use ext_calc::output::{DisplacementOutput, DisplacementWindOutput};

use crate::chart_build::{DISPLACEMENT_WIND_X_IMAGE, DISPLACEMENT_WIND_Y_IMAGE, aggregate_story_max};
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
    // Use only the direction-specific displacement columns so X and Y charts are independent.
    let story_values = aggregate_story_max(displacement.rows.iter().map(|row| {
        let value = if is_x {
            row.max_disp_x_pos_ft.abs().max(row.max_disp_x_neg_ft.abs())
        } else {
            row.max_disp_y_pos_ft.abs().max(row.max_disp_y_neg_ft.abs())
        };
        (row.story.clone(), value)
    }));

    // Scale chart height with story count.
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
                categories: story_values
                    .iter()
                    .map(|(story, _)| story.clone())
                    .collect(),
                swap_axes: true,
                series: vec![
                    CartesianSeries {
                        name: "Demand (ft)".to_string(),
                        data: story_values.iter().map(|(_, value)| *value).collect(),
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                    CartesianSeries {
                        name: "Limit (ft)".to_string(),
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

#[cfg(test)]
mod tests {
    use ext_calc::output::{
        DisplacementEnvelopeRow, DisplacementOutput, DisplacementWindOutput,
        JointDisplacementResult, Quantity,
    };
    use crate::chart_types::{ChartKind, RenderConfig};
    use super::build;

    fn make_disp_row(
        story: &str,
        x_pos: f64,
        x_neg: f64,
        y_pos: f64,
        y_neg: f64,
    ) -> DisplacementEnvelopeRow {
        DisplacementEnvelopeRow {
            story: story.to_string(),
            group_name: "ALL".to_string(),
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
            make_disp_row("L1", 0.050, 0.040, 0.005, 0.005),
            make_disp_row("L2", 0.005, 0.005, 0.050, 0.045),
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

        assert_ne!(x_demands, y_demands, "X and Y displacement charts must differ when inputs differ");

        // X chart: L1 leads with 0.050 ft
        assert!((x_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.050).abs() < 1e-9);
        // Y chart: L2 leads with 0.050 ft
        assert!((y_demands.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 0.050).abs() < 1e-9);
    }
}
