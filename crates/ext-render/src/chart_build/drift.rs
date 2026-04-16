use ext_calc::output::{DriftOutput, DriftSeismicOutput, DriftWindOutput};

use crate::chart_build::{
    DRIFT_SEISMIC_X_IMAGE, DRIFT_SEISMIC_Y_IMAGE, DRIFT_WIND_X_IMAGE, DRIFT_WIND_Y_IMAGE,
    story_display_order,
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
    // Build bottom→top display order for the swapped Y-axis.
    // Fall back to insertion order from rows when story_order is empty.
    let categories = if drift.story_order.is_empty() {
        // No story_order: collect unique stories in appearance order then reverse.
        let raw = ordered_unique(drift.rows.iter().map(|row| row.story.clone()));
        story_display_order(&raw, |_| true)
    } else {
        story_display_order(&drift.story_order, |s| {
            drift.rows.iter().any(|r| r.story == s)
        })
    };
    let groups = ordered_unique(drift.rows.iter().map(|row| row.group_name.clone()));

    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#8c564b", "#17becf", "#bcbd22", "#7f7f7f",
    ];
    let mut series = Vec::new();
    for (idx, group) in groups.iter().enumerate() {
        let mut by_story = std::collections::HashMap::new();
        for row in drift.rows.iter().filter(|row| row.group_name == *group) {
            let demand = if is_x {
                row.max_drift_x_pos.abs().max(row.max_drift_x_neg.abs())
            } else {
                row.max_drift_y_pos.abs().max(row.max_drift_y_neg.abs())
            };
            let entry = by_story.entry(row.story.as_str()).or_insert(0.0_f64);
            *entry = entry.max(demand);
        }
        series.push(CartesianSeries {
            name: group.clone(),
            data: categories
                .iter()
                .map(|story| by_story.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some(palette[idx % palette.len()].to_string()),
            line_style: Some(LinePattern::Solid),
            smooth: false,
        });
    }

    series.push(CartesianSeries {
        name: "Limit".to_string(),
        data: vec![drift.allowable_ratio; categories.len()],
        kind: SeriesType::Line,
        color: Some("#cc0000".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });

    // Scale chart height so bars don't compress on tall buildings (35+ stories).
    let story_count = categories.len() as u32;
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
                series,
            },
        },
    }
}

fn ordered_unique(iter: impl Iterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in iter {
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::build_wind;
    use crate::chart_types::{ChartKind, RenderConfig};
    use ext_calc::output::{DriftEnvelopeRow, DriftOutput, DriftWindOutput, StoryDriftResult};

    fn make_drift_row(
        story: &str,
        group: &str,
        x_pos: f64,
        x_neg: f64,
        y_pos: f64,
        y_neg: f64,
    ) -> DriftEnvelopeRow {
        DriftEnvelopeRow {
            story: story.to_string(),
            group_name: group.to_string(),
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
            story_order: vec!["L1".to_string(), "L2".to_string()],
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
            make_drift_row("L1", "ALL", 0.010, 0.008, 0.001, 0.001),
            make_drift_row("L2", "ALL", 0.001, 0.001, 0.010, 0.009),
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
        assert_ne!(
            x_demands, y_demands,
            "X and Y charts must differ when inputs differ"
        );

        // X chart: largest demand from L1 (0.010), then L2 (0.001)
        assert!((x_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.010).abs() < 1e-9);
        // Y chart: largest demand from L2 (0.010), L1 has 0.001
        assert!((y_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.010).abs() < 1e-9);
    }

    #[test]
    fn drift_chart_supports_more_than_four_groups() {
        let mut rows = Vec::new();
        for group in ["G1", "G2", "G3", "G4", "G5"] {
            rows.push(make_drift_row("L2", group, 0.001, 0.0, 0.0, 0.0));
            rows.push(make_drift_row("L1", group, 0.002, 0.0, 0.0, 0.0));
        }

        let wind = DriftWindOutput {
            x: dummy_drift_output(rows.clone()),
            y: dummy_drift_output(rows),
        };
        let charts = build_wind(&wind, &RenderConfig::default());
        let ChartKind::Cartesian { series, .. } = &charts[0].spec.kind else {
            panic!("expected cartesian chart");
        };
        // 5 tracking-group series + 1 limit series.
        assert_eq!(series.len(), 6);
    }
}
