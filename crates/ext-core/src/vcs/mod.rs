// ext-core::vcs — git version-control operations.
//
// Split into two files with distinct concerns:
//   subprocess.rs — all git WRITE ops via std::process::Command
//   read.rs       — all git READ ops via std::process::Command (git subprocess)
//
// Phase 1 uses subprocess for both reads and writes for simplicity and
// zero extra dependencies.  gix can be layered in later for performance.

pub mod read;
pub mod subprocess;

pub use read::{
    CommitInfo, diff_commits, latest_version_number, list_commits, next_version_id, read_blob,
};
pub use subprocess::{
    git_add, git_checkout_branch, git_commit, git_config, git_create_branch, git_delete_branch,
};
