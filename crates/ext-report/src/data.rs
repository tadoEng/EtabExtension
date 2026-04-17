use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use typst::foundations::Bytes;

use ext_calc::output::{
    BaseReactionsOutput, CalcOutput, DisplacementOutput, DriftOutput, ModalOutput,
    PierAxialStressOutput, StoryForcesOutput, TorsionalDirectionOutput, TorsionalOutput,
};

use crate::theme::PageTheme;

// ── Project metadata (moved from report_types.rs, now Serialize) ────────────

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ReportProjectMeta {
    pub project_name: String,
    pub project_number: String,
    pub reference: String,
    pub engineer: String,
    pub checker: String,
    pub date: String,
    pub subject: String,
    pub scale: String,
    pub revision: String,
    pub sheet_prefix: String,
}

// ── ReportData — the serialization gateway ──────────────────────────────────

pub struct ReportData {
    pub files: HashMap<PathBuf, Bytes>,
}

impl ReportData {
    pub fn from_calc(
        calc: &CalcOutput,
        project: &ReportProjectMeta,
        theme: &PageTheme,
        svg_map: HashMap<String, String>,
    ) -> Result<Self> {
        let mut files = HashMap::new();

        // Always present
        files.insert(pb("theme.json"), to_json(theme)?);
        files.insert(pb("project.json"), to_json(project)?);

        // Summary — always present
        files.insert(pb("summary.json"), to_json(&build_summary(calc))?);

        // Per-check — only inject if Some
        if let Some(v) = &calc.modal {
            files.insert(pb("modal.json"), to_json(&build_modal(v))?);
        }
        if let Some(v) = &calc.base_reactions {
            files.insert(
                pb("base_reactions.json"),
                to_json(&build_base_reactions(v))?,
            );
        }
        if let Some(v) = &calc.story_forces {
            files.insert(pb("story_forces.json"), to_json(&build_story_forces(v))?);
        }
        if let Some(v) = &calc.drift_wind {
            files.insert(
                pb("drift_wind.json"),
                to_json(&DriftReportData {
                    x: build_drift_dir(&v.x),
                    y: build_drift_dir(&v.y),
                })?,
            );
        }
        if let Some(v) = &calc.drift_seismic {
            files.insert(
                pb("drift_seismic.json"),
                to_json(&DriftReportData {
                    x: build_drift_dir(&v.x),
                    y: build_drift_dir(&v.y),
                })?,
            );
        }
        if let Some(v) = &calc.displacement_wind {
            files.insert(
                pb("displacement_wind.json"),
                to_json(&DisplacementReportData {
                    x: build_displacement_dir(&v.x),
                    y: build_displacement_dir(&v.y),
                })?,
            );
        }
        if let Some(v) = &calc.torsional {
            files.insert(pb("torsional.json"), to_json(&build_torsional(v))?);
        }
        if let Some(v) = &calc.pier_axial_stress {
            files.insert(pb("pier_axial_stress.json"), to_json(&build_pier_axial(v))?);
        }
        if let Some(v) = &calc.pier_shear_stress_wind {
            files.insert(pb("pier_shear_wind.json"), to_json(&build_pier_shear(v))?);
        }
        if let Some(v) = &calc.pier_shear_stress_seismic {
            files.insert(
                pb("pier_shear_seismic.json"),
                to_json(&build_pier_shear(v))?,
            );
        }

        // SVG charts from ext-render
        for (name, svg) in svg_map {
            files.insert(PathBuf::from(&name), Bytes::new(svg.into_bytes()));
        }

        Ok(Self { files })
    }
}

