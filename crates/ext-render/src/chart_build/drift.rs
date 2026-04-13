use ext_calc::output::{DriftOutput, DriftWindOutput, DriftSeismicOutput};

use crate::chart_build::{DRIFT_SEISMIC_X_IMAGE, DRIFT_SEISMIC_Y_IMAGE, DRIFT_WIND_X_IMAGE, DRIFT_WIND_Y_IMAGE, aggregate_story_max};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(drift: &DriftWindOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_chart(
            DRIFT_WIND_X_IMAGE,
            "Wind Drift Envelope (X)",
            "Maximum drift ratio per story under wind loading (X).",
            &drift.x,
            config,
        ),
        build_chart(
            DRIFT_WIND_Y_IMAGE,
            "Wind Drift Envelope (Y)",
            "Maximum drift ratio per story under wind loading (Y).",
            &drift.y,
            config,
        ),
    ]
}

pub fn build_seismic(drift: &DriftSeismicOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_chart(
            DRIFT_SEISMIC_X_IMAGE,
            "Seismic Drift Envelope (X)",
            "Maximum drift ratio per story under seismic loading (X).",
            &drift.x,
            config,
        ),
        build_chart(
            DRIFT_SEISMIC_Y_IMAGE,
            "Seismic Drift Envelope (Y)",
            "Maximum drift ratio per story under seismic loading (Y).",
            &drift.y,
            config,
        ),
    ]
}

fn build_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    drift: &DriftOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let story_values = aggregate_story_max(drift.rows.iter().map(|row| {
        let value = [
            row.max_drift_x_pos.abs(),
            row.max_drift_x_neg.abs(),
            row.max_drift_y_pos.abs(),
            row.max_drift_y_neg.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        (row.story.clone(), value)
    }));

    let categories = story_values
        .iter()
        .map(|(story, _)| story.clone())
        .collect();
    let values = story_values.iter().map(|(_, value)| *value).collect();
    let limits = vec![drift.allowable_ratio; story_values.len()];

    // Scale chart height so bars don't compress on tall buildings (35+ stories).
    let story_count = story_values.len() as u32;
    let height = config.height.max(story_count * 18 + 100);

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height,
            kind: ChartKind::Cartesian {
                categories,
                swap_axes: true,
                series: vec![
                    CartesianSeries {
                        name: "Demand".to_string(),
                        data: values,
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: limits,
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
