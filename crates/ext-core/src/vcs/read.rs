// ext-core::vcs::read — git READ operations via git subprocess.
//
// Phase 1 uses git subprocess for reads (same as writes) for reliability
// and to avoid gix API churn across versions. gix can be introduced later
// for performance-sensitive paths once the API stabilises.
//
// Key design choices:
//   • `next_version_id` reads from git history, NOT from the filesystem.
//     Partial vN/ folders from interrupted commits don't affect the counter.
//   • `list_commits` filters "ext:" prefix by default so ext log never
//     exposes internal plumbing commits to the user.
//   • Version numbers are parsed from manifest.json blobs inside the git tree.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitInfo {
    /// Short 8-char hash.
    pub hash: String,
    /// Commit message (filtered if include_internal=false).
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    /// "v3" parsed from the commit message or None for internal commits.
    pub version_id: Option<String>,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("Failed to spawn git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            anyhow::bail!("git {} failed", args.join(" "));
        }
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Parse "v3" → 3 from a manifest `id` field.
fn parse_version_number(id: &str) -> Option<u32> {
    id.strip_prefix('v')?.parse().ok()
}

fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

fn branch_exists(repo: &Path, branch: &str) -> bool {
    let branch_ref = branch_ref(branch);
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &branch_ref])
        .current_dir(repo)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Try to read a manifest.json blob from a commit tree and extract the `id`.
///
/// Uses `git ls-tree -r` to list all blobs, then `git show` to read any
/// manifest.json found.  Returns None on any error.
fn version_id_from_commit(repo: &Path, branch: &str, commit_hash: &str) -> Option<String> {
    // List all blobs in the commit tree recursively.
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", commit_hash])
        .current_dir(repo)
        .output()
        .ok()?;

    let listing = String::from_utf8_lossy(&output.stdout);

    let mut max_version: Option<u32> = None;

    let branch_prefix = format!("{branch}/");
    for manifest_path in listing
        .lines()
        .filter(|l| l.ends_with("manifest.json"))
        .filter(|l| l.starts_with(&branch_prefix))
    {
        let blob_ref = format!("{commit_hash}:{manifest_path}");
        let content = Command::new("git")
            .args(["show", &blob_ref])
            .current_dir(repo)
            .output()
            .ok()?;

        let json: serde_json::Value = serde_json::from_slice(&content.stdout).ok()?;
        let id = json.get("id")?.as_str()?;
        let version = parse_version_number(id)?;
        max_version = Some(max_version.map_or(version, |current| current.max(version)));
    }

    max_version.map(|version| format!("v{version}"))
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Walk the commit log on the current branch.
///
/// When `include_internal` is `false`, commits whose message starts with
/// `"ext:"` are filtered out — the standard for user-visible `ext log`.
pub fn list_commits(
    repo_dir: &Path,
    branch: &str,
    include_internal: bool,
) -> Result<Vec<CommitInfo>> {
    if !branch_exists(repo_dir, branch) {
        return Ok(vec![]);
    }
    let branch_ref = branch_ref(branch);
    // Format: hash|author|unix-timestamp|message  (one line per commit)
    let raw = run_git(repo_dir, &["log", &branch_ref, "--format=%H|%an|%at|%s"])?;

    if raw.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut commits = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        let hash = parts[0][..8.min(parts[0].len())].to_string();
        let author = parts[1].to_string();
        let ts: i64 = parts[2].parse().unwrap_or(0);
        let message = parts[3].to_string();

        if !include_internal && message.starts_with("ext:") {
            continue;
        }

        let timestamp = DateTime::from_timestamp(ts, 0).unwrap_or_default();
        let version_id = version_id_from_commit(repo_dir, branch, parts[0]);

        commits.push(CommitInfo {
            hash,
            message,
            author,
            timestamp,
            version_id,
        });
    }

    Ok(commits)
}

