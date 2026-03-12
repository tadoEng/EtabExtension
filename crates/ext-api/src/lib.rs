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
//
// Sidecar path resolution lives here — NOT in ext-core — because
// resolution requires ext-db::Config, and ext-db depends on ext-core.
// Putting it here keeps the dependency graph acyclic.

pub mod analyze;
pub mod branch;
pub mod checkout;
pub mod commit;
pub mod config_cmd;
pub mod context;
pub mod diff;
pub mod etabs;
pub mod init;
pub mod remote;
pub mod report;
pub mod stash;
pub mod status;
pub mod switch;

pub use context::AppContext;
