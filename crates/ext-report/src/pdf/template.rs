use crate::pdf::sections;
use crate::report_types::{ReportDocument, ReportSection};

pub fn build_typst_document(document: &ReportDocument) -> String {
    let mut doc = String::new();
    doc.push_str("#set text(font: \"Libertinus Serif\", size: 9pt)\n");
    doc.push_str("#set page(width: 17in, height: 11in, margin: (top: 0.5in, left: 1.0in, right: 0.5in, bottom: 0.5in))\n");
    doc.push_str("#set par(justify: false)\n\n");
    doc.push_str(&title_block_fn());
    doc.push_str(&content_rect_fn());

    for (index, section) in document.sections.iter().enumerate() {
        if index > 0 {
            doc.push_str("\n#pagebreak()\n");
        }

        let body = match section {
            ReportSection::SummaryPage { title, lines } => {
                sections::summary::render_summary_page(document, title, lines)
            }
            ReportSection::SingleChartPage { title, chart } => {
                sections::chart::render_single_chart_page(title, chart)
            }
            ReportSection::TwoChartsPage { title, charts } => {
                sections::chart::render_two_charts_page(title, charts)
            }
            ReportSection::ChartAndTablePage {
                title,
                chart,
                table,
                table_emphasis,
            } => sections::chart::render_chart_and_table_page(title, chart, table, *table_emphasis),
            ReportSection::TableOnlyPage { title, table } => {
                sections::table::render_table_page(title, table)
            }
            ReportSection::CalculationPage { title, blocks } => {
                sections::calculations::render_calculation_page(title, blocks)
            }
        };
        doc.push_str(&wrap_page(document, index + 1, &body));
    }

    doc
}

pub(crate) fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('"', "\\\"")
        .replace('@', "\\@")
        .replace('*', "\\*")
        .replace('_', "\\_")
}

fn title_block_fn() -> String {
    r##"
#let title_block(project, proj_num, reference, engineer, checker, date, subject, scale, sheet, revision) = {
  place(bottom + left)[
    #set text(font: "Libertinus Serif")
    #table(
      columns: (1.35in, 3.2in, 4.0in, 1.6in, 2.0in, 3.35in),
      stroke: 1pt + black,
      inset: 5pt,

      [
        #align(center + horizon)[
          #stack(spacing: 0pt,
            text(size: 11pt, weight: "bold")[Thornton],
            text(size: 11pt, weight: "bold")[Tomasetti],
          )
        ]
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[PROJECT],
          text(size: 8pt, weight: "bold")[#project],
          text(size: 5.5pt, fill: luma(110))[PROJECT NO.],
          text(size: 7.5pt)[#proj_num],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[DRAWING TITLE],
          text(size: 8.5pt, weight: "bold")[#subject],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[REFERENCE],
          text(size: 7.5pt)[#reference],
          text(size: 5.5pt, fill: luma(110))[REVISION],
          text(size: 8pt, weight: "bold")[#revision],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[DRAWN BY],
          text(size: 8pt, weight: "bold")[#engineer],
          text(size: 5.5pt, fill: luma(110))[CHECKED BY],
          text(size: 8pt, weight: "bold")[#checker],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[DATE],
          text(size: 7.5pt)[#date],
          text(size: 5.5pt, fill: luma(110))[SCALE / SHEET],
          text(size: 8pt)[#scale],
          text(size: 14pt, weight: "bold")[#sheet],
        )
      ],
    )
  ]
}

"##
    .to_string()
}

fn wrap_page(document: &ReportDocument, page_number: usize, body: &str) -> String {
    let sheet_prefix = if document.project.sheet_prefix.is_empty() {
        "SK".to_string()
    } else {
        document.project.sheet_prefix.clone()
    };
    let sheet = format!("{sheet_prefix}-{page_number:02}");
    format!(
        "#title_block([{}], [{}], [{}], [{}], [{}], [{}], [{}], [{}], [{}], [{}])\n#content_rect([\n{}\n])\n",
        escape_text(&document.project.project_name),
        escape_text(&document.project.project_number),
        escape_text(&document.project.reference),
        escape_text(&document.project.engineer),
        escape_text(&document.project.checker),
        escape_text(&document.project.date),
        escape_text(&document.project.subject),
        escape_text(&document.project.scale),
        escape_text(&sheet),
        escape_text(&document.project.revision),
        body,
    )
}

fn content_rect_fn() -> String {
    r##"
#let content_rect(body) = rect(
  width: 100%,
  height: 8.85in,
  stroke: 1.2pt + black,
  inset: 18pt,
  body,
)

"##
    .to_string()
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
                sheet_prefix: "SK".to_string(),
            },
        );

        let source = super::build_typst_document(&document);
        assert!(source.contains("width: 17in"));
        assert!(source.contains("height: 11in"));
        assert!(source.contains("left: 1.0in"));
        assert!(source.contains("bottom: 0.5in"));
        assert!(source.contains("#let title_block("));
        assert!(source.contains("font: \"Libertinus Serif\""));
    }

    #[test]
    fn escape_text_handles_quotes_and_mentions() {
        let escaped = super::escape_text("Tower \"A\" @ Main");
        assert_eq!(escaped, "Tower \\\"A\\\" \\@ Main");
    }

    #[test]
    fn escape_text_handles_load_case_names() {
        // ETABS load cases like "DBE_X*Cd/R" must have _ and * escaped
        // to prevent Typst markup interpretation (subscript and bold/emphasis)
        let escaped = super::escape_text("DBE_X*Cd/R");
        assert_eq!(escaped, "DBE\\_X\\*Cd/R");
    }
}
