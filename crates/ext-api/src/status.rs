use crate::context::AppContext;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ext_core::{
    sidecar::{GetStatusData, GetStatusUnitSystem, SidecarClient},
    state::{self, ResolveInput, WorkingFileStatus},
};
use ext_db::StateFile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sysinfo::System;

use crate::path_utils::normalize_path;

/// Resolve the current working file status from a freshly loaded StateFile.
/// Shared helper used by commit, switch, checkout, stash, and guards.
pub fn resolve_working_file_status(
    state: &StateFile,
    project_root: &std::path::Path,
) -> WorkingFileStatus {
    let working_path = working_model_path(state, project_root);
    let fast_status = resolve_fast_status(state, &working_path);
    apply_persisted_closed_status(fast_status, state)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusOptions {
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarStatusReport {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub etabs_version: Option<String>,
    pub open_file_path: Option<String>,
    pub is_model_open: bool,
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_system: Option<GetStatusUnitSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub project_name: Option<String>,
    pub project_root: PathBuf,
    pub working_model_path: PathBuf,
    pub working_status: WorkingFileStatus,
    pub based_on_version: Option<String>,
    pub etabs_pid: Option<u32>,
    pub last_known_mtime: Option<DateTime<Utc>>,
    pub current_mtime: Option<DateTime<Utc>>,
    pub sidecar_status: Option<SidecarStatusReport>,
    pub sidecar_warning: Option<String>,
    pub onedrive_warning: Option<String>,
}

fn mtime(path: &Path) -> Option<DateTime<Utc>> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(modified.into())
}

fn is_pid_alive(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    system.process(sysinfo::Pid::from_u32(pid)).is_some()
}

fn working_model_path(state: &StateFile, project_root: &Path) -> PathBuf {
    state
        .working_file
        .as_ref()
        .map(|w| w.path.clone())
        .unwrap_or_else(|| {
            project_root
                .join(".etabs-ext")
                .join("main")
                .join("working")
                .join("model.edb")
        })
}

fn resolve_fast_status(state: &StateFile, working_path: &Path) -> WorkingFileStatus {
    let wf = state.working_file.as_ref();
    let current_mtime = mtime(working_path);

    state::resolve(ResolveInput {
        file_exists: working_path.exists(),
        etabs_pid: wf.and_then(|w| w.etabs_pid),
        pid_alive: wf
            .and_then(|w| w.etabs_pid)
            .map(is_pid_alive)
            .unwrap_or(false),
        based_on_version: wf.and_then(|w| w.based_on_version.clone()),
        last_known_mtime: wf.and_then(|w| w.last_known_mtime),
        current_mtime,
    })
}

fn apply_persisted_closed_status(
    fast_status: WorkingFileStatus,
    state: &StateFile,
) -> WorkingFileStatus {
    if fast_status != WorkingFileStatus::Clean {
        return fast_status;
    }

    match state.working_file.as_ref().map(|wf| wf.status) {
        Some(WorkingFileStatus::Analyzed) => WorkingFileStatus::Analyzed,
        Some(WorkingFileStatus::Locked) => WorkingFileStatus::Locked,
        _ => fast_status,
    }
}

fn normalize_path_string(path: &Path) -> String {
    normalize_path(path).display().to_string()
}

fn paths_match(left: &Path, right: &Path) -> bool {
    normalize_path_string(left).eq_ignore_ascii_case(&normalize_path_string(right))
}

fn sidecar_status_report(data: GetStatusData) -> SidecarStatusReport {
    SidecarStatusReport {
        is_running: data.is_running,
        pid: data.pid,
        etabs_version: data.etabs_version,
        open_file_path: data
            .open_file_path
            .map(|p| normalize_path_string(Path::new(&p))),
        is_model_open: data.is_model_open,
        is_locked: data.is_locked,
        is_analyzed: data.is_analyzed,
        unit_system: data.unit_system,
    }
}

fn apply_sidecar_resolution(
    status: WorkingFileStatus,
    working_file: &Path,
    data: &GetStatusData,
) -> WorkingFileStatus {
    if status != WorkingFileStatus::Clean || !data.is_running || !data.is_model_open {
        return status;
    }

    let Some(open_file) = data.open_file_path.as_deref() else {
        return status;
    };

    if !paths_match(Path::new(open_file), working_file) {
        return status;
    }

    if data.is_locked == Some(true) {
        WorkingFileStatus::Locked
    } else if data.is_analyzed == Some(true) {
        WorkingFileStatus::Analyzed
    } else {
        status
    }
}

pub async fn resolve_with_sidecar(
    fast_status: WorkingFileStatus,
    sidecar: Option<&SidecarClient>,
    working_file: &Path,
) -> WorkingFileStatus {
    if fast_status != WorkingFileStatus::Clean {
        return fast_status;
    }

    let Some(sidecar) = sidecar else {
        return fast_status;
    };

    match sidecar.get_status().await {
        Ok(data) => apply_sidecar_resolution(fast_status, working_file, &data),
        Err(_) => fast_status,
    }
}

pub async fn project_status(ctx: &AppContext, options: StatusOptions) -> Result<StatusReport> {
    let state = ctx
        .load_state()
        .with_context(|| "Failed to load state.json".to_string())?;
    let working_file = state.working_file.as_ref();

    let working_model_path_raw = working_model_path(&state, &ctx.project_root);
    let current_mtime = mtime(&working_model_path_raw);
    let last_known_mtime = working_file.and_then(|w| w.last_known_mtime);
    let etabs_pid = working_file.and_then(|w| w.etabs_pid);
    let based_on_version = working_file.and_then(|w| w.based_on_version.clone());
    let fast_status = resolve_fast_status(&state, &working_model_path_raw);
    let mut status = apply_persisted_closed_status(fast_status, &state);

    let mut sidecar_status = None;
    let mut sidecar_warning = None;
    if options.verbose {
        // EXCEPTION: direct sidecar access is permitted here.
        // `ext status --verbose` degrades gracefully — missing sidecar is a
        // warning, not a hard failure. All other callers must use require_sidecar().
        // See agents.md §Sidecar Integration for the rule and its rationale.
        if let Some(sidecar) = ctx.sidecar.as_ref() {
            match sidecar.get_status().await {
                Ok(data) => {
                    status = apply_sidecar_resolution(status, &working_model_path_raw, &data);
                    sidecar_status = Some(sidecar_status_report(data));
                }
                Err(err) => {
                    sidecar_warning = Some(format!("Failed to query sidecar: {err}"));
                }
            }
        } else {
            sidecar_warning = Some(
                "Sidecar not configured (set project.sidecar-path or ETABS_SIDECAR_PATH)"
                    .to_string(),
            );
        }
    }

    let onedrive_warning = if crate::init::is_onedrive_path(&ctx.project_root)
        && !ctx.config.onedrive.acknowledged_sync_or_default()
    {
        Some(
            "Project is inside a OneDrive-synced folder. This can corrupt .edb files during sync."
                .to_string(),
        )
    } else {
        None
    };

    Ok(StatusReport {
        project_name: ctx.config.project.name.clone(),
        project_root: normalize_path(&ctx.project_root),
        working_model_path: normalize_path(&working_model_path_raw),
        working_status: status,
        based_on_version,
        etabs_pid,
        last_known_mtime,
        current_mtime,
        sidecar_status,
        sidecar_warning,
        onedrive_warning,
    })
}
