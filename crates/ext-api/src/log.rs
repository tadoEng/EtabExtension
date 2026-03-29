// ext-api::log — list_versions and show_version.

use anyhow::{Context, Result, bail};
use ext_core::{
    vcs::{CommitInfo, current_branch, list_commits},
    version::{VersionManifest, manifest::AnalysisSummary},
};
use serde::{Deserialize, Serialize};

use crate::context::AppContext;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResult {
    pub branch: String,
    pub commits: Vec<CommitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetail {
    pub manifest: VersionManifest,
    pub analysis: Option<AnalysisSummary>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// List committed versions on `branch` (default: current branch).
///
/// `include_internal: false` filters commits whose message starts with `"ext:"`.
pub async fn list_versions(
    ctx: &AppContext,
    branch: Option<&str>,
    include_internal: bool,
) -> Result<ListVersionsResult> {
    let ext_dir = ctx.ext_dir();
    let cur = current_branch(&ext_dir)?;
    let target = branch.unwrap_or(&cur).to_string();

    // Switch git view to the target branch temporarily using git log --branch.
    let commits = list_commits(&ext_dir, &target, include_internal)
        .with_context(|| format!("Failed to read git log for branch '{target}'"))?;

    Ok(ListVersionsResult {
        branch: target,
        commits,
    })
}

/// Show the manifest (and optional analysis summary) for a specific version.
///
/// `version_ref` accepts: `"v3"`, `"main/v3"`, `"steel-columns/v1"`.
pub async fn show_version(ctx: &AppContext, version_ref: &str) -> Result<VersionDetail> {
    let ext_dir = ctx.ext_dir();
    let cur = current_branch(&ext_dir)?;

    let (branch, version) = if let Some((b, v)) = version_ref.split_once('/') {
        (b.to_string(), v.to_string())
    } else {
        (cur, version_ref.to_string())
    };

    let version_dir = ext_dir.join(&branch).join(&version);
    if !version_dir.exists() {
        bail!("✗ Version '{branch}/{version}' not found\n  Run: ext log to see available versions");
    }

    let mut manifest = VersionManifest::read_from(&version_dir)
        .with_context(|| format!("Failed to read manifest for '{branch}/{version}'"))?;
    if let Some(commit) = list_commits(&ext_dir, &branch, false)?
        .into_iter()
        .find(|commit| commit.version_id.as_deref() == Some(version.as_str()))
    {
        manifest.git_commit_hash = Some(commit.hash);
    }

    let analysis = AnalysisSummary::read_from(&version_dir).ok();

    Ok(VersionDetail { manifest, analysis })
}
