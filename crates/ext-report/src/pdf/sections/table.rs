use crate::pdf::template::escape_text;
use crate::report_types::KeyValueTable;

/// Map a row annotation tag to a Typst fill expression string.
/// Returns `"none"` when no fill is needed.
fn annotation_fill(tag: &str) -> &'static str {
    match tag {
        "ux_threshold" => "rgb(\"#cfe2ff\")",  // blue tint
        "uy_threshold" => "rgb(\"#fff3cd\")",  // amber tint
        "ux_uy_threshold" => "rgb(\"#d1c4e9\")", // purple tint
        "high" => "rgb(\"#e8f5e9\")",           // light green – high participation
        "pass" => "rgb(\"#d4edda\")",           // green
        "warn" => "rgb(\"#fff3cd\")",           // amber
        "fail" => "rgb(\"#f8d7da\")",           // red
        _ => "none",
    }
}

pub fn render_table(table: &KeyValueTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return String::new();
    }

    let col_count = table.headers.len();
    let column_spec = vec!["1fr"; col_count].join(", ");
    let has_annotations = !table.row_annotations.is_empty();

    let mut out = String::new();
    if let Some(title) = table.title.as_ref() {
        out.push_str(&format!("#text(size: 9pt, weight: \"bold\")[{}]\n", escape_text(title)));
        out.push_str("#v(6pt)\n");
    }
    out.push_str(&format!("#table(columns: ({column_spec}), stroke: 0.5pt + luma(180), inset: 5pt,\n"));

    // Header row — always grey background
    out.push_str("  table.header(\n");
    for header in &table.headers {
        out.push_str(&format!("    table.cell(fill: luma(220))[*{}*],\n", escape_text(header)));
    }
    out.push_str("  ),\n");

    for (row_idx, row) in table.rows.iter().enumerate() {
        let fill = if has_annotations {
            table
                .row_annotations
                .get(row_idx)
                .and_then(|a| a.as_deref())
                .map(annotation_fill)
                .unwrap_or("none")
        } else {
            "none"
        };

        // Alternate stripe on un-annotated even rows for readability
        let effective_fill = if fill == "none" && row_idx % 2 == 0 {
            "luma(248)"
        } else {
            fill
        };

        for (col_idx, cell) in row.iter().enumerate() {
            // Right-align numeric columns: anything past col 1 that parses as f64.
            // Col 0 (story/mode) and col 1 (case/pier) stay left-aligned.
            let align = if col_idx >= 2 && cell.trim_end_matches('%').parse::<f64>().is_ok() {
                "right"
            } else {
                "left"
            };
            out.push_str(&format!(
                "  table.cell(fill: {}, align: {})[{}],\n",
                effective_fill,
                align,
                escape_text(cell)
            ));
        }
    }

    out.push_str(")\n");
    out
}

pub fn render_table_page(title: &str, table: &KeyValueTable) -> String {
    let mut out = format!("#text(size: 16pt, weight: \"bold\")[{}]\n", escape_text(title));
    out.push_str("#v(10pt)\n");
    out.push_str(&render_table(table));
    out
}
