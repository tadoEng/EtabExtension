use ext_calc::output::PierShearOutput;

use crate::chart_build::{PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE, top_pier_values};
use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

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
    let governing = top_pier_values(
        output
            .piers
            .iter()
            .map(|row| (format!("{} {}", row.story, row.pier_label), row.dcr)),
    );

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: governing.iter().map(|(label, _)| label.clone()).collect(),
                series: vec![CartesianSeries {
                    name: "DCR".to_string(),
                    data: governing.iter().map(|(_, value)| *value).collect(),
                    kind: SeriesType::Bar,
                }],
            },
        },
    }
}