/// Return the currently checked-out git branch.
///
/// Returns an error when the repository is missing, HEAD is detached, or git
/// does not report a usable branch name.
pub fn current_branch(repo_dir: &Path) -> Result<String> {
    let raw = run_git(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = raw.trim();
    if branch.is_empty() {
        anyhow::bail!("✗ Could not determine current branch");
    }
    if branch == "HEAD" {
        anyhow::bail!("✗ Detached HEAD state\n  Run: ext branch to see all branches");
    }
    Ok(branch.to_string())
}

/// Return the highest version number committed on the current branch.
///
/// Reads manifest.json from each commit's tree to find `id` fields like
/// `"v3"` → 3. Returns 0 when no versions exist yet.
pub fn latest_version_number(repo_dir: &Path, branch: &str) -> Result<u32> {
    if !branch_exists(repo_dir, branch) {
        return Ok(0);
    }
    let branch_ref = branch_ref(branch);
    // Get all commit hashes on the current branch (newest first).
    let raw = run_git(repo_dir, &["log", &branch_ref, "--format=%H"])?;

    if raw.trim().is_empty() {
        return Ok(0);
    }

    let mut max = 0u32;
    for hash in raw.lines() {
        let hash = hash.trim();
        if hash.is_empty() {
            continue;
        }
        if let Some(id_str) = version_id_from_commit(repo_dir, branch, hash)
            && let Some(n) = parse_version_number(&id_str) {
                max = max.max(n);
            }
    }

    Ok(max)
}

/// Return the next version id, e.g. `"v4"` when latest is `v3`.
pub fn next_version_id(repo_dir: &Path, branch: &str) -> Result<String> {
    Ok(format!("v{}", latest_version_number(repo_dir, branch)? + 1))
}

/// Read the raw text content of a file at a specific commit.
///
/// `file_path` is relative to the repo root (e.g. `"main/v3/manifest.json"`).
pub fn read_blob(repo_dir: &Path, commit_hash: &str, file_path: &str) -> Result<String> {
    let blob_ref = format!("{commit_hash}:{file_path}");
    let output = Command::new("git")
        .args(["show", &blob_ref])
        .current_dir(repo_dir)
        .output()
        .with_context(|| format!("git show {blob_ref}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "✗ Path not found in commit: {file_path}\n  {}",
            stderr.trim()
        );
    }

    String::from_utf8(output.stdout)
        .with_context(|| format!("File is not valid UTF-8: {file_path}"))
}

/// Return a unified diff between two commits as a plain string.
///
/// `path_filter` — if `Some("model.e2k")` only diffs files whose name ends
/// with that suffix.
pub fn diff_commits(
    repo_dir: &Path,
    from_hash: &str,
    to_hash: &str,
    path_filter: Option<&str>,
) -> Result<String> {
    let mut args = vec!["diff", from_hash, to_hash];

    let filter_owned;
    if let Some(filter) = path_filter {
        args.push("--");
        filter_owned = format!("*{filter}");
        args.push(&filter_owned);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_dir)
        .output()
        .with_context(|| "Failed to run git diff")?;

    // git diff exits 1 when there are differences — that is expected.
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_repo(dir: &Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn write_and_commit(repo: &Path, file: &str, content: &str, msg: &str) {
        if let Some(parent) = Path::new(file).parent()
            && !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(repo.join(parent)).unwrap();
            }
        std::fs::write(repo.join(file), content).unwrap();
        Command::new("git")
            .args(["add", file])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(repo)
            .output()
            .unwrap();
    }

    #[test]
    fn next_version_id_returns_v1_on_empty_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        assert_eq!(next_version_id(tmp.path(), "main").unwrap(), "v1");
    }

    #[test]
    fn next_version_id_v1_with_no_manifests() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);
        write_and_commit(repo, "readme.txt", "hi", "init");
        // No manifest.json → latest = 0 → next = v1
        assert_eq!(next_version_id(repo, "main").unwrap(), "v1");
    }

    #[test]
    fn latest_version_number_increases_with_commits() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);

        write_and_commit(
            repo,
            "main/v1/manifest.json",
            r#"{"id":"v1","branch":"main","message":"a","author":"A","timestamp":"2026-01-01T00:00:00Z","isAnalyzed":false,"e2kGenerated":true,"materialsExtracted":false,"edbSizeBytes":1}"#,
            "v1 commit",
        );
        // Internal commit — counter must still read v1 correctly
        write_and_commit(
            repo,
            "main/v1/summary.json",
            "{}",
            "ext: analysis results v1",
        );

        write_and_commit(
            repo,
            "main/v2/manifest.json",
            r#"{"id":"v2","branch":"main","message":"b","author":"A","timestamp":"2026-01-02T00:00:00Z","isAnalyzed":false,"e2kGenerated":true,"materialsExtracted":false,"edbSizeBytes":1}"#,
            "v2 commit",
        );

        assert_eq!(latest_version_number(repo, "main").unwrap(), 2);
        assert_eq!(next_version_id(repo, "main").unwrap(), "v3");
    }

    #[test]
    fn current_branch_reads_active_branch() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);
        write_and_commit(repo, "readme.txt", "hi", "init");

        assert_eq!(current_branch(repo).unwrap(), "main");

        Command::new("git")
            .args(["checkout", "-b", "alt"])
            .current_dir(repo)
            .output()
            .unwrap();

        assert_eq!(current_branch(repo).unwrap(), "alt");
    }

    #[test]
    fn current_branch_errors_for_detached_head() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);
        write_and_commit(repo, "readme.txt", "hi", "init");

        Command::new("git")
            .args(["checkout", "HEAD~0"])
            .current_dir(repo)
            .output()
            .unwrap();

        let err = current_branch(repo).unwrap_err();
        assert!(err.to_string().contains("Detached HEAD"));
    }

    #[test]
    fn current_branch_errors_outside_git_repo() {
        let tmp = TempDir::new().unwrap();
        let err = current_branch(tmp.path()).unwrap_err();
        assert!(!err.to_string().contains("main"));
    }

    #[test]
    fn list_commits_filters_internal() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);

        write_and_commit(repo, "a.txt", "user", "user commit");
        write_and_commit(repo, "b.txt", "int", "ext: internal commit");

        let visible = list_commits(repo, "main", false).unwrap();
        let all = list_commits(repo, "main", true).unwrap();

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].message, "user commit");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn next_version_id_starts_at_v1_for_new_branch() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);

        write_and_commit(
            repo,
            "main/v1/manifest.json",
            r#"{"id":"v1","branch":"main","message":"a","author":"A","timestamp":"2026-01-01T00:00:00Z","isAnalyzed":false,"e2kGenerated":true,"materialsExtracted":false,"edbSizeBytes":1}"#,
            "v1 commit",
        );

        Command::new("git")
            .args(["checkout", "-b", "steel-columns"])
            .current_dir(repo)
            .output()
            .unwrap();

        assert_eq!(latest_version_number(repo, "steel-columns").unwrap(), 0);
        assert_eq!(next_version_id(repo, "steel-columns").unwrap(), "v1");
    }
}
