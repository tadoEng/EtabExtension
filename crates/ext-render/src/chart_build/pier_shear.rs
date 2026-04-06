use ext_calc::output::PierShearOutput;

use crate::chart_build::{PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(output: &PierShearOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_WIND_IMAGE,
        "Pier Shear Wind DCR",
        "Top governing pier shear DCR values for wind combinations.",
        output,
        config,
    )
}

pub fn build_seismic(output: &PierShearOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_SEISMIC_IMAGE,
        "Pier Shear Seismic DCR",
        "Top governing pier shear DCR values for seismic combinations.",
        output,
        config,
    )
}

fn build_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let governing = governing_story_stress(output);

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: governing.iter().map(|(story, _)| story.clone()).collect(),
                swap_axes: true,
                series: vec![
                    CartesianSeries {
                        name: "Stress".to_string(),
                        data: governing.iter().map(|(_, value)| value.0).collect(),
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: governing.iter().map(|(_, value)| value.1).collect(),
                        kind: SeriesType::Line,
                        color: Some("#cc0000".to_string()),
                        line_style: Some(LinePattern::Dashed),
                        smooth: false,
                    },
                ],
            },
        },
    }
}

fn governing_story_stress(output: &PierShearOutput) -> Vec<(String, (f64, f64))> {
    let mut values: Vec<(String, (f64, f64, f64))> = Vec::new();

    for row in &output.piers {
        let demand = row.vu.value / row.acv.value;
        let limit = row.phi_vn.value / row.acv.value;
        let dcr = row.dcr;

        if let Some((_, existing)) = values.iter_mut().find(|(story, _)| story == &row.story) {
            if dcr > existing.2 {
                *existing = (demand, limit, dcr);
            }
        } else {
            values.push((row.story.clone(), (demand, limit, dcr)));
        }
    }

    values
        .into_iter()
        .map(|(story, (demand, limit, _))| (story, (demand, limit)))
        .collect()
}
