use ext_calc::output::StoryForcesOutput;

use crate::chart_build::{
    STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType,
};

/// Build 4 story-force charts: VX, VY, MY, MX.
///
/// VX + MY represent the X-direction excitation.
/// VY + MX represent the Y-direction excitation.
pub fn build(output: &StoryForcesOutput, _config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_force_chart(
            STORY_FORCE_VX_IMAGE,
            "Story Shear VX",
            "Maximum shear force Vx per story (X-direction excitation, kip).",
            output,
            |row| row.max_vx_kip,
            "#1f77b4",
            "Vx (kip)",
        ),
        build_force_chart(
            STORY_FORCE_VY_IMAGE,
            "Story Shear VY",
            "Maximum shear force Vy per story (Y-direction excitation, kip).",
            output,
            |row| row.max_vy_kip,
            "#ff7f0e",
            "Vy (kip)",
        ),
        build_force_chart(
            STORY_FORCE_MY_IMAGE,
            "Story Moment MY",
            "Maximum moment My per story (X-direction excitation, kip·ft).",
            output,
            |row| row.max_my_kip_ft,
            "#2ca02c",
            "My (kip·ft)",
        ),
        build_force_chart(
            STORY_FORCE_MX_IMAGE,
            "Story Moment MX",
            "Maximum moment Mx per story (Y-direction excitation, kip·ft).",
            output,
            |row| row.max_mx_kip_ft,
            "#d62728",
            "Mx (kip·ft)",
        ),
    ]
}

fn build_force_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &StoryForcesOutput,
    value_fn: impl Fn(&ext_calc::output::StoryForceEnvelopeRow) -> f64,
    color: &str,
    series_name: &str,
) -> NamedChartSpec {
    // Rows are already sorted top-down from calc; reverse for bottom-up chart display.
    let rows: Vec<_> = output.rows.iter().rev().collect();

    let categories: Vec<String> = rows.iter().map(|r| r.story.clone()).collect();
    let values: Vec<f64> = rows.iter().map(|r| value_fn(r).abs()).collect();

    let story_count = categories.len() as u32;
    let height = config_height_for_story_count(story_count);

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: 620,
            height,
            kind: ChartKind::Cartesian {
                categories,
                swap_axes: true, // Y-axis = story, X-axis = force magnitude
                series: vec![CartesianSeries {
                    name: series_name.to_string(),
                    data: values,
                    kind: SeriesType::Bar,
                    color: Some(color.to_string()),
                    line_style: None,
                    smooth: false,
                }],
            },
        },
    }
}

fn config_height_for_story_count(count: u32) -> u32 {
    // 20 px per story, minimum 400.
    (count * 20 + 100).max(400)
}

#[cfg(test)]
mod tests {
    use ext_calc::output::{StoryForceEnvelopeRow, StoryForcesOutput};
    use crate::chart_build::{
        STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
    };
    use crate::chart_types::RenderConfig;
    use super::build;

    fn make_output() -> StoryForcesOutput {
        StoryForcesOutput {
            rows: vec![
                StoryForceEnvelopeRow {
                    story: "L3".to_string(),
                    max_vx_kip: 100.0,
                    max_my_kip_ft: 500.0,
                    max_vy_kip: 80.0,
                    max_mx_kip_ft: 400.0,
                },
                StoryForceEnvelopeRow {
                    story: "L2".to_string(),
                    max_vx_kip: 200.0,
                    max_my_kip_ft: 1000.0,
                    max_vy_kip: 160.0,
                    max_mx_kip_ft: 800.0,
                },
                StoryForceEnvelopeRow {
                    story: "L1".to_string(),
                    max_vx_kip: 300.0,
                    max_my_kip_ft: 1500.0,
                    max_vy_kip: 240.0,
                    max_mx_kip_ft: 1200.0,
                },
            ],
        }
    }

    #[test]
    fn story_forces_build_returns_four_assets() {
        let output = make_output();
        let config = RenderConfig::default();
        let charts = build(&output, &config);

        assert_eq!(charts.len(), 4, "expected 4 story-force chart assets");

        let names: Vec<&str> = charts.iter().map(|c| c.logical_name.as_str()).collect();
        assert!(names.contains(&STORY_FORCE_VX_IMAGE), "missing VX chart");
        assert!(names.contains(&STORY_FORCE_VY_IMAGE), "missing VY chart");
        assert!(names.contains(&STORY_FORCE_MY_IMAGE), "missing MY chart");
        assert!(names.contains(&STORY_FORCE_MX_IMAGE), "missing MX chart");
    }

    #[test]
    fn story_forces_vx_and_vy_are_independent() {
        let output = make_output();
        let config = RenderConfig::default();
        let charts = build(&output, &config);

        use crate::chart_types::ChartKind;
        let get_values = |name: &str| {
            let chart = charts.iter().find(|c| c.logical_name == name).unwrap();
            match &chart.spec.kind {
                ChartKind::Cartesian { series, .. } => series[0].data.clone(),
                _ => panic!("expected cartesian"),
            }
        };

        let vx_vals = get_values(STORY_FORCE_VX_IMAGE);
        let vy_vals = get_values(STORY_FORCE_VY_IMAGE);
        // VX max is 300 (L1 bottom), VY max is 240 — must differ.
        assert_ne!(vx_vals, vy_vals, "VX and VY series must differ");
    }
}
