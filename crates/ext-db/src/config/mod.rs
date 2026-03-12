// ext-db::config — two-tier config resolution
//
// Config lives at: <project_root>/.etabs-ext/config.toml        (git-tracked)
//                  <project_root>/.etabs-ext/config.local.toml  (git-ignored)
//
// Resolution order: config.local.toml → config.toml → built-in defaults
//
// AI keys (ai.apiKey) MUST ONLY appear in config.local.toml.
// config.toml is git-tracked and pushed to OneDrive — never write secrets there.

pub mod extract;
pub mod llm;
pub mod project;

pub use extract::{ExtractConfig, TableConfig, TableSelections};
pub use llm::LlmConfig;
pub use project::ProjectConfig;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONFIG_DIR: &str = ".etabs-ext";
pub const CONFIG_FILE: &str = "config.toml";
pub const CONFIG_LOCAL_FILE: &str = "config.local.toml";

/// Fully resolved configuration for a project.
/// Produced by Config::load() — callers never touch the raw TOML files directly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub extract: ExtractConfig,

    #[serde(default)]
    pub llm: LlmConfig,
}

impl Config {
    /// Load and merge config.toml + config.local.toml from project root.
    ///
    /// config.local.toml keys win over config.toml keys on every field
    /// via the merge() impl on each sub-config struct.
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_dir = project_root.join(CONFIG_DIR);
        let base = Self::load_file(&config_dir.join(CONFIG_FILE)).unwrap_or_default();
        let local = Self::load_file(&config_dir.join(CONFIG_LOCAL_FILE)).unwrap_or_default();
        Ok(base.merge(local))
    }

    /// Load a single TOML file, returning None if the file doesn't exist.
    fn load_file(path: &PathBuf) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        toml::from_str(&text)
            .with_context(|| format!("Config parse error in {}", path.display()))
            .ok()
    }

    /// Merge `other` over `self`. other's Some values win; None leaves self unchanged.
    fn merge(self, other: Self) -> Self {
        Self {
            project: self.project.merge(other.project),
            extract: self.extract.merge(other.extract),
            llm: self.llm.merge(other.llm),
        }
    }

    /// Path to the .etabs-ext config directory for a given project root.
    pub fn config_dir(project_root: &Path) -> PathBuf {
        project_root.join(CONFIG_DIR)
    }

    /// Resolve the sidecar path from config + env var + PATH.
    ///
    /// Called by AppContext during construction in ext-api.
    /// The resolved path is then passed into SidecarClient::new(path).
    ///
    /// Order: project.sidecar-path → ETABS_SIDECAR_PATH env → PATH lookup
    pub fn resolve_sidecar_path(&self, project_root: &Path) -> Option<PathBuf> {
        // 1. Explicit config path
        if let Some(ref p) = self.project.sidecar_path {
            let raw = PathBuf::from(p);
            let path = if raw.is_absolute() {
                raw
            } else {
                project_root.join(raw)
            };
            if path.exists() {
                return Some(path);
            }
        }

        // 2. Env var override
        if let Ok(env_path) = std::env::var("ETABS_SIDECAR_PATH") {
            let raw = PathBuf::from(env_path);
            let path = if raw.is_absolute() {
                raw
            } else {
                std::env::current_dir().ok()?.join(raw)
            };
            if path.exists() {
                return Some(path);
            }
        }

        // 3. PATH lookup — look for etab-cli or etab-cli.exe
        let name = if cfg!(windows) { "etab-cli.exe" } else { "etab-cli" };
        std::env::var_os("PATH")
            .as_ref()
            .and_then(|path_var| {
                std::env::split_paths(path_var).find_map(|dir| {
                    let candidate = dir.join(name);
                    candidate.exists().then_some(candidate)
                })
            })
    }
}
