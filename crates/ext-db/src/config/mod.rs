// ext-db::config — two-tier config resolution
//
// Config lives at: <project_root>/.etabs-ext/config.toml        (git-tracked)
//                  <project_root>/.etabs-ext/config.local.toml  (git-ignored)
//
// Resolution order: config.local.toml → config.toml → built-in defaults
//
// AI keys (ai.apiKey) MUST ONLY appear in config.local.toml.
// config.toml is git-tracked and pushed to OneDrive — never write secrets there.

pub mod calc;
pub mod extract;
pub mod git;
pub mod llm;
pub mod onedrive;
pub mod paths;
pub mod project;

pub use calc::CalcConfig;
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
    pub calc: CalcConfig,

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

    #[serde(default)]
    pub calc: CalcConfig,
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
            calc: base.calc,
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
            project: config.project.shared_only(),
            extract: config.extract.clone(),
            calc: config.calc.clone(),
        };
        Self::write_file(&config_dir.join(CONFIG_FILE), &shared)
    }

    pub fn write_local(project_root: &Path, config: &Self) -> Result<()> {
        let config_dir = project_root.join(CONFIG_DIR);
        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config dir: {}", config_dir.display()))?;

        let local = LocalConfigFile {
            project: config.project.local_only(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_config() -> Config {
        let mut config = Config::default();
        config.project.name = Some("Tower A".into());
        config.project.sidecar_path = Some("C:\\sidecar\\etab-cli.exe".into());
        config.project.units = Some("kip-ft-F".into());
        config.calc.code = Some("ACI318-14".into());
        config.extract.tables.group_assignments = Some(TableConfig::default());
        config
    }

    #[test]
    fn write_shared_omits_local_only_project_fields() {
        let dir = tempdir().unwrap();
        let config = sample_config();

        Config::write_shared(dir.path(), &config).unwrap();
        let text = std::fs::read_to_string(dir.path().join(CONFIG_DIR).join(CONFIG_FILE)).unwrap();

        assert!(text.contains("name = \"Tower A\""));
        assert!(text.contains("[calc]"));
        assert!(!text.contains("sidecar-path"));
        assert!(!text.contains("units = "));
    }

    #[test]
    fn write_local_only_persists_local_project_fields() {
        let dir = tempdir().unwrap();
        let config = sample_config();

        Config::write_local(dir.path(), &config).unwrap();
        let text =
            std::fs::read_to_string(dir.path().join(CONFIG_DIR).join(CONFIG_LOCAL_FILE)).unwrap();

        assert!(text.contains("sidecar-path"));
        assert!(text.contains("units = \"kip-ft-F\""));
        assert!(!text.contains("name = \"Tower A\""));
    }

    #[test]
    fn load_merges_legacy_shared_project_fields_with_local_override() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join(CONFIG_DIR);
        std::fs::create_dir_all(&config_dir).unwrap();

        std::fs::write(
            config_dir.join(CONFIG_FILE),
            r#"
[project]
name = "Tower A"
sidecar-path = "legacy-sidecar.exe"
units = "kN-m-C"

[calc]
code = "ACI318-14"
"#,
        )
        .unwrap();

        std::fs::write(
            config_dir.join(CONFIG_LOCAL_FILE),
            r#"
[project]
units = "kip-ft-F"
"#,
        )
        .unwrap();

        let loaded = Config::load(dir.path()).unwrap();
        assert_eq!(loaded.project.name.as_deref(), Some("Tower A"));
        assert_eq!(loaded.project.sidecar_path.as_deref(), Some("legacy-sidecar.exe"));
        assert_eq!(loaded.project.units.as_deref(), Some("kip-ft-F"));
        assert_eq!(loaded.calc.code_or_default(), "ACI318-14");
    }
}
