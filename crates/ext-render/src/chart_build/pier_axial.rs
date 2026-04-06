use ext_calc::output::PierAxialOutput;

use crate::chart_build::PIER_AXIAL_IMAGE;
use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

pub fn build(output: &PierAxialOutput, config: &RenderConfig) -> NamedChartSpec {
    let governing = top_axial_values(
        output
            .piers
            .iter()
            .map(|row| (format!("{} {}", row.story, row.pier_label), row.fa.value)),
    );

    NamedChartSpec {
        logical_name: PIER_AXIAL_IMAGE.to_string(),
        caption: "Signed axial stress envelope showing both compression and tension.".to_string(),
        spec: ChartSpec {
            title: "Pier Axial Stress".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: governing.iter().map(|(label, _)| label.clone()).collect(),
                swap_axes: false,
                series: vec![CartesianSeries {
                    name: "Axial Stress".to_string(),
                    data: governing.iter().map(|(_, value)| *value).collect(),
                    kind: SeriesType::Bar,
                    color: Some("#4c78a8".to_string()),
                    line_style: None,
                    smooth: false,
                }],
            },
        },
    }
}

fn top_axial_values(iter: impl Iterator<Item = (String, f64)>) -> Vec<(String, f64)> {
    let mut values = iter.collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.abs().total_cmp(&left.1.abs()));
    values.truncate(8);
    values.reverse();
    values
}
