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
        files.insert(pb("theme.json"),   to_json(theme)?);
        files.insert(pb("project.json"), to_json(project)?);

        // Summary — always present
        files.insert(pb("summary.json"), to_json(&build_summary(calc))?);

        // Per-check — only inject if Some
        if let Some(v) = &calc.modal {
            files.insert(pb("modal.json"), to_json(&build_modal(v))?);
        }
        if let Some(v) = &calc.base_reactions {
            files.insert(pb("base_reactions.json"), to_json(&build_base_reactions(v))?);
        }
        if let Some(v) = &calc.story_forces {
            files.insert(pb("story_forces.json"), to_json(&build_story_forces(v))?);
        }
        if let Some(v) = &calc.drift_wind {
            files.insert(pb("drift_wind.json"), to_json(&DriftReportData {
                x: build_drift_dir(&v.x),
                y: build_drift_dir(&v.y),
            })?);
        }
        if let Some(v) = &calc.drift_seismic {
            files.insert(pb("drift_seismic.json"), to_json(&DriftReportData {
                x: build_drift_dir(&v.x),
                y: build_drift_dir(&v.y),
            })?);
        }
        if let Some(v) = &calc.displacement_wind {
            files.insert(pb("displacement_wind.json"), to_json(&DisplacementReportData {
                x: build_displacement_dir(&v.x),
                y: build_displacement_dir(&v.y),
            })?);
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
            files.insert(pb("pier_shear_seismic.json"), to_json(&build_pier_shear(v))?);
        }

        // SVG charts from ext-render
        for (name, svg) in svg_map {
            files.insert(PathBuf::from(&name), Bytes::new(svg.into_bytes()));
        }

        Ok(Self { files })
    }
}

fn pb(s: &str) -> PathBuf { PathBuf::from(s) }

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
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SummaryLineReport {
    key: String,
    status: String,
    message: String,
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
        lines: calc.summary.lines.iter().map(|l| SummaryLineReport {
            key: l.key.clone(),
            status: l.status.clone(),
            message: l.message.clone(),
        }).collect(),
    }
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
    sum_ux: f64,
    sum_uy: f64,
}

