use crate::pdf::sections;
use crate::report_types::{ReportDocument, ReportSection};

pub fn build_typst_document(document: &ReportDocument) -> String {
    let mut doc = String::new();
    doc.push_str("#set text(font: \"Arial\", size: 9pt)\n");
    doc.push_str("#set page(width: 17in, height: 11in, margin: (top: 0.5in, left: 0.5in, right: 0.5in, bottom: 0.7in))\n");
    doc.push_str("#set par(justify: false)\n\n");

    doc.push_str(&format!(
        "#align(center + horizon)[\n  #text(size: 26pt, weight: \"bold\")[{}]\n  #v(10pt)\n  #text(size: 16pt)[Structural Check Report]\n  #v(6pt)\n  #text(fill: rgb(\"#555555\"))[{}]\n]\n\n",
        escape_text(&document.project.project_name),
        escape_text(&document.project.subject),
    ));

    doc.push_str("#grid(columns: (1fr, 1fr), gutter: 18pt,\n");
    doc.push_str(&format!(
        "  [*Reference:* {}\\\n*Project No.:* {}\\\n*Revision:* {}],\n",
        escape_text(&document.project.reference),
        escape_text(&document.project.project_number),
        escape_text(&document.project.revision),
    ));
    doc.push_str(&format!(
        "  [*Engineer:* {}\\\n*Checker:* {}\\\n*Date:* {}],\n)\n\n",
        escape_text(&document.project.engineer),
        escape_text(&document.project.checker),
        escape_text(&document.project.date),
    ));

    for (index, section) in document.sections.iter().enumerate() {
        if index > 0 {
            doc.push_str("\n#pagebreak()\n");
        }

        match section {
            ReportSection::SummaryText { title, lines } => {
                doc.push_str(&sections::summary::render_summary_section(title, lines));
            }
            ReportSection::ChartBlock {
                title,
                layout,
                charts,
                table,
            } => {
                doc.push_str(&sections::chart::render_chart_section(title, layout, charts, table.as_ref()));
            }
            ReportSection::CalculationNotes { title, blocks } => {
                doc.push_str(&sections::calculations::render_calculation_section(title, blocks));
            }
        }
    }

    doc
}

pub(crate) fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
}

#[cfg(test)]
mod tests {
    use crate::build_report_document;
    use crate::report_types::{ChartRef, ReportProjectMeta};
    use ext_calc::output::CalcOutput;
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn typst_document_uses_tabloid_landscape() {
        let calc = fixture_calc_output();
        let document = build_report_document(
            &calc,
            &[ChartRef {
                logical_name: "images/sample.svg".to_string(),
                caption: "Rendered proof chart".to_string(),
            }],
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
            },
        );

        let source = super::build_typst_document(&document);
        assert!(source.contains("width: 17in"));
        assert!(source.contains("height: 11in"));
    }
}
