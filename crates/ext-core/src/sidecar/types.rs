// ext-core::sidecar::types — JSON envelope and shared data types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Top-level JSON envelope from every etab-cli command.
///
/// C# contract:
///   { "success": bool, "data"?: T, "error"?: string, "timestamp": string }
///
/// success=false at this level = fatal error (ETABS crash, file not found).
/// Per-table partial failures in extract-results are inside data.tables[slug].
#[derive(Debug, Deserialize)]
pub struct SidecarResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Unit system snapshot present in every command that touches numeric data.
///
/// Mirrors C# UnitInfo exactly. The raw_* fields are ETABSv1 enum ints stored
/// so RestoreAsync can round-trip back to the exact original preset.
///
/// Serialised into result JSON — used by ext-api to record which units
/// the extracted data is in.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitInfo {
    /// e.g. "kip", "kN", "N", "lb", "kgf", "tonf"
    pub force: String,
    /// e.g. "ft", "in", "m", "cm", "mm"
    pub length: String,
    /// "F" or "C"
    pub temperature: String,
    pub is_us: bool,
    pub is_metric: bool,
    /// Raw ETABSv1.eForce enum int
    pub raw_force: i32,
    /// Raw ETABSv1.eLength enum int
    pub raw_length: i32,
    /// Raw ETABSv1.eTemperature enum int
    pub raw_temperature: i32,
}
