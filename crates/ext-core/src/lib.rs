// ext-core — pure domain logic
//
// DEPENDENCY RULE: ext-core depends only on ext-error.
// It never imports ext-db. AppContext (owned by ext-api) passes
// already-resolved paths and config values down into ext-core functions.
//
// This keeps the dependency graph acyclic:
//   ext-error → ext-core → ext-db → ext-api → ext / ext-agent

pub mod sidecar;
pub mod state;
pub mod vcs;
pub mod branch;
pub mod version;
pub mod stash;
pub mod reports;
pub mod remote;
pub mod fs;
