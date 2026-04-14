use std::collections::HashMap;

use ext_calc::output::{
    BaseReactionsOutput, CalcOutput, DisplacementOutput, DriftOutput, ModalOutput,
    PierShearStressOutput, StoryForcesOutput, TorsionalDirectionOutput,
};
use ext_render::{
    BASE_REACTIONS_IMAGE,
    DISPLACEMENT_WIND_X_IMAGE, DISPLACEMENT_WIND_Y_IMAGE,
    DRIFT_SEISMIC_X_IMAGE, DRIFT_SEISMIC_Y_IMAGE, DRIFT_WIND_X_IMAGE, DRIFT_WIND_Y_IMAGE,
    PIER_AXIAL_GRAVITY_IMAGE, PIER_AXIAL_SEISMIC_IMAGE, PIER_AXIAL_WIND_IMAGE,
    PIER_SHEAR_STRESS_SEISMIC_IMAGE, PIER_SHEAR_STRESS_WIND_IMAGE,
    STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
};

use crate::report_types::{
    CalculationBlock, ChartRef, KeyValueTable, ReportDocument, ReportProjectMeta, ReportSection,
};

pub fn build_report_document(
    calc: &CalcOutput,
    charts: &[ChartRef],
    project: ReportProjectMeta,
) -> ReportDocument {
    let chart_lookup = charts
        .iter()
        .cloned()
        .map(|chart| (chart.logical_name.clone(), chart))
        .collect::<HashMap<_, _>>();

    let mut sections = vec![ReportSection::SummaryPage {
        title: "Report Summary".to_string(),
        lines: build_summary_lines(calc),
    }];

    // ── Modal ────────────────────────────────────────────────────────────────
    if let Some(modal) = calc.modal.as_ref() {
        sections.push(ReportSection::TableOnlyPage {
            title: "Modal Participation".to_string(),
            table: build_modal_table(modal),
        });
    }

    // ── Base Reactions ───────────────────────────────────────────────────────
    if let (Some(base_shear), Some(chart)) = (
        calc.base_reactions.as_ref(),
        chart_lookup.get(BASE_REACTIONS_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Base Reaction Review".to_string(),
            chart,
            table: build_base_shear_table(base_shear),
            table_emphasis: false,
        });
    }

    // ── Story Forces — X page (VX + MY) ─────────────────────────────────────
    if let Some(sf) = calc.story_forces.as_ref() {
        if let (Some(vx_chart), Some(my_chart)) = (
            chart_lookup.get(STORY_FORCE_VX_IMAGE).cloned(),
            chart_lookup.get(STORY_FORCE_MY_IMAGE).cloned(),
        ) {
            sections.push(ReportSection::TwoChartsPage {
                title: "Story Forces — X Direction".to_string(),
                charts: vec![vx_chart, my_chart],
            });
        }

        // ── Story Forces — Y page (VY + MX) ─────────────────────────────────
        if let (Some(vy_chart), Some(mx_chart)) = (
            chart_lookup.get(STORY_FORCE_VY_IMAGE).cloned(),
            chart_lookup.get(STORY_FORCE_MX_IMAGE).cloned(),
        ) {
            sections.push(ReportSection::TwoChartsPage {
                title: "Story Forces — Y Direction".to_string(),
                charts: vec![vy_chart, mx_chart],
            });
        }

        // Fall back: story forces table if no charts were found
        if chart_lookup.get(STORY_FORCE_VX_IMAGE).is_none()
            && chart_lookup.get(STORY_FORCE_VY_IMAGE).is_none()
        {
            sections.push(ReportSection::TableOnlyPage {
                title: "Story Forces".to_string(),
                table: build_story_forces_table(sf),
            });
        }
    }

    // ── Wind Drift ───────────────────────────────────────────────────────────
    if let Some(drift) = calc.drift_wind.as_ref() {
        if let Some(chart) = chart_lookup.get(DRIFT_WIND_X_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Wind Drift Review (X)".to_string(),
                chart,
                table: build_drift_table(&drift.x),
                table_emphasis: true,
            });
        }
        if let Some(chart) = chart_lookup.get(DRIFT_WIND_Y_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Wind Drift Review (Y)".to_string(),
                chart,
                table: build_drift_table(&drift.y),
                table_emphasis: true,
            });
        }
    }

    // ── Seismic Drift ────────────────────────────────────────────────────────
    if let Some(drift) = calc.drift_seismic.as_ref() {
        if let Some(chart) = chart_lookup.get(DRIFT_SEISMIC_X_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Seismic Drift Review (X)".to_string(),
                chart,
                table: build_drift_table(&drift.x),
                table_emphasis: true,
            });
        }
        if let Some(chart) = chart_lookup.get(DRIFT_SEISMIC_Y_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Seismic Drift Review (Y)".to_string(),
                chart,
                table: build_drift_table(&drift.y),
                table_emphasis: true,
            });
        }
    }

    // ── Wind Displacement ────────────────────────────────────────────────────
    if let Some(displacement) = calc.displacement_wind.as_ref() {
        if let Some(chart) = chart_lookup.get(DISPLACEMENT_WIND_X_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Wind Displacement Review (X)".to_string(),
                chart,
                table: build_displacement_table(&displacement.x),
                table_emphasis: true,
            });
        }
        if let Some(chart) = chart_lookup.get(DISPLACEMENT_WIND_Y_IMAGE).cloned() {
            sections.push(ReportSection::ChartAndTablePage {
                title: "Wind Displacement Review (Y)".to_string(),
                chart,
                table: build_displacement_table(&displacement.y),
                table_emphasis: true,
            });
        }
    }

    // ── Torsional — X table page ─────────────────────────────────────────────
    if let Some(torsional) = calc.torsional.as_ref() {
        if !torsional.x.rows.is_empty() {
            sections.push(ReportSection::TableOnlyPage {
                title: "Torsional Irregularity — X Direction".to_string(),
                table: build_torsional_table(&torsional.x),
            });
        }
        if !torsional.y.rows.is_empty() {
            sections.push(ReportSection::TableOnlyPage {
                title: "Torsional Irregularity — Y Direction".to_string(),
                table: build_torsional_table(&torsional.y),
            });
        }
    }

    // ── Pier Shear ───────────────────────────────────────────────────────────
    if let (Some(pier), Some(chart)) = (
        calc.pier_shear_stress_wind.as_ref(),
        chart_lookup.get(PIER_SHEAR_STRESS_WIND_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Pier Shear Wind Review".to_string(),
            chart,
            table: build_pier_shear_table(pier),
            table_emphasis: false,
        });
    }

    if let (Some(pier), Some(chart)) = (
        calc.pier_shear_stress_seismic.as_ref(),
        chart_lookup.get(PIER_SHEAR_STRESS_SEISMIC_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Pier Shear Seismic Review".to_string(),
            chart,
            table: build_pier_shear_table(pier),
            table_emphasis: false,
        });
    }

    // ── Pier Axial — 3 category pages ────────────────────────────────────────
    if calc.pier_axial_stress.is_some() {
        let axial_chart_data = [
            (PIER_AXIAL_GRAVITY_IMAGE, "Pier Axial — Gravity"),
            (PIER_AXIAL_WIND_IMAGE,    "Pier Axial — Wind"),
            (PIER_AXIAL_SEISMIC_IMAGE, "Pier Axial — Seismic"),
        ];
        for (key, title) in axial_chart_data {
            if let Some(chart) = chart_lookup.get(key).cloned() {
                sections.push(ReportSection::SingleChartPage {
                    title: title.to_string(),
                    chart,
                });
            }
        }

        // Axial assumptions note (always shown when axial check is present).
        sections.push(ReportSection::CalculationPage {
            title: "Pier Axial Assumptions".to_string(),
            blocks: vec![CalculationBlock {
                heading: "Conservative Capacity Basis".to_string(),
                lines: vec![
                    "Nominal capacity uses Po = 0.85f'cAg and ϕPo = ϕ × Po.".to_string(),
                    "Rebar contribution is intentionally excluded from this preliminary axial check.".to_string(),
                    "Fallback f'c reuses the pier section material default when pier/story material matching is unavailable.".to_string(),
                    "Results are split by load category: gravity combos (gravity), wind combos (wind), seismic combos (seismic).".to_string(),
                ],
            }],
        });
    }

    ReportDocument {
        project,
        branch: calc.meta.branch.clone(),
        version_id: calc.meta.version_id.clone(),
        overall_status: calc.summary.overall_status.clone(),
        check_count: calc.summary.check_count,
        pass_count: calc.summary.pass_count,
        fail_count: calc.summary.fail_count,
        sections,
    }
}

