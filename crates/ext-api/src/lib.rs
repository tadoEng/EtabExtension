// ext-api — single source of truth for all application workflows
//
// ARCHITECTURE (from agents.md):
//   ext-api is the ONLY layer that orchestrates across ext-core + ext-db.
//   The CLI and agent both call ext-api exclusively.
//   Neither may call ext-core or ext-db directly for operations.
//
// AppContext is constructed here and owns:
//   - project root path
//   - resolved Config (merged config.toml + config.local.toml)
//   - resolved SidecarClient (path looked up from config → env → PATH)
//   - current StateFile (loaded fresh on each API call, never cached)

pub mod analyze;
pub mod branch;
pub mod checkout;
pub mod commit;
pub mod config_cmd;
pub mod context;
pub mod diff;
pub mod etabs;
pub mod guards;
pub mod init;
pub mod log;
mod path_utils;
pub mod remote;
pub mod report;
pub mod stash;
pub mod status;
pub mod switch;

pub use analyze::{AnalyzeOptions, AnalyzeResult, analyze_version};
pub use config_cmd::{
    ConfigEntry, ConfigListResult, ConfigSetResult, get_config, list_config, set_config,
};
pub use context::AppContext;
pub use etabs::{
    CloseMode, EtabsCloseConflict, EtabsCloseResult, EtabsOpenResult, EtabsRecoverConflict,
    EtabsRecoverResult, EtabsStatusResult, EtabsUnlockResult, RecoveryChoice, etabs_close,
    etabs_open, etabs_recover, etabs_status, etabs_unlock,
};
pub use report::{
    CalcArtifacts, RenderArtifact, RenderArtifacts, ReportArtifacts, load_calc_output,
    render_version, report_version, run_calc,
};
