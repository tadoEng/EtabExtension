use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{
    branch,
    fs::{atomic_copy, check_disk_space},
    sidecar::{GetStatusData, GetStatusUnitSystem},
    state::WorkingFileStatus,
    vcs::current_branch,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    path_utils::normalize_path,
    status::{resolve_with_sidecar, resolve_working_file_status},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CloseMode {
    Interactive,
    Save,
    NoSave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryChoice {
    KeepChanges,
    RestoreFromVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsOpenResult {
    pub opened_file: PathBuf,
    pub pid: u32,
    pub is_snapshot: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsCloseResult {
    pub saved: bool,
    pub arrival_status: WorkingFileStatus,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsStatusResult {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub open_file_path: Option<PathBuf>,
    pub etabs_version: Option<String>,
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_system: Option<GetStatusUnitSystem>,
    pub working_file_status: WorkingFileStatus,
    pub sidecar_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsUnlockResult {
    pub file: PathBuf,
    pub arrival_status: WorkingFileStatus,
    pub reopened_for_unlock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsRecoverResult {
    pub choice: RecoveryChoice,
    pub arrival_status: WorkingFileStatus,
    pub working_file: PathBuf,
}

#[derive(Debug)]
pub struct EtabsCloseConflict {
    pub pid: u32,
    pub open_file: PathBuf,
}

impl std::fmt::Display for EtabsCloseConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "⚠ ETABS has unsaved changes\n  File: {}\n  [s] Save and close  [d] Discard changes  [x] Cancel",
            self.open_file.display()
        )
    }
}

impl std::error::Error for EtabsCloseConflict {}

#[derive(Debug)]
pub struct EtabsRecoverConflict {
    pub pid: u32,
    pub working_file: PathBuf,
    pub based_on_version: Option<String>,
}

impl std::fmt::Display for EtabsRecoverConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "⚠ ETABS appears to have crashed while editing {}\n  [k] Keep file changes  [r] Restore from last committed version  [x] Cancel",
            self.working_file.display()
        )
    }
}

impl std::error::Error for EtabsRecoverConflict {}

fn mtime(path: &Path) -> Option<chrono::DateTime<Utc>> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(Into::into)
}

fn normalize_display(path: &Path) -> PathBuf {
    normalize_path(path)
}

fn paths_match(left: &Path, right: &Path) -> bool {
    normalize_display(left)
        .display()
        .to_string()
        .eq_ignore_ascii_case(&normalize_display(right).display().to_string())
}

fn working_file_path(ctx: &AppContext, state: &ext_db::StateFile) -> PathBuf {
    state
        .working_file
        .as_ref()
        .map(|wf| wf.path.clone())
        .unwrap_or_else(|| {
            ctx.project_root
                .join(".etabs-ext")
                .join("main")
                .join("working")
                .join("model.edb")
        })
}

fn close_state_from_sidecar(
    data: &GetStatusData,
    working_file: &Path,
) -> Option<WorkingFileStatus> {
    let Some(open_file) = data.open_file_path.as_deref() else {
        return None;
    };
    if !data.is_running || !data.is_model_open || !paths_match(Path::new(open_file), working_file) {
        return None;
    }

    if data.is_locked == Some(true) {
        Some(WorkingFileStatus::Locked)
    } else if data.is_analyzed == Some(true) {
        Some(WorkingFileStatus::Analyzed)
    } else {
        None
    }
}

fn resolve_version_model(
    ctx: &AppContext,
    current_branch_name: &str,
    version_ref: &str,
) -> Result<PathBuf> {
    let (branch_name, version_id) =
        if let Some((branch_name, version_id)) = version_ref.split_once('/') {
            (branch_name.to_string(), version_id.to_string())
        } else {
            (current_branch_name.to_string(), version_ref.to_string())
        };

    let model = ctx
        .ext_dir()
        .join(&branch_name)
        .join(&version_id)
        .join("model.edb");
    if !model.exists() {
        bail!(
            "✗ Version '{branch_name}/{version_id}' not found\n  Run: ext log to see available versions"
        );
    }
    Ok(model)
}

