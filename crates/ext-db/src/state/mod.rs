// ext-db::state — state.json read/write
//
// State is resolved fresh at the start of every ext-api function — never cached.
// The resolve() logic itself lives in ext-core::state (pure domain logic).
// This module owns only the serialisation schema and disk I/O.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
pub use ext_core::stash::StashEntry;
use ext_core::state::WorkingFileStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const STATE_FILE: &str = ".etabs-ext/state.json";
pub const STATE_SCHEMA_VERSION: u32 = 2;

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

    /// One stash slot per branch.  Key = branch name.
    #[serde(default)]
    pub stashes: HashMap<String, StashEntry>,

    pub updated_at: DateTime<Utc>,
}

impl StateFile {
    pub fn new_empty() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            working_file: None,
            stashes: HashMap::new(),
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
        let mut state: Self = serde_json::from_str(&text)
            .with_context(|| format!("State file corrupted: {}", path.display()))?;
        // Schema migration: v1 → v2 (adds stashes map)
        if state.schema_version < 2 {
            state.stashes = HashMap::new();
            state.schema_version = STATE_SCHEMA_VERSION;
        }
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

        let text = serde_json::to_string_pretty(self).context("Failed to serialise state")?;
        std::fs::write(&tmp, text)
            .with_context(|| format!("Failed to write tmp state: {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("Failed to rename state file: {}", path.display()))?;
        Ok(())
    }
}
