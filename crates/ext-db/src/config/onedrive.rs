use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OneDriveConfig {
    #[serde(
        default,
        rename = "acknowledgedSync",
        alias = "acknowledged_sync",
        alias = "acknowledged-sync"
    )]
    pub acknowledged_sync: Option<bool>,
}

impl OneDriveConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            acknowledged_sync: other.acknowledged_sync.or(self.acknowledged_sync),
        }
    }

    pub fn acknowledged_sync_or_default(&self) -> bool {
        self.acknowledged_sync.unwrap_or(false)
    }
}
