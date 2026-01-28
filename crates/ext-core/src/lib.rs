use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use ts_rs::TS;
use std::collections::HashMap;

/// Main project domain model
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct Project {
    #[ts(type = "string")]
    pub id: Uuid,

    pub name: String,

    pub description: String,

    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Complete project state including all branches
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct ProjectState {
    pub project_name: String,
    pub project_path: String,
    pub current_branch: String,
    pub branches: HashMap<String, BranchData>,
}

/// Information about a branch
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct BranchData {
    pub name: String,
    pub description: Option<String>,
    pub versions: Vec<VersionInfo>,
    pub latest_version: String,
    pub parent_branch: Option<String>,
    pub parent_version: Option<String>,

    #[ts(type = "string")]
    pub created: Option<DateTime<Utc>>,
    
    pub working_file: Option<WorkingFileInfo>,
}

/// Information about a version/commit
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct VersionInfo {
    pub id: String,
    pub message: String,
    pub author: Option<String>,

    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,

    pub e2k_path: Option<String>,
    pub analyzed: bool,
}

/// Working file status
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct WorkingFileInfo {
    pub exists: bool,
    pub path: String,
    pub is_open: bool,
    pub has_unsaved_changes: bool,
    pub source_version: Option<String>,
}

/// ETABS application status
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct EtabsStatus {
    pub is_running: bool,
    pub version: Option<String>,
    pub current_file: Option<String>,
}

/// Generic CLI result wrapper
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct CliResult<T> {
    pub success: bool,
    pub error: Option<String>,

    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,

    pub data: Option<T>,
}

/// ETABS file validation data
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct ValidationData {
    pub etabs_installed: bool,
    pub etabs_version: Option<String>,
    pub file_valid: Option<bool>,
    pub file_path: Option<String>,
    pub file_exists: Option<bool>,
    pub file_extension: Option<String>,
    pub is_analyzed: Option<bool>,
    pub validation_messages: Vec<String>,
}

/// E2K generation result data
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct GenerateE2KData {
    pub input_file: String,
    pub output_file: Option<String>,
    pub file_exists: bool,
    pub file_extension: Option<String>,
    pub output_exists: Option<bool>,
    pub generation_successful: Option<bool>,
    pub file_size_bytes: Option<u64>,
    pub generation_time_ms: Option<u64>,
    pub messages: Vec<String>,
}

/// E2K file comparison result
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct E2KDiffResult {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub changes: Vec<E2KChange>,
    pub raw_diff: String,
}

/// Individual E2K change
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct E2KChange {
    #[ts(rename = "type")]
    pub change_type: String, // "add", "remove", "modify"
    pub category: String,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// 3D geometry comparison result
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct GeometryDiffResult {
    pub members_added: Vec<String>,
    pub members_removed: Vec<String>,
    pub members_modified: Vec<String>,
    pub total_changes: usize,
}

/// Request to create a new branch
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct CreateBranchRequest {
    pub project_path: String,
    pub branch_name: String,
    pub from_branch: String,
    pub from_version: String,
    pub description: Option<String>,
}

/// Request to save a version
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct SaveVersionRequest {
    pub project_path: String,
    pub branch_name: String,
    pub message: String,
    pub generate_e2k: bool,
}

/// Request to compare two versions
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct CompareVersionsRequest {
    pub project_path: String,
    pub version1: VersionIdentifier,
    pub version2: VersionIdentifier,
    pub diff_type: String, // "e2k", "geometry", "both"
}

/// Identifies a specific version
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = concat!(env!("CARGO_MANIFEST_DIR"), "/../../packages/shared/src/types/")
)]
pub struct VersionIdentifier {
    pub branch: String,
    pub version_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        let project = Project::new(
            "Test Project".to_string(),
            "A test project".to_string()
        );
        assert_eq!(project.name, "Test Project");
    }

    #[test]
    fn test_export_typescript_bindings() {
        // Export all types
        Project::export().expect("Failed to export Project");
        ProjectState::export().expect("Failed to export ProjectState");
        BranchData::export().expect("Failed to export BranchData");
        VersionInfo::export().expect("Failed to export VersionInfo");
        WorkingFileInfo::export().expect("Failed to export WorkingFileInfo");
        EtabsStatus::export().expect("Failed to export EtabsStatus");
        ValidationData::export().expect("Failed to export ValidationData");
        GenerateE2KData::export().expect("Failed to export GenerateE2KData");
        E2KDiffResult::export().expect("Failed to export E2KDiffResult");
        E2KChange::export().expect("Failed to export E2KChange");
        GeometryDiffResult::export().expect("Failed to export GeometryDiffResult");
        CreateBranchRequest::export().expect("Failed to export CreateBranchRequest");
        SaveVersionRequest::export().expect("Failed to export SaveVersionRequest");
        CompareVersionsRequest::export().expect("Failed to export CompareVersionsRequest");
        VersionIdentifier::export().expect("Failed to export VersionIdentifier");
    }
}