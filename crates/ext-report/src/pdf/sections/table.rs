use crate::pdf::template::escape_text;
use crate::report_types::KeyValueTable;

pub fn render_table(table: &KeyValueTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return String::new();
    }

    let column_spec = vec!["1fr"; table.headers.len()].join(", ");
    let mut out = String::new();
    if let Some(title) = table.title.as_ref() {
        out.push_str(&format!("#text(size: 9pt, weight: \"bold\")[{}]\n", escape_text(title)));
        out.push_str("#v(6pt)\n");
    }
    out.push_str(&format!("#table(columns: ({column_spec}), stroke: 0.5pt + luma(180), inset: 5pt,\n"));

    for header in &table.headers {
        out.push_str(&format!("[*{}*],\n", escape_text(header)));
    }

    for row in &table.rows {
        for cell in row {
            out.push_str(&format!("[{}],\n", escape_text(cell)));
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
