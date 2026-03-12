// ext-db — config, state, and storage layer
//
// Owns:
//   config.toml + config.local.toml resolution  (config/)
//   state.json read/write                        (state/)
//   SQLite project registry + session history    (registry/) — Phase 2

pub mod config;
pub mod state;
pub mod registry;

pub use config::{Config, ExtractConfig, LlmConfig, ProjectConfig, TableConfig, TableSelections};
pub use state::{StateFile, WorkingFileState, WorkingFileStatus};
