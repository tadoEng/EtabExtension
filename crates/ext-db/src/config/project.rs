// ext-db::config::project — [project] section of config.toml

use serde::{Deserialize, Serialize};

/// Committed config — shared across all machines via OneDrive.
/// Never put secrets or machine-specific paths here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Human-readable project name, e.g. "HighRise Tower A"
    pub name: Option<String>,

    /// Relative or absolute path to etab-cli.exe.
    /// Override per-machine via config.local.toml or ETABS_SIDECAR_PATH env.
    pub sidecar_path: Option<String>,

    /// Default unit preset for sidecar operations.
    /// e.g. "kip-in-F", "kN-m-C"
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
        self.units.as_deref().unwrap_or("kip-in-F")
    }
}
