use ext_calc::output::DriftOutput;

use crate::chart_build::{DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE, aggregate_story_max};
use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

pub fn build_wind(drift: &DriftOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        DRIFT_WIND_IMAGE,
        "Wind Drift Envelope",
        "Maximum drift ratio per story under wind loading.",
        drift,
        config,
    )
}

pub fn build_seismic(drift: &DriftOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        DRIFT_SEISMIC_IMAGE,
        "Seismic Drift Envelope",
        "Maximum drift ratio per story under seismic loading.",
        drift,
        config,
    )
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

    let categories = story_values.iter().map(|(story, _)| story.clone()).collect();
    let values = story_values.iter().map(|(_, value)| *value).collect();
    let limits = vec![drift.allowable_ratio; story_values.len()];

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories,
                series: vec![
                    CartesianSeries {
                        name: "Demand".to_string(),
                        data: values,
                        kind: SeriesType::Line,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: limits,
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}
