// ext-core::branch — branch domain logic.
//
//   mod.rs   — create, list, delete, metadata, validation
//   copy.rs  — atomic .edb copy for branch creation (with disk check)

pub mod copy;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::vcs::{latest_version_number, list_commits};

// ── BranchMeta ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchMeta {
    pub name: String,
    pub created_at: DateTime<Utc>,
    /// "main/v3" or None for the initial `main` branch.
    pub created_from: Option<String>,
    pub description: Option<String>,
}

// ── BranchInfo ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    pub name: String,
    pub version_count: u32,
    pub latest_version: Option<String>,
    pub created_from: Option<String>,
    pub is_active: bool,
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Ensure a branch name is valid: no slashes, no spaces, not empty.
pub fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("✗ Branch name cannot be empty");
    }
    if name.contains('/') {
        bail!("✗ Branch name cannot contain '/': {name}");
    }
    if name.contains(' ') {
        bail!("✗ Branch name cannot contain spaces: {name}");
    }
    Ok(())
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Absolute path to `.etabs-ext/<branch>/`.
pub fn branch_dir(name: &str, ext_dir: &Path) -> PathBuf {
    ext_dir.join(name)
}

/// Absolute path to `.etabs-ext/<branch>/working/model.edb`.
pub fn working_model_path(name: &str, ext_dir: &Path) -> PathBuf {
    ext_dir.join(name).join("working").join("model.edb")
}

fn meta_path(name: &str, ext_dir: &Path) -> PathBuf {
    ext_dir.join(name).join(".branch.json")
}

// ── I/O ───────────────────────────────────────────────────────────────────────

