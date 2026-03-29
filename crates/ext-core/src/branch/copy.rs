// ext-core::branch::copy — atomic .edb copy for branch creation.
//
// Separate from fs::atomic_copy so branch-specific pre-checks
// (disk space guard) are co-located with the copy call.

use anyhow::Result;
use std::path::Path;

use crate::fs::{atomic_copy, check_disk_space};

/// Copy `src` to `dst`, checking disk space first.
///
/// Used when creating a new branch to copy a committed snapshot or the
/// current working file into the new branch's working directory.
pub fn branch_copy(src: &Path, dst: &Path) -> Result<()> {
    let dst_parent = dst.parent().unwrap_or(dst);
    check_disk_space(src, dst_parent)?;
    atomic_copy(src, dst)
}
