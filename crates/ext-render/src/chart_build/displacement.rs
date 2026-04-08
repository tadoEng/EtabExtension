use ext_calc::output::DisplacementOutput;

use crate::chart_build::{DISPLACEMENT_WIND_IMAGE, aggregate_story_max};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(displacement: &DisplacementOutput, config: &RenderConfig) -> NamedChartSpec {
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
        logical_name: DISPLACEMENT_WIND_IMAGE.to_string(),
        caption: "Maximum roof and story displacement demand under wind loading.".to_string(),
        spec: ChartSpec {
            title: "Wind Displacement Envelope".to_string(),
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
