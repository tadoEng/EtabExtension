use crate::pdf::template::escape_text;
use crate::report_types::CalculationBlock;

pub fn render_calculation_section(title: &str, blocks: &[CalculationBlock]) -> String {
    let mut out = format!("== {}\n\n", escape_text(title));

    for block in blocks {
        out.push_str(&format!("=== {}\n\n", escape_text(&block.heading)));
        for line in &block.lines {
            out.push_str(&format!("- {}\n", escape_text(line)));
        }
        out.push('\n');
    }

    out
}
