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
pub struct GetStatusData {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub etabs_version: Option<String>,
    pub open_file_path: Option<String>,
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenModelData {
    pub opened_file: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseModelData {
    pub closed_file: Option<String>,
    pub saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockModelData {
    pub unlocked_file: String,
}

// ── Mode B types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateE2kData {
    pub output_path: String,
    pub file_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAnalysisData {
    pub cases_run: Vec<String>,
    pub elapsed_seconds: f64,
    pub units: UnitInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractMaterialsData {
    pub output_path: String,
    pub row_count: u64,
    pub units: UnitInfo,
}

/// Per-table outcome inside ExtractResultsData.
/// success=false here is a partial failure — other tables may have succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub row_count: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractResultsData {
    /// Key = table slug (e.g. "storyForces", "basReactions")
    /// Caller must check each entry's .success — partial failures are normal.
    pub tables: std::collections::HashMap<String, TableResult>,
    pub units: UnitInfo,
}

// ── extract-results request shape ─────────────────────────────────────────
// Serialised to JSON → passed as the single --request flag.
// Mirrors TableSelections in ext-db::config exactly.

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
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "open-model returned no data".into(),
        ))
    }

    /// Close the currently open ETABS model.
    pub async fn close_model(&self, save: bool) -> ExtResult<CloseModelData> {
        let flag = if save { "--save" } else { "--no-save" };
        let resp = self.run::<CloseModelData>(&["close-model", flag]).await?;
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "close-model returned no data".into(),
        ))
    }

    /// Unlock an ETABS model file that was left in a locked state.
    pub async fn unlock_model(&self, file: &Path) -> ExtResult<UnlockModelData> {
        let file_str = file.display().to_string();
        let resp = self
            .run::<UnlockModelData>(&["unlock-model", "--file", &file_str])
            .await?;
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "unlock-model returned no data".into(),
        ))
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
        let mut args = vec![
            "generate-e2k",
            "--file",
            &file_str,
            "--output",
            &output_str,
        ];
        let overwrite_flag;
        if overwrite {
            overwrite_flag = "--overwrite";
            args.push(overwrite_flag);
        }
        let resp = self.run::<GenerateE2kData>(&args).await?;
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "generate-e2k returned no data".into(),
        ))
    }

    /// Run analysis on a committed snapshot. Never called on working/model.edb.
    pub async fn run_analysis(
        &self,
        file: &Path,
        cases: Option<&[String]>,
        units: &str,
    ) -> ExtResult<RunAnalysisData> {
        let file_str = file.display().to_string();
        let cases_str;
        let mut args = vec!["run-analysis", "--file", &file_str, "--units", units];
        if let Some(c) = cases {
            cases_str = c.join(",");
            args.push("--cases");
            args.push(&cases_str);
        }
        let resp = self.run::<RunAnalysisData>(&args).await?;
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "run-analysis returned no data".into(),
        ))
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
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "extract-materials returned no data".into(),
        ))
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
        resp.data.ok_or_else(|| ext_error::ExtError::SidecarParse(
            "extract-results returned no data".into(),
        ))
    }
}
