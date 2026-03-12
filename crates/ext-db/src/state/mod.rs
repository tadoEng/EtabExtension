// ext-db::state — state.json read/write
//
// State is resolved fresh at the start of every ext-api function — never cached.
// The resolve() logic itself lives in ext-core::state (pure domain logic).
// This module owns only the serialisation schema and disk I/O.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const STATE_FILE: &str = ".etabs-ext/state.json";
pub const STATE_SCHEMA_VERSION: u32 = 1;

/// The 9 working file states from agents.md §State Machine.
///
/// Resolution priority (decide from top to bottom):
///   1. Missing   — working/model.edb doesn't exist
///   2. Orphaned  — ETABS PID alive but file not open in it
///   3. OpenModified / OpenClean — ETABS PID alive and file is open
///   4. Analyzed  — .etabs-ext/analysis/ results exist for current commit
///   5. Modified  — mtime > lastKnownMtime
///   6. Clean     — mtime == lastKnownMtime, basedOnVersion is set
///   7. Untracked — basedOnVersion is not set (never been committed)
///
/// Locked is a sub-state of Clean/Analyzed — the .edb has a lock file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkingFileStatus {
    /// working/model.edb does not exist on disk
    Missing,
    /// ETABS is running with this file open, no unsaved changes detected
    OpenClean,
    /// ETABS is running with this file open, unsaved changes detected
    OpenModified,
    /// ETABS PID is alive but this file is not its open file — state unknown
    Orphaned,
    /// File exists, tracked, no changes since last commit, analysis results present
    Analyzed,
    /// File exists, tracked, no changes since last commit, no analysis results
    Clean,
    /// File exists, tracked, mtime changed since last commit
    Modified,
    /// File exists but has never been committed (basedOnVersion is None)
    Untracked,
    /// .edb has an ETABS lock file (sub-state of Clean/Analyzed)
    Locked,
}

impl std::fmt::Display for WorkingFileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WorkingFileStatus::Missing => "Missing",
            WorkingFileStatus::OpenClean => "OpenClean",
            WorkingFileStatus::OpenModified => "OpenModified",
            WorkingFileStatus::Orphaned => "Orphaned",
            WorkingFileStatus::Analyzed => "Analyzed",
            WorkingFileStatus::Clean => "Clean",
            WorkingFileStatus::Modified => "Modified",
            WorkingFileStatus::Untracked => "Untracked",
            WorkingFileStatus::Locked => "Locked",
        };
        write!(f, "{s}")
    }
}

/// Persisted state for the tracked working file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingFileState {
    /// Absolute path to working/model.edb
    pub path: PathBuf,

    pub status: WorkingFileStatus,

    /// PID of the ETABS process that has this file open (OpenClean/OpenModified/Orphaned)
    pub etabs_pid: Option<u32>,

    /// Git commit hash the working file was checked out from
    pub last_commit_hash: Option<String>,

    /// Version tag the working file is based on, e.g. "v3"
    pub based_on_version: Option<String>,

    /// mtime at the time of last commit/checkout — used for Modified detection
    pub last_known_mtime: Option<DateTime<Utc>>,

    pub status_changed_at: DateTime<Utc>,
}

/// Root structure of .etabs-ext/state.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFile {
    pub schema_version: u32,

    /// None if no working file is tracked yet (before ext init or after ext checkout --discard)
    pub working_file: Option<WorkingFileState>,

    pub updated_at: DateTime<Utc>,
}

impl StateFile {
    pub fn new_empty() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            working_file: None,
            updated_at: Utc::now(),
        }
    }

    /// Load state.json from the project root. Returns an empty state if the file
    /// doesn't exist yet (before first ext init commit).
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(STATE_FILE);
        if !path.exists() {
            return Ok(Self::new_empty());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state file: {}", path.display()))?;
        let state: Self = serde_json::from_str(&text)
            .with_context(|| format!("State file corrupted: {}", path.display()))?;
        Ok(state)
    }

    /// Write state.json atomically (write to .tmp then rename).
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = project_root.join(STATE_FILE);
        let tmp = path.with_extension("json.tmp");

        // Ensure .etabs-ext/ exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir: {}", parent.display()))?;
        }

        let text = serde_json::to_string_pretty(self)
            .context("Failed to serialise state")?;
        std::fs::write(&tmp, text)
            .with_context(|| format!("Failed to write tmp state: {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("Failed to rename state file: {}", path.display()))?;
        Ok(())
    }
}
