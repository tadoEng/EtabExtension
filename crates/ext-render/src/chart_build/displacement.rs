use ext_calc::output::{DisplacementOutput, DisplacementWindOutput};

use crate::chart_build::{DISPLACEMENT_WIND_X_IMAGE, DISPLACEMENT_WIND_Y_IMAGE, aggregate_story_max};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(displacement: &DisplacementWindOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_inner(DISPLACEMENT_WIND_X_IMAGE, "Wind Displacement (X)", "Maximum wind displacement (X).", &displacement.x, config),
        build_inner(DISPLACEMENT_WIND_Y_IMAGE, "Wind Displacement (Y)", "Maximum wind displacement (Y).", &displacement.y, config)
    ]
}

fn build_inner(logical_name: &str, title: &str, caption: &str, displacement: &DisplacementOutput, config: &RenderConfig) -> NamedChartSpec {
    let story_values = aggregate_story_max(displacement.rows.iter().map(|row| {
        let value = [
            row.max_disp_x_pos_ft.abs(),
            row.max_disp_x_neg_ft.abs(),
            row.max_disp_y_pos_ft.abs(),
            row.max_disp_y_neg_ft.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        (row.story.clone(), value)
    }));

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: story_values
                    .iter()
                    .map(|(story, _)| story.clone())
                    .collect(),
                swap_axes: true,
                series: vec![
                    CartesianSeries {
                        name: "Demand".to_string(),
                        data: story_values.iter().map(|(_, value)| *value).collect(),
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: vec![displacement.disp_limit.value; story_values.len()],
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