fn sidecar_target_matches(data: &GetStatusData, working_file: &Path) -> bool {
    let Some(open_file) = data.open_file_path.as_deref() else {
        return false;
    };
    data.is_running && data.is_model_open && paths_match(Path::new(open_file), working_file)
}

pub async fn etabs_open(ctx: &AppContext, version_ref: Option<&str>) -> Result<EtabsOpenResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let working_status = resolve_working_file_status(&state, &ctx.project_root);
    let working_file = working_file_path(ctx, &state);

    match check_state_guard(Command::EtabsOpen, &working_status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let full_status =
        resolve_with_sidecar(working_status, ctx.sidecar.as_ref(), &working_file).await;
    if full_status == WorkingFileStatus::Locked {
        bail!("✗ Model is locked\n  Run: ext etabs unlock before opening");
    }

    let branch_name = current_branch(&ext_dir)?;
    let (target_file, is_snapshot, warning) = if let Some(version_ref) = version_ref {
        (
            resolve_version_model(ctx, &branch_name, version_ref)?,
            true,
            Some("Opening a snapshot — changes will be discarded".to_string()),
        )
    } else {
        if !working_file.exists() {
            bail!("✗ Working file missing\n  Run: ext checkout vN");
        }
        (working_file.clone(), false, None)
    };

    let sidecar = ctx.require_sidecar()?;
    let opened = sidecar
        .open_model(&target_file, false, false)
        .await
        .with_context(|| format!("Failed to launch ETABS for {}", target_file.display()))?;

    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = Some(opened.pid);
        wf.status = WorkingFileStatus::OpenClean;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(EtabsOpenResult {
        opened_file: normalize_display(&target_file),
        pid: opened.pid,
        is_snapshot,
        warning,
    })
}

pub async fn etabs_close(ctx: &AppContext, mode: CloseMode) -> Result<EtabsCloseResult> {
    let mut state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);
    let status = resolve_working_file_status(&state, &ctx.project_root);

    match check_state_guard(Command::EtabsClose, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let sidecar = ctx.require_sidecar()?;
    let pre_close = sidecar.get_status().await?;
    if !pre_close.is_running || !pre_close.is_model_open {
        bail!("✗ ETABS is not running\n  Nothing to close");
    }

    let save = match (status, mode) {
        (WorkingFileStatus::OpenModified, CloseMode::Interactive) => {
            return Err(anyhow::Error::new(EtabsCloseConflict {
                pid: pre_close.pid.unwrap_or_default(),
                open_file: pre_close
                    .open_file_path
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| working_file.clone()),
            }));
        }
        (WorkingFileStatus::OpenModified, CloseMode::Save) => true,
        (WorkingFileStatus::OpenModified, CloseMode::NoSave) => false,
        _ => false,
    };

    let closed_state = close_state_from_sidecar(&pre_close, &working_file);
    let close_result = sidecar.close_model(save).await?;

    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = None;
        wf.status_changed_at = Utc::now();
        wf.status = closed_state.unwrap_or(WorkingFileStatus::Clean);
    }

    let arrival_status = if let Some(closed_state) = closed_state {
        closed_state
    } else {
        resolve_working_file_status(&state, &ctx.project_root)
    };

    if let Some(wf) = state.working_file.as_mut() {
        wf.status = arrival_status;
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(EtabsCloseResult {
        saved: close_result.was_saved,
        arrival_status,
        warning: None,
    })
}

pub async fn etabs_status(ctx: &AppContext) -> Result<EtabsStatusResult> {
    let state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);
    let mut working_status = resolve_working_file_status(&state, &ctx.project_root);

    if let Some(sidecar) = ctx.sidecar.as_ref() {
        let data = sidecar.get_status().await?;
        if working_status == WorkingFileStatus::Clean
            && sidecar_target_matches(&data, &working_file)
        {
            if data.is_locked == Some(true) {
                working_status = WorkingFileStatus::Locked;
            } else if data.is_analyzed == Some(true) {
                working_status = WorkingFileStatus::Analyzed;
            }
        }

        return Ok(EtabsStatusResult {
            is_running: data.is_running,
            pid: data.pid,
            open_file_path: data
                .open_file_path
                .map(PathBuf::from)
                .map(|p| normalize_display(&p)),
            etabs_version: data.etabs_version,
            is_locked: data.is_locked,
            is_analyzed: data.is_analyzed,
            unit_system: data.unit_system,
            working_file_status: working_status,
            sidecar_available: true,
        });
    }

    Ok(EtabsStatusResult {
        is_running: false,
        pid: None,
        open_file_path: None,
        etabs_version: None,
        is_locked: None,
        is_analyzed: None,
        unit_system: None,
        working_file_status: working_status,
        sidecar_available: false,
    })
}

