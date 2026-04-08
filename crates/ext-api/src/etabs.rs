// ext-api::etabs — ETABS lifecycle commands.
//
// Commands: open, close, status, unlock, recover.
//
// All write-path functions follow this pattern:
//   1. Load state + fast resolve status
//   2. resolve_with_sidecar() when ANALYZED/LOCKED detection is required
//      (must precede the guard so the guard sees the full status)
//   3. check_state_guard() — hard block or warn
//   4. Call sidecar via ctx.require_sidecar()
//   5. Update state.json
//   6. Return structured result

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{
    branch,
    fs::{atomic_copy, check_disk_space},
    sidecar::{GetStatusData, GetStatusUnitSystem, OpenModelData, SidecarClient},
    state::WorkingFileStatus,
    vcs::current_branch,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    path_utils::normalize_path,
    status::{apply_sidecar_resolution, resolve_with_sidecar, resolve_working_file_status},
};

// ── Public enums ──────────────────────────────────────────────────────────────

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

// ── Result structs ────────────────────────────────────────────────────────────

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
    /// True when the model was not open in ETABS and had to be reopened to unlock.
    pub reopened_for_unlock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtabsRecoverResult {
    pub choice: RecoveryChoice,
    pub arrival_status: WorkingFileStatus,
    pub working_file: PathBuf,
}

// ── Conflict types (downcastable errors) ──────────────────────────────────────

/// Returned when `etabs_close` is called in Interactive mode and the working
/// file has unsaved changes. The CLI should prompt [s/d/x] and re-call.
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

/// Returned by `etabs_recover` Phase 1 (choice: None) so the CLI can present
/// an informed prompt to the user before executing the recovery.
#[derive(Debug)]
pub struct EtabsRecoverConflict {
    pub pid: u32,
    pub working_file: PathBuf,
    pub based_on_version: Option<String>,
    /// True when the file's mtime is newer than `last_known_mtime`, meaning
    /// the engineer made changes before the crash.
    pub file_was_modified: bool,
}

impl std::fmt::Display for EtabsRecoverConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "⚠ ETABS appears to have crashed while editing {}\n  File modified: {}\n  Last version: {}\n  [k] Keep file changes  [r] Restore from last committed version  [x] Cancel",
            self.working_file.display(),
            if self.file_was_modified { "Yes" } else { "No" },
            self.based_on_version.as_deref().unwrap_or("none"),
        )
    }
}

impl std::error::Error for EtabsRecoverConflict {}

// ── Private helpers ───────────────────────────────────────────────────────────

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

fn sidecar_target_matches(data: &GetStatusData, working_file: &Path) -> bool {
    let Some(open_file) = data.open_file_path.as_deref() else {
        return false;
    };
    data.is_running && data.is_model_open && paths_match(Path::new(open_file), working_file)
}

