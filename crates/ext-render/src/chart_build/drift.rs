use ext_calc::output::{DriftOutput, DriftWindOutput, DriftSeismicOutput};

use crate::chart_build::{
    DRIFT_SEISMIC_X_IMAGE, DRIFT_SEISMIC_Y_IMAGE, DRIFT_WIND_X_IMAGE, DRIFT_WIND_Y_IMAGE,
    aggregate_story_max,
};
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
            true,
        ),
        build_chart(
            DRIFT_WIND_Y_IMAGE,
            "Wind Drift Envelope (Y)",
            "Maximum drift ratio per story under wind loading (Y).",
            &drift.y,
            config,
            false,
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
            true,
        ),
        build_chart(
            DRIFT_SEISMIC_Y_IMAGE,
            "Seismic Drift Envelope (Y)",
            "Maximum drift ratio per story under seismic loading (Y).",
            &drift.y,
            config,
            false,
        ),
    ]
}

fn build_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    drift: &DriftOutput,
    config: &RenderConfig,
    is_x: bool,
) -> NamedChartSpec {
    // Use only the direction-specific drift columns so X and Y charts are truly independent.
    let story_values = aggregate_story_max(drift.rows.iter().map(|row| {
        let value = if is_x {
            row.max_drift_x_pos.abs().max(row.max_drift_x_neg.abs())
        } else {
            row.max_drift_y_pos.abs().max(row.max_drift_y_neg.abs())
        };
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

#[cfg(test)]
mod tests {
    use ext_calc::output::{DriftEnvelopeRow, DriftOutput, DriftWindOutput, StoryDriftResult};
    use crate::chart_types::{ChartKind, RenderConfig};
    use super::build_wind;

    fn make_drift_row(
        story: &str,
        x_pos: f64,
        x_neg: f64,
        y_pos: f64,
        y_neg: f64,
    ) -> DriftEnvelopeRow {
        DriftEnvelopeRow {
            story: story.to_string(),
            group_name: "ALL".to_string(),
            output_case: "WIND".to_string(),
            max_disp_x_pos_ft: 0.0,
            max_disp_x_neg_ft: 0.0,
            max_disp_y_pos_ft: 0.0,
            max_disp_y_neg_ft: 0.0,
            max_drift_x_pos: x_pos,
            max_drift_x_neg: x_neg,
            max_drift_y_pos: y_pos,
            max_drift_y_neg: y_neg,
        }
    }

    fn dummy_drift_output(rows: Vec<DriftEnvelopeRow>) -> DriftOutput {
        DriftOutput {
            allowable_ratio: 0.004,
            rows,
            governing: StoryDriftResult {
                story: "L1".to_string(),
                group_name: "ALL".to_string(),
                output_case: "WIND".to_string(),
                direction: "X".to_string(),
                sense: "Pos".to_string(),
                drift_ratio: 0.001,
                dcr: 0.25,
                pass: true,
            },
            pass: true,
            roof_disp_x: None,
            roof_disp_y: None,
            disp_limit: None,
            disp_pass: None,
        }
    }

    #[test]
    fn drift_x_and_y_charts_differ_when_xy_inputs_differ() {
        // L1: large X drift, tiny Y drift
        // L2: tiny X drift, large Y drift
        let rows = vec![
            make_drift_row("L1", 0.010, 0.008, 0.001, 0.001),
            make_drift_row("L2", 0.001, 0.001, 0.010, 0.009),
        ];
        let wind = DriftWindOutput {
            x: dummy_drift_output(rows.clone()),
            y: dummy_drift_output(rows),
        };
        let config = RenderConfig::default();
        let charts = build_wind(&wind, &config);
        assert_eq!(charts.len(), 2);

        let extract_demands = |chart: &crate::chart_types::NamedChartSpec| match &chart.spec.kind {
            ChartKind::Cartesian { series, .. } => series[0].data.clone(),
            _ => panic!("expected cartesian"),
        };

        let x_demands = extract_demands(&charts[0]);
        let y_demands = extract_demands(&charts[1]);

        // Charts must differ because X/Y inputs differ.
        assert_ne!(x_demands, y_demands, "X and Y charts must differ when inputs differ");

        // X chart: largest demand from L1 (0.010), then L2 (0.001)
        assert!((x_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.010).abs() < 1e-9);
        // Y chart: largest demand from L2 (0.010), L1 has 0.001
        assert!((y_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.010).abs() < 1e-9);
    }
}
