use crate::pdf::sections::table::render_table;
use crate::pdf::template::escape_text;
use crate::report_types::{ChartLayout, ChartRef, KeyValueTable};

pub fn render_chart_section(
    title: &str,
    layout: &ChartLayout,
    charts: &[ChartRef],
    table: Option<&KeyValueTable>,
) -> String {
    let mut out = format!("== {}\n\n", escape_text(title));

    match layout {
        ChartLayout::SingleChart => {
            let chart = &charts[0];
            out.push_str(&render_single_chart(chart, "6.6in"));
        }
        ChartLayout::TwoCharts => {
            let left = &charts[0];
            let right = &charts[1];
            out.push_str(&format!(
                "#grid(columns: (1fr, 1fr), gutter: 14pt,\n  [#figure(image(\"{}\", height: 3.2in), caption: [{}])],\n  [#figure(image(\"{}\", height: 3.2in), caption: [{}])],\n)\n",
                escape_text(&left.logical_name),
                escape_text(&left.caption),
                escape_text(&right.logical_name),
                escape_text(&right.caption),
            ));
        }
        ChartLayout::ChartAndTable => {
            let chart = &charts[0];
            let table_markup = table.map(render_table).unwrap_or_default();
            out.push_str(&format!(
                "#grid(columns: (1fr, 1fr), gutter: 16pt,\n  [{}],\n  [#figure(image(\"{}\", height: 5.8in), caption: [{}])],\n)\n",
                table_markup,
                escape_text(&chart.logical_name),
                escape_text(&chart.caption),
            ));
        }
        ChartLayout::TableOnly => {
            if let Some(table) = table {
                out.push_str(&render_table(table));
            }
        }
    }

    out
}

fn render_single_chart(chart: &ChartRef, height: &str) -> String {
    format!(
        "#figure(\n  image(\"{}\", height: {}),\n  caption: [{}],\n)\n",
        escape_text(&chart.logical_name),
        height,
        escape_text(&chart.caption),
    )
}
