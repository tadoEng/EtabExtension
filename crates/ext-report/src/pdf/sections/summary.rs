use crate::pdf::template::escape_text;
use crate::report_types::ReportDocument;

pub fn render_summary_page(document: &ReportDocument, title: &str, lines: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "#align(center + horizon)[\n  #text(size: 24pt, weight: \"bold\")[{}]\n  #v(6pt)\n  #text(size: 14pt)[Structural Check Report]\n  #v(4pt)\n  #text(fill: luma(90))[{}]\n]\n",
        escape_text(&document.project.project_name),
        escape_text(&document.project.subject),
    ));
    out.push_str("#v(12pt)\n");
    out.push_str("#grid(columns: (1fr, 1fr), gutter: 16pt,\n");
    out.push_str(&format!(
        "  [*Reference:* {}\\\n*Project No.:* {}\\\n*Revision:* {}\\\n*Branch / Version:* {}/{}],\n",
        escape_text(&document.project.reference),
        escape_text(&document.project.project_number),
        escape_text(&document.project.revision),
        escape_text(&document.branch),
        escape_text(&document.version_id),
    ));
    out.push_str(&format!(
        "  [*Engineer:* {}\\\n*Checker:* {}\\\n*Date:* {}\\\n*Status:* {}],\n)\n",
        escape_text(&document.project.engineer),
        escape_text(&document.project.checker),
        escape_text(&document.project.date),
        escape_text(&document.overall_status),
    ));
    out.push_str("#v(12pt)\n");
    out.push_str(&format!("#text(size: 16pt, weight: \"bold\")[{}]\n", escape_text(title)));
    out.push_str("#v(8pt)\n");
    for line in lines {
        out.push_str(&format!("- {}\n", escape_text(line)));
    }
    out
}
