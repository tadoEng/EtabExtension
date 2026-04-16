use ext_calc::output::{StoryForceCaseProfile, StoryForcesOutput};

use crate::chart_build::{
    STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
    story_display_order,
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
            "Story shear Vx per load case (X-direction excitation, kip).",
            &output.x_profiles,
            output,
            |row| row.vx_kip,
            |row| row.max_vx_kip,
        ),
        build_force_chart(
            STORY_FORCE_VY_IMAGE,
            "Story Shear VY",
            "Story shear Vy per load case (Y-direction excitation, kip).",
            &output.y_profiles,
            output,
            |row| row.vy_kip,
            |row| row.max_vy_kip,
        ),
        build_force_chart(
            STORY_FORCE_MY_IMAGE,
            "Story Moment MY",
            "Story moment My per load case (X-direction excitation, kip·ft).",
            &output.x_profiles,
            output,
            |row| row.my_kip_ft,
            |row| row.max_my_kip_ft,
        ),
        build_force_chart(
            STORY_FORCE_MX_IMAGE,
            "Story Moment MX",
            "Story moment Mx per load case (Y-direction excitation, kip·ft).",
            &output.y_profiles,
            output,
            |row| row.mx_kip_ft,
            |row| row.max_mx_kip_ft,
        ),
    ]
}

fn build_force_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    profiles: &[StoryForceCaseProfile],
    output: &StoryForcesOutput,
    value_fn: impl Fn(&ext_calc::output::StoryForceCaseRow) -> f64,
    fallback_value_fn: impl Fn(&ext_calc::output::StoryForceEnvelopeRow) -> f64,
) -> NamedChartSpec {
    let categories = if output.story_order.is_empty() {
        output
            .rows
            .iter()
            .map(|row| row.story.clone())
            .collect::<Vec<_>>()
    } else {
        output.story_order.clone()
    };
    // Shared story-order utility keeps swapped-axis category ordering consistent across all charts.
    let display_categories = story_display_order(&categories, |_| true);

    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#8c564b", "#17becf", "#bcbd22", "#7f7f7f",
    ];

    let mut series = Vec::new();
    for (idx, profile) in profiles.iter().enumerate() {
        let mut by_story = std::collections::HashMap::new();
        for row in &profile.rows {
            by_story.insert(row.story.as_str(), value_fn(row).abs());
        }
        series.push(CartesianSeries {
            name: profile.output_case.clone(),
            data: display_categories
                .iter()
                .map(|story| by_story.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some(palette[idx % palette.len()].to_string()),
            line_style: None,
            smooth: false,
        });
    }

    if series.is_empty() {
        let mut by_story = std::collections::HashMap::new();
        for row in &output.rows {
            by_story.insert(row.story.as_str(), fallback_value_fn(row).abs());
        }
        series.push(CartesianSeries {
            name: "Envelope".to_string(),
            data: display_categories
                .iter()
                .map(|story| by_story.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some("#1f77b4".to_string()),
            line_style: None,
            smooth: false,
        });
    }

    let story_count = display_categories.len() as u32;
    let height = config_height_for_story_count(story_count);

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: 620,
            height,
            kind: ChartKind::Cartesian {
                categories: display_categories,
                swap_axes: true, // Y-axis = story, X-axis = force magnitude
                series,
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
    use super::build;
    use crate::chart_build::{
        STORY_FORCE_MX_IMAGE, STORY_FORCE_MY_IMAGE, STORY_FORCE_VX_IMAGE, STORY_FORCE_VY_IMAGE,
    };
    use crate::chart_types::RenderConfig;
    use ext_calc::output::{StoryForceEnvelopeRow, StoryForcesOutput};

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
            story_order: vec!["L3".to_string(), "L2".to_string(), "L1".to_string()],
            x_profiles: vec![],
            y_profiles: vec![],
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

    #[test]
    fn story_forces_chart_reverses_category_order_for_swapped_axis() {
        use crate::chart_types::ChartKind;
        let output = make_output();
        let charts = build(&output, &RenderConfig::default());
        let vx = charts
            .iter()
            .find(|chart| chart.logical_name == STORY_FORCE_VX_IMAGE)
            .unwrap();
        let ChartKind::Cartesian { categories, .. } = &vx.spec.kind else {
            panic!("expected cartesian");
        };
        assert_eq!(categories, &vec!["L1", "L2", "L3"]);
    }
}
