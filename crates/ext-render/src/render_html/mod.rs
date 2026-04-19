use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::chart_build::{build_chart, build_report_charts};
use crate::chart_types::{ChartSpec, RenderConfig};

pub fn render_html(spec: &ChartSpec, chart_id: &str) -> Result<String> {
    charming::HtmlRenderer::new(chart_id, spec.width as u64, spec.height as u64)
        .render(&build_chart(spec))
        .context("charming HTML render failed")
}

pub fn render_all_html(
    calc: &ext_calc::output::CalcOutput,
    config: &RenderConfig,
) -> Result<HashMap<String, String>> {
    let mut html_map = HashMap::new();

    for chart in build_report_charts(calc, config) {
        html_map.insert(
            chart.logical_name.clone(),
            render_html(&chart.spec, chart.logical_name.as_str())?,
        );
    }

    Ok(html_map)
}

#[cfg(test)]
mod tests {
    use super::render_html;
    use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, LinePattern, SeriesType};

    #[test]
    fn rendered_line_charts_disable_point_symbols() {
        let spec = ChartSpec {
            title: "line".to_string(),
            width: 800,
            height: 400,
            kind: ChartKind::Cartesian {
                categories: vec!["L1".to_string(), "L2".to_string()],
                swap_axes: true,
                x_axis_label: Some("Ratio".to_string()),
                y_axis_label: Some("Story".to_string()),
                series: vec![
                    CartesianSeries {
                        name: "Series A".to_string(),
                        data: vec![0.2, 0.4],
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: false,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: vec![1.0, 1.0],
                        kind: SeriesType::Line,
                        color: Some("#cc0000".to_string()),
                        line_style: Some(LinePattern::Dashed),
                        smooth: false,
                    },
                ],
            },
        };

        let html = render_html(&spec, "symbol_test").expect("line chart should render");
        assert!(html.contains("showSymbol"));
        assert!(html.contains("false"));
    }
}