// ── Table builders ───────────────────────────────────────────────────────────

fn build_summary_lines(calc: &CalcOutput) -> Vec<String> {
    let mut summary_lines = vec![
        format!("Overall status: {}", calc.summary.overall_status),
        format!("Active checks: {}", calc.summary.check_count),
        format!("Passed: {}", calc.summary.pass_count),
        format!("Failed: {}", calc.summary.fail_count),
        format!(
            "Branch/version: {}/{}",
            calc.meta.branch, calc.meta.version_id
        ),
    ];
    for line in &calc.summary.lines {
        summary_lines.push(format!("{} [{}] {}", line.key, line.status, line.message));
    }
    summary_lines
}

fn build_modal_table(modal: &ModalOutput) -> KeyValueTable {
    let mut rows = Vec::new();
    let mut row_annotations: Vec<Option<String>> = Vec::new();

    for row in &modal.rows {
        let is_ux = modal.mode_reaching_ux == Some(row.mode);
        let is_uy = modal.mode_reaching_uy == Some(row.mode);
        let annotation = match (is_ux, is_uy) {
            (true, true) => Some("ux_uy_threshold".to_string()),
            (true, false) => Some("ux_threshold".to_string()),
            (false, true) => Some("uy_threshold".to_string()),
            (false, false) => {
                if row.ux >= 0.10 || row.uy >= 0.10 {
                    Some("high".to_string())
                } else {
                    None
                }
            }
        };
        let highlight_label = match annotation.as_deref() {
            Some("ux_threshold") => "UX threshold",
            Some("uy_threshold") => "UY threshold",
            Some("ux_uy_threshold") => "UX/UY threshold",
            _ => "",
        };
        rows.push(vec![
            row.mode.to_string(),
            format!("{:.3}", row.period),
            format!("{:.1}%", row.ux * 100.0),
            format!("{:.1}%", row.uy * 100.0),
            format!("{:.1}%", row.sum_ux * 100.0),
            format!("{:.1}%", row.sum_uy * 100.0),
            highlight_label.to_string(),
        ]);
        row_annotations.push(annotation);
    }

    KeyValueTable {
        title: Some(format!(
            "Mass participation threshold = {:.0}%",
            modal.threshold * 100.0
        )),
        headers: vec![
            "Mode".to_string(),
            "Period".to_string(),
            "UX".to_string(),
            "UY".to_string(),
            "Sum UX".to_string(),
            "Sum UY".to_string(),
            "Highlight".to_string(),
        ],
        rows,
        row_annotations,
    }
}

