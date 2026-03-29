// ext-core::sidecar — C# etab-cli IPC layer
//
// DESIGN: SidecarClient takes an already-resolved sidecar path.
// Path resolution (config → env var → PATH) lives in ext-api::context,
// keeping ext-core free of any ext-db dependency.
//
// CALLING CONVENTION (from agents.md):
//   stdin  — nothing
//   stdout — exactly one JSON object: { success, data?, error?, timestamp }
//   stderr — progress lines (ℹ ✓ ✗ ⚠) forwarded live to terminal
//   exit 0 — success, exit 1 — failure
//
// Mode A commands (attach — ETABS must be running):
//   get_status, open_model, close_model, unlock_model
//
// Mode B commands (hidden — spawns its own ETABS, no live instance needed):
//   generate_e2k, extract_materials, run_analysis, extract_results

pub mod client;
pub mod commands;
pub mod types;

pub use client::SidecarClient;
pub use commands::{
    CloseModelData, ExtractMaterialsData, ExtractResultsData, ExtractResultsRequest,
    GenerateE2kData, GetStatusData, GetStatusUnitSystem, OpenModelData, RunAnalysisData,
    TableResult, TableSelection, TableSelections, UnitPreset, UnlockModelData,
};
pub use types::{SidecarResponse, UnitInfo};
