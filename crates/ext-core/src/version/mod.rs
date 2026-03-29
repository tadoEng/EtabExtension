// ext-core::version — version snapshot domain logic.
//
//   manifest.rs  — manifest.json + summary.json schemas and I/O
//   snapshot.rs  — .partial sentinel, RAII guard, disk check
//   mod.rs       — commit sequence, list, show

pub mod manifest;
pub mod snapshot;

pub use manifest::{AnalysisSummary, VersionManifest};
pub use snapshot::{PartialGuard, begin_snapshot, cleanup_partial_snapshots, complete_snapshot};