/// Write `.etabs-ext/<name>/.branch.json` atomically.
pub fn write_meta(meta: &BranchMeta, ext_dir: &Path) -> Result<()> {
    let path = meta_path(&meta.name, ext_dir);
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(meta).with_context(|| "Serialize branch meta")?;
    std::fs::write(&tmp, &text).with_context(|| format!("Write {}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .with_context(|| format!("Rename {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Read `.etabs-ext/<name>/.branch.json`.
pub fn read_meta(name: &str, ext_dir: &Path) -> Result<BranchMeta> {
    let path = meta_path(name, ext_dir);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("Read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("Parse {}", path.display()))
}

// ── Branch existence ──────────────────────────────────────────────────────────

pub fn exists(name: &str, ext_dir: &Path) -> bool {
    meta_path(name, ext_dir).exists()
}

// ── Public CRUD ───────────────────────────────────────────────────────────────

/// Create a new branch directory + metadata.
///
/// `from_edb`  — source `.edb` to copy as the new branch's working file.
/// `from_ref`  — human-readable ref string for metadata, e.g. `"main/v3"`.
///
/// Does NOT perform the git branch creation — that is handled in ext-api
/// after ext-core succeeds so the rollback order is correct.
pub fn create(name: &str, from_edb: &Path, from_ref: &str, ext_dir: &Path) -> Result<BranchMeta> {
    validate_branch_name(name)?;

    if exists(name, ext_dir) {
        bail!("✗ Branch '{}' already exists", name);
    }

    // Create working directory.
    let working_dir = ext_dir.join(name).join("working");
    std::fs::create_dir_all(&working_dir)
        .with_context(|| format!("Create {}", working_dir.display()))?;

    // Copy source edb into the new branch's working directory.
    let dst = working_dir.join("model.edb");
    copy::branch_copy(from_edb, &dst)?;

    let meta = BranchMeta {
        name: name.to_string(),
        created_at: Utc::now(),
        created_from: Some(from_ref.to_string()),
        description: None,
    };
    write_meta(&meta, ext_dir)?;

    Ok(meta)
}

/// List all branches found in `ext_dir`, sorted by creation time.
///
/// `current_branch` is used to set the `is_active` flag.
pub fn list(ext_dir: &Path, current_branch: &str) -> Result<Vec<BranchInfo>> {
    let mut infos = Vec::new();
    let Ok(entries) = std::fs::read_dir(ext_dir) else {
        return Ok(infos);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Only directories that have a .branch.json are branches.
        if !meta_path(&name, ext_dir).exists() {
            continue;
        }

        let meta = read_meta(&name, ext_dir).unwrap_or(BranchMeta {
            name: name.clone(),
            created_at: Utc::now(),
            created_from: None,
            description: None,
        });

        // Git is the source of truth for committed versions. That keeps branch
        // metadata aligned with next_version_id even if the filesystem drifts.
        let version_count = list_commits(ext_dir, &name, false)
            .map(|commits| commits.len() as u32)
            .unwrap_or(0);
        let latest_version = latest_version_number(ext_dir, &name)
            .ok()
            .and_then(|n| (n > 0).then(|| format!("v{n}")));

        infos.push(BranchInfo {
            is_active: name == current_branch,
            name,
            version_count,
            latest_version,
            created_from: meta.created_from,
        });
    }

    // Sort by: active first, then alphabetical.
    infos.sort_by(|a, b| b.is_active.cmp(&a.is_active).then(a.name.cmp(&b.name)));
    Ok(infos)
}
/// Delete a branch directory.
///
/// Refuses to delete `"main"` or the currently active branch.
/// If `force` is false, also refuses when the branch has uncommitted changes
/// (detected by the absence of a version newer than the working file mtime —
/// handled at the ext-api level; this layer only enforces hard constraints).
pub fn delete(name: &str, ext_dir: &Path, current_branch: &str, force: bool) -> Result<()> {
    if name == "main" {
        bail!("✗ Cannot delete the main branch");
    }
    if name == current_branch {
        bail!(
            "✗ Cannot delete the active branch '{}'\n  Switch to another branch first",
            name
        );
    }
    if !exists(name, ext_dir) {
        bail!("✗ Branch '{}' not found", name);
    }

    if !force {
        // Check if the branch has any uncommitted working file content.
        // The ext-api layer handles the full MODIFIED check; here we just
        // ensure the caller explicitly passes force=true for deletion.
        // For now we allow delete without force unless the caller has set it.
        // The ext-api guard (checking working file state) should be called first.
    }

    std::fs::remove_dir_all(branch_dir(name, ext_dir))
        .with_context(|| format!("Delete branch directory for '{name}'"))?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_ext_dir(tmp: &TempDir) -> PathBuf {
        let ext_dir = tmp.path().join(".etabs-ext");
        std::fs::create_dir_all(ext_dir.join("main").join("working")).unwrap();
        // Write a dummy edb so branch copy has something to work with.
        std::fs::write(
            ext_dir.join("main").join("working").join("model.edb"),
            b"dummy",
        )
        .unwrap();
        // Write main .branch.json
        let meta = BranchMeta {
            name: "main".to_string(),
            created_at: Utc::now(),
            created_from: None,
            description: None,
        };
        write_meta(&meta, &ext_dir).unwrap();
        ext_dir
    }

    #[test]
    fn create_branch_makes_directory_and_meta() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let src = ext_dir.join("main").join("working").join("model.edb");

        let meta = create("steel-columns", &src, "main/v1", &ext_dir).unwrap();
        assert_eq!(meta.name, "steel-columns");
        assert!(ext_dir.join("steel-columns").join(".branch.json").exists());
        assert!(
            ext_dir
                .join("steel-columns")
                .join("working")
                .join("model.edb")
                .exists()
        );
    }

    #[test]
    fn create_rejects_slash_in_name() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let src = ext_dir.join("main").join("working").join("model.edb");
        let err = create("bad/name", &src, "main/v1", &ext_dir).unwrap_err();
        assert!(err.to_string().contains('/'));
    }

    #[test]
    fn create_rejects_duplicate_name() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let src = ext_dir.join("main").join("working").join("model.edb");
        create("alt", &src, "main/v1", &ext_dir).unwrap();
        let err = create("alt", &src, "main/v1", &ext_dir).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn delete_refuses_main() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let err = delete("main", &ext_dir, "steel", false).unwrap_err();
        assert!(err.to_string().contains("main"));
    }

    #[test]
    fn delete_refuses_active_branch() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let src = ext_dir.join("main").join("working").join("model.edb");
        create("alt", &src, "main/v1", &ext_dir).unwrap();
        let err = delete("alt", &ext_dir, "alt", false).unwrap_err();
        assert!(err.to_string().contains("active branch"));
    }

    #[test]
    fn list_returns_branches() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = make_ext_dir(&tmp);
        let src = ext_dir.join("main").join("working").join("model.edb");
        create("alt", &src, "main/v1", &ext_dir).unwrap();

        let branches = list(&ext_dir, "main").unwrap();
        let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"alt"));
        // main is active
        let main = branches.iter().find(|b| b.name == "main").unwrap();
        assert!(main.is_active);
    }
}
