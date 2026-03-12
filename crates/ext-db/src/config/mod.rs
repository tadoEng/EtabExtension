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
pub mod git;
pub mod llm;
pub mod onedrive;
pub mod paths;
pub mod project;

pub use extract::{ExtractConfig, TableConfig, TableSelections};
pub use git::GitConfig;
pub use llm::LlmConfig;
pub use onedrive::OneDriveConfig;
pub use paths::PathsConfig;
pub use project::ProjectConfig;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
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

    #[serde(default)]
    pub git: GitConfig,

    #[serde(default)]
    pub paths: PathsConfig,

    #[serde(default)]
    pub onedrive: OneDriveConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SharedConfigFile {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub extract: ExtractConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LocalConfigFile {
    #[serde(default)]
    pub project: ProjectConfig,

    #[serde(default)]
    pub llm: LlmConfig,

    #[serde(default)]
    pub git: GitConfig,

    #[serde(default)]
    pub paths: PathsConfig,

    #[serde(default)]
    pub onedrive: OneDriveConfig,
}

impl Config {
    /// Load and merge config.toml + config.local.toml from project root.
    ///
    /// config.local.toml keys win over config.toml keys on every field
    /// via the merge() impl on each sub-config struct.
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_dir = project_root.join(CONFIG_DIR);
        let base =
            Self::load_file::<SharedConfigFile>(&config_dir.join(CONFIG_FILE))?.unwrap_or_default();
        let local = Self::load_file::<LocalConfigFile>(&config_dir.join(CONFIG_LOCAL_FILE))?
            .unwrap_or_default();

        Ok(Self {
            project: base.project.merge(local.project),
            extract: base.extract,
            llm: local.llm,
            git: local.git,
            paths: local.paths,
            onedrive: local.onedrive,
        })
    }

    pub fn write_shared(project_root: &Path, config: &Self) -> Result<()> {
        let config_dir = project_root.join(CONFIG_DIR);
        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config dir: {}", config_dir.display()))?;

        let shared = SharedConfigFile {
            project: config.project.clone(),
            extract: config.extract.clone(),
        };
        Self::write_file(&config_dir.join(CONFIG_FILE), &shared)
    }

    pub fn write_local(project_root: &Path, config: &Self) -> Result<()> {
        let config_dir = project_root.join(CONFIG_DIR);
        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config dir: {}", config_dir.display()))?;

        let local = LocalConfigFile {
            project: config.project.clone(),
            llm: config.llm.clone(),
            git: config.git.clone(),
            paths: config.paths.clone(),
            onedrive: config.onedrive.clone(),
        };
        Self::write_file(&config_dir.join(CONFIG_LOCAL_FILE), &local)
    }

    /// Load a single TOML file, returning None if the file doesn't exist.
    fn load_file<T: DeserializeOwned>(path: &PathBuf) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let parsed = toml::from_str(&text)
            .with_context(|| format!("Config parse error in {}", path.display()))?;
        Ok(Some(parsed))
    }

    fn write_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
        let tmp = path.with_extension("toml.tmp");
        let content = toml::to_string_pretty(value)
            .with_context(|| format!("Failed to serialize TOML: {}", path.display()))?;
        std::fs::write(&tmp, content)
            .with_context(|| format!("Failed to write tmp config: {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Failed to replace config file: {}", path.display()))?;
        Ok(())
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
        let name = if cfg!(windows) {
            "etab-cli.exe"
        } else {
            "etab-cli"
        };
        std::env::var_os("PATH").as_ref().and_then(|path_var| {
            std::env::split_paths(path_var).find_map(|dir| {
                let candidate = dir.join(name);
                candidate.exists().then_some(candidate)
            })
        })
    }
}
