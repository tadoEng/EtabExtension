// ext::utils::metrics — telemetry stub
//
// Telemetry is not required for ext Phase 1.
// This stub keeps the module path intact so future metrics can be added
// without restructuring imports across the codebase.
//
// To add telemetry later: replace this file with a real implementation.
// Do NOT use PostHog or any external analytics service without explicit
// user consent and a clear opt-in in config.local.toml.

/// No-op context — exists so call sites can be added without #[cfg] guards.
pub struct MetricsContext;

impl MetricsContext {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetricsContext {
    fn default() -> Self {
        Self::new()
    }
}
