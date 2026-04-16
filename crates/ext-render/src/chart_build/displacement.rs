use ext_calc::output::{DisplacementOutput, DisplacementWindOutput};

use crate::chart_build::{DISPLACEMENT_WIND_X_IMAGE, DISPLACEMENT_WIND_Y_IMAGE};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(displacement: &DisplacementWindOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_inner(
            DISPLACEMENT_WIND_X_IMAGE,
            "Wind Displacement (X)",
            "Maximum wind displacement per story (X direction).",
            &displacement.x,
            config,
            true,
        ),
        build_inner(
            DISPLACEMENT_WIND_Y_IMAGE,
            "Wind Displacement (Y)",
            "Maximum wind displacement per story (Y direction).",
            &displacement.y,
            config,
            false,
        ),
    ]
}

fn build_inner(
    logical_name: &str,
    title: &str,
    caption: &str,
    displacement: &DisplacementOutput,
    config: &RenderConfig,
    is_x: bool,
) -> NamedChartSpec {
    let categories = if displacement.story_order.is_empty() {
        ordered_unique(displacement.rows.iter().map(|row| row.story.clone()))
    } else {
        displacement.story_order.clone()
    };
    // Swapped-axis category plots place the first category at the bottom.
    // Reverse here so visual Y order remains top story -> bottom story.
    let display_categories = categories.iter().rev().cloned().collect::<Vec<_>>();
    let groups = ordered_unique(displacement.rows.iter().map(|row| row.group_name.clone()));

    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#8c564b", "#17becf", "#bcbd22", "#7f7f7f",
    ];
    let mut series = Vec::new();
    for (idx, group) in groups.iter().enumerate() {
        let mut by_story = std::collections::HashMap::new();
        for row in displacement
            .rows
            .iter()
            .filter(|row| row.group_name == *group)
        {
            let value = if is_x {
                row.max_disp_x_pos_ft.abs().max(row.max_disp_x_neg_ft.abs())
            } else {
                row.max_disp_y_pos_ft.abs().max(row.max_disp_y_neg_ft.abs())
            };
            let entry = by_story.entry(row.story.as_str()).or_insert(0.0_f64);
            *entry = entry.max(value);
        }

        series.push(CartesianSeries {
            name: group.clone(),
            data: display_categories
                .iter()
                .map(|story| by_story.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some(palette[idx % palette.len()].to_string()),
            line_style: Some(LinePattern::Solid),
            smooth: false,
        });
    }

    let mut limits_by_story = std::collections::HashMap::new();
    for row in &displacement.story_limits {
        limits_by_story.insert(row.story.as_str(), row.limit_ft);
    }
    series.push(CartesianSeries {
        name: "Limit (ft)".to_string(),
        data: display_categories
            .iter()
            .map(|story| {
                limits_by_story
                    .get(story.as_str())
                    .copied()
                    .unwrap_or(displacement.disp_limit.value)
            })
            .collect(),
        kind: SeriesType::Line,
        color: Some("#cc0000".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });

    // Scale chart height with story count.
    let story_count = display_categories.len() as u32;
    let height = config.height.max(story_count * 18 + 100);

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height,
            kind: ChartKind::Cartesian {
                categories: display_categories,
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
    use super::build;
    use crate::chart_types::{ChartKind, RenderConfig};
    use ext_calc::output::{
        DisplacementEnvelopeRow, DisplacementOutput, DisplacementWindOutput,
        JointDisplacementResult, Quantity,
    };

    fn make_disp_row(
        story: &str,
        group: &str,
        x_pos: f64,
        x_neg: f64,
        y_pos: f64,
        y_neg: f64,
    ) -> DisplacementEnvelopeRow {
        DisplacementEnvelopeRow {
            story: story.to_string(),
            group_name: group.to_string(),
            output_case: "WIND".to_string(),
            max_disp_x_pos_ft: x_pos,
            max_disp_x_neg_ft: x_neg,
            max_disp_y_pos_ft: y_pos,
            max_disp_y_neg_ft: y_neg,
        }
    }

    fn dummy_governing() -> JointDisplacementResult {
        JointDisplacementResult {
            story: "L1".to_string(),
            group_name: "ALL".to_string(),
            output_case: "WIND".to_string(),
            direction: "X".to_string(),
            sense: "Pos".to_string(),
            displacement: Quantity::new(0.01, "ft"),
            dcr: 0.5,
            pass: true,
        }
    }

    fn dummy_disp_output(rows: Vec<DisplacementEnvelopeRow>) -> DisplacementOutput {
        DisplacementOutput {
            rows,
            story_order: vec!["L1".to_string(), "L2".to_string()],
            story_limits: vec![],
            governing: dummy_governing(),
            disp_limit: Quantity::new(0.10, "ft"),
            pass: true,
        }
    }

    #[test]
    fn displacement_x_and_y_charts_differ_when_xy_inputs_differ() {
        // L1: large X displacement, tiny Y displacement
        // L2: tiny X displacement, large Y displacement
        let rows = vec![
            make_disp_row("L1", "ALL", 0.050, 0.040, 0.005, 0.005),
            make_disp_row("L2", "ALL", 0.005, 0.005, 0.050, 0.045),
        ];
        let wind = DisplacementWindOutput {
            x: dummy_disp_output(rows.clone()),
            y: dummy_disp_output(rows),
        };
        let config = RenderConfig::default();
        let charts = build(&wind, &config);
        assert_eq!(charts.len(), 2);

        let extract_demands = |chart: &crate::chart_types::NamedChartSpec| match &chart.spec.kind {
            ChartKind::Cartesian { series, .. } => series[0].data.clone(),
            _ => panic!("expected cartesian"),
        };

        let x_demands = extract_demands(&charts[0]);
        let y_demands = extract_demands(&charts[1]);

        assert_ne!(
            x_demands, y_demands,
            "X and Y displacement charts must differ when inputs differ"
        );

        // X chart: L1 leads with 0.050 ft
        assert!((x_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.050).abs() < 1e-9);
        // Y chart: L2 leads with 0.050 ft
        assert!((y_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.050).abs() < 1e-9);
    }

    #[test]
    fn displacement_chart_supports_more_than_four_groups() {
        let mut rows = Vec::new();
        for group in ["G1", "G2", "G3", "G4", "G5"] {
            rows.push(make_disp_row("L2", group, 0.01, 0.0, 0.0, 0.0));
            rows.push(make_disp_row("L1", group, 0.02, 0.0, 0.0, 0.0));
        }
        let wind = DisplacementWindOutput {
            x: dummy_disp_output(rows.clone()),
            y: dummy_disp_output(rows),
        };
        let charts = build(&wind, &RenderConfig::default());
        let ChartKind::Cartesian { series, .. } = &charts[0].spec.kind else {
            panic!("expected cartesian chart");
        };
        // 5 tracking-group series + 1 limit series.
        assert_eq!(series.len(), 6);
    }

    #[test]
    fn displacement_chart_reverses_category_order_for_swapped_axis() {
        let rows = vec![
            make_disp_row("L1", "ALL", 0.050, 0.040, 0.005, 0.005),
            make_disp_row("L2", "ALL", 0.005, 0.005, 0.050, 0.045),
        ];
        let wind = DisplacementWindOutput {
            x: dummy_disp_output(rows.clone()),
            y: dummy_disp_output(rows),
        };
        let charts = build(&wind, &RenderConfig::default());
        let ChartKind::Cartesian { categories, .. } = &charts[0].spec.kind else {
            panic!("expected cartesian");
        };
        assert_eq!(categories, &vec!["L2", "L1"]);
    }
}
