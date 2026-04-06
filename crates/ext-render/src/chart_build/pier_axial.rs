use ext_calc::output::PierAxialOutput;

use crate::chart_build::{PIER_AXIAL_IMAGE, top_pier_values};
use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

pub fn build(output: &PierAxialOutput, config: &RenderConfig) -> NamedChartSpec {
    let governing = top_pier_values(
        output
            .piers
            .iter()
            .map(|row| (format!("{} {}", row.story, row.pier_label), row.dcr)),
    );

    NamedChartSpec {
        logical_name: PIER_AXIAL_IMAGE.to_string(),
        caption: "Top governing pier axial demand-capacity ratios.".to_string(),
        spec: ChartSpec {
            title: "Pier Axial DCR".to_string(),
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
