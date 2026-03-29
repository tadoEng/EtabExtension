// ext-api::switch — switch_branch and switch_and_create.
//
// Switch updates git HEAD and repoints state.json at the target branch's
// working file. If ETABS is open, the operation is hard-blocked.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{branch, state::WorkingFileStatus, vcs::git_checkout_branch};
use serde::{Deserialize, Serialize};

use crate::{
    branch::create_branch,
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    status::resolve_working_file_status,
};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchResult {
    pub branch: String,
    pub arrival_status: WorkingFileStatus,
    pub departure_warning: Option<String>,
    pub arrival_warning: Option<String>,
}

// ── Internal helper ───────────────────────────────────────────────────────────

fn current_branch_name(ext_dir: &std::path::Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(ext_dir)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Switch to an existing branch.
///
/// Copies the target branch's working/model.edb to the current working path
/// and updates state.json.
pub async fn switch_branch(ctx: &AppContext, name: &str) -> Result<SwitchResult> {
    let ext_dir = ctx.ext_dir();
    let mut state = ctx.load_state()?;
    let cur_status = resolve_working_file_status(&state, &ctx.project_root);

    // Guard — blocks if ETABS open or orphaned.
    let departure_warning = match check_state_guard(Command::Switch, &cur_status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(msg) => Some(msg),
        GuardOutcome::Allow => None,
    };

    let cur_branch = current_branch_name(&ext_dir);
    if name == cur_branch {
        bail!("✗ Already on branch '{name}'");
    }

    if !branch::exists(name, &ext_dir) {
        bail!("✗ Branch '{name}' not found\n  Run: ext branch to list branches");
    }

    // git checkout
    git_checkout_branch(&ext_dir, name).with_context(|| format!("git checkout '{name}' failed"))?;

    // Update state.json for the target branch.
    let new_working_path = branch::working_model_path(name, &ext_dir);
    let mtime: Option<chrono::DateTime<Utc>> = std::fs::metadata(&new_working_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(Into::into);
    let latest_version = branch::list(&ext_dir, name).ok().and_then(|branches| {
        branches
            .into_iter()
            .find(|b| b.name == name)
            .and_then(|b| b.latest_version)
    });
    let based_on_version = latest_version.or_else(|| {
        branch::read_meta(name, &ext_dir)
            .ok()
            .and_then(|meta| meta.created_from)
            .and_then(|from| from.split_once('/').map(|(_, version)| version.to_string()))
    });

    if let Some(ref mut wf) = state.working_file {
        wf.path = new_working_path.clone();
        wf.based_on_version = based_on_version;
        wf.last_known_mtime = mtime;
        wf.etabs_pid = None;
        wf.status_changed_at = Utc::now();
    }
    let resolved_status = resolve_working_file_status(&state, &ctx.project_root);
    if let Some(ref mut wf) = state.working_file {
        wf.status = resolved_status;
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    // Re-resolve arrival status.
    let state2 = ctx.load_state()?;
    let arrival_status = resolve_working_file_status(&state2, &ctx.project_root);
    let arrival_warning = match arrival_status {
        WorkingFileStatus::Missing => Some("⚠ Working file is missing on this branch".into()),
        WorkingFileStatus::Orphaned => {
            Some("⚠ ETABS was left open on this branch previously".into())
        }
        _ => None,
    };

    Ok(SwitchResult {
        branch: name.to_string(),
        arrival_status,
        departure_warning,
        arrival_warning,
    })
}

/// Create a new branch and switch to it in one operation (`ext switch -c <name>`).
pub async fn switch_and_create(
    ctx: &AppContext,
    name: &str,
    from_ref: Option<&str>,
) -> Result<SwitchResult> {
    // Create the branch first (ext-core + git branch).
    create_branch(ctx, name, from_ref).await?;
    // Then switch to it.
    switch_branch(ctx, name).await
}
