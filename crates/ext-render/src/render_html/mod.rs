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
