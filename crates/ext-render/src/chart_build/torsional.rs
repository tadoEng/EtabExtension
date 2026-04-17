use std::collections::HashMap;

use ext_calc::output::{TorsionalDirectionOutput, TorsionalOutput};

use crate::chart_build::{TORSIONAL_X_IMAGE, TORSIONAL_Y_IMAGE, story_display_order};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(output: &TorsionalOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_dir(
            &output.x,
            config,
            TORSIONAL_X_IMAGE,
            "Torsional Ratio (X Direction)",
            "Story-wise governing torsional ratio (DeltaMax/DeltaAvg) with Type A and Type B thresholds.",
        ),
        build_dir(
            &output.y,
            config,
            TORSIONAL_Y_IMAGE,
            "Torsional Ratio (Y Direction)",
            "Story-wise governing torsional ratio (DeltaMax/DeltaAvg) with Type A and Type B thresholds.",
        ),
    ]
}

fn build_dir(
    dir: &TorsionalDirectionOutput,
    config: &RenderConfig,
    logical_name: &str,
    title: &str,
    caption: &str,
) -> NamedChartSpec {
    let story_order_top_to_bottom = ordered_unique(dir.rows.iter().map(|row| row.story.clone()));
    let categories = story_display_order(&story_order_top_to_bottom, |_| true);

    let mut by_story = HashMap::new();
    for row in &dir.rows {
        let entry = by_story.entry(row.story.as_str()).or_insert(0.0_f64);
        *entry = entry.max(row.governing_ratio);
    }

    let mut series = vec![CartesianSeries {
        name: "Governing Ratio".to_string(),
        data: categories
            .iter()
            .map(|story| by_story.get(story.as_str()).copied().unwrap_or(0.0))
            .collect(),
        kind: SeriesType::Line,
        color: Some("#1f77b4".to_string()),
        line_style: Some(LinePattern::Solid),
        smooth: false,
    }];
    series.push(CartesianSeries {
        name: "Type A (1.2)".to_string(),
        data: vec![1.2; categories.len()],
        kind: SeriesType::Line,
        color: Some("#ff7f0e".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });
    series.push(CartesianSeries {
        name: "Type B (1.4)".to_string(),
        data: vec![1.4; categories.len()],
        kind: SeriesType::Line,
        color: Some("#cc0000".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories,
                series,
                swap_axes: true,
                x_axis_label: Some("Torsional Ratio (DeltaMax/DeltaAvg)".to_string()),
                y_axis_label: Some("Story".to_string()),
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
    use super::build;
    use crate::chart_types::{ChartKind, RenderConfig};
    use ext_calc::output::{TorsionalDirectionOutput, TorsionalOutput, TorsionalRow};

    fn sample_row(story: &str, ratio: f64) -> TorsionalRow {
        TorsionalRow {
            story: story.to_string(),
            case: "ELF_X".to_string(),
            joint_a: "J1".to_string(),
            joint_b: "J2".to_string(),
            drift_a_steps: vec![0.1],
            drift_b_steps: vec![0.2],
            delta_max_steps: vec![0.2],
            delta_avg_steps: vec![0.15],
            ratio,
            governing_step: 1,
            governing_drift_a: 0.1,
            governing_drift_b: 0.2,
            governing_delta_max: 0.2,
            governing_delta_avg: 0.15,
            governing_ratio: ratio,
            ax: 1.0,
            ecc_ft: 0.5,
            rho: 1.0,
            is_type_a: ratio > 1.2,
            is_type_b: ratio > 1.4,
        }
    }

    fn direction(rows: Vec<TorsionalRow>) -> TorsionalDirectionOutput {
        TorsionalDirectionOutput {
            rows,
            no_data: vec![],
            governing_story: "L2".to_string(),
            governing_case: "ELF_X".to_string(),
            governing_joints: vec!["J1".to_string(), "J2".to_string()],
            governing_step: Some(1),
            max_ratio: 1.3,
            has_type_a: true,
            has_type_b: false,
        }
    }

    #[test]
    fn torsional_chart_builds_x_and_y_with_threshold_series() {
        let output = TorsionalOutput {
            x: direction(vec![
                sample_row("L3", 1.1),
                sample_row("L2", 1.3),
                sample_row("L1", 1.0),
            ]),
            y: direction(vec![sample_row("L2", 1.2)]),
            pass: true,
        };
        let charts = build(&output, &RenderConfig::default());
        assert_eq!(charts.len(), 2);

        let ChartKind::Cartesian {
            categories, series, ..
        } = &charts[0].spec.kind
        else {
            panic!("expected cartesian chart");
        };
        assert_eq!(categories, &vec!["L1", "L2", "L3"]);
        assert_eq!(series.len(), 3);
        assert_eq!(series[1].name, "Type A (1.2)");
        assert_eq!(series[2].name, "Type B (1.4)");
    }
}
