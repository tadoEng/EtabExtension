// ext-core::version::snapshot — .partial sentinel and RAII rollback guard.
//
// Every in-progress vN/ folder is marked with a `.partial` file at creation.
// The marker is deleted only when the entire commit sequence succeeds.
//
// This covers two failure modes:
//   1. Runtime error     — PartialGuard fires on Drop, deletes the folder.
//   2. Process kill      — cleanup_partial_snapshots() called on next startup.
//
// The two mechanisms are complementary: (1) handles clean errors, (2) handles
// SIGKILL / power loss.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

// ── PartialGuard ──────────────────────────────────────────────────────────────

/// RAII guard that deletes `version_dir` when dropped unless disarmed.
///
/// Created by `begin_snapshot()`.  Disarmed by `complete_snapshot()`.
pub struct PartialGuard {
    version_dir: PathBuf,
    armed: bool,
}

impl PartialGuard {
    fn new(version_dir: PathBuf) -> Self {
        Self {
            version_dir,
            armed: true,
        }
    }

    /// Disarm the guard so the version directory is NOT deleted on drop.
    pub fn disarm(&mut self) {
        self.armed = false;
    }

    pub fn version_dir(&self) -> &Path {
        &self.version_dir
    }
}

impl Drop for PartialGuard {
    fn drop(&mut self) {
        if self.armed {
            // Best-effort: log but never panic in Drop.
            if let Err(e) = std::fs::remove_dir_all(&self.version_dir) {
                tracing::warn!(
                    "PartialGuard: failed to clean up {}: {e}",
                    self.version_dir.display()
                );
            }
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Create `version_dir/` and write the `.partial` sentinel inside it.
///
/// Returns a `PartialGuard` that will delete the directory on drop unless
/// `complete_snapshot()` is called first.
pub fn begin_snapshot(version_dir: &Path) -> Result<PartialGuard> {
    std::fs::create_dir_all(version_dir)
        .with_context(|| format!("Create version dir {}", version_dir.display()))?;

    std::fs::write(version_dir.join(".partial"), b"")
        .with_context(|| format!("Write .partial in {}", version_dir.display()))?;

    Ok(PartialGuard::new(version_dir.to_path_buf()))
}

/// Delete the `.partial` sentinel and disarm the guard.
///
/// Must be called as the last step of a successful commit sequence.
pub fn complete_snapshot(mut guard: PartialGuard) -> Result<()> {
    let partial = guard.version_dir.join(".partial");
    std::fs::remove_file(&partial)
        .with_context(|| format!("Remove .partial from {}", partial.display()))?;
    guard.disarm();
    Ok(())
}

/// Scan `branch_dir` for vN/ subfolders that contain `.partial` and delete them.
///
/// Returns the list of directories that were cleaned up.
/// Called once during AppContext construction so stale partial snapshots from
/// a previous crash are removed before any new operation begins.
pub fn cleanup_partial_snapshots(branch_dir: &Path) -> Vec<PathBuf> {
    let mut removed = Vec::new();
    let Ok(entries) = std::fs::read_dir(branch_dir) else {
        return removed;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join(".partial").exists()
            && std::fs::remove_dir_all(&path).is_ok() {
                removed.push(path);
            }
    }
    removed
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn begin_snapshot_creates_partial_file() {
        let tmp = TempDir::new().unwrap();
        let vdir = tmp.path().join("v1");
        let _guard = begin_snapshot(&vdir).unwrap();
        assert!(vdir.join(".partial").exists());
    }

    #[test]
    fn complete_snapshot_removes_partial_and_disarms() {
        let tmp = TempDir::new().unwrap();
        let vdir = tmp.path().join("v1");
        let guard = begin_snapshot(&vdir).unwrap();
        complete_snapshot(guard).unwrap();
        // .partial is gone
        assert!(!vdir.join(".partial").exists());
        // directory itself still exists
        assert!(vdir.exists());
    }

    #[test]
    fn partial_guard_rollback_on_drop() {
        let tmp = TempDir::new().unwrap();
        let vdir = tmp.path().join("v1");
        {
            let _guard = begin_snapshot(&vdir).unwrap();
            // guard dropped here without disarming
        }
        assert!(!vdir.exists(), "guard should have deleted the version dir");
    }

    #[test]
    fn cleanup_partial_snapshots_removes_partial_dirs() {
        let tmp = TempDir::new().unwrap();
        // create a partial v1 folder
        let v1 = tmp.path().join("v1");
        std::fs::create_dir(&v1).unwrap();
        std::fs::write(v1.join(".partial"), b"").unwrap();
        // create a complete v2 folder (no .partial)
        let v2 = tmp.path().join("v2");
        std::fs::create_dir(&v2).unwrap();

        let removed = cleanup_partial_snapshots(tmp.path());

        assert_eq!(removed.len(), 1);
        assert!(!v1.exists());
        assert!(v2.exists());
    }

    #[test]
    fn cleanup_partial_snapshots_empty_dir_is_fine() {
        let tmp = TempDir::new().unwrap();
        let removed = cleanup_partial_snapshots(tmp.path());
        assert!(removed.is_empty());
    }
}
