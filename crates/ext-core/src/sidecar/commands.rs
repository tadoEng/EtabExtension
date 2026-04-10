// ext-core::sidecar::commands — all 8 etab-cli command methods
//
// ── Mode A (attach — ETABS must already be running) ───────────────────────
//   get_status, open_model, close_model, unlock_model
//
// ── Mode B (hidden — sidecar spawns its own headless ETABS) ──────────────
//   generate_e2k, run_analysis, extract_materials, extract_results
//
// IMPORTANT: extract_results uses a different calling convention to all
// others. --request takes a serialised JSON blob (not flat flags) because
// the table selection tree is too deep to flatten into CLI args.

use crate::sidecar::client::SidecarClient;
use ext_error::ExtResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::UnitInfo;

// ── Shared ────────────────────────────────────────────────────────────────

/// Unit preset string accepted by all Mode B commands that produce numbers.
/// Passed as --units to the sidecar.
/// Matches the presets defined in C# EtabSharp.System.Models.Units.
pub type UnitPreset = String;

// ── Mode A types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusUnitSystem {
    pub force: String,
    pub length: String,
    pub temperature: String,
    #[serde(alias = "isUS")]
    pub is_us: bool,
    pub is_metric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusData {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub etabs_version: Option<String>,
    pub open_file_path: Option<String>,
    pub is_model_open: bool,
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_system: Option<GetStatusUnitSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenModelData {
    pub file_path: String,
    pub previous_file_path: Option<String>,
    pub pid: Option<u32>,
    pub opened_in_new_instance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseModelData {
    pub closed_file_path: Option<String>,
    pub was_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockModelData {
    pub file_path: String,
    pub was_locked: bool,
}

// ── Mode B types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateE2kData {
    pub input_file: String,
    #[serde(alias = "outputPath")]
    pub output_file: String,
    pub file_size_bytes: u64,
    pub generation_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAnalysisData {
    pub file_path: String,
    pub cases_requested: Option<Vec<String>>,
    pub case_count: u64,
    pub finished_case_count: u64,
    pub analysis_time_ms: u64,
    pub units: Option<UnitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractMaterialsData {
    pub file_path: String,
    #[serde(alias = "outputPath")]
    pub output_file: Option<String>,
    pub table_key: String,
    pub row_count: u64,
    pub discarded_row_count: u64,
    pub units: Option<UnitInfo>,
    pub extraction_time_ms: u64,
}

/// Per-table outcome inside ExtractResultsData.
/// success=false here is a partial failure — other tables may have succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableResult {
    pub success: bool,
    pub output_file: Option<String>,
    pub row_count: u64,
    pub discarded_row_count: u64,
    pub error: Option<String>,
    pub extraction_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractResultsData {
    pub file_path: String,
    pub output_dir: String,
    /// Key = table slug (e.g. "storyForces", "basReactions")
    /// Caller must check each entry's .success — partial failures are normal.
    pub tables: std::collections::HashMap<String, TableResult>,
    pub total_row_count: u64,
    pub succeeded_count: u64,
    pub failed_count: u64,
    pub units: Option<UnitInfo>,
    pub extraction_time_ms: u64,
}

// ── extract-results request shape ─────────────────────────────────────────
// Serialised to JSON → passed as the single --request flag.
// Keep this in sync with ext-db::config::TableSelections.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractResultsRequest {
    pub units: UnitPreset,
    pub tables: TableSelections,
}

/// Per-table selection. None = skip. ["*"] = all. ["X", "Y"] = named items.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableSelections {
    pub story_definitions: Option<TableSelection>,
    pub pier_section_properties: Option<TableSelection>,
    pub base_reactions: Option<TableSelection>,
    pub story_forces: Option<TableSelection>,
    pub joint_drifts: Option<TableSelection>,
    pub pier_forces: Option<TableSelection>,
    pub modal_participating_mass_ratios: Option<TableSelection>,
    pub group_assignments: Option<TableSelection>,
    pub material_properties_concrete_data: Option<TableSelection>,
    pub material_list_by_story: Option<TableSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSelection {
    pub load_cases: Option<Vec<String>>,
    pub load_combos: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub field_keys: Option<Vec<String>>,
}

// ── Mode A commands ───────────────────────────────────────────────────────

impl SidecarClient {
    /// Check whether ETABS is running and what file (if any) is open.
    pub async fn get_status(&self) -> ExtResult<GetStatusData> {
        let resp = self.run::<GetStatusData>(&["get-status"]).await?;
        Ok(resp.data.unwrap_or(GetStatusData {
            is_running: false,
            pid: None,
            etabs_version: None,
            open_file_path: None,
            is_model_open: false,
            is_locked: None,
            is_analyzed: None,
            unit_system: None,
        }))
    }

    /// Open an ETABS model. `save_on_close` and `new_instance` are optional.
    pub async fn open_model(
        &self,
        file: &Path,
        save_on_close: bool,
        new_instance: bool,
    ) -> ExtResult<OpenModelData> {
        let file_str = file.display().to_string();
        let mut args = vec!["open-model", "--file", &file_str];
        let save_flag;
        let instance_flag;
        if save_on_close {
            save_flag = "--save";
            args.push(save_flag);
        }
        if new_instance {
            instance_flag = "--new-instance";
            args.push(instance_flag);
        }
        let resp = self.run::<OpenModelData>(&args).await?;
        resp.data
            .ok_or_else(|| ext_error::ExtError::SidecarParse("open-model returned no data".into()))
    }

    /// Close the currently open ETABS model.
    pub async fn close_model(&self, save: bool) -> ExtResult<CloseModelData> {
        let flag = if save { "--save" } else { "--no-save" };
        let resp = self.run::<CloseModelData>(&["close-model", flag]).await?;
        resp.data
            .ok_or_else(|| ext_error::ExtError::SidecarParse("close-model returned no data".into()))
    }

    /// Unlock an ETABS model file that was left in a locked state.
    pub async fn unlock_model(&self, file: &Path) -> ExtResult<UnlockModelData> {
        let file_str = file.display().to_string();
        let resp = self
            .run::<UnlockModelData>(&["unlock-model", "--file", &file_str])
            .await?;
        resp.data.ok_or_else(|| {
            ext_error::ExtError::SidecarParse("unlock-model returned no data".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExtractMaterialsData, ExtractResultsRequest, GenerateE2kData, OpenModelData,
        TableSelection, TableSelections,
    };

    #[test]
    fn open_model_data_accepts_null_pid() {
        let data: OpenModelData = serde_json::from_str(
            r#"{
                "filePath":"C:\\Models\\tower.edb",
                "previousFilePath":null,
                "pid":null,
                "openedInNewInstance":true
            }"#,
        )
        .unwrap();

        assert_eq!(data.file_path, r"C:\Models\tower.edb");
        assert_eq!(data.pid, None);
        assert!(data.opened_in_new_instance);
    }

    #[test]
    fn generate_e2k_data_accepts_current_output_file() {
        let data: GenerateE2kData = serde_json::from_str(
            r#"{
                "inputFile":"C:\\Models\\tower.edb",
                "outputFile":"C:\\Models\\tower.e2k",
                "fileSizeBytes":2048,
                "generationTimeMs":120
            }"#,
        )
        .unwrap();

        assert_eq!(data.input_file, r"C:\Models\tower.edb");
        assert_eq!(data.output_file, r"C:\Models\tower.e2k");
        assert_eq!(data.file_size_bytes, 2048);
        assert_eq!(data.generation_time_ms, Some(120));
    }

    #[test]
    fn generate_e2k_data_accepts_legacy_output_path_alias() {
        let data: GenerateE2kData = serde_json::from_str(
            r#"{
                "inputFile":"C:\\Models\\tower.edb",
                "outputPath":"C:\\Models\\tower.e2k",
                "fileSizeBytes":2048
            }"#,
        )
        .unwrap();

        assert_eq!(data.input_file, r"C:\Models\tower.edb");
        assert_eq!(data.output_file, r"C:\Models\tower.e2k");
        assert_eq!(data.file_size_bytes, 2048);
        assert_eq!(data.generation_time_ms, None);
    }

    #[test]
    fn extract_materials_data_accepts_output_file() {
        let data: ExtractMaterialsData = serde_json::from_str(
            r#"{
                "filePath":"C:\\Models\\tower.edb",
                "outputFile":"C:\\Temp\\material_list_by_story.parquet",
                "tableKey":"Material List by Story",
                "rowCount":12,
                "discardedRowCount":2,
                "units":{
                    "force":"kip",
                    "length":"ft",
                    "temperature":"F",
                    "isUs":true,
                    "isMetric":false,
                    "rawForce":1,
                    "rawLength":2,
                    "rawTemperature":3
                },
                "extractionTimeMs":300
            }"#,
        )
        .unwrap();

        assert_eq!(data.file_path, r"C:\Models\tower.edb");
        assert_eq!(
            data.output_file.as_deref(),
            Some(r"C:\Temp\material_list_by_story.parquet")
        );
        assert_eq!(data.table_key, "Material List by Story");
        assert_eq!(data.row_count, 12);
        assert_eq!(data.discarded_row_count, 2);
        assert!(data.units.is_some());
        assert_eq!(data.extraction_time_ms, 300);
    }

    #[test]
    fn extract_materials_data_accepts_null_output_file() {
        let data: ExtractMaterialsData = serde_json::from_str(
            r#"{
                "filePath":"C:\\Models\\tower.edb",
                "outputFile":null,
                "tableKey":"Material List by Story",
                "rowCount":0,
                "discardedRowCount":0,
                "units":null,
                "extractionTimeMs":180
            }"#,
        )
        .unwrap();

        assert_eq!(data.output_file, None);
        assert_eq!(data.row_count, 0);
        assert!(data.units.is_none());
        assert_eq!(data.extraction_time_ms, 180);
    }

    #[test]
    fn extract_results_request_serializes_new_table_keys_in_camel_case() {
        let request = ExtractResultsRequest {
            units: "US_Kip_Ft".to_string(),
            tables: TableSelections {
                group_assignments: Some(TableSelection {
                    load_cases: None,
                    load_combos: None,
                    groups: Some(vec!["Core".to_string()]),
                    field_keys: None,
                }),
                material_properties_concrete_data: Some(TableSelection {
                    load_cases: None,
                    load_combos: None,
                    groups: None,
                    field_keys: Some(vec!["Fc".to_string(), "Ec".to_string()]),
                }),
                material_list_by_story: Some(TableSelection {
                    load_cases: None,
                    load_combos: None,
                    groups: None,
                    field_keys: Some(vec!["Story".to_string()]),
                }),
                ..TableSelections::default()
            },
        };

        let json = serde_json::to_value(&request).unwrap();

        assert!(json["tables"]["groupAssignments"].is_object());
        assert!(json["tables"]["materialPropertiesConcreteData"].is_object());
        assert!(json["tables"]["materialListByStory"].is_object());
    }
}

// ── Mode B commands ───────────────────────────────────────────────────────

impl SidecarClient {
    /// Export a .edb snapshot to E2K text format.
    pub async fn generate_e2k(
        &self,
        file: &Path,
        output: &Path,
        overwrite: bool,
    ) -> ExtResult<GenerateE2kData> {
        let file_str = file.display().to_string();
        let output_str = output.display().to_string();
        let mut args = vec!["generate-e2k", "--file", &file_str, "--output", &output_str];
        let overwrite_flag;
        if overwrite {
            overwrite_flag = "--overwrite";
            args.push(overwrite_flag);
        }
        let resp = self.run::<GenerateE2kData>(&args).await?;
        resp.data.ok_or_else(|| {
            ext_error::ExtError::SidecarParse("generate-e2k returned no data".into())
        })
    }

    /// Run analysis on a committed snapshot. Never called on working/model.edb.
    pub async fn run_analysis(
        &self,
        file: &Path,
        cases: Option<&[String]>,
        units: &str,
    ) -> ExtResult<RunAnalysisData> {
        let file_str = file.display().to_string();
        let mut args = vec!["run-analysis", "--file", &file_str, "--units", units];
        if let Some(c) = cases {
            args.push("--cases");
            for case in c {
                args.push(case.as_str());
            }
        }
        let resp = self.run::<RunAnalysisData>(&args).await?;
        resp.data.ok_or_else(|| {
            ext_error::ExtError::SidecarParse("run-analysis returned no data".into())
        })
    }

    /// Extract material takeoff to a Parquet file.
    pub async fn extract_materials(
        &self,
        file: &Path,
        output_dir: &Path,
        units: &str,
        table_key: Option<&str>,
        field_keys: Option<&[String]>,
    ) -> ExtResult<ExtractMaterialsData> {
        let file_str = file.display().to_string();
        let output_str = output_dir.display().to_string();
        let table_key_owned;
        let field_keys_str;
        let mut args = vec![
            "extract-materials",
            "--file",
            &file_str,
            "--output-dir",
            &output_str,
            "--units",
            units,
        ];
        if let Some(k) = table_key {
            table_key_owned = k.to_string();
            args.push("--table-key");
            args.push(&table_key_owned);
        }
        if let Some(fk) = field_keys {
            field_keys_str = fk.join(",");
            args.push("--field-keys");
            args.push(&field_keys_str);
        }
        let resp = self.run::<ExtractMaterialsData>(&args).await?;
        resp.data.ok_or_else(|| {
            ext_error::ExtError::SidecarParse("extract-materials returned no data".into())
        })
    }

    /// Extract analysis results to Parquet files.
    ///
    /// NOTE: calling convention differs from all other commands.
    /// --file and --output-dir are flat flags.
    /// --request takes the ENTIRE TableSelections tree serialised as one JSON string.
    ///
    /// Returns ExtractResultsData whose .tables map must be checked per-entry —
    /// partial failure (some tables succeeded, some failed) is expected and normal.
    pub async fn extract_results(
        &self,
        file: &Path,
        output_dir: &Path,
        request: &ExtractResultsRequest,
    ) -> ExtResult<ExtractResultsData> {
        let file_str = file.display().to_string();
        let output_str = output_dir.display().to_string();
        // Serialise the entire request struct → one JSON string arg.
        // TOML snake_case is already converted to camelCase by #[serde(rename_all)].
        let request_json = serde_json::to_string(request).map_err(|e| {
            ext_error::ExtError::SidecarParse(format!("Failed to serialise request: {e}"))
        })?;

        let resp = self
            .run::<ExtractResultsData>(&[
                "extract-results",
                "--file",
                &file_str,
                "--output-dir",
                &output_str,
                "--request",
                &request_json,
            ])
            .await?;
        resp.data.ok_or_else(|| {
            ext_error::ExtError::SidecarParse("extract-results returned no data".into())
        })
    }
}
