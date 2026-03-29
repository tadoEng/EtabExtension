// ext-core::version::manifest — manifest.json and summary.json schemas.
//
// Both files are written as pretty-printed JSON using atomic write
// (tmp → rename) so a crash mid-write never leaves a corrupt file.
//
// manifest.json is git-tracked.  summary.json is git-tracked only after
// analysis has run.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── VersionManifest ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    /// Version identifier, e.g. "v3".
    pub id: String,
    pub branch: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    /// Parent version id, e.g. "v2".  None for the first commit on a branch.
    pub parent: Option<String>,
    pub edb_size_bytes: u64,
    /// None when --no-e2k was passed.
    pub e2k_size_bytes: Option<u64>,
    pub is_analyzed: bool,
    /// false when --no-e2k was passed.
    pub e2k_generated: bool,
    pub materials_extracted: bool,
    /// Filled in after the git commit completes.
    pub git_commit_hash: Option<String>,
}

impl VersionManifest {
    /// Write the manifest to `<version_dir>/manifest.json` atomically.
    pub fn write_to(&self, version_dir: &Path) -> Result<()> {
        write_json(version_dir, "manifest.json", self)
    }

    /// Read the manifest from `<version_dir>/manifest.json`.
    pub fn read_from(version_dir: &Path) -> Result<Self> {
        read_json(version_dir, "manifest.json")
    }
}

// ── AnalysisSummary ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalSummary {
    pub num_modes: u32,
    /// Dominant period in X direction (seconds).
    pub dominant_period_x: Option<f64>,
    /// Dominant period in Y direction (seconds).
    pub dominant_period_y: Option<f64>,
    /// Mass participation ratio in X (0–1).
    pub mass_participation_x: Option<f64>,
    /// Mass participation ratio in Y (0–1).
    pub mass_participation_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseReactionSummary {
    pub max_base_shear_x: Option<f64>,
    pub max_base_shear_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftSummary {
    /// Maximum story drift ratio across all stories and load cases.
    pub max_drift: Option<f64>,
    /// Story label where max drift occurs.
    pub max_drift_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub analyzed_at: DateTime<Utc>,
    pub load_cases: Vec<String>,
    pub modal: ModalSummary,
    pub base_reaction: BaseReactionSummary,
    pub drift: DriftSummary,
}

impl AnalysisSummary {
    /// Write to `<version_dir>/summary.json` atomically.
    pub fn write_to(&self, version_dir: &Path) -> Result<()> {
        write_json(version_dir, "summary.json", self)
    }

    /// Read from `<version_dir>/summary.json`.
    pub fn read_from(version_dir: &Path) -> Result<Self> {
        read_json(version_dir, "summary.json")
    }
}

// ── I/O helpers ───────────────────────────────────────────────────────────────

fn write_json<T: serde::Serialize>(dir: &Path, filename: &str, value: &T) -> Result<()> {
    let path = dir.join(filename);
    let tmp = path.with_extension("json.tmp");

    let text =
        serde_json::to_string_pretty(value).with_context(|| format!("Serialize {filename}"))?;

    std::fs::write(&tmp, &text).with_context(|| format!("Write tmp {}", tmp.display()))?;

    std::fs::rename(&tmp, &path)
        .with_context(|| format!("Rename {} → {}", tmp.display(), path.display()))?;

    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(dir: &Path, filename: &str) -> Result<T> {
    let path = dir.join(filename);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("Read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("Parse {}", path.display()))
}

/// Return the path to manifest.json inside a version directory.
pub fn manifest_path(version_dir: &Path) -> PathBuf {
    version_dir.join("manifest.json")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn sample_manifest(id: &str) -> VersionManifest {
        VersionManifest {
            id: id.to_string(),
            branch: "main".to_string(),
            message: "Test commit".to_string(),
            author: "Alice".to_string(),
            timestamp: Utc::now(),
            parent: None,
            edb_size_bytes: 1024,
            e2k_size_bytes: Some(512),
            is_analyzed: false,
            e2k_generated: true,
            materials_extracted: false,
            git_commit_hash: None,
        }
    }

    #[test]
    fn manifest_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let m = sample_manifest("v1");
        m.write_to(tmp.path()).unwrap();
        let m2 = VersionManifest::read_from(tmp.path()).unwrap();
        assert_eq!(m2.id, "v1");
        assert_eq!(m2.branch, "main");
    }

    #[test]
    fn manifest_no_tmp_left_on_success() {
        let tmp = TempDir::new().unwrap();
        sample_manifest("v1").write_to(tmp.path()).unwrap();
        assert!(!tmp.path().join("manifest.json.tmp").exists());
    }
}
