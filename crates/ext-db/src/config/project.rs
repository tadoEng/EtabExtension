// ext-db::config::project — [project] section of config.toml

use serde::{Deserialize, Serialize};

/// Project config supports legacy reads from shared config, but new writes split
/// shared vs local fields explicitly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Human-readable project name, e.g. "HighRise Tower A"
    pub name: Option<String>,

    /// Relative or absolute path to etab-cli.exe.
    /// Machine-local going forward; shared reads are tolerated for compatibility.
    pub sidecar_path: Option<String>,

    /// Default unit preset for sidecar operations.
    /// Machine-local going forward; shared reads are tolerated for compatibility.
    pub units: Option<String>,
}

impl ProjectConfig {
    /// Merge other over self — other's Some values win.
    pub fn merge(self, other: Self) -> Self {
        Self {
            name: other.name.or(self.name),
            sidecar_path: other.sidecar_path.or(self.sidecar_path),
            units: other.units.or(self.units),
        }
    }

    pub fn units_or_default(&self) -> &str {
        self.units.as_deref().unwrap_or("kip-ft-F")
    }

    pub fn shared_only(&self) -> Self {
        Self {
            name: self.name.clone(),
            sidecar_path: None,
            units: None,
        }
    }

    pub fn local_only(&self) -> Self {
        Self {
            name: None,
            sidecar_path: self.sidecar_path.clone(),
            units: self.units.clone(),
        }
    }
}