fn pb(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn to_json<T: Serialize>(v: &T) -> Result<Bytes> {
    Ok(Bytes::new(serde_json::to_vec(v)?))
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Report-ready JSON types — kebab-case for Typst field access
// ═══════════════════════════════════════════════════════════════════════════════

// ── Summary ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SummaryReportData {
    overall_status: String,
    check_count: u32,
    pass_count: u32,
    fail_count: u32,
    branch: String,
    version_id: String,
    code: String,
    lines: Vec<SummaryLineReport>,
    checker_rows: Vec<SummaryCheckerRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SummaryLineReport {
    key: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SummaryCheckerRow {
    check: String,
    status: String,
    governing_case: String,
    governing_story: String,
    demand: String,
    limit: String,
    utilization: String,
    margin: String,
    reason: String,
}

fn build_summary(calc: &CalcOutput) -> SummaryReportData {
    SummaryReportData {
        overall_status: calc.summary.overall_status.clone(),
        check_count: calc.summary.check_count,
        pass_count: calc.summary.pass_count,
        fail_count: calc.summary.fail_count,
        branch: calc.meta.branch.clone(),
        version_id: calc.meta.version_id.clone(),
        code: calc.meta.code.clone(),
        lines: calc
            .summary
            .lines
            .iter()
            .map(|l| SummaryLineReport {
                key: l.key.clone(),
                status: l.status.clone(),
                message: l.message.clone(),
            })
            .collect(),
        checker_rows: build_checker_rows(calc),
    }
}

fn build_checker_rows(calc: &CalcOutput) -> Vec<SummaryCheckerRow> {
    let mut rows = Vec::new();

    if let Some(value) = &calc.drift_wind {
        rows.push(build_drift_checker_row("Drift (Wind)", &value.x, &value.y));
    }
    if let Some(value) = &calc.drift_seismic {
        rows.push(build_drift_checker_row(
            "Drift (Seismic)",
            &value.x,
            &value.y,
        ));
    }
    if let Some(value) = &calc.displacement_wind {
        rows.push(build_displacement_checker_row(
            "Displacement (Wind)",
            &value.x,
            &value.y,
        ));
    }
    if let Some(value) = &calc.torsional {
        rows.push(build_torsional_checker_row("Torsional", value));
    }
    if let Some(value) = &calc.base_reactions {
        rows.push(build_base_reactions_checker_row("Base Reactions", value));
    }
    if let Some(value) = &calc.pier_shear_stress_wind {
        rows.push(build_pier_shear_checker_row("Pier Shear (Wind)", value));
    }
    if let Some(value) = &calc.pier_shear_stress_seismic {
        rows.push(build_pier_shear_checker_row("Pier Shear (Seismic)", value));
    }
    if let Some(value) = &calc.pier_axial_stress {
        rows.push(build_pier_axial_checker_row("Pier Axial Screening", value));
    }

    rows
}

fn build_drift_checker_row(
    check: &str,
    x: &ext_calc::output::DriftOutput,
    y: &ext_calc::output::DriftOutput,
) -> SummaryCheckerRow {
    let (governing, direction) = if x.governing.dcr >= y.governing.dcr {
        (&x.governing, "X")
    } else {
        (&y.governing, "Y")
    };
    let limit = if x.allowable_ratio > 0.0 {
        x.allowable_ratio
    } else {
        y.allowable_ratio
    };
    let utilization = governing.dcr;
    let margin = 1.0 - utilization;
    let status = if x.pass && y.pass { "pass" } else { "fail" };
    SummaryCheckerRow {
        check: check.to_string(),
        status: status.to_string(),
        governing_case: governing.output_case.clone(),
        governing_story: governing.story.clone(),
        demand: fmt_float(governing.drift_ratio),
        limit: fmt_float(limit),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: format!(
            "Direction {direction}, drift ratio demand/allowable check ({:.5} / {:.5})",
            governing.drift_ratio, limit
        ),
    }
}

fn build_displacement_checker_row(
    check: &str,
    x: &ext_calc::output::DisplacementOutput,
    y: &ext_calc::output::DisplacementOutput,
) -> SummaryCheckerRow {
    let (governing, selected, direction) = if x.governing.dcr >= y.governing.dcr {
        (&x.governing, x, "X")
    } else {
        (&y.governing, y, "Y")
    };
    let limit_ft = selected
        .story_limits
        .iter()
        .find(|row| row.story == governing.story)
        .map(|row| row.limit_ft)
        .unwrap_or(selected.disp_limit.value);
    let utilization = governing.dcr;
    let margin = 1.0 - utilization;
    let status = if x.pass && y.pass { "pass" } else { "fail" };
    SummaryCheckerRow {
        check: check.to_string(),
        status: status.to_string(),
        governing_case: governing.output_case.clone(),
        governing_story: governing.story.clone(),
        demand: fmt_with_unit(governing.displacement.value * 12.0, "in"),
        limit: fmt_with_unit(limit_ft * 12.0, "in"),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: format!(
            "Direction {direction}, per-level displacement limit from elevation/limit ratio"
        ),
    }
}

fn build_torsional_checker_row(check: &str, torsional: &TorsionalOutput) -> SummaryCheckerRow {
    let x_ratio = torsional.x.max_ratio;
    let y_ratio = torsional.y.max_ratio;
    let (direction, governing) = if x_ratio >= y_ratio {
        ("X", &torsional.x)
    } else {
        ("Y", &torsional.y)
    };
    let threshold = 1.4;
    let utilization = if threshold > 0.0 {
        governing.max_ratio / threshold
    } else {
        0.0
    };
    let margin = threshold - governing.max_ratio;
    let classification = if governing.has_type_b {
        "Type B"
    } else if governing.has_type_a {
        "Type A"
    } else {
        "None"
    };
    SummaryCheckerRow {
        check: check.to_string(),
        status: if torsional.pass { "pass" } else { "fail" }.to_string(),
        governing_case: governing.governing_case.clone(),
        governing_story: governing.governing_story.clone(),
        demand: fmt_float(governing.max_ratio),
        limit: fmt_float(threshold),
        utilization: fmt_percent(utilization),
        margin: fmt_float(margin),
        reason: format!(
            "Direction {direction}, ratio basis dmax/davg, classification {classification}"
        ),
    }
}

fn build_base_reactions_checker_row(check: &str, base: &BaseReactionsOutput) -> SummaryCheckerRow {
    let min_ratio = base.direction_x.ratio.min(base.direction_y.ratio);
    let utilization = min_ratio;
    let margin = min_ratio - 1.0;
    let pass = base.direction_x.pass && base.direction_y.pass;
    SummaryCheckerRow {
        check: check.to_string(),
        status: if pass { "pass" } else { "fail" }.to_string(),
        governing_case: format!(
            "X {} / Y {}",
            base.direction_x.rsa_case, base.direction_y.rsa_case
        ),
        governing_story: "-".to_string(),
        demand: fmt_float(min_ratio),
        limit: ">= 1.000".to_string(),
        utilization: fmt_percent(utilization),
        margin: fmt_float(margin),
        reason: format!(
            "RSA/ELF ratio check, X={:.5}, Y={:.5}",
            base.direction_x.ratio, base.direction_y.ratio
        ),
    }
}

fn build_pier_shear_checker_row(
    check: &str,
    value: &ext_calc::output::PierShearStressOutput,
) -> SummaryCheckerRow {
    if !value.supported {
        return SummaryCheckerRow {
            check: check.to_string(),
            status: "warn".to_string(),
            governing_case: "-".to_string(),
            governing_story: "-".to_string(),
            demand: "-".to_string(),
            limit: "-".to_string(),
            utilization: "-".to_string(),
            margin: "-".to_string(),
            reason: value
                .support_note
                .clone()
                .unwrap_or_else(|| "Check is not available for configured code".to_string()),
        };
    }

    let util_individual = if value.limit_individual > 0.0 {
        value.max_individual_ratio / value.limit_individual
    } else {
        0.0
    };
    let util_average = if value.limit_average > 0.0 {
        value.max_average_ratio / value.limit_average
    } else {
        0.0
    };
    let utilization = util_individual.max(util_average);
    let margin = 1.0 - utilization;
    SummaryCheckerRow {
        check: check.to_string(),
        status: if value.pass { "pass" } else { "fail" }.to_string(),
        governing_case: "-".to_string(),
        governing_story: "-".to_string(),
        demand: format!(
            "ind {:.3}, avg {:.3}",
            value.max_individual_ratio, value.max_average_ratio
        ),
        limit: format!(
            "ind {:.3}, avg {:.3}",
            value.limit_individual, value.limit_average
        ),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: "Stress-ratio checks against individual and average limits".to_string(),
    }
}

fn build_pier_axial_checker_row(check: &str, value: &PierAxialStressOutput) -> SummaryCheckerRow {
    SummaryCheckerRow {
        check: check.to_string(),
        status: if value.pass { "pass" } else { "fail" }.to_string(),
        governing_case: value.governing.combo.clone(),
        governing_story: value.governing.story.clone(),
        demand: format!(
            "{} (signed {})",
            fmt_with_unit(value.governing.fa.value, "ksi"),
            fmt_with_unit(value.governing.fa_signed.value, "ksi")
        ),
        limit: fmt_with_unit(0.85 * value.governing.fc_ksi, "ksi"),
        utilization: fmt_percent(value.governing.dcr),
        margin: fmt_percent(1.0 - value.governing.dcr),
        reason: "Preliminary axial screening check".to_string(),
    }
}

fn fmt_float(value: f64) -> String {
    format!("{value:.3}")
}

fn fmt_percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

fn fmt_with_unit(value: f64, unit: &str) -> String {
    format!("{} {}", fmt_float(value), unit)
}

// ── Modal ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ModalReportData {
    threshold: f64,
    pass: bool,
    rows: Vec<ModalReportRow>,
    annotations: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ModalReportRow {
    mode: i64,
    period: f64,
    ux: f64,
    uy: f64,
    uz: f64,
    sum_ux: f64,
    sum_uy: f64,
    sum_uz: f64,
}

fn build_modal(modal: &ModalOutput) -> ModalReportData {
    let mut rows = Vec::with_capacity(modal.rows.len());
    let mut annotations = Vec::with_capacity(modal.rows.len());

    for row in &modal.rows {
        let is_ux = modal.mode_reaching_ux == Some(row.mode);
        let is_uy = modal.mode_reaching_uy == Some(row.mode);
        let annotation = match (is_ux, is_uy) {
            (true, true) => "ux_uy_threshold",
            (true, false) => "ux_threshold",
            (false, true) => "uy_threshold",
            (false, false) => {
                if row.ux >= 0.10 || row.uy >= 0.10 {
                    "high"
                } else {
                    ""
                }
            }
        };
        annotations.push(annotation.to_string());
        rows.push(ModalReportRow {
            mode: row.mode,
            period: row.period,
            ux: row.ux,
            uy: row.uy,
            uz: row.uz,
            sum_ux: row.sum_ux,
            sum_uy: row.sum_uy,
            sum_uz: row.sum_uz,
        });
    }

    ModalReportData {
        threshold: modal.threshold,
        pass: modal.pass,
        rows,
        annotations,
    }
}

// ── Base Reactions ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct BaseReactionsReportData {
    rows: Vec<BaseReactionsReportRow>,
    annotations: Vec<String>,
    pass: bool,
    ratio_x: f64,
    ratio_y: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct BaseReactionsReportRow {
    output_case: String,
    case_type: String,
    step_type: String,
    step_number: Option<f64>,
    fx_kip: f64,
    fy_kip: f64,
    fz_kip: f64,
    mx_kip_ft: f64,
    my_kip_ft: f64,
    mz_kip_ft: f64,
}

fn build_base_reactions(base: &BaseReactionsOutput) -> BaseReactionsReportData {
    let rows = base
        .rows
        .iter()
        .filter(|row| !should_exclude_base_case_type(&row.case_type))
        .map(|row| BaseReactionsReportRow {
            output_case: wrap_load_case_label(&row.output_case),
            case_type: row.case_type.clone(),
            step_type: row.step_type.clone(),
            step_number: row.step_number,
            fx_kip: row.fx_kip.abs(),
            fy_kip: row.fy_kip.abs(),
            fz_kip: row.fz_kip.abs(),
            mx_kip_ft: row.mx_kip_ft.abs(),
            my_kip_ft: row.my_kip_ft.abs(),
            mz_kip_ft: row.mz_kip_ft.abs(),
        })
        .collect::<Vec<_>>();
    let row_count = rows.len();

    BaseReactionsReportData {
        rows,
        annotations: vec![String::new(); row_count],
        pass: base.direction_x.pass && base.direction_y.pass,
        ratio_x: base.direction_x.ratio,
        ratio_y: base.direction_y.ratio,
    }
}

// ── Story Forces ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StoryForcesReportData {
    rows: Vec<StoryForcesReportRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StoryForcesReportRow {
    story: String,
    max_vx_kip: f64,
    max_my_kip_ft: f64,
    max_vy_kip: f64,
    max_mx_kip_ft: f64,
}

fn build_story_forces(story_forces: &StoryForcesOutput) -> StoryForcesReportData {
    StoryForcesReportData {
        rows: story_forces
            .rows
            .iter()
            .map(|row| StoryForcesReportRow {
                story: row.story.clone(),
                max_vx_kip: row.max_vx_kip,
                max_my_kip_ft: row.max_my_kip_ft,
                max_vy_kip: row.max_vy_kip,
                max_mx_kip_ft: row.max_mx_kip_ft,
            })
            .collect(),
    }
}

fn wrap_load_case_label(value: &str) -> String {
    const SOFT_WRAP: char = '\u{200B}';
    let mut out = String::with_capacity(value.len() + 16);
    for ch in value.chars() {
        out.push(ch);
        if matches!(ch, '_' | '+' | '-' | '/' | ':' | ')') {
            out.push(SOFT_WRAP);
        }
    }
    out
}

fn round5(value: f64) -> f64 {
    (value * 100_000.0).round() / 100_000.0
}

fn should_exclude_base_case_type(case_type: &str) -> bool {
    let normalized = case_type.trim().to_ascii_lowercase();
    normalized == "combination" || normalized == "linmodritz" || normalized == "eigen"
}

// ── Drift ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DriftReportData {
    x: DriftDirReport,
    y: DriftDirReport,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DriftDirReport {
    levels: Vec<String>,
    groups: Vec<String>,
    matrix: Vec<Vec<Option<f64>>>,
    allowable_ratio: f64,
    governing_demand_ratio: f64,
    governing_utilization: f64,
    governing_margin_ratio: f64,
    governing_story: String,
    governing_direction: String,
    governing_case: String,
    pass: bool,
}

fn build_drift_dir(drift: &DriftOutput) -> DriftDirReport {
    let levels = drift
        .story_order
        .iter()
        .filter(|story| drift.rows.iter().any(|row| &row.story == *story))
        .cloned()
        .collect::<Vec<_>>();
    let groups = ordered_unique(drift.rows.iter().map(|row| row.group_name.clone()));
    let is_x = drift.governing.direction.eq_ignore_ascii_case("X");

    let mut values: HashMap<(String, String), f64> = HashMap::new();
    for row in &drift.rows {
        let demand = if is_x {
            row.max_drift_x_pos.abs().max(row.max_drift_x_neg.abs())
        } else {
            row.max_drift_y_pos.abs().max(row.max_drift_y_neg.abs())
        };
        let key = (row.story.clone(), row.group_name.clone());
        let entry = values.entry(key).or_insert(0.0);
        *entry = entry.max(demand);
    }

    let mut matrix = Vec::with_capacity(levels.len());
    for level in &levels {
        let mut row_values = Vec::with_capacity(groups.len());
        for group in &groups {
            row_values.push(values.get(&(level.clone(), group.clone())).copied());
        }
        matrix.push(row_values);
    }

    DriftDirReport {
        levels,
        groups,
        matrix,
        allowable_ratio: drift.allowable_ratio,
        governing_demand_ratio: drift.governing.drift_ratio,
        governing_utilization: drift.governing.dcr,
        governing_margin_ratio: drift.allowable_ratio - drift.governing.drift_ratio,
        governing_story: drift.governing.story.clone(),
        governing_direction: drift.governing.direction.clone(),
        governing_case: drift.governing.output_case.clone(),
        pass: drift.pass,
    }
}

// ── Displacement ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DisplacementReportData {
    x: DisplacementDirReport,
    y: DisplacementDirReport,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DisplacementDirReport {
    levels: Vec<String>,
    groups: Vec<String>,
    matrix_in: Vec<Vec<Option<f64>>>,
    level_elevations_ft: Vec<f64>,
    level_limits_in: Vec<f64>,
    level_max_demand_in: Vec<f64>,
    level_utilization: Vec<f64>,
    governing_limit_in: f64,
    governing_utilization: f64,
    governing_margin: f64,
    governing_story: String,
    governing_direction: String,
    governing_case: String,
    pass: bool,
}

fn build_displacement_dir(disp: &DisplacementOutput) -> DisplacementDirReport {
    let to_in = |ft: f64| ft * 12.0;
    let levels = disp
        .story_order
        .iter()
        .filter(|story| disp.rows.iter().any(|row| &row.story == *story))
        .cloned()
        .collect::<Vec<_>>();
    let groups = ordered_unique(disp.rows.iter().map(|row| row.group_name.clone()));
    let is_x = disp.governing.direction.eq_ignore_ascii_case("X");

    let mut values: HashMap<(String, String), f64> = HashMap::new();
    for row in &disp.rows {
        let demand_ft = if is_x {
            row.max_disp_x_pos_ft.abs().max(row.max_disp_x_neg_ft.abs())
        } else {
            row.max_disp_y_pos_ft.abs().max(row.max_disp_y_neg_ft.abs())
        };
        let key = (row.story.clone(), row.group_name.clone());
        let entry = values.entry(key).or_insert(0.0);
        *entry = entry.max(demand_ft);
    }

    let mut limits_by_level = HashMap::new();
    let mut elevations_by_level = HashMap::new();
    for row in &disp.story_limits {
        limits_by_level.insert(row.story.clone(), to_in(row.limit_ft));
        elevations_by_level.insert(row.story.clone(), row.elevation_ft);
    }

    let mut matrix_in = Vec::with_capacity(levels.len());
    let mut level_max_demand_in = Vec::with_capacity(levels.len());
    let mut level_utilization = Vec::with_capacity(levels.len());
    let level_elevations_ft = levels
        .iter()
        .map(|level| elevations_by_level.get(level).copied().unwrap_or(0.0))
        .collect::<Vec<_>>();
    for level in &levels {
        let mut row_values = Vec::with_capacity(groups.len());
        for group in &groups {
            row_values.push(
                values
                    .get(&(level.clone(), group.clone()))
                    .copied()
                    .map(to_in),
            );
        }
        let max_demand = row_values.iter().flatten().copied().fold(0.0_f64, f64::max);
        let limit_in = limits_by_level
            .get(level)
            .copied()
            .unwrap_or(to_in(disp.disp_limit.value));
        let utilization = if limit_in > 1e-9 {
            max_demand / limit_in
        } else {
            0.0
        };
        matrix_in.push(row_values);
        level_max_demand_in.push(max_demand);
        level_utilization.push(utilization);
    }
    let level_limits_in = levels
        .iter()
        .map(|level| {
            limits_by_level
                .get(level)
                .copied()
                .unwrap_or(to_in(disp.disp_limit.value))
        })
        .collect();
    let governing_limit_in = limits_by_level
        .get(&disp.governing.story)
        .copied()
        .unwrap_or(to_in(disp.disp_limit.value));

    DisplacementDirReport {
        levels,
        groups,
        matrix_in,
        level_elevations_ft,
        level_limits_in,
        level_max_demand_in,
        level_utilization,
        governing_limit_in,
        governing_utilization: disp.governing.dcr,
        governing_margin: 1.0 - disp.governing.dcr,
        governing_story: disp.governing.story.clone(),
        governing_direction: disp.governing.direction.clone(),
        governing_case: disp.governing.output_case.clone(),
        pass: disp.pass,
    }
}

// ── Torsional ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct TorsionalReportData {
    x: TorsionalDirReport,
    y: TorsionalDirReport,
    pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct TorsionalDirReport {
    rows: Vec<TorsionalReportRow>,
    annotations: Vec<String>,
    governing_story: String,
    governing_case: String,
    governing_joint_a: String,
    governing_joint_b: String,
    governing_step: Option<i32>,
    governing_drift_a: f64,
    governing_drift_b: f64,
    governing_delta_max: f64,
    governing_delta_avg: f64,
    governing_ratio: f64,
    type_a_threshold: f64,
    type_b_threshold: f64,
    classification: String,
    has_type_a: bool,
    has_type_b: bool,
    has_rows: bool,
    no_data_note: String,
    no_data_contexts: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
struct TorsionalReportRow {
    story: String,
    case: String,
    joint_a: String,
    joint_b: String,
    governing_step: i32,
    drift_a: f64,
    drift_b: f64,
    delta_max: f64,
    delta_avg: f64,
    ratio: f64,
    is_type_a: bool,
    is_type_b: bool,
    ax: f64,
    ecc_ft: f64,
}

fn build_torsional(torsional: &TorsionalOutput) -> TorsionalReportData {
    TorsionalReportData {
        x: build_torsional_dir(&torsional.x),
        y: build_torsional_dir(&torsional.y),
        pass: torsional.pass,
    }
}

fn build_torsional_dir(dir: &TorsionalDirectionOutput) -> TorsionalDirReport {
    let type_a_threshold = 1.2;
    let type_b_threshold = 1.4;
    let mut rows = Vec::with_capacity(dir.rows.len());
    let mut annotations = Vec::with_capacity(dir.rows.len());

    for row in &dir.rows {
        let annotation = if row.is_type_b {
            "fail"
        } else if row.is_type_a {
            "warn"
        } else {
            ""
        };
        annotations.push(annotation.to_string());
        rows.push(TorsionalReportRow {
            story: row.story.clone(),
            case: wrap_load_case_label(&row.case),
            joint_a: row.joint_a.clone(),
            joint_b: row.joint_b.clone(),
            governing_step: row.governing_step,
            drift_a: row.governing_drift_a,
            drift_b: row.governing_drift_b,
            delta_max: row.governing_delta_max,
            delta_avg: row.governing_delta_avg,
            ratio: row.governing_ratio,
            is_type_a: row.is_type_a,
            is_type_b: row.is_type_b,
            ax: row.ax,
            ecc_ft: row.ecc_ft,
        });
    }

    let governing_row = rows
        .iter()
        .max_by(|left, right| {
            left.ratio
                .partial_cmp(&right.ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();
    let classification = if dir.has_type_b {
        "Type B".to_string()
    } else if dir.has_type_a {
        "Type A".to_string()
    } else if rows.is_empty() {
        "No data".to_string()
    } else {
        "None".to_string()
    };
    let has_rows = !rows.is_empty();

    TorsionalDirReport {
        rows,
        annotations,
        governing_story: dir.governing_story.clone(),
        governing_case: dir.governing_case.clone(),
        governing_joint_a: governing_row
            .as_ref()
            .map(|row| row.joint_a.clone())
            .or_else(|| dir.governing_joints.first().cloned())
            .unwrap_or_default(),
        governing_joint_b: governing_row
            .as_ref()
            .map(|row| row.joint_b.clone())
            .or_else(|| dir.governing_joints.get(1).cloned())
            .unwrap_or_default(),
        governing_step: dir.governing_step,
        governing_drift_a: governing_row
            .as_ref()
            .map(|row| row.drift_a)
            .unwrap_or_default(),
        governing_drift_b: governing_row
            .as_ref()
            .map(|row| row.drift_b)
            .unwrap_or_default(),
        governing_delta_max: governing_row
            .as_ref()
            .map(|row| row.delta_max)
            .unwrap_or_default(),
        governing_delta_avg: governing_row
            .as_ref()
            .map(|row| row.delta_avg)
            .unwrap_or_default(),
        governing_ratio: dir.max_ratio,
        type_a_threshold,
        type_b_threshold,
        classification,
        has_type_a: dir.has_type_a,
        has_type_b: dir.has_type_b,
        has_rows,
        no_data_note: if has_rows {
            String::new()
        } else {
            "No qualifying rows for configured pairs and cases.".to_string()
        },
        no_data_contexts: dir
            .no_data
            .iter()
            .map(|row| {
                format!(
                    "story={} case={} pair={}/{} step={} missing={}",
                    row.story,
                    row.case,
                    row.joint_a,
                    row.joint_b,
                    row.step,
                    row.missing.join(", ")
                )
            })
            .collect(),
    }
}

// ── Pier Axial (minimal — for assumptions page) ──────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PierAxialReportData {
    phi_axial: f64,
    pass: bool,
}

fn build_pier_axial(axial: &PierAxialStressOutput) -> PierAxialReportData {
    PierAxialReportData {
        phi_axial: axial.phi_axial,
        pass: axial.pass,
    }
}

// ── Pier Shear ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PierShearReportData {
    supported: bool,
    support_note: String,
    levels: Vec<String>,
    piers: Vec<String>,
    matrix_ratio: Vec<Vec<Option<f64>>>,
    limit_individual_ratio: f64,
    average_rows: Vec<PierShearAverageReportRow>,
    limit_average_ratio: f64,
    max_individual_ratio: f64,
    max_average_ratio: f64,
    pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PierShearAverageReportRow {
    story: String,
    limit_average: f64,
    x_average_stress_ratio: Option<f64>,
    y_average_stress_ratio: Option<f64>,
}

fn build_pier_shear(pier: &ext_calc::output::PierShearStressOutput) -> PierShearReportData {
    if !pier.supported {
        return PierShearReportData {
            supported: false,
            support_note: pier.support_note.clone().unwrap_or_default(),
            levels: Vec::new(),
            piers: Vec::new(),
            matrix_ratio: Vec::new(),
            limit_individual_ratio: pier.limit_individual,
            average_rows: Vec::new(),
            limit_average_ratio: pier.limit_average,
            max_individual_ratio: pier.max_individual_ratio,
            max_average_ratio: pier.max_average_ratio,
            pass: pier.pass,
        };
    }

    let levels = pier.story_order.clone();
    let mut piers = pier
        .per_pier
        .iter()
        .map(|row| row.pier.clone())
        .filter(|label| !is_default_pier_label(label))
        .collect::<Vec<_>>();
    piers = ordered_unique(piers.into_iter());
    piers = normalized_pier_labels(piers);

    let mut values: HashMap<(String, String), f64> = HashMap::new();
    for row in &pier.per_pier {
        if is_default_pier_label(&row.pier) {
            continue;
        }
        let key = (row.story.clone(), row.pier.clone());
        let entry = values.entry(key).or_insert(0.0);
        *entry = entry.max(row.stress_ratio);
    }

    let mut matrix_ratio = Vec::with_capacity(levels.len());
    for level in &levels {
        let mut row_values = Vec::with_capacity(piers.len());
        for pier_name in &piers {
            row_values.push(values.get(&(level.clone(), pier_name.clone())).copied());
        }
        matrix_ratio.push(row_values);
    }

    let mut x_avg = HashMap::new();
    let mut y_avg = HashMap::new();
    for row in &pier.x_average {
        x_avg.insert(row.story.clone(), row.avg_stress_ratio);
    }
    for row in &pier.y_average {
        y_avg.insert(row.story.clone(), row.avg_stress_ratio);
    }
    let average_rows = levels
        .iter()
        .map(|story| PierShearAverageReportRow {
            story: story.clone(),
            limit_average: pier.limit_average,
            x_average_stress_ratio: x_avg.get(story).copied(),
            y_average_stress_ratio: y_avg.get(story).copied(),
        })
        .collect::<Vec<_>>();

    PierShearReportData {
        supported: true,
        support_note: String::new(),
        levels,
        piers,
        matrix_ratio,
        limit_individual_ratio: pier.limit_individual,
        average_rows,
        limit_average_ratio: pier.limit_average,
        max_individual_ratio: pier.max_individual_ratio,
        max_average_ratio: pier.max_average_ratio,
        pass: pier.pass,
    }
}

fn ordered_unique(iter: impl Iterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in iter {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn is_default_pier_label(label: &str) -> bool {
    let trimmed = label.trim();
    trimmed.is_empty() || trimmed == "0"
}

fn normalized_pier_labels(labels: Vec<String>) -> Vec<String> {
    let mut out = labels;
    out.sort_by(|left, right| compare_pier_labels(left, right));
    out
}

fn compare_pier_labels(left: &str, right: &str) -> std::cmp::Ordering {
    let left_key = pier_label_key(left);
    let right_key = pier_label_key(right);
    left_key
        .0
        .cmp(&right_key.0)
        .then_with(|| left_key.1.cmp(&right_key.1))
        .then_with(|| natural_cmp(left, right))
}

fn pier_label_key(label: &str) -> (u8, u32) {
    if let Some(num) = parse_prefixed_number(label, "PX") {
        return (0, num);
    }
    if let Some(num) = parse_prefixed_number(label, "PY") {
        return (1, num);
    }
    (2, u32::MAX)
}

fn parse_prefixed_number(label: &str, prefix: &str) -> Option<u32> {
    let upper = label.trim().to_ascii_uppercase();
    if !upper.starts_with(prefix) {
        return None;
    }
    let suffix = &upper[prefix.len()..];
    if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    suffix.parse::<u32>().ok()
}

fn natural_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let mut li = left.chars().peekable();
    let mut ri = right.chars().peekable();

    loop {
        match (li.peek(), ri.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lc), Some(rc)) if lc.is_ascii_digit() && rc.is_ascii_digit() => {
                let mut l_num = String::new();
                let mut r_num = String::new();
                while let Some(ch) = li.peek() {
                    if ch.is_ascii_digit() {
                        l_num.push(*ch);
                        li.next();
                    } else {
                        break;
                    }
                }
                while let Some(ch) = ri.peek() {
                    if ch.is_ascii_digit() {
                        r_num.push(*ch);
                        ri.next();
                    } else {
                        break;
                    }
                }
                let l_val = l_num.parse::<u64>().unwrap_or(0);
                let r_val = r_num.parse::<u64>().unwrap_or(0);
                match l_val.cmp(&r_val) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (Some(_), Some(_)) => {
                let l = li.next().unwrap().to_ascii_lowercase();
                let r = ri.next().unwrap().to_ascii_lowercase();
                match l.cmp(&r) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::TABLOID_LANDSCAPE;
    use ext_calc::CalcRunner;
    use ext_calc::code_params::CodeParams;
    use ext_db::config::Config;

    fn fixture_calc_output() -> CalcOutput {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic");
        let path = fixture_dir.join("calc_output.json");
        if path.exists() {
            let text = std::fs::read_to_string(path).unwrap();
            serde_json::from_str(&text).unwrap()
        } else {
            let config = Config::load(&fixture_dir).unwrap();
            let params = CodeParams::from_config(&config).unwrap();
            CalcRunner::run_all(
                fixture_dir.as_path(),
                fixture_dir.as_path(),
                &params,
                "fixture",
                "main",
            )
            .unwrap()
        }
    }

    fn sample_project() -> ReportProjectMeta {
        ReportProjectMeta {
            project_name: "Project Test".to_string(),
            project_number: "v1".to_string(),
            reference: "main/v1".to_string(),
            engineer: "Preview".to_string(),
            checker: "Preview".to_string(),
            date: "2026-04-15".to_string(),
            subject: "Fixture report".to_string(),
            scale: "NTS".to_string(),
            revision: "0".to_string(),
            sheet_prefix: "SK".to_string(),
        }
    }

    #[test]
    fn report_data_includes_story_forces_json_when_present() {
        let calc = fixture_calc_output();
        assert!(calc.story_forces.is_some());
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        assert!(
            report_data
                .files
                .contains_key(&PathBuf::from("story_forces.json"))
        );
    }

    #[test]
    fn summary_json_includes_calc_code() {
        let calc = fixture_calc_output();
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        let bytes = report_data
            .files
            .get(&PathBuf::from("summary.json"))
            .expect("summary.json must exist");
        let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();
        assert_eq!(
            value.get("code").and_then(|v| v.as_str()),
            Some(calc.meta.code.as_str())
        );
    }

    #[test]
    fn base_reactions_json_excludes_unscoped_case_types() {
        let calc = fixture_calc_output();
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        let bytes = report_data
            .files
            .get(&PathBuf::from("base_reactions.json"))
            .expect("base_reactions.json must exist");
        let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();
        let rows = value
            .get("rows")
            .and_then(|node| node.as_array())
            .expect("rows should be array");
        assert!(rows.iter().all(|row| {
            let case_type = row
                .get("case-type")
                .and_then(|node| node.as_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            case_type != "combination" && case_type != "linmodritz" && case_type != "eigen"
        }));
    }
}