pub async fn etabs_unlock(ctx: &AppContext) -> Result<EtabsUnlockResult> {
    let mut state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);
    let status = resolve_working_file_status(&state, &ctx.project_root);

    match check_state_guard(Command::EtabsUnlock, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let sidecar = ctx.require_sidecar()?;
    let mut reopened_for_unlock = false;
    let sidecar_status = sidecar.get_status().await?;

    if !sidecar_target_matches(&sidecar_status, &working_file) {
        let opened = sidecar
            .open_model(&working_file, false, false)
            .await
            .with_context(|| format!("Failed to reopen {} for unlock", working_file.display()))?;
        reopened_for_unlock = true;
        if let Some(wf) = state.working_file.as_mut() {
            wf.etabs_pid = Some(opened.pid);
            wf.status = WorkingFileStatus::OpenClean;
            wf.status_changed_at = Utc::now();
        }
    }

    sidecar
        .unlock_model(&working_file)
        .await
        .with_context(|| format!("Failed to unlock {}", working_file.display()))?;
    let post_unlock = sidecar.get_status().await?;

    let arrival_status = if sidecar_target_matches(&post_unlock, &working_file)
        && post_unlock.is_analyzed == Some(true)
    {
        WorkingFileStatus::Analyzed
    } else {
        WorkingFileStatus::Clean
    };

    sidecar.close_model(false).await?;

    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = None;
        wf.status = arrival_status;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(EtabsUnlockResult {
        file: normalize_display(&working_file),
        arrival_status,
        reopened_for_unlock,
    })
}

pub async fn etabs_recover(
    ctx: &AppContext,
    choice: Option<RecoveryChoice>,
) -> Result<EtabsRecoverResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);
    let status = resolve_working_file_status(&state, &ctx.project_root);

    match check_state_guard(Command::EtabsRecover, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let selected = match choice {
        Some(choice) => choice,
        None => {
            let wf = state
                .working_file
                .as_ref()
                .context("Missing working file state for recovery")?;
            return Err(anyhow::Error::new(EtabsRecoverConflict {
                pid: wf.etabs_pid.unwrap_or_default(),
                working_file: working_file.clone(),
                based_on_version: wf.based_on_version.clone(),
            }));
        }
    };

    match selected {
        RecoveryChoice::KeepChanges => {
            if let Some(wf) = state.working_file.as_mut() {
                wf.etabs_pid = None;
                wf.status = WorkingFileStatus::Modified;
                wf.status_changed_at = Utc::now();
            }
        }
        RecoveryChoice::RestoreFromVersion => {
            let branch_name = current_branch(&ext_dir)?;
            let based_on_version = state
                .working_file
                .as_ref()
                .and_then(|wf| wf.based_on_version.clone())
                .context("No last committed version available for recovery")?;
            let snapshot = ext_dir
                .join(&branch_name)
                .join(&based_on_version)
                .join("model.edb");
            if !snapshot.exists() {
                bail!("Snapshot missing: {}", snapshot.display());
            }
            check_disk_space(&snapshot, working_file.parent().unwrap_or(&ext_dir))?;
            atomic_copy(&snapshot, &working_file)?;

            if let Some(wf) = state.working_file.as_mut() {
                wf.etabs_pid = None;
                wf.last_known_mtime = mtime(&working_file);
                wf.status = WorkingFileStatus::Clean;
                wf.status_changed_at = Utc::now();
            }
        }
    }

    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    let arrival_status = state
        .working_file
        .as_ref()
        .map(|wf| wf.status)
        .unwrap_or(WorkingFileStatus::Missing);

    Ok(EtabsRecoverResult {
        choice: selected,
        arrival_status,
        working_file: branch::working_model_path(&current_branch(&ext_dir)?, &ext_dir),
    })
}
