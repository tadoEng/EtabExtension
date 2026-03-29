// ext-api::diff — diff two committed versions.

use anyhow::{Context, Result};
use ext_core::vcs::diff_commits;
use serde::{Deserialize, Serialize};

use crate::context::AppContext;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub from_ref: String,
    pub to_ref: String,
    pub diff_text: String,
    /// Set when either version was committed with --no-e2k.
    pub no_e2k_warning: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve a ref like `"main/v3"` or `"v3"` to a git commit hash.
///
/// Looks for the commit that added `<branch>/<version>/manifest.json`.
fn resolve_ref_to_hash(ext_dir: &std::path::Path, branch: &str, version: &str) -> Result<String> {
    // Find the commit that introduced this manifest via `git log --diff-filter=A`.
    let path = format!("{branch}/{version}/manifest.json");
    let branch_ref = format!("refs/heads/{branch}");
    let output = std::process::Command::new("git")
        .args([
            "log",
            &branch_ref,
            "--diff-filter=A",
            "--format=%H",
            "--",
            &path,
        ])
        .current_dir(ext_dir)
        .output()
        .with_context(|| "git log failed")?;

    let hash = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned);

    hash.ok_or_else(|| {
        anyhow::anyhow!(
            "✗ Version '{branch}/{version}' not found in git history\n  \
             Run: ext log to see available versions"
        )
    })
}

fn parse_ref<'a>(r: &'a str, current_branch: &'a str) -> (&'a str, &'a str) {
    if let Some(pos) = r.find('/') {
        (&r[..pos], &r[pos + 1..])
    } else {
        (current_branch, r)
    }
}

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

/// Check whether a version's manifest says e2kGenerated=false.
fn e2k_missing(ext_dir: &std::path::Path, branch: &str, version: &str) -> bool {
    let manifest_path = ext_dir.join(branch).join(version).join("manifest.json");
    let Ok(text) = std::fs::read_to_string(&manifest_path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    json.get("e2kGenerated")
        .and_then(|v| v.as_bool())
        .map(|b| !b)
        .unwrap_or(false)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Diff two committed versions.
///
/// Refs accept: `"v3"`, `"main/v3"`, `"steel-columns/v1"`.
/// Only `.e2k` files are diffed — binary `.edb` files are excluded.
pub async fn diff_versions(ctx: &AppContext, from_ref: &str, to_ref: &str) -> Result<DiffResult> {
    let ext_dir = ctx.ext_dir();
    let cur_branch = current_branch_name(&ext_dir);

    let (from_branch, from_version) = parse_ref(from_ref, &cur_branch);
    let (to_branch, to_version) = parse_ref(to_ref, &cur_branch);

    // Warn if either version has no E2K.
    let mut no_e2k_warning: Option<String> = None;
    let from_missing = e2k_missing(&ext_dir, from_branch, from_version);
    let to_missing = e2k_missing(&ext_dir, to_branch, to_version);
    if from_missing || to_missing {
        let which = match (from_missing, to_missing) {
            (true, true) => format!("{from_ref} and {to_ref}"),
            (true, false) => from_ref.to_string(),
            (false, true) => to_ref.to_string(),
            _ => unreachable!(),
        };
        no_e2k_warning = Some(format!(
            "⚠ No E2K generated for {which}.\n  \
             Re-commit without --no-e2k to enable diff."
        ));
    }

    // Resolve refs to git hashes.
    let from_hash = resolve_ref_to_hash(&ext_dir, from_branch, from_version)
        .with_context(|| format!("Failed to resolve '{from_ref}'"))?;
    let to_hash = resolve_ref_to_hash(&ext_dir, to_branch, to_version)
        .with_context(|| format!("Failed to resolve '{to_ref}'"))?;

    // Diff only *.e2k files.
    let diff_text = diff_commits(&ext_dir, &from_hash, &to_hash, Some(".e2k"))
        .with_context(|| format!("git diff {from_ref}..{to_ref} failed"))?;

    Ok(DiffResult {
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        diff_text,
        no_e2k_warning,
    })
}
