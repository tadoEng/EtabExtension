use crate::pdf::template::escape_text;
use crate::report_types::KeyValueTable;

pub fn render_table(table: &KeyValueTable) -> String {
    if table.headers.is_empty() || table.rows.is_empty() {
        return String::new();
    }

    let column_spec = vec!["1fr"; table.headers.len()].join(", ");
    let mut out = format!("#table(columns: ({column_spec}),\n");

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
