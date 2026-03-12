use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(
        default,
        rename = "oneDriveDir",
        alias = "one_drive_dir",
        alias = "one-drive-dir"
    )]
    pub one_drive_dir: Option<String>,
    #[serde(
        default,
        rename = "reportsDir",
        alias = "reports_dir",
        alias = "reports-dir"
    )]
    pub reports_dir: Option<String>,
}

impl PathsConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            one_drive_dir: other.one_drive_dir.or(self.one_drive_dir),
            reports_dir: other.reports_dir.or(self.reports_dir),
        }
    }
}
