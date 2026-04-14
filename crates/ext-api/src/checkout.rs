// ext-api::checkout — restore a committed version as the working file.
//
// Two-phase flow for the CLI:
//   Phase 1: call with conflict_resolution: None → detect conflict → return Err(CheckoutConflict)
//   Phase 2: call with conflict_resolution: Some(resolution) → execute
//
// Tauri and agent pass the resolution in one call (no two-phase needed).
// --force maps to CheckoutConflictResolution::Discard.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{
    branch,
    fs::{atomic_copy, check_disk_space},
    state::WorkingFileStatus,
    vcs::current_branch,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    status::resolve_working_file_status,
};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CheckoutOptions {
    /// None = detect conflict only; Some = execute with chosen resolution.
    pub conflict_resolution: Option<CheckoutConflictResolution>,
}


#[derive(Debug, Clone)]
pub enum CheckoutConflictResolution {
    /// Commit the working file before checking out.
    CommitFirst { message: String },
    /// Stash the working file before checking out.
    Stash,
    /// Discard working file changes (--force).
    Discard,
}

/// Returned as an error when conflict_resolution is None and the working file
/// is MODIFIED — the CLI should prompt the user and re-call with a resolution.
#[derive(Debug)]
pub struct CheckoutConflict {
    pub current_status: WorkingFileStatus,
    pub target_version: String,
    pub current_branch: String,
    pub stash_exists: bool,
}

impl std::fmt::Display for CheckoutConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "✗ Working file has uncommitted changes (status: {})\n  \
             Target: {}/{}\n  \
             Options: [c] commit first  [s] stash  [d] discard  [x] cancel",
            self.current_status, self.current_branch, self.target_version
        )
    }
}

impl std::error::Error for CheckoutConflict {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutResult {
    pub version_id: String,
    pub branch: String,
    pub working_model_path: PathBuf,
}

/// Parse `"main/v3"` → `("main", "v3")` or `"v3"` → `(current_branch, "v3")`.
fn parse_version_ref(version_ref: &str, current_branch: &str) -> (String, String) {
    if let Some((branch, version)) = version_ref.split_once('/') {
        (branch.to_string(), version.to_string())
    } else {
        (current_branch.to_string(), version_ref.to_string())
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn checkout_version(
    ctx: &AppContext,
    version_ref: &str,
    opts: CheckoutOptions,
) -> Result<CheckoutResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let initial_status = resolve_working_file_status(&state, &ctx.project_root);
    let initial_branch = current_branch(&ext_dir)?;

    // Hard blocks (ETABS open, analyzed, orphaned).
    match check_state_guard(Command::Checkout, &initial_status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let (target_branch, target_version) = parse_version_ref(version_ref, &initial_branch);

    // Handle cross-branch checkout — switch first, then checkout version.
    if target_branch != initial_branch {
        // Switch (re-uses switch_branch logic internally).
        crate::switch::switch_branch(ctx, &target_branch).await?;
        // Reload state after switch.
        state = ctx.load_state()?;
    }
    let active_branch = current_branch(&ext_dir)?;
    let cur_status = resolve_working_file_status(&state, &ctx.project_root);

    let version_dir = ext_dir.join(&target_branch).join(&target_version);
    if !version_dir.exists() {
        bail!(
            "✗ Version '{target_branch}/{target_version}' not found\n  Run: ext log to see available versions"
        );
    }
    let src_edb = version_dir.join("model.edb");
    if !src_edb.exists() {
        bail!("✗ model.edb missing in version '{target_branch}/{target_version}'");
    }

    let working_file = branch::working_model_path(&active_branch, &ext_dir);

    // MODIFIED conflict handling.
    if cur_status == WorkingFileStatus::Modified {
        match opts.conflict_resolution {
            None => {
                let stash_exists = state.stashes.contains_key(&active_branch);
                return Err(anyhow::Error::new(CheckoutConflict {
                    current_status: cur_status,
                    target_version: target_version.clone(),
                    current_branch: active_branch.clone(),
                    stash_exists,
                }));
            }
            Some(CheckoutConflictResolution::CommitFirst { ref message }) => {
                crate::commit::commit_version(
                    ctx,
                    message,
                    crate::commit::CommitOptions {
                        no_e2k: true,
                        analyze: false,
                    },
                )
                .await
                .with_context(|| "Auto-commit before checkout failed")?;
            }
            Some(CheckoutConflictResolution::Stash) => {
                let based_on = state
                    .working_file
                    .as_ref()
                    .and_then(|w| w.based_on_version.clone());
                ext_core::stash::save(
                    &active_branch,
                    &working_file,
                    &ext_dir,
                    Some("auto-stash before checkout"),
                    &mut state.stashes,
                    based_on,
                    false,
                )
                .with_context(|| "Auto-stash before checkout failed")?;
                ctx.save_state(&state)?;
            }
            Some(CheckoutConflictResolution::Discard) => {
                // Just overwrite — no save.
            }
        }
    }

    // Copy the version's model.edb to working.
    check_disk_space(&src_edb, working_file.parent().unwrap_or(&ext_dir))
        .with_context(|| "Disk space check failed before checkout")?;
    atomic_copy(&src_edb, &working_file)
        .with_context(|| format!("Failed to copy {target_version}/model.edb to working"))?;

    // Update state.json.
    let mtime: Option<chrono::DateTime<Utc>> = std::fs::metadata(&working_file)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(Into::into);

    state = ctx.load_state()?;
    if let Some(ref mut wf) = state.working_file {
        wf.path = working_file.clone();
        wf.based_on_version = Some(target_version.clone());
        wf.last_known_mtime = mtime;
        wf.etabs_pid = None;
        wf.status = WorkingFileStatus::Clean;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(CheckoutResult {
        version_id: target_version,
        branch: active_branch,
        working_model_path: working_file,
    })
}
