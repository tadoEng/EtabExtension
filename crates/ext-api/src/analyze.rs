use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{
    sidecar::{ExtractResultsRequest, RunAnalysisData, TableSelection, TableSelections},
    vcs::{current_branch, git_add, git_commit},
    version::{
        AnalysisSummary, VersionManifest,
        manifest::{BaseReactionSummary, DriftSummary, ModalSummary},
    },
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    status::resolve_working_file_status,
};

#[derive(Debug, Clone, Default)]
pub struct AnalyzeOptions {
    pub cases: Option<Vec<String>>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResult {
    pub version_id: String,
    pub branch: String,
    pub results_dir: PathBuf,
    pub elapsed_ms: u64,
    pub warning: Option<String>,
    pub already_analyzed: bool,
}

#[derive(Debug, Clone)]
pub struct AnalyzeSnapshotOutcome {
    pub summary_path: PathBuf,
    pub results_dir: PathBuf,
    pub extract_warning: Option<String>,
}

pub async fn analyze_snapshot(
    ctx: &AppContext,
    version_dir: &Path,
    cases: Option<&[String]>,
) -> Result<AnalyzeSnapshotOutcome> {
    let sidecar = ctx.require_sidecar()?;
    let units = resolve_sidecar_units(ctx)?;
    let edb = version_dir.join("model.edb");

    let run_data = sidecar
        .run_analysis(&edb, cases, &units)
        .await
        .with_context(|| format!("run-analysis failed for {}", edb.display()))?;

    let results_dir = version_dir.join("results");
    std::fs::create_dir_all(&results_dir)
        .with_context(|| format!("Create results dir {}", results_dir.display()))?;

    let request = ExtractResultsRequest {
        units: units.clone(),
        tables: build_extract_request(&ctx.config.extract.tables),
    };

    let extract_warning = match sidecar.extract_results(&edb, &results_dir, &request).await {
        Ok(data) => {
            if data.failed_count > 0 {
                Some(format!(
                    "⚠ Analysis finished, but {} result table(s) failed to extract",
                    data.failed_count
                ))
            } else {
                None
            }
        }
        Err(err) => Some(format!(
            "⚠ Analysis finished, but result extraction failed: {err}"
        )),
    };

    let summary = build_summary(&run_data);
    summary
        .write_to(version_dir)
        .with_context(|| format!("Write summary.json in {}", version_dir.display()))?;

    Ok(AnalyzeSnapshotOutcome {
        summary_path: version_dir.join("summary.json"),
        results_dir,
        extract_warning,
    })
}

pub async fn analyze_version(
    ctx: &AppContext,
    version_ref: &str,
    opts: AnalyzeOptions,
) -> Result<AnalyzeResult> {
    let t0 = std::time::Instant::now();
    let state = ctx.load_state()?;
    let status = resolve_working_file_status(&state, &ctx.project_root);
    match check_state_guard(Command::Analyze, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(_) | GuardOutcome::Allow => {}
    }

    let ext_dir = ctx.ext_dir();
    let current = current_branch(&ext_dir)?;
    let (branch, version_id) = if let Some((branch, version_id)) = version_ref.split_once('/') {
        (branch.to_string(), version_id.to_string())
    } else {
        (current, version_ref.to_string())
    };

    let version_dir = ext_dir.join(&branch).join(&version_id);
    if !version_dir.exists() {
        bail!(
            "✗ Version '{branch}/{version_id}' not found\n  Run: ext log to see available versions"
        );
    }
    let edb = version_dir.join("model.edb");
    if !edb.exists() {
        bail!("✗ model.edb missing in version '{branch}/{version_id}'");
    }

    let mut manifest = VersionManifest::read_from(&version_dir)
        .with_context(|| format!("Failed to read manifest for '{branch}/{version_id}'"))?;
    if manifest.is_analyzed && !opts.force {
        return Ok(AnalyzeResult {
            version_id,
            branch,
            results_dir: version_dir.join("results"),
            elapsed_ms: t0.elapsed().as_millis() as u64,
            warning: Some("Version is already analyzed. Use --force to re-run.".to_string()),
            already_analyzed: true,
        });
    }

    let outcome = analyze_snapshot(ctx, &version_dir, opts.cases.as_deref()).await?;

    manifest.is_analyzed = true;
    manifest
        .write_to(&version_dir)
        .with_context(|| format!("Failed to rewrite manifest for '{branch}/{version_id}'"))?;

    let manifest_path = version_dir.join("manifest.json");
    git_add(&ext_dir, &[&manifest_path, &outcome.summary_path])
        .with_context(|| "Failed to stage analysis metadata")?;

    let author = ctx.config.git.author_or_default();
    let email = ctx.config.git.email_or_default();
    git_commit(
        &ext_dir,
        &format!("ext: analysis results {}", version_id),
        author,
        email,
    )
    .with_context(|| "Failed to commit analysis results")?;

    Ok(AnalyzeResult {
        version_id,
        branch,
        results_dir: outcome.results_dir,
        elapsed_ms: t0.elapsed().as_millis() as u64,
        warning: outcome.extract_warning,
        already_analyzed: false,
    })
}

pub fn resolve_sidecar_units(ctx: &AppContext) -> Result<String> {
    let raw = ctx
        .config
        .extract
        .units
        .as_deref()
        .or(ctx.config.project.units.as_deref());

    let key = raw.unwrap_or("US_Kip_Ft").trim().to_ascii_uppercase();
    let canonical = match key.as_str() {
        "US_KIP_FT" | "KIP_FT" | "KIP-FT-F" | "KIP/FT/F" => "US_Kip_Ft",
        "US_KIP_IN" | "KIP_IN" | "KIP-IN-F" | "KIP/IN/F" | "KIP-IN" => "US_Kip_In",
        "US_LB_FT" | "LB_FT" | "LB-FT-F" | "LB/FT/F" => "US_Lb_Ft",
        "US_LB_IN" | "LB_IN" | "LB-IN-F" | "LB/IN/F" => "US_Lb_In",
        "SI_KN_M" | "KN_M" | "KN-M-C" | "KN/M/C" => "SI_kN_m",
        "SI_KN_MM" | "KN_MM" | "KN-MM-C" | "KN/MM/C" => "SI_kN_mm",
        "SI_N_M" | "N_M" | "N-M-C" | "N/M/C" => "SI_N_m",
        "SI_N_MM" | "N_MM" | "N-MM-C" | "N/MM/C" => "SI_N_mm",
        "SI_KGF_M" | "KGF_M" | "KGF-M-C" | "KGF/M/C" => "SI_kgf_m",
        "SI_TONF_M" | "TONF_M" | "TONF-M-C" | "TONF/M/C" => "SI_tonf_m",
        other => anyhow::bail!(
            "Unknown ETABS unit preset '{}'\n  Valid values: US_Kip_Ft, US_Kip_In, US_Lb_Ft, US_Lb_In, SI_kN_m, SI_kN_mm, SI_N_m, SI_N_mm, SI_kgf_m, SI_tonf_m",
            other
        ),
    };

    Ok(canonical.to_string())
}

fn build_extract_request(config: &ext_db::TableSelections) -> TableSelections {
    if !config.is_empty() {
        return TableSelections {
            story_definitions: config.story_definitions.clone().map(convert_table_config),
            pier_section_properties: config
                .pier_section_properties
                .clone()
                .map(convert_table_config),
            base_reactions: config.base_reactions.clone().map(convert_table_config),
            story_forces: config.story_forces.clone().map(convert_table_config),
            joint_drifts: config.joint_drifts.clone().map(convert_table_config),
            pier_forces: config.pier_forces.clone().map(convert_table_config),
            modal_participating_mass_ratios: config
                .modal_participating_mass_ratios
                .clone()
                .map(convert_table_config),
        };
    }

    TableSelections {
        base_reactions: Some(TableSelection {
            load_cases: Some(vec!["*".to_string()]),
            load_combos: Some(vec!["*".to_string()]),
            groups: None,
            field_keys: None,
        }),
        story_forces: Some(TableSelection {
            load_cases: Some(vec!["*".to_string()]),
            load_combos: Some(vec!["*".to_string()]),
            groups: None,
            field_keys: None,
        }),
        joint_drifts: None,
        modal_participating_mass_ratios: Some(TableSelection {
            load_cases: None,
            load_combos: None,
            groups: None,
            field_keys: None,
        }),
        story_definitions: None,
        pier_section_properties: None,
        pier_forces: None,
    }
}

fn convert_table_config(config: ext_db::TableConfig) -> TableSelection {
    TableSelection {
        load_cases: config.load_cases,
        load_combos: config.load_combos,
        groups: config.groups,
        field_keys: config.field_keys,
    }
}

fn build_summary(run_data: &RunAnalysisData) -> AnalysisSummary {
    AnalysisSummary {
        analyzed_at: Utc::now(),
        load_cases: run_data
            .cases_requested
            .clone()
            .unwrap_or_else(|| vec!["*".to_string()]),
        modal: ModalSummary {
            num_modes: 0,
            dominant_period_x: None,
            dominant_period_y: None,
            mass_participation_x: None,
            mass_participation_y: None,
        },
        base_reaction: BaseReactionSummary {
            max_base_shear_x: None,
            max_base_shear_y: None,
        },
        drift: DriftSummary {
            max_drift: None,
            max_drift_story: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ext_db::config::Config;

    #[test]
    fn resolve_sidecar_units_accepts_legacy_project_units() {
        let mut config = Config::default();
        config.project.units = Some("kip-in-F".to_string());
        let ctx = AppContext::for_test(std::env::temp_dir(), config);
        assert_eq!(resolve_sidecar_units(&ctx).unwrap(), "US_Kip_In");
    }
}
