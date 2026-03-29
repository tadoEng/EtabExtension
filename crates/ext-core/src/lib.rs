// ext-core — pure domain logic
//
// DEPENDENCY RULE: ext-core depends only on ext-error and anyhow.
// TODO(week5-6+): replace broad anyhow usage with typed error surfaces where
// it materially improves API contracts.
// It never imports ext-db. AppContext (owned by ext-api) passes
// already-resolved paths and config values down into ext-core functions.
//
// This keeps the dependency graph acyclic:
//   ext-error → ext-core → ext-db → ext-api → ext / ext-agent

pub mod branch; // branch/mod.rs + branch/copy.rs
pub mod fs; // atomic_copy, check_disk_space, cleanup_stale_tmp
pub mod remote; // stub — implemented Week 9-10
pub mod reports; // stub — implemented Week 9-10
pub mod sidecar; // sidecar/mod.rs + client.rs + commands.rs + types.rs
pub mod stash; // stash/mod.rs
pub mod state; // state.rs (working file state machine)
pub mod vcs; // vcs/mod.rs + subprocess.rs + read.rs
pub mod version; // version/mod.rs + manifest.rs + snapshot.rs