async fn confirm_open_pid(
    sidecar: &SidecarClient,
    target_file: &Path,
    opened: &OpenModelData,
) -> Result<u32> {
    if let Some(pid) = opened.pid {
        return Ok(pid);
    }

    let status = sidecar.get_status().await.with_context(|| {
        format!(
            "Failed to confirm ETABS status for {}",
            target_file.display()
        )
    })?;

    if sidecar_target_matches(&status, target_file) {
        if let Some(pid) = status.pid {
            return Ok(pid);
        }
    }

    bail!(
        "✗ ETABS opened but PID could not be confirmed\n  File: {}\n  Close ETABS and try again",
        target_file.display()
    );
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

// ── Public API ────────────────────────────────────────────────────────────────

/// Open the working file (or a snapshot) in ETABS.
///
/// `version_ref` — if None, opens the working file. If Some("v3") or
/// Some("main/v3"), opens that committed snapshot (read-only recommended).
pub async fn etabs_open(ctx: &AppContext, version_ref: Option<&str>) -> Result<EtabsOpenResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let working_status = resolve_working_file_status(&state, &ctx.project_root);
    let working_file = working_file_path(ctx, &state);

    // Guard check uses the fast status — OPEN_CLEAN/MODIFIED/MISSING/ORPHANED
    // are all detectable without a sidecar call.
    match check_state_guard(Command::EtabsOpen, &working_status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    // LOCKED state looks like CLEAN from mtime alone — must ask sidecar.
    let full_status =
        resolve_with_sidecar(working_status, ctx.sidecar.as_ref(), &working_file).await;
    if full_status == WorkingFileStatus::Locked {
        bail!("✗ Model is locked\n  Run: ext etabs unlock before opening");
    }

    let branch_name = current_branch(&ext_dir)?;
    let (target_file, is_snapshot, warning) = if let Some(vref) = version_ref {
        (
            resolve_version_model(ctx, &branch_name, vref)?,
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
        .open_model(&target_file, false, true)
        .await
        .with_context(|| format!("Failed to launch ETABS for {}", target_file.display()))?;
    let confirmed_pid = confirm_open_pid(sidecar, &target_file, &opened).await?;

    // Record the mtime at open time so OPEN_CLEAN vs OPEN_MODIFIED detection
    // is accurate: ETABS saving a file bumps the mtime above this baseline.
    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = Some(confirmed_pid);
        wf.last_known_mtime = mtime(&target_file);
        wf.status = WorkingFileStatus::OpenClean;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(EtabsOpenResult {
        opened_file: normalize_display(&target_file),
        pid: confirmed_pid,
        is_snapshot,
        warning,
    })
}

/// Close ETABS.
///
/// In `Interactive` mode, returns `Err(EtabsCloseConflict)` when there are
/// unsaved changes so the CLI can prompt the user. Re-call with `Save` or
/// `NoSave` to proceed.
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
        // ANALYZED / LOCKED / OPEN_CLEAN — nothing to save
        _ => false,
    };

    let close_result = sidecar.close_model(save).await?;

    // Capture mtime immediately after close so the next status call does not
    // incorrectly report MODIFIED if ETABS saved changes during the session.
    let mtime_after = mtime(&working_file);

    // Determine arrival status. Use the pre-close sidecar data to detect
    // ANALYZED/LOCKED (set by analysis run inside ETABS before close).
    // NOTE: Phase 1 limitation — this reflects the state at pre-close query
    // time. If ETABS set the analysis flag at the moment of close, a follow-up
    // sidecar.get_status() post-close may be more accurate but is skipped here
    // to avoid an unnecessary process spawn. See Week 7-8 for refinement.
    let arrival_status = {
        let maybe_analyzed_or_locked = if sidecar_target_matches(&pre_close, &working_file) {
            if pre_close.is_locked == Some(true) {
                Some(WorkingFileStatus::Locked)
            } else if pre_close.is_analyzed == Some(true) {
                Some(WorkingFileStatus::Analyzed)
            } else {
                None
            }
        } else {
            None
        };

        maybe_analyzed_or_locked.unwrap_or_else(|| {
            // No analysis state from sidecar — fall back to mtime comparison.
            // If ETABS saved the file, mtime_after > last_known_mtime → MODIFIED.
            // If nothing changed (NoSave or OpenClean), mtime is unchanged → CLEAN.
            if save {
                // File was saved — determine MODIFIED vs CLEAN by mtime comparison
                // against what was recorded at open time.
                let last = state
                    .working_file
                    .as_ref()
                    .and_then(|wf| wf.last_known_mtime);
                match (last, mtime_after) {
                    (Some(l), Some(a)) if a > l => WorkingFileStatus::Modified,
                    _ => WorkingFileStatus::Clean,
                }
            } else {
                WorkingFileStatus::Clean
            }
        })
    };

    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = None;
        wf.last_known_mtime = mtime_after;
        wf.status = arrival_status;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(EtabsCloseResult {
        saved: close_result.was_saved,
        arrival_status,
        warning: None,
    })
}

/// Query the current ETABS state via sidecar.
///
/// Always calls the sidecar — this is a dedicated inquiry command and is the
/// one place where a live sidecar query is always expected.
pub async fn etabs_status(ctx: &AppContext) -> Result<EtabsStatusResult> {
    let state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);
    let fast_status = resolve_working_file_status(&state, &ctx.project_root);

    if let Some(sidecar) = ctx.sidecar.as_ref() {
        let data = sidecar.get_status().await?;

        // Use the shared helper to avoid duplicating ANALYZED/LOCKED detection.
        let working_file_status = apply_sidecar_resolution(fast_status, &working_file, &data);

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
            working_file_status,
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
        working_file_status: fast_status,
        sidecar_available: false,
    })
}

