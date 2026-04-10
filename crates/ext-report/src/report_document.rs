use std::collections::HashMap;

use ext_calc::output::{
    BaseShearOutput, CalcOutput, DisplacementOutput, DriftOutput, ModalOutput, PierShearOutput,
};
use ext_render::{
    BASE_SHEAR_IMAGE, DISPLACEMENT_WIND_IMAGE, DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE,
    PIER_AXIAL_IMAGE, PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE,
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

    if let Some(modal) = calc.modal.as_ref() {
        sections.push(ReportSection::TableOnlyPage {
            title: "Modal Participation".to_string(),
            table: build_modal_table(modal),
        });
    }

    if let (Some(base_shear), Some(chart)) = (
        calc.base_shear.as_ref(),
        chart_lookup.get(BASE_SHEAR_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Base Reaction Review".to_string(),
            chart,
            table: build_base_shear_table(base_shear),
            table_emphasis: false,
        });
    }

    if let (Some(drift), Some(chart)) = (
        calc.drift_wind.as_ref(),
        chart_lookup.get(DRIFT_WIND_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Wind Drift Review".to_string(),
            chart,
            table: build_drift_table(drift),
            table_emphasis: true,
        });
    }

    if let (Some(drift), Some(chart)) = (
        calc.drift_seismic.as_ref(),
        chart_lookup.get(DRIFT_SEISMIC_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Seismic Drift Review".to_string(),
            chart,
            table: build_drift_table(drift),
            table_emphasis: true,
        });
    }

    if let (Some(displacement), Some(chart)) = (
        calc.displacement_wind.as_ref(),
        chart_lookup.get(DISPLACEMENT_WIND_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Wind Displacement Review".to_string(),
            chart,
            table: build_displacement_table(displacement),
            table_emphasis: true,
        });
    }

    if let (Some(pier), Some(chart)) = (
        calc.pier_shear_wind.as_ref(),
        chart_lookup.get(PIER_SHEAR_WIND_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Pier Shear Wind Review".to_string(),
            chart,
            table: build_pier_shear_table(pier),
            table_emphasis: false,
        });
    }

    if let (Some(pier), Some(chart)) = (
        calc.pier_shear_seismic.as_ref(),
        chart_lookup.get(PIER_SHEAR_SEISMIC_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::ChartAndTablePage {
            title: "Pier Shear Seismic Review".to_string(),
            chart,
            table: build_pier_shear_table(pier),
            table_emphasis: false,
        });
    }

    if let (Some(_axial), Some(chart)) = (
        calc.pier_axial.as_ref(),
        chart_lookup.get(PIER_AXIAL_IMAGE).cloned(),
    ) {
        sections.push(ReportSection::SingleChartPage {
            title: "Pier Axial Review".to_string(),
            chart,
        });
        sections.push(ReportSection::CalculationPage {
            title: "Pier Axial Assumptions".to_string(),
            blocks: vec![CalculationBlock {
                heading: "Conservative Capacity Basis".to_string(),
                lines: vec![
                    "Nominal capacity uses Po = 0.85f'cAg and ϕPo = ϕ × Po.".to_string(),
                    "Rebar contribution is intentionally excluded from this preliminary axial check.".to_string(),
                    "Fallback f'c currently reuses the seismic pier-shear default when pier/story material matching is unavailable.".to_string(),
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
                // Highlight rows with individually high contribution (>=10%)
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

fn build_base_shear_table(base_shear: &BaseShearOutput) -> KeyValueTable {
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

fn build_drift_table(drift: &DriftOutput) -> KeyValueTable {
    // Group by story: pick the governing-case row (max demand) per story to keep
    // the table concise while showing every story level.
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
    // Convert ft → in for readability; limit.unit carries the authoritative label.
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

fn build_pier_shear_table(pier: &PierShearOutput) -> KeyValueTable {
    // ACI 318-19 §18.10.4.4: maximum nominal shear stress = 8√f'c (psi)
    // fc_ksi is per-row; use the governing pier's value for the limit column header,
    // but compute per-row limit so mixed-fc walls are handled correctly.
    let mut rows: Vec<Vec<String>> = pier
        .piers
        .iter()
        .map(|row| {
            let vu_acv = row.vu.value / row.acv.value; // psi
            let phi_vn_acv = row.phi_vn.value / row.acv.value; // psi
            let fc_psi = row.fc_ksi * 1000.0;
            let limit_8sqrt_fc = 8.0 * fc_psi.sqrt(); // psi
            vec![
                row.story.clone(),
                row.pier_label.clone(),
                row.combo.clone(),
                format!("{:.1}", vu_acv),
                format!("{:.1}", limit_8sqrt_fc),
                format!("{:.1}", phi_vn_acv),
                format!("{:.3}", row.dcr),
                pass_fail(row.pass),
            ]
        })
        .collect();
    // Sort by DCR descending so worst cases appear first.
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
    // Annotate the governing row explicitly.
    if let Some(first) = row_annotations.first_mut() {
        if first.is_none() {
            *first = Some("pass".to_string());
        }
    }

    KeyValueTable {
        title: Some(format!(
            "Governing: {} {} {} ({})",
            pier.governing.story,
            pier.governing.pier_label,
            pier.governing.combo,
            pass_fail(pier.pass)
        )),
        headers: vec![
            "Story".to_string(),
            "Pier".to_string(),
            "Combo".to_string(),
            "Vu/Acv (psi)".to_string(),
            "8\u{221A}f'c (psi)".to_string(),
            "\u{03C6}Vn/Acv (psi)".to_string(),
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

#[cfg(test)]
mod tests {
    use super::build_report_document;
    use crate::report_types::{ChartRef, ReportProjectMeta, ReportSection};
    use ext_calc::output::CalcOutput;
    use ext_render::{
        BASE_SHEAR_IMAGE, DISPLACEMENT_WIND_IMAGE, DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE,
        MODAL_IMAGE, PIER_AXIAL_IMAGE, PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE,
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
            BASE_SHEAR_IMAGE,
            DRIFT_WIND_IMAGE,
            DRIFT_SEISMIC_IMAGE,
            DISPLACEMENT_WIND_IMAGE,
            PIER_SHEAR_WIND_IMAGE,
            PIER_SHEAR_SEISMIC_IMAGE,
            PIER_AXIAL_IMAGE,
        ]
        .into_iter()
        .map(|logical_name| ChartRef {
            logical_name: logical_name.to_string(),
            caption: logical_name.to_string(),
        })
        .collect()
    }

    #[test]
    fn build_report_document_matches_available_fixture_pages() {
        let calc = fixture_calc_output();
        let document = build_report_document(
            &calc,
            &all_chart_refs(),
            ReportProjectMeta {
                project_name: "Proof Tower".to_string(),
                project_number: "P-001".to_string(),
                reference: "CLI-PROOF".to_string(),
                engineer: "Tester".to_string(),
                checker: "Reviewer".to_string(),
                date: "2026-04-06".to_string(),
                subject: "CLI proof report".to_string(),
                scale: "NTS".to_string(),
                revision: "0".to_string(),
                sheet_prefix: "SK".to_string(),
            },
        );

        assert_eq!(document.sections.len(), 6);
        assert!(matches!(
            document.sections[0],
            ReportSection::SummaryPage { .. }
        ));
        assert!(matches!(
            document.sections[1],
            ReportSection::TableOnlyPage { .. }
        ));
        assert!(matches!(
            document.sections[2],
            ReportSection::ChartAndTablePage { .. }
        ));
        assert!(matches!(
            document.sections[5],
            ReportSection::ChartAndTablePage { .. }
        ));
    }
}
