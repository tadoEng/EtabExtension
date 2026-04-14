// ext-core::stash — one stash slot per branch.
//
// Stash metadata lives in state.json (StateFile.stashes HashMap).
// The .edb binary lives at .etabs-ext/stash/<branch>.edb on disk.
//
// The caller (ext-api) is responsible for:
//   • ETABS running check before any stash operation.
//   • Saving state.json after the stash operation mutates it.

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::fs::{atomic_copy, check_disk_space};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    /// Version the working file was based on at stash time.
    pub based_on: Option<String>,
    pub stashed_at: DateTime<Utc>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashListEntry {
    pub branch: String,
    pub based_on: Option<String>,
    pub stashed_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// Returned by `save()` when a stash already exists for the branch.
#[derive(Debug)]
pub struct StashExists {
    pub branch: String,
    pub description: Option<String>,
    pub stashed_at: DateTime<Utc>,
}

impl std::fmt::Display for StashExists {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "✗ Stash already exists for branch '{}'\n  \
             Description: {}\n  \
             Stashed at: {}\n  \
             Use --overwrite to replace it",
            self.branch,
            self.description.as_deref().unwrap_or("(none)"),
            self.stashed_at.format("%Y-%m-%d %H:%M:%S UTC"),
        )
    }
}

impl std::error::Error for StashExists {}

// ── Path helper ───────────────────────────────────────────────────────────────

pub fn stash_edb_path(branch: &str, ext_dir: &Path) -> PathBuf {
    ext_dir.join("stash").join(format!("{branch}.edb"))
}

// ── Public operations ─────────────────────────────────────────────────────────

/// Save the working file into the stash slot for `branch`.
///
/// Returns `Err(StashExists)` (as an anyhow error) if a stash already exists
/// and `overwrite` is false — the caller should surface the overwrite prompt.
///
/// Does NOT save state.json.  The caller must call `state.save()` after.
pub fn save(
    branch: &str,
    working_file: &Path,
    ext_dir: &Path,
    description: Option<&str>,
    stashes: &mut std::collections::HashMap<String, StashEntry>,
    based_on: Option<String>,
    overwrite: bool,
) -> Result<()> {
    if let Some(existing) = stashes.get(branch)
        && !overwrite {
            return Err(anyhow::Error::new(StashExists {
                branch: branch.to_string(),
                description: existing.description.clone(),
                stashed_at: existing.stashed_at,
            }));
        }

    let stash_dir = ext_dir.join("stash");
    std::fs::create_dir_all(&stash_dir)
        .with_context(|| format!("Create stash dir {}", stash_dir.display()))?;

    let dst = stash_edb_path(branch, ext_dir);
    check_disk_space(working_file, &stash_dir)?;
    atomic_copy(working_file, &dst)?;

    stashes.insert(
        branch.to_string(),
        StashEntry {
            based_on,
            stashed_at: Utc::now(),
            description: description.map(str::to_owned),
        },
    );

    Ok(())
}

/// Restore the stash for `branch` into `working_file`.
///
/// Returns the `StashEntry` so the caller can update state.json.
/// Removes the stash entry from `stashes` on success.
/// Does NOT save state.json.
pub fn pop(
    branch: &str,
    working_file: &Path,
    ext_dir: &Path,
    stashes: &mut std::collections::HashMap<String, StashEntry>,
) -> Result<StashEntry> {
    let entry = stashes
        .remove(branch)
        .ok_or_else(|| anyhow::anyhow!("✗ No stash found for branch '{branch}'"))?;

    let src = stash_edb_path(branch, ext_dir);
    if !src.exists() {
        bail!(
            "✗ Stash file missing: {}\n  The stash metadata exists but the .edb was deleted",
            src.display()
        );
    }

    check_disk_space(&src, working_file.parent().unwrap_or(working_file))?;
    atomic_copy(&src, working_file)?;

    // Remove the stash file after successful restore.
    let _ = std::fs::remove_file(&src);

    Ok(entry)
}

/// Drop the stash for `branch` without restoring.
///
/// Does NOT save state.json.
pub fn drop_stash(
    branch: &str,
    ext_dir: &Path,
    stashes: &mut std::collections::HashMap<String, StashEntry>,
) -> Result<()> {
    if stashes.remove(branch).is_none() {
        bail!("✗ No stash found for branch '{branch}'");
    }
    let path = stash_edb_path(branch, ext_dir);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Delete stash file {}", path.display()))?;
    }
    Ok(())
}

/// Build a list of stash entries for display.
pub fn list(stashes: &std::collections::HashMap<String, StashEntry>) -> Vec<StashListEntry> {
    let mut entries: Vec<StashListEntry> = stashes
        .iter()
        .map(|(branch, e)| StashListEntry {
            branch: branch.clone(),
            based_on: e.based_on.clone(),
            stashed_at: e.stashed_at,
            description: e.description.clone(),
        })
        .collect();
    entries.sort_by(|a, b| b.stashed_at.cmp(&a.stashed_at));
    entries
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup(tmp: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
        let ext_dir = tmp.path().join(".etabs-ext");
        std::fs::create_dir_all(ext_dir.join("main").join("working")).unwrap();
        let working = ext_dir.join("main").join("working").join("model.edb");
        std::fs::write(&working, b"model content").unwrap();
        let ext_dir_clone = ext_dir.clone();
        (ext_dir, working, ext_dir_clone)
    }

    #[test]
    fn save_creates_stash_file_and_entry() {
        let tmp = TempDir::new().unwrap();
        let (ext_dir, working, _) = setup(&tmp);
        let mut stashes = HashMap::new();

        save(
            "main",
            &working,
            &ext_dir,
            Some("test stash"),
            &mut stashes,
            Some("v1".to_string()),
            false,
        )
        .unwrap();

        assert!(stash_edb_path("main", &ext_dir).exists());
        assert!(stashes.contains_key("main"));
    }

    #[test]
    fn save_returns_stash_exists_error() {
        let tmp = TempDir::new().unwrap();
        let (ext_dir, working, _) = setup(&tmp);
        let mut stashes = HashMap::new();

        save("main", &working, &ext_dir, None, &mut stashes, None, false).unwrap();
        let err = save("main", &working, &ext_dir, None, &mut stashes, None, false).unwrap_err();

        assert!(err.is::<StashExists>());
    }

    #[test]
    fn pop_restores_file_and_removes_entry() {
        let tmp = TempDir::new().unwrap();
        let (ext_dir, working, _) = setup(&tmp);
        let mut stashes = HashMap::new();

        save(
            "main",
            &working,
            &ext_dir,
            None,
            &mut stashes,
            Some("v1".to_string()),
            false,
        )
        .unwrap();

        // Overwrite working file to simulate changes after stash.
        std::fs::write(&working, b"changed").unwrap();

        let entry = pop("main", &working, &ext_dir, &mut stashes).unwrap();
        assert_eq!(entry.based_on.as_deref(), Some("v1"));
        assert!(!stashes.contains_key("main"));
        assert_eq!(std::fs::read(&working).unwrap(), b"model content");
    }

    #[test]
    fn drop_removes_entry_and_file() {
        let tmp = TempDir::new().unwrap();
        let (ext_dir, working, _) = setup(&tmp);
        let mut stashes = HashMap::new();

        save("main", &working, &ext_dir, None, &mut stashes, None, false).unwrap();
        drop_stash("main", &ext_dir, &mut stashes).unwrap();

        assert!(!stashes.contains_key("main"));
        assert!(!stash_edb_path("main", &ext_dir).exists());
    }
}