/// Clear the analysis lock on the working file.
///
/// LOCKED state is only detectable via sidecar, so `resolve_with_sidecar`
/// is called BEFORE the guard check — this is the correct order.
pub async fn etabs_unlock(ctx: &AppContext) -> Result<EtabsUnlockResult> {
    let mut state = ctx.load_state()?;
    let working_file = working_file_path(ctx, &state);

    // Fast resolve first, then upgrade to full status via sidecar.
    // The guard for EtabsUnlock requires LOCKED — which is only returned by
    // the full (sidecar) resolver, never by the fast (mtime-only) resolver.
    let fast_status = resolve_working_file_status(&state, &ctx.project_root);
    let full_status = resolve_with_sidecar(fast_status, ctx.sidecar.as_ref(), &working_file).await;

    match check_state_guard(Command::EtabsUnlock, &full_status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let sidecar = ctx.require_sidecar()?;
    let mut reopened_for_unlock = false;
    let sidecar_status = sidecar.get_status().await?;

    // unlock_model requires the model to be open in ETABS (Mode A).
    // If it is not already open, open it in hidden mode, unlock, then close.
    if !sidecar_target_matches(&sidecar_status, &working_file) {
        let opened = sidecar
            .open_model(&working_file, false, false)
            .await
            .with_context(|| format!("Failed to reopen {} for unlock", working_file.display()))?;
        reopened_for_unlock = true;
        if let Some(wf) = state.working_file.as_mut() {
            wf.etabs_pid = opened.pid;
            wf.status = WorkingFileStatus::OpenClean;
            wf.status_changed_at = Utc::now();
        }
    }

    sidecar
        .unlock_model(&working_file)
        .await
        .with_context(|| format!("Failed to unlock {}", working_file.display()))?;

    // Query post-unlock state to determine arrival status accurately.
    let post_unlock = sidecar.get_status().await?;
    let arrival_status = if sidecar_target_matches(&post_unlock, &working_file)
        && post_unlock.is_analyzed == Some(true)
    {
        WorkingFileStatus::Analyzed
    } else {
        WorkingFileStatus::Clean
    };

    sidecar.close_model(false).await?;

    let mtime_after = mtime(&working_file);
    if let Some(wf) = state.working_file.as_mut() {
        wf.etabs_pid = None;
        wf.last_known_mtime = mtime_after;
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

/// Recover from an ETABS crash (ORPHANED state).
///
/// Two-phase flow:
///   Phase 1 — call with `choice: None` → returns `Err(EtabsRecoverConflict)`
///             containing enough context for the CLI to build a useful prompt.
///   Phase 2 — call with `choice: Some(RecoveryChoice)` → executes the choice.
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

    // Phase 1 — detect and surface conflict for the CLI prompt.
    let selected = match choice {
        Some(c) => c,
        None => {
            let wf = state
                .working_file
                .as_ref()
                .context("Missing working file state for recovery")?;

            // Compute whether the file was modified after the crash.
            // This is the key information the engineer needs to choose [k] vs [r].
            let file_was_modified = wf
                .last_known_mtime
                .zip(mtime(&working_file))
                .map(|(last, current)| current > last)
                .unwrap_or(false);

            return Err(anyhow::Error::new(EtabsRecoverConflict {
                pid: wf.etabs_pid.unwrap_or_default(),
                working_file: working_file.clone(),
                based_on_version: wf.based_on_version.clone(),
                file_was_modified,
            }));
        }
    };

    // Phase 2 — execute the chosen recovery action.
    // Resolve branch once; used in both match arms and the final return.
    let branch_name = current_branch(&ext_dir)?;

    match selected {
        RecoveryChoice::KeepChanges => {
            if let Some(wf) = state.working_file.as_mut() {
                wf.etabs_pid = None;
                wf.status = WorkingFileStatus::Modified;
                wf.status_changed_at = Utc::now();
            }
        }
        RecoveryChoice::RestoreFromVersion => {
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
                bail!(
                    "✗ Snapshot missing: {}\n  The committed version file was not found",
                    snapshot.display()
                );
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
        working_file: branch::working_model_path(&branch_name, &ext_dir),
    })
}
