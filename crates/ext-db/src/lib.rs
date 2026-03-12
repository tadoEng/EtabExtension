// ext-db — config, state, and storage layer
//
// Owns:
//   config.toml + config.local.toml resolution  (config/)
//   state.json read/write                        (state/)
//   SQLite project registry + session history    (registry/) — Phase 2

pub mod config;
pub mod registry;
pub mod state;

pub use config::{
    Config, ExtractConfig, GitConfig, LlmConfig, OneDriveConfig, PathsConfig, ProjectConfig,
    TableConfig, TableSelections,
};
pub use ext_core::state::WorkingFileStatus;
pub use state::{StateFile, WorkingFileState};
