use crate::pdf::template::escape_text;

pub fn render_summary_section(title: &str, lines: &[String]) -> String {
    let mut out = format!("== {}\n\n", escape_text(title));
    for line in lines {
        out.push_str(&format!("- {}\n", escape_text(line)));
    }
    out
}
