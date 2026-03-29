// ext-api::branch — create, list, delete branch operations.

use anyhow::{Context, Result, bail};
use ext_core::branch::{self, BranchInfo, BranchMeta};
use ext_core::vcs::{current_branch, git_create_branch, git_delete_branch, next_version_id};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::context::AppContext;

// ── Result types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBranchResult {
    pub name: String,
    pub created_from: String,
    pub working_model_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBranchesResult {
    pub branches: Vec<BranchInfo>,
    pub current_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBranchResult {
    pub name: String,
}

/// Create a new branch from an existing committed version.
///
/// `from_ref` — optional `"<branch>/vN"` or `"vN"` (relative to current branch).
/// When None, uses the latest committed version of the current branch.
pub async fn create_branch(
    ctx: &AppContext,
    name: &str,
    from_ref: Option<&str>,
) -> Result<CreateBranchResult> {
    let ext_dir = ctx.ext_dir();
    let cur_branch = current_branch(&ext_dir)?;

    // Resolve the source branch and version.
    let (source_branch, source_version) = if let Some(r) = from_ref {
        if let Some((b, v)) = r.split_once('/') {
            (b.to_string(), v.to_string())
        } else {
            // Just a version id — use current branch.
            (cur_branch.clone(), r.to_string())
        }
    } else {
        // Latest version on current branch.
        let latest = next_version_id(&ext_dir, &cur_branch)?;
        // next_version_id returns vN+1, so the latest committed is vN-1.
        // Walk back to find the actual latest committed version dir.
        let latest_n: u32 = latest
            .strip_prefix('v')
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        if latest_n == 1 {
            bail!(
                "✗ No committed versions on branch '{cur_branch}'\n  Run: ext commit \"message\" first"
            );
        }
        (cur_branch.clone(), format!("v{}", latest_n - 1))
    };

    let from_version_dir = ext_dir.join(&source_branch).join(&source_version);
    if !from_version_dir.exists() {
        bail!(
            "✗ Version '{source_branch}/{source_version}' not found\n  Run: ext log to see available versions"
        );
    }
    let source_edb = from_version_dir.join("model.edb");
    if !source_edb.exists() {
        bail!("✗ model.edb missing in {}", from_version_dir.display());
    }

    let from_ref_str = format!("{source_branch}/{source_version}");

    // ext-core creates the directory structure and copies the edb.
    let meta: BranchMeta = branch::create(name, &source_edb, &from_ref_str, &ext_dir)
        .with_context(|| format!("Failed to create branch '{name}'"))?;

    // git branch creation (after ext-core succeeds so rollback order is correct).
    git_create_branch(&ext_dir, name)
        .with_context(|| format!("Failed to create git branch '{name}'"))?;

    let working_model = branch::working_model_path(name, &ext_dir);

    Ok(CreateBranchResult {
        name: meta.name,
        created_from: from_ref_str,
        working_model_path: working_model,
    })
}

/// List all branches in the project.
pub async fn list_branches(ctx: &AppContext) -> Result<ListBranchesResult> {
    let ext_dir = ctx.ext_dir();
    let cur = current_branch(&ext_dir)?;
    let branches = branch::list(&ext_dir, &cur).with_context(|| "Failed to list branches")?;
    Ok(ListBranchesResult {
        branches,
        current_branch: cur,
    })
}

/// Delete a branch by name.
pub async fn delete_branch(
    ctx: &AppContext,
    name: &str,
    force: bool,
) -> Result<DeleteBranchResult> {
    let ext_dir = ctx.ext_dir();
    let cur = current_branch(&ext_dir)?;

    branch::delete(name, &ext_dir, &cur, force)
        .with_context(|| format!("Failed to delete branch '{name}'"))?;

    // Git branch deletion — after ext-core succeeds.
    git_delete_branch(&ext_dir, name, force)
        .with_context(|| format!("Failed to delete git branch '{name}'"))?;

    Ok(DeleteBranchResult {
        name: name.to_string(),
    })
}