fn build_base_shear_table(base_shear: &BaseReactionsOutput) -> KeyValueTable {
    let mut grouped = Vec::<(String, String, f64, f64, f64)>::new();
    for row in &base_shear.rows {
        if let Some(existing) = grouped.iter_mut().find(|entry| entry.0 == row.output_case) {
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
    KeyValueTable {
        title: Some("All extracted base reaction load cases. Gravity pie includes configured gravity cases only.".to_string()),
        headers: vec![
            "Load Case".to_string(),
            "Type".to_string(),
            "Fx (kip)".to_string(),
            "Fy (kip)".to_string(),
            "Fz (kip)".to_string(),
        ],
        rows: grouped
            .into_iter()
            .map(|(case_name, case_type, fx, fy, fz)| {
                vec![
                    case_name,
                    case_type,
                    format!("{fx:.1}"),
                    format!("{fy:.1}"),
                    format!("{fz:.1}"),
                ]
            })
            .collect(),
        row_annotations: vec![None; row_count],
    }
}

fn build_story_forces_table(sf: &StoryForcesOutput) -> KeyValueTable {
    let row_count = sf.rows.len();
    KeyValueTable {
        title: Some("Story force envelope: maximum absolute values per story.".to_string()),
        headers: vec![
            "Story".to_string(),
            "Vx (kip)".to_string(),
            "Vy (kip)".to_string(),
            "My (kip·ft)".to_string(),
            "Mx (kip·ft)".to_string(),
        ],
        rows: sf
            .rows
            .iter()
            .map(|r| {
                vec![
                    r.story.clone(),
                    format!("{:.1}", r.max_vx_kip),
                    format!("{:.1}", r.max_vy_kip),
                    format!("{:.1}", r.max_my_kip_ft),
                    format!("{:.1}", r.max_mx_kip_ft),
                ]
            })
            .collect(),
        row_annotations: vec![None; row_count],
    }
}

fn build_drift_table(drift: &DriftOutput) -> KeyValueTable {
    let mut story_max: Vec<(String, String, f64)> = Vec::new();
    for row in &drift.rows {
        let demand = [
            row.max_drift_x_pos.abs(),
            row.max_drift_x_neg.abs(),
            row.max_drift_y_pos.abs(),
            row.max_drift_y_neg.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        if let Some(entry) = story_max.iter_mut().find(|e| e.0 == row.story) {
            if demand > entry.2 {
                entry.1 = row.output_case.clone();
                entry.2 = demand;
            }
        } else {
            story_max.push((row.story.clone(), row.output_case.clone(), demand));
        }
    }

    let mut row_annotations: Vec<Option<String>> = Vec::new();
    let rows = story_max
        .iter()
        .map(|(story, case, demand)| {
            let dcr = demand / drift.allowable_ratio;
            let annotation = if dcr >= 1.0 {
                Some("fail".to_string())
            } else if dcr >= 0.85 {
                Some("warn".to_string())
            } else {
                None
            };
            row_annotations.push(annotation);
            vec![
                story.clone(),
                case.clone(),
                format!("{:.5}", demand),
                format!("{:.5}", drift.allowable_ratio),
                format!("{:.3}", dcr),
            ]
        })
        .collect::<Vec<_>>();

    KeyValueTable {
        title: Some(format!(
            "Governing: {} {} {} ({})",
            drift.governing.story,
            drift.governing.direction,
            drift.governing.output_case,
            pass_fail(drift.pass)
        )),
        headers: vec![
            "Story".to_string(),
            "Case".to_string(),
            "Demand (ratio)".to_string(),
            "Limit (ratio)".to_string(),
            "DCR".to_string(),
        ],
        rows,
        row_annotations,
    }
}

fn build_displacement_table(displacement: &DisplacementOutput) -> KeyValueTable {
    let to_in = |ft: f64| ft * 12.0;
    let limit_in = to_in(displacement.disp_limit.value);

    let mut story_max: Vec<(String, String, f64)> = Vec::new();
    for row in &displacement.rows {
        let demand = [
            row.max_disp_x_pos_ft.abs(),
            row.max_disp_x_neg_ft.abs(),
            row.max_disp_y_pos_ft.abs(),
            row.max_disp_y_neg_ft.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        if let Some(entry) = story_max.iter_mut().find(|e| e.0 == row.story) {
            if demand > entry.2 {
                entry.1 = row.output_case.clone();
                entry.2 = demand;
            }
        } else {
            story_max.push((row.story.clone(), row.output_case.clone(), demand));
        }
    }

    let mut row_annotations: Vec<Option<String>> = Vec::new();
    let rows = story_max
        .iter()
        .map(|(story, case, demand_ft)| {
            let demand_in = to_in(*demand_ft);
            let dcr = demand_in / limit_in;
            let annotation = if dcr >= 1.0 {
                Some("fail".to_string())
            } else if dcr >= 0.85 {
                Some("warn".to_string())
            } else {
                None
            };
            row_annotations.push(annotation);
            vec![
                story.clone(),
                case.clone(),
                format!("{:.4}", demand_in),
                format!("{:.4}", limit_in),
                format!("{:.3}", dcr),
            ]
        })
        .collect::<Vec<_>>();

    KeyValueTable {
        title: Some(format!(
            "Governing: {} {} {} ({})",
            displacement.governing.story,
            displacement.governing.direction,
            displacement.governing.output_case,
            pass_fail(displacement.pass)
        )),
        headers: vec![
            "Story".to_string(),
            "Case".to_string(),
            "Demand (in)".to_string(),
            "Limit (in)".to_string(),
            "DCR".to_string(),
        ],
        rows,
        row_annotations,
    }
}

fn build_torsional_table(dir: &TorsionalDirectionOutput) -> KeyValueTable {
    let row_count = dir.rows.len();
    let mut row_annotations: Vec<Option<String>> = Vec::with_capacity(row_count);

    let rows = dir
        .rows
        .iter()
        .map(|row| {
            let annotation = if row.is_type_b {
                Some("fail".to_string())
            } else if row.is_type_a {
                Some("warn".to_string())
            } else {
                None
            };
            row_annotations.push(annotation);
            vec![
                row.story.clone(),
                row.case.clone(),
                row.joint_a.clone(),
                row.joint_b.clone(),
                format!("{:.3}", row.ratio),
                format!("{}", if row.is_type_a { "Type A" } else { "—" }),
                format!("{}", if row.is_type_b { "Type B" } else { "—" }),
                format!("{:.2}", row.ax),
                format!("{:.2}", row.ecc_ft),
            ]
        })
        .collect::<Vec<_>>();

    KeyValueTable {
        title: Some(format!(
            "Governing story: {} | Case: {} | Max ratio: {:.3} | {}",
            dir.governing_story,
            dir.governing_case,
            dir.max_ratio,
            if dir.has_type_b {
                "⚠ TYPE B IRREGULARITY"
            } else if dir.has_type_a {
                "Type A irregularity"
            } else {
                "No irregularity"
            }
        )),
        headers: vec![
            "Story".to_string(),
            "Case".to_string(),
            "Joint A".to_string(),
            "Joint B".to_string(),
            "Δmax/Δavg".to_string(),
            "Type A (>1.2)".to_string(),
            "Type B (>1.4)".to_string(),
            "Ax".to_string(),
            "Ecc (ft)".to_string(),
        ],
        rows,
        row_annotations,
    }
}

fn build_pier_shear_table(pier: &PierShearStressOutput) -> KeyValueTable {
    let mut rows: Vec<Vec<String>> = pier
        .per_pier
        .iter()
        .map(|row| {
            let dcr = row.stress_ratio / row.limit_individual;
            vec![
                row.story.clone(),
                row.pier.clone(),
                row.combo.clone(),
                format!("{:.1}", row.stress_psi),
                format!("{:.1}", row.limit_individual),
                format!("{:.3}", row.stress_ratio),
                format!("{:.3}", dcr),
                pass_fail(row.pass),
            ]
        })
        .collect();
    rows.sort_by(|left, right| {
        let right_dcr = right[6].parse::<f64>().unwrap_or(0.0);
        let left_dcr = left[6].parse::<f64>().unwrap_or(0.0);
        right_dcr.total_cmp(&left_dcr)
    });

    let mut row_annotations: Vec<Option<String>> = rows
        .iter()
        .map(|row| {
            let dcr = row[6].parse::<f64>().unwrap_or(0.0);
            if dcr >= 1.0 {
                Some("fail".to_string())
            } else if dcr >= 0.85 {
                Some("warn".to_string())
            } else {
                None
            }
        })
        .collect();
    if let Some(first) = row_annotations.first_mut() {
        if first.is_none() {
            *first = Some("pass".to_string());
        }
    }

    KeyValueTable {
        title: Some(format!(
            "Pier Shear Results. Passed = {}",
            pass_fail(pier.pass)
        )),
        headers: vec![
            "Story".to_string(),
            "Pier".to_string(),
            "Combo".to_string(),
            "Vu/Acv (psi)".to_string(),
            "Limit (n\u{221A}f'c)".to_string(),
            "Stress Ratio".to_string(),
            "DCR".to_string(),
            "Status".to_string(),
        ],
        rows,
        row_annotations,
    }
}

fn pass_fail(pass: bool) -> String {
    if pass {
        "PASS".to_string()
    } else {
        "FAIL".to_string()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::build_report_document;
    use crate::report_types::{ChartRef, ReportProjectMeta, ReportSection};
    use ext_calc::output::CalcOutput;
    use ext_render::{
        BASE_REACTIONS_IMAGE,
        DISPLACEMENT_WIND_X_IMAGE, DISPLACEMENT_WIND_Y_IMAGE,
        DRIFT_SEISMIC_X_IMAGE, DRIFT_SEISMIC_Y_IMAGE, DRIFT_WIND_X_IMAGE, DRIFT_WIND_Y_IMAGE,
        MODAL_IMAGE,
        PIER_AXIAL_GRAVITY_IMAGE, PIER_AXIAL_SEISMIC_IMAGE, PIER_AXIAL_WIND_IMAGE,
        PIER_SHEAR_STRESS_SEISMIC_IMAGE, PIER_SHEAR_STRESS_WIND_IMAGE,
        STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
    };
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn all_chart_refs() -> Vec<ChartRef> {
        [
            MODAL_IMAGE,
            BASE_REACTIONS_IMAGE,
            STORY_FORCE_VX_IMAGE,
            STORY_FORCE_VY_IMAGE,
            STORY_FORCE_MY_IMAGE,
            STORY_FORCE_MX_IMAGE,
            DRIFT_WIND_X_IMAGE,
            DRIFT_WIND_Y_IMAGE,
            DRIFT_SEISMIC_X_IMAGE,
            DRIFT_SEISMIC_Y_IMAGE,
            DISPLACEMENT_WIND_X_IMAGE,
            DISPLACEMENT_WIND_Y_IMAGE,
            PIER_SHEAR_STRESS_WIND_IMAGE,
            PIER_SHEAR_STRESS_SEISMIC_IMAGE,
            PIER_AXIAL_GRAVITY_IMAGE,
            PIER_AXIAL_WIND_IMAGE,
            PIER_AXIAL_SEISMIC_IMAGE,
        ]
        .into_iter()
        .map(|logical_name| ChartRef {
            logical_name: logical_name.to_string(),
            caption: logical_name.to_string(),
        })
        .collect()
    }

    fn test_project() -> ReportProjectMeta {
        ReportProjectMeta {
            project_name: "Proof Tower".to_string(),
            project_number: "P-001".to_string(),
            reference: "CLI-PROOF".to_string(),
            engineer: "Tester".to_string(),
            checker: "Reviewer".to_string(),
            date: "2026-04-14".to_string(),
            subject: "CLI proof report".to_string(),
            scale: "NTS".to_string(),
            revision: "0".to_string(),
            sheet_prefix: "SK".to_string(),
        }
    }

    #[test]
    fn build_report_document_matches_available_fixture_pages() {
        let calc = fixture_calc_output();
        let document = build_report_document(&calc, &all_chart_refs(), test_project());

        // The fixture calc_output.json has these active checks:
        //   modal, baseReactions, storyForces, driftWind, driftSeismic,
        //   displacementWind, pierShearStressWind, pierShearStressSeismic, pierAxialStress
        //   torsional = null (not configured in fixture)
        //
        // Sections generated:
        //   1  SummaryPage
        //   2  TableOnlyPage          modal
        //   3  ChartAndTablePage      base_reactions
        //   4  TwoChartsPage          story_forces X (VX + MY)
        //   5  TwoChartsPage          story_forces Y (VY + MX)
        //   6  ChartAndTablePage      drift_wind_x
        //   7  ChartAndTablePage      drift_wind_y
        //   8  ChartAndTablePage      drift_seismic_x
        //   9  ChartAndTablePage      drift_seismic_y
        //  10  ChartAndTablePage      displacement_wind_x
        //  11  ChartAndTablePage      displacement_wind_y
        //     [torsional = null → 0 pages]
        //  12  ChartAndTablePage      pier_shear_wind
        //  13  ChartAndTablePage      pier_shear_seismic
        //  14  SingleChartPage        pier_axial gravity
        //  15  SingleChartPage        pier_axial wind
        //  16  SingleChartPage        pier_axial seismic
        //  17  CalculationPage        pier_axial assumptions
        //  = 17 sections
        assert_eq!(document.sections.len(), 17);

        assert!(matches!(document.sections[0], ReportSection::SummaryPage { .. }));
        assert!(matches!(document.sections[1], ReportSection::TableOnlyPage { .. })); // modal
        assert!(matches!(document.sections[2], ReportSection::ChartAndTablePage { .. })); // base reactions
        assert!(matches!(document.sections[3], ReportSection::TwoChartsPage { .. })); // story forces X
        assert!(matches!(document.sections[4], ReportSection::TwoChartsPage { .. })); // story forces Y
        assert!(matches!(document.sections[16], ReportSection::CalculationPage { .. })); // pier axial assumptions
    }

    #[test]
    fn report_builds_when_torsional_is_absent() {
        let mut calc = fixture_calc_output();
        calc.torsional = None;
        let document = build_report_document(&calc, &all_chart_refs(), test_project());
        // Sections count stays at 17 (torsional was already null in fixture).
        assert_eq!(document.sections.len(), 17);
    }

    #[test]
    fn report_builds_when_story_forces_is_absent() {
        let mut calc = fixture_calc_output();
        calc.story_forces = None;
        let document = build_report_document(&calc, &all_chart_refs(), test_project());
        // Loses the 2 story-forces pages → 15 sections.
        assert_eq!(document.sections.len(), 15);
    }

    #[test]
    fn report_builds_when_one_axial_category_absent_in_chart_refs() {
        let calc = fixture_calc_output();
        // Omit the seismic axial chart ref — only gravity + wind refs provided.
        let charts: Vec<ChartRef> = all_chart_refs()
            .into_iter()
            .filter(|c| c.logical_name != PIER_AXIAL_SEISMIC_IMAGE)
            .collect();
        let document = build_report_document(&calc, &charts, test_project());
        // Loses 1 axial page (seismic) → 16 sections.
        assert_eq!(document.sections.len(), 16);
    }
}