fn build_modal(modal: &ModalOutput) -> ModalReportData {
    let mut rows = Vec::with_capacity(modal.rows.len());
    let mut annotations = Vec::with_capacity(modal.rows.len());

    for row in &modal.rows {
        let is_ux = modal.mode_reaching_ux == Some(row.mode);
        let is_uy = modal.mode_reaching_uy == Some(row.mode);
        let annotation = match (is_ux, is_uy) {
            (true, true)   => "ux_uy_threshold",
            (true, false)  => "ux_threshold",
            (false, true)  => "uy_threshold",
            (false, false) => {
                if row.ux >= 0.10 || row.uy >= 0.10 { "high" } else { "" }
            }
        };
        annotations.push(annotation.to_string());
        rows.push(ModalReportRow {
            mode: row.mode,
            period: row.period,
            ux: row.ux,
            uy: row.uy,
            sum_ux: row.sum_ux,
            sum_uy: row.sum_uy,
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
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct BaseReactionsReportRow {
    output_case: String,
    case_type: String,
    fx_kip: f64,
    fy_kip: f64,
    fz_kip: f64,
}

fn build_base_reactions(base: &BaseReactionsOutput) -> BaseReactionsReportData {
    // Group by output_case, take max absolute values
    let mut grouped = Vec::<(String, String, f64, f64, f64)>::new();
    for row in &base.rows {
        if let Some(existing) = grouped.iter_mut().find(|e| e.0 == row.output_case) {
            existing.2 = existing.2.max(row.fx_kip.abs());
            existing.3 = existing.3.max(row.fy_kip.abs());
            existing.4 = existing.4.max(row.fz_kip.abs());
        } else {
            grouped.push((
                row.output_case.clone(),
                row.case_type.clone(),
                row.fx_kip.abs(),
                row.fy_kip.abs(),
                row.fz_kip.abs(),
            ));
        }
    }

    let row_count = grouped.len();
    let rows = grouped.into_iter().map(|(case, case_type, fx, fy, fz)| {
        BaseReactionsReportRow {
            output_case: wrap_load_case_label(&case),
            case_type,
            fx_kip: fx,
            fy_kip: fy,
            fz_kip: fz,
        }
    }).collect();

    BaseReactionsReportData {
        rows,
        annotations: vec![String::new(); row_count],
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
        rows: story_forces.rows.iter().map(|row| StoryForcesReportRow {
            story: row.story.clone(),
            max_vx_kip: row.max_vx_kip,
            max_my_kip_ft: row.max_my_kip_ft,
            max_vy_kip: row.max_vy_kip,
            max_mx_kip_ft: row.max_mx_kip_ft,
        }).collect(),
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
    rows: Vec<DriftReportRow>,
    annotations: Vec<String>,
    allowable_ratio: f64,
    governing_story: String,
    governing_direction: String,
    governing_case: String,
    pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DriftReportRow {
    story: String,
    output_case: String,
    drift_ratio: f64,
    dcr: f64,
}

fn build_drift_dir(drift: &DriftOutput) -> DriftDirReport {
    // Aggregate per-story max drift
    let mut story_max: Vec<(String, String, f64)> = Vec::new();
    for row in &drift.rows {
        let demand = [
            row.max_drift_x_pos.abs(),
            row.max_drift_x_neg.abs(),
            row.max_drift_y_pos.abs(),
            row.max_drift_y_neg.abs(),
        ].into_iter().fold(0.0_f64, f64::max);

        if let Some(entry) = story_max.iter_mut().find(|e| e.0 == row.story) {
            if demand > entry.2 {
                entry.1 = row.output_case.clone();
                entry.2 = demand;
            }
        } else {
            story_max.push((row.story.clone(), row.output_case.clone(), demand));
        }
    }

    let mut rows = Vec::with_capacity(story_max.len());
    let mut annotations = Vec::with_capacity(story_max.len());

    for (story, case, demand) in &story_max {
        let dcr = demand / drift.allowable_ratio;
        let annotation = if dcr >= 1.0 { "fail" }
                         else if dcr >= 0.85 { "warn" }
                         else { "" };
        annotations.push(annotation.to_string());
        rows.push(DriftReportRow {
            story: story.clone(),
            output_case: case.clone(),
            drift_ratio: *demand,
            dcr,
        });
    }

    DriftDirReport {
        rows,
        annotations,
        allowable_ratio: drift.allowable_ratio,
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
    rows: Vec<DisplacementReportRow>,
    annotations: Vec<String>,
    limit_in: f64,
    governing_story: String,
    governing_direction: String,
    governing_case: String,
    pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct DisplacementReportRow {
    story: String,
    output_case: String,
    demand_in: f64,
    dcr: f64,
}

fn build_displacement_dir(disp: &DisplacementOutput) -> DisplacementDirReport {
    let to_in = |ft: f64| ft * 12.0;
    let limit_in = to_in(disp.disp_limit.value);

    // Aggregate per-story max displacement
    let mut story_max: Vec<(String, String, f64)> = Vec::new();
    for row in &disp.rows {
        let demand = [
            row.max_disp_x_pos_ft.abs(),
            row.max_disp_x_neg_ft.abs(),
            row.max_disp_y_pos_ft.abs(),
            row.max_disp_y_neg_ft.abs(),
        ].into_iter().fold(0.0_f64, f64::max);

        if let Some(entry) = story_max.iter_mut().find(|e| e.0 == row.story) {
            if demand > entry.2 {
                entry.1 = row.output_case.clone();
                entry.2 = demand;
            }
        } else {
            story_max.push((row.story.clone(), row.output_case.clone(), demand));
        }
    }

    let mut rows = Vec::with_capacity(story_max.len());
    let mut annotations = Vec::with_capacity(story_max.len());

    for (story, case, demand_ft) in &story_max {
        let demand_in = to_in(*demand_ft);
        let dcr = demand_in / limit_in;
        let annotation = if dcr >= 1.0 { "fail" }
                         else if dcr >= 0.85 { "warn" }
                         else { "" };
        annotations.push(annotation.to_string());
        rows.push(DisplacementReportRow {
            story: story.clone(),
            output_case: case.clone(),
            demand_in,
            dcr,
        });
    }

    DisplacementDirReport {
        rows,
        annotations,
        limit_in,
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
    max_ratio: f64,
    has_type_a: bool,
    has_type_b: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct TorsionalReportRow {
    story: String,
    case: String,
    joint_a: String,
    joint_b: String,
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
    let mut rows = Vec::with_capacity(dir.rows.len());
    let mut annotations = Vec::with_capacity(dir.rows.len());

    for row in &dir.rows {
        let annotation = if row.is_type_b { "fail" }
                         else if row.is_type_a { "warn" }
                         else { "" };
        annotations.push(annotation.to_string());
        rows.push(TorsionalReportRow {
            story: row.story.clone(),
            case: wrap_load_case_label(&row.case),
            joint_a: row.joint_a.clone(),
            joint_b: row.joint_b.clone(),
            ratio: row.ratio,
            is_type_a: row.is_type_a,
            is_type_b: row.is_type_b,
            ax: row.ax,
            ecc_ft: row.ecc_ft,
        });
    }

    TorsionalDirReport {
        rows,
        annotations,
        governing_story: dir.governing_story.clone(),
        governing_case: dir.governing_case.clone(),
        max_ratio: dir.max_ratio,
        has_type_a: dir.has_type_a,
        has_type_b: dir.has_type_b,
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
    rows: Vec<PierShearReportRow>,
    annotations: Vec<String>,
    pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PierShearReportRow {
    story: String,
    pier: String,
    combo: String,
    stress_psi: f64,
    limit_individual: f64,
    stress_ratio: f64,
    dcr: f64,
    pass: bool,
}

fn build_pier_shear(pier: &ext_calc::output::PierShearStressOutput) -> PierShearReportData {
    let mut rows: Vec<PierShearReportRow> = pier
        .per_pier
        .iter()
        .map(|row| {
            let dcr = row.stress_ratio / row.limit_individual;
            PierShearReportRow {
                story: row.story.clone(),
                pier: row.pier.clone(),
                combo: wrap_load_case_label(&row.combo),
                stress_psi: row.stress_psi,
                limit_individual: row.limit_individual,
                stress_ratio: row.stress_ratio,
                dcr,
                pass: row.pass,
            }
        })
        .collect();

    // Sort by DCR descending (approximating old logic)
    rows.sort_by(|left, right| right.dcr.total_cmp(&left.dcr));

    let annotations: Vec<String> = rows
        .iter()
        .map(|row| {
            if row.dcr >= 1.0 {
                "fail".to_string()
            } else if row.dcr >= 0.85 {
                "warn".to_string()
            } else {
                "".to_string()
            }
        })
        .collect();

    PierShearReportData {
        rows,
        annotations,
        pass: pier.pass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::TABLOID_LANDSCAPE;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
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
        assert_eq!(value.get("code").and_then(|v| v.as_str()), Some(calc.meta.code.as_str()));
    }
}
