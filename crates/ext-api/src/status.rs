use crate::context::AppContext;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ext_core::state::{self, ResolveInput, WorkingFileStatus};
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
    let wf = state.working_file.as_ref();
    let working_path = wf.map(|w| w.path.clone()).unwrap_or_else(|| {
        project_root
            .join(".etabs-ext")
            .join("main")
            .join("working")
            .join("model.edb")
    });
    let current_mtime: Option<chrono::DateTime<Utc>> = std::fs::metadata(&working_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(Into::into);

    state::resolve(ext_core::state::ResolveInput {
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
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_system: Option<String>,
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

pub async fn project_status(ctx: &AppContext, options: StatusOptions) -> Result<StatusReport> {
    let state = ctx
        .load_state()
        .with_context(|| "Failed to load state.json".to_string())?;
    let working_file = state.working_file.as_ref();

    let working_model_path_raw = working_file.map(|w| w.path.clone()).unwrap_or_else(|| {
        ctx.project_root
            .join(".etabs-ext")
            .join("main")
            .join("working")
            .join("model.edb")
    });
    let current_mtime = mtime(&working_model_path_raw);
    let last_known_mtime = working_file.and_then(|w| w.last_known_mtime.as_ref().cloned());
    let etabs_pid = working_file.and_then(|w| w.etabs_pid);
    let based_on_version = working_file.and_then(|w| w.based_on_version.clone());

    let status = state::resolve(ResolveInput {
        file_exists: working_model_path_raw.exists(),
        etabs_pid,
        pid_alive: etabs_pid.map(is_pid_alive).unwrap_or(false),
        based_on_version: based_on_version.clone(),
        last_known_mtime,
        current_mtime,
    });

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
                    sidecar_status = Some(SidecarStatusReport {
                        is_running: data.is_running,
                        pid: data.pid,
                        etabs_version: data.etabs_version,
                        open_file_path: data
                            .open_file_path
                            .map(|p| normalize_path(Path::new(&p)).display().to_string()),
                        is_locked: data.is_locked,
                        is_analyzed: data.is_analyzed,
                        unit_system: data.unit_system,
                    });
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
