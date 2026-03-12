// ext-core::state — 9-state working file status machine
//
// TODO Week 2: implement resolve() — correctness spine.
// This is the highest-priority implementation target.
//
// Resolution order (from agents.md §State Detection):
//   1. Does working/model.edb exist?       → Missing if no
//   2. Is ETABS PID alive?                 → OpenClean/OpenModified/Orphaned
//   3. Is basedOnVersion set in state.json?→ Untracked if no
//   4. mtime vs lastKnownMtime             → Modified or Clean
//
// The 9 states and their resolution priority logic must be implemented
// before any command guards can be correct.
//
// IMPORTANT: resolve() must call ext-core::sidecar::get_status()
// for the ETABS PID check — not ext-db directly. The AppContext
// (owned by ext-api) provides the resolved SidecarClient path.

// pub fn resolve(project_path: &std::path::Path, sidecar: &crate::sidecar::SidecarClient)
//     -> ext_error::ExtResult<WorkingFileStatus> { todo!() }
