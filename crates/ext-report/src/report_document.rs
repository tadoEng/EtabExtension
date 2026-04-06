use ext_calc::output::CalcOutput;

use crate::report_types::{ChartLayout, ChartRef, ReportDocument, ReportProjectMeta, ReportSection};

pub fn build_report_document(
    calc: &CalcOutput,
    charts: &[ChartRef],
    project: ReportProjectMeta,
) -> ReportDocument {
    let mut sections = Vec::new();

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
    sections.push(ReportSection::SummaryText {
        title: "Summary".to_string(),
        lines: summary_lines,
    });

    for chart in charts {
        sections.push(ReportSection::ChartBlock {
            title: chart.caption.clone(),
            layout: ChartLayout::SingleChart,
            charts: vec![chart.clone()],
            table: None,
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
