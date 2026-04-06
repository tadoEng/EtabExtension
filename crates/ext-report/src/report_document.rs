use std::collections::HashMap;

use ext_calc::output::{
    BaseShearOutput, CalcOutput, DisplacementOutput, DriftOutput, ModalOutput, PierShearOutput,
};
use ext_render::{
    BASE_SHEAR_IMAGE, DISPLACEMENT_WIND_IMAGE, DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE,
    PIER_AXIAL_IMAGE, PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE,
};

use crate::report_types::{ChartRef, KeyValueTable, ReportDocument, ReportProjectMeta, ReportSection};

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
        format!("Branch/version: {}/{}", calc.meta.branch, calc.meta.version_id),
    ];
    for line in &calc.summary.lines {
        summary_lines.push(format!("{} [{}] {}", line.key, line.status, line.message));
    }
    summary_lines
}

fn build_modal_table(modal: &ModalOutput) -> KeyValueTable {
    let rows = modal
        .rows
        .iter()
        .map(|row| {
            let highlight = match (
                modal.mode_reaching_ux == Some(row.mode),
                modal.mode_reaching_uy == Some(row.mode),
            ) {
                (true, true) => "UX/UY threshold",
                (true, false) => "UX threshold",
                (false, true) => "UY threshold",
                (false, false) => "",
            };
            vec![
                row.mode.to_string(),
                format!("{:.3}", row.period),
                format!("{:.1}%", row.ux * 100.0),
                format!("{:.1}%", row.uy * 100.0),
                format!("{:.1}%", row.sum_ux * 100.0),
                format!("{:.1}%", row.sum_uy * 100.0),
                highlight.to_string(),
            ]
        })
        .collect();

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

    KeyValueTable {
        title: Some("All extracted base reaction load cases. Gravity pie includes configured gravity cases only.".to_string()),
        headers: vec![
            "Load Case".to_string(),
            "Type".to_string(),
            "Fx".to_string(),
            "Fy".to_string(),
            "Fz".to_string(),
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
    }
}

fn build_drift_table(drift: &DriftOutput) -> KeyValueTable {
    let mut rows = drift
        .rows
        .iter()
        .map(|row| {
            let demand = [
                row.max_drift_x_pos.abs(),
                row.max_drift_x_neg.abs(),
                row.max_drift_y_pos.abs(),
                row.max_drift_y_neg.abs(),
            ]
            .into_iter()
            .fold(0.0_f64, f64::max);
            vec![
                row.story.clone(),
                row.output_case.clone(),
                format!("{:.5}", demand),
                format!("{:.5}", drift.allowable_ratio),
                format!("{:.3}", demand / drift.allowable_ratio),
            ]
        })
        .collect::<Vec<_>>();
    rows.truncate(12);

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
            "Demand".to_string(),
            "Limit".to_string(),
            "DCR".to_string(),
        ],
        rows,
    }
}

fn build_displacement_table(displacement: &DisplacementOutput) -> KeyValueTable {
    let mut rows = displacement
        .rows
        .iter()
        .map(|row| {
            let demand = [
                row.max_disp_x_pos_ft.abs(),
                row.max_disp_x_neg_ft.abs(),
                row.max_disp_y_pos_ft.abs(),
                row.max_disp_y_neg_ft.abs(),
            ]
            .into_iter()
            .fold(0.0_f64, f64::max);
            vec![
                row.story.clone(),
                row.output_case.clone(),
                format!("{:.4}", demand),
                format!("{:.4}", displacement.disp_limit.value),
                format!("{:.3}", demand / displacement.disp_limit.value),
            ]
        })
        .collect::<Vec<_>>();
    rows.truncate(12);

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
            "Demand".to_string(),
            "Limit".to_string(),
            "DCR".to_string(),
        ],
        rows,
    }
}

fn build_pier_shear_table(pier: &PierShearOutput) -> KeyValueTable {
    let mut rows = pier
        .piers
        .iter()
        .map(|row| {
            vec![
                row.story.clone(),
                row.pier_label.clone(),
                row.combo.clone(),
                format!("{:.3}", row.vu.value / row.acv.value),
                format!("{:.3}", row.phi_vn.value / row.acv.value),
                format!("{:.3}", row.dcr),
                pass_fail(row.pass),
            ]
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        let right_dcr = right[5].parse::<f64>().unwrap_or(0.0);
        let left_dcr = left[5].parse::<f64>().unwrap_or(0.0);
        right_dcr.total_cmp(&left_dcr)
    });
    rows.truncate(12);

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
            "Vu/Acv".to_string(),
            "Phi Vn/Acv".to_string(),
            "DCR".to_string(),
            "Status".to_string(),
        ],
        rows,
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
        assert!(matches!(document.sections[0], ReportSection::SummaryPage { .. }));
        assert!(matches!(document.sections[1], ReportSection::TableOnlyPage { .. }));
        assert!(matches!(document.sections[2], ReportSection::ChartAndTablePage { .. }));
        assert!(matches!(document.sections[5], ReportSection::ChartAndTablePage { .. }));
    }
}
