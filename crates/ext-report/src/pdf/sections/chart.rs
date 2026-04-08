use crate::pdf::sections::table::render_table;
use crate::pdf::template::escape_text;
use crate::report_types::{ChartRef, KeyValueTable};

pub fn render_single_chart_page(title: &str, chart: &ChartRef) -> String {
    let mut out = format!(
        "#text(size: 16pt, weight: \"bold\")[{}]\n",
        escape_text(title)
    );
    out.push_str("#v(10pt)\n");
    out.push_str(&render_single_chart(chart, "6.8in"));
    out
}

pub fn render_two_charts_page(title: &str, charts: &[ChartRef]) -> String {
    let left = &charts[0];
    let right = &charts[1];
    format!(
        "#text(size: 16pt, weight: \"bold\")[{}]\n#v(10pt)\n#grid(columns: (1fr, 1fr), gutter: 14pt,\n  [#figure(image(\"{}\", height: 6.0in), caption: [{}])],\n  [#figure(image(\"{}\", height: 6.0in), caption: [{}])],\n)\n",
        escape_text(title),
        escape_text(&left.logical_name),
        escape_text(&left.caption),
        escape_text(&right.logical_name),
        escape_text(&right.caption),
    )
}

pub fn render_chart_and_table_page(
    title: &str,
    chart: &ChartRef,
    table: &KeyValueTable,
    table_emphasis: bool,
) -> String {
    let (columns, chart_height) = if table_emphasis {
        ("(1.08fr, 0.92fr)", "5.7in")
    } else {
        ("(0.82fr, 1.18fr)", "6.4in")
    };
    format!(
        "#text(size: 16pt, weight: \"bold\")[{}]\n#v(10pt)\n#grid(columns: {}, gutter: 14pt,\n  [#align(top)[{}]],\n  [#align(center)[#figure(image(\"{}\", height: {}), caption: [{}])]],\n)\n",
        escape_text(title),
        columns,
        render_table(table),
        escape_text(&chart.logical_name),
        chart_height,
        escape_text(&chart.caption),
    )
}

fn render_single_chart(chart: &ChartRef, height: &str) -> String {
    format!(
        "#figure(\n  image(\"{}\", height: {}),\n  caption: [{}],\n)\n",
        escape_text(&chart.logical_name),
        height,
        escape_text(&chart.caption),
    )
}
