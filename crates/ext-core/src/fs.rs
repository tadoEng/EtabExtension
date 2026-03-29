// ext-core::fs — atomic file operations used across all domain modules.
//
// All .edb copies use write-to-temp-then-rename to prevent partial writes
// if the process is killed mid-copy.  On the same filesystem, rename() is
// atomic on both Windows (NTFS) and Linux (ext4/tmpfs).
//
// Disk-space checks require a 10% buffer over the source file size so that
// ETABS has room to write temp files during open/analysis.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

// ── Atomic copy ───────────────────────────────────────────────────────────────

/// Copy `src` to `dst` atomically via a `.edb.tmp` staging file.
///
/// On success `dst` contains an exact copy of `src`.
/// On error the staging file is cleaned up (best-effort) and `dst` is unchanged.
pub fn atomic_copy(src: &Path, dst: &Path) -> Result<()> {
    let tmp = dst.with_extension("edb.tmp");

    // Best-effort remove stale .tmp from a previous interrupted copy.
    let _ = std::fs::remove_file(&tmp);

    std::fs::copy(src, &tmp)
        .with_context(|| format!("Failed to copy {} → {}", src.display(), tmp.display()))?;

    std::fs::rename(&tmp, dst).with_context(|| {
        // Rename failed — clean up tmp so it does not accumulate.
        let _ = std::fs::remove_file(&tmp);
        format!("Failed to rename {} → {}", tmp.display(), dst.display())
    })?;

    Ok(())
}

// ── Disk-space guard ──────────────────────────────────────────────────────────

/// Return the number of free bytes on the filesystem that contains `path`.
///
/// Uses platform-specific syscalls via `libc` (Unix) or `windows_sys` (Windows).
/// Falls back to `u64::MAX` (effectively skipping the check) when the query
/// fails so callers on unsupported platforms are not hard-blocked.
fn available_bytes(path: &Path) -> u64 {
    // Probe the parent directory when the path itself doesn't exist yet.
    let probe: PathBuf = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = probe
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut free: u64 = 0;
        let ok = winapi_inner(&mut wide, &mut free);
        if ok { free } else { u64::MAX }
    }

    #[cfg(not(windows))]
    {
        use std::ffi::CString;
        let Ok(c_path) = CString::new(probe.to_string_lossy().as_ref()) else {
            return u64::MAX;
        };
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        // SAFETY: c_path is a valid null-terminated C string; stat is zeroed.
        if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
            (stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64)
        } else {
            u64::MAX
        }
    }
}

#[cfg(windows)]
fn winapi_inner(wide: &mut Vec<u16>, free: &mut u64) -> bool {
    // Call GetDiskFreeSpaceExW through the standard windows crate subset
    // that ships with the Windows SDK.  We use an inline extern block so
    // we don't need to add windows-sys as a direct dependency.
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }
    // SAFETY: wide is null-terminated; free is a valid mutable pointer.
    let ret = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    ret != 0
}

/// Verify that `dst_parent` has enough free space to hold a copy of `src`
/// plus a 10 % buffer for ETABS temp files.
///
/// Returns `Ok(())` when space is sufficient or the query is unavailable.
/// Returns `Err` with a human-readable message when space is provably insufficient.
pub fn check_disk_space(src: &Path, dst_parent: &Path) -> Result<()> {
    let meta = std::fs::metadata(src)
        .with_context(|| format!("Cannot stat source file: {}", src.display()))?;
    let required = meta.len();
    let required_with_buffer = required + required / 10; // 10 % overhead
    let available = available_bytes(dst_parent);

    if available < required_with_buffer {
        let required_mb = required_with_buffer / (1024 * 1024);
        let available_mb = available / (1024 * 1024);
        bail!(
            "✗ Insufficient disk space\n  Need {} MB, have {} MB\n  Check: {}",
            required_mb,
            available_mb,
            dst_parent.display()
        );
    }
    Ok(())
}

// ── Stale temp cleanup ────────────────────────────────────────────────────────

/// Remove any `*.edb.tmp` files found directly inside `dir`.
///
/// Called on startup to clean up temp files left by interrupted copies.
/// Errors on individual files are logged but do not abort the scan.
pub fn cleanup_stale_tmp(dir: &Path) -> Vec<PathBuf> {
    let mut removed = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return removed;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "tmp")
            .unwrap_or(false)
        {
            if std::fs::remove_file(&path).is_ok() {
                removed.push(path);
            }
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
    fn atomic_copy_produces_identical_file() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.edb");
        let dst = tmp.path().join("dst.edb");
        std::fs::write(&src, b"hello edb").unwrap();

        atomic_copy(&src, &dst).unwrap();

        assert!(dst.exists());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello edb");
    }

    #[test]
    fn atomic_copy_no_tmp_left_on_success() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.edb");
        let dst = tmp.path().join("dst.edb");
        std::fs::write(&src, b"data").unwrap();

        atomic_copy(&src, &dst).unwrap();

        assert!(!dst.with_extension("edb.tmp").exists());
    }

    #[test]
    fn cleanup_stale_tmp_removes_tmp_files() {
        let tmp = TempDir::new().unwrap();
        let stale = tmp.path().join("model.edb.tmp");
        std::fs::write(&stale, b"partial").unwrap();

        let removed = cleanup_stale_tmp(tmp.path());

        assert_eq!(removed.len(), 1);
        assert!(!stale.exists());
    }

    #[test]
    fn check_disk_space_ok_for_small_file() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.edb");
        std::fs::write(&src, b"small").unwrap();
        // Should not error — any machine has room for 5 bytes.
        check_disk_space(&src, tmp.path()).unwrap();
    }
}
