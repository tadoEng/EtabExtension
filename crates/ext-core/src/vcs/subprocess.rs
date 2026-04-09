// ext-core::vcs::subprocess — git WRITE operations via std::process::Command.
//
// Every function in this module is synchronous and calls the `git` binary as a
// child process.  We never use git2 or gix for writes — the subprocess approach
// is simpler, more reliable across platform edge-cases, and easier to reason
// about for operations that touch the index or HEAD.
//
// All functions take `repo: &Path` which is the `.etabs-ext/` directory (the
// git repo root, NOT the project root).
//
// Error messages always include the failing git invocation and stderr so the
// user can diagnose config problems.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

// ── Internal helper ───────────────────────────────────────────────────────────

fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("Failed to spawn git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            bail!("git {} failed (exit {})", args.join(" "), output.status);
        }
        bail!(
            "git {} failed (exit {}): {}",
            args.join(" "),
            output.status,
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Stage one or more files relative to `repo`.
///
/// Paths should be relative to `repo` (the `.etabs-ext/` directory).
pub fn git_add(repo: &Path, paths: &[&Path]) -> Result<()> {
    let path_strs: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    // ext stages explicit generated artifacts, including versioned parquet files
    // that may still match broad ignore patterns in older repos.
    let mut args = vec!["add", "-f", "--"];
    let borrowed: Vec<&str> = path_strs.iter().map(|s| s.as_str()).collect();
    args.extend_from_slice(&borrowed);
    run_git(repo, &args)?;
    Ok(())
}

/// Create a commit with `message` authored by `author <email>`.
///
/// Returns the short (8-char) commit hash on success.
pub fn git_commit(repo: &Path, message: &str, author: &str, email: &str) -> Result<String> {
    let author_str = format!("{author} <{email}>");
    run_git(repo, &["commit", "--author", &author_str, "-m", message])?;
    // Read the hash of the commit we just created.
    let hash = run_git(repo, &["rev-parse", "--short=8", "HEAD"])?;
    Ok(hash)
}

/// Amend the last commit in place without changing its message.
///
/// Returns the new short (8-char) commit hash on success.
pub fn git_amend_no_edit(repo: &Path, author: &str, email: &str) -> Result<String> {
    let author_str = format!("{author} <{email}>");
    run_git(
        repo,
        &["commit", "--amend", "--no-edit", "--author", &author_str],
    )?;
    let hash = run_git(repo, &["rev-parse", "--short=8", "HEAD"])?;
    Ok(hash)
}

/// Create a new git branch at the current HEAD without switching to it.
pub fn git_create_branch(repo: &Path, name: &str) -> Result<()> {
    run_git(repo, &["branch", name])?;
    Ok(())
}

/// Check out (switch to) an existing git branch.
pub fn git_checkout_branch(repo: &Path, name: &str) -> Result<()> {
    run_git(repo, &["checkout", name])?;
    Ok(())
}

/// Delete a git branch (`-d` — refuses if not fully merged).
/// Use `--force` at the caller level to pass `-D` when needed.
pub fn git_delete_branch(repo: &Path, name: &str, force: bool) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    run_git(repo, &["branch", flag, name])?;
    Ok(())
}

/// Set a git config key inside `repo`.
pub fn git_config(repo: &Path, key: &str, value: &str) -> Result<()> {
    run_git(repo, &["config", key, value])?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_repo(dir: &Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn make_initial_commit(repo: &Path) {
        std::fs::write(repo.join("init.txt"), "init").unwrap();
        Command::new("git")
            .args(["add", "init.txt"])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(repo)
            .output()
            .unwrap();
    }

    #[test]
    fn git_add_and_commit_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);

        let file = repo.join("manifest.json");
        std::fs::write(&file, r#"{"id":"v1"}"#).unwrap();
        git_add(repo, &[Path::new("manifest.json")]).unwrap();
        let hash = git_commit(repo, "v1 commit", "Alice", "alice@example.com").unwrap();
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn git_amend_updates_hash_without_changing_message() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);

        let file = repo.join("manifest.json");
        std::fs::write(&file, r#"{"id":"v1"}"#).unwrap();
        git_add(repo, &[Path::new("manifest.json")]).unwrap();
        let hash_before = git_commit(repo, "v1 commit", "Alice", "alice@example.com").unwrap();

        std::fs::write(&file, r#"{"id":"v1","gitCommitHash":"pending"}"#).unwrap();
        git_add(repo, &[Path::new("manifest.json")]).unwrap();
        let hash_after = git_amend_no_edit(repo, "Alice", "alice@example.com").unwrap();

        assert_ne!(hash_before, hash_after);
        let message = run_git(repo, &["log", "-1", "--format=%s"]).unwrap();
        assert_eq!(message, "v1 commit");
    }

    #[test]
    fn git_create_and_delete_branch() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);
        make_initial_commit(repo);

        git_create_branch(repo, "feature").unwrap();
        git_delete_branch(repo, "feature", false).unwrap();
    }

    #[test]
    fn git_checkout_branch_switches() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path();
        init_repo(repo);
        make_initial_commit(repo);

        git_create_branch(repo, "alt").unwrap();
        git_checkout_branch(repo, "alt").unwrap();

        let head = run_git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap();
        assert_eq!(head, "alt");
    }
}
