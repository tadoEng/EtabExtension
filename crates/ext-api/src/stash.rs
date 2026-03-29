// ext-api::stash — save, pop, drop, list operations.
//
// Delegates to ext-core::stash for file I/O.
// This layer handles the guard check, state loading, and state saving.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::stash::{self, StashListEntry};
use ext_core::state::WorkingFileStatus;
use ext_core::vcs::current_branch;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    status::resolve_working_file_status,
};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashSaveResult {
    pub branch: String,
    pub based_on: Option<String>,
    pub stash_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashPopResult {
    pub branch: String,
    pub restored_based_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashDropResult {
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashListResult {
    pub stashes: Vec<StashListEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct StashPopOptions {
    pub conflict_resolution: Option<StashPopConflictResolution>,
}

#[derive(Debug, Clone)]
pub enum StashPopConflictResolution {
    Overwrite,
}

#[derive(Debug)]
pub struct StashPopConflict {
    pub branch: String,
    pub current_status: WorkingFileStatus,
}

impl std::fmt::Display for StashPopConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "✗ Working file has uncommitted changes (status: {})\n  \
             Restoring stash for branch '{}'\n  \
             Options: [o] overwrite  [x] cancel",
            self.current_status, self.branch
        )
    }
}

impl std::error::Error for StashPopConflict {}

// ── Public API ────────────────────────────────────────────────────────────────

/// Stash the current working file for the current branch.
///
/// Returns `Err(StashExists)` (downcastable) when a stash already exists
/// and `overwrite` is false — the CLI should prompt [o]verwrite / [x]cancel.
pub async fn stash_save(
    ctx: &AppContext,
    description: Option<&str>,
    overwrite: bool,
) -> Result<StashSaveResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let status = resolve_working_file_status(&state, &ctx.project_root);

    match check_state_guard(Command::StashSave, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let branch = current_branch(&ext_dir)?;
    let working_file = ext_core::branch::working_model_path(&branch, &ext_dir);
    let based_on = state
        .working_file
        .as_ref()
        .and_then(|w| w.based_on_version.clone());

    stash::save(
        &branch,
        &working_file,
        &ext_dir,
        description,
        &mut state.stashes,
        based_on.clone(),
        overwrite,
    )
    .with_context(|| "Stash save failed")?;

    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    let stash_path = stash::stash_edb_path(&branch, &ext_dir);
    Ok(StashSaveResult {
        branch,
        based_on,
        stash_path,
    })
}

/// Restore the stash for the current branch.
pub async fn stash_pop(ctx: &AppContext, opts: StashPopOptions) -> Result<StashPopResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let status = resolve_working_file_status(&state, &ctx.project_root);
    let branch = current_branch(&ext_dir)?;
    let working_file = ext_core::branch::working_model_path(&branch, &ext_dir);

    match check_state_guard(Command::StashPop, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }
    if status == ext_core::state::WorkingFileStatus::Modified && opts.conflict_resolution.is_none()
    {
        return Err(anyhow::Error::new(StashPopConflict {
            branch,
            current_status: status,
        }));
    }

    let entry = stash::pop(&branch, &working_file, &ext_dir, &mut state.stashes)
        .with_context(|| "Stash pop failed")?;

    // Update working file state.
    let mtime: Option<chrono::DateTime<Utc>> = std::fs::metadata(&working_file)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(Into::into);

    if let Some(ref mut wf) = state.working_file {
        wf.based_on_version = entry.based_on.clone();
        wf.last_known_mtime = mtime;
        wf.status = ext_core::state::WorkingFileStatus::Modified;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(StashPopResult {
        branch,
        restored_based_on: entry.based_on,
    })
}

/// Drop the stash for the current branch without restoring.
pub async fn stash_drop(ctx: &AppContext, force: bool) -> Result<StashDropResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let branch = current_branch(&ext_dir)?;

    if !state.stashes.contains_key(&branch) && !force {
        bail!("✗ No stash found for branch '{branch}'\n  Nothing to drop");
    }

    stash::drop_stash(&branch, &ext_dir, &mut state.stashes)
        .with_context(|| "Stash drop failed")?;

    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    Ok(StashDropResult { branch })
}

/// List all stashes across all branches.
pub async fn stash_list(ctx: &AppContext) -> Result<StashListResult> {
    let state = ctx.load_state()?;
    let stashes = stash::list(&state.stashes);
    Ok(StashListResult { stashes })
}
