use ext_calc::output::PierShearStressOutput;

use crate::chart_build::{
    PIER_SHEAR_STRESS_SEISMIC_AVG_IMAGE, PIER_SHEAR_STRESS_SEISMIC_X_IMAGE,
    PIER_SHEAR_STRESS_SEISMIC_Y_IMAGE, PIER_SHEAR_STRESS_WIND_AVG_IMAGE,
    PIER_SHEAR_STRESS_WIND_X_IMAGE, PIER_SHEAR_STRESS_WIND_Y_IMAGE, is_default_pier_label,
    normalized_pier_labels, story_display_order,
};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(output: &PierShearStressOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_individual_direction_chart(
            PIER_SHEAR_STRESS_WIND_X_IMAGE,
            "Pier Shear Wind - X Wall Direction",
            "X-wall-direction pier stress-ratio trends by story with limit line (10.0).",
            output,
            config,
            "X",
        ),
        build_individual_direction_chart(
            PIER_SHEAR_STRESS_WIND_Y_IMAGE,
            "Pier Shear Wind - Y Wall Direction",
            "Y-wall-direction pier stress-ratio trends by story with limit line (10.0).",
            output,
            config,
            "Y",
        ),
    ]
}

pub fn build_seismic(output: &PierShearStressOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    vec![
        build_individual_direction_chart(
            PIER_SHEAR_STRESS_SEISMIC_X_IMAGE,
            "Pier Shear Seismic - X Wall Direction",
            "X-wall-direction pier stress-ratio trends by story with limit line (10.0).",
            output,
            config,
            "X",
        ),
        build_individual_direction_chart(
            PIER_SHEAR_STRESS_SEISMIC_Y_IMAGE,
            "Pier Shear Seismic - Y Wall Direction",
            "Y-wall-direction pier stress-ratio trends by story with limit line (10.0).",
            output,
            config,
            "Y",
        ),
    ]
}

pub fn build_wind_average(output: &PierShearStressOutput, config: &RenderConfig) -> NamedChartSpec {
    build_average_chart(
        PIER_SHEAR_STRESS_WIND_AVG_IMAGE,
        "Pier Shear Wind - Average Stress Ratio by Story",
        "Average pier shear stress-ratio trends by story with average limit line (8.0).",
        output,
        config,
    )
}

pub fn build_seismic_average(
    output: &PierShearStressOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    build_average_chart(
        PIER_SHEAR_STRESS_SEISMIC_AVG_IMAGE,
        "Pier Shear Seismic - Average Stress Ratio by Story",
        "Average pier shear stress-ratio trends by story with average limit line (8.0).",
        output,
        config,
    )
}

fn build_individual_direction_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearStressOutput,
    config: &RenderConfig,
    wall_direction: &str,
) -> NamedChartSpec {
    let directional_rows = output
        .per_pier
        .iter()
        .filter(|row| row.wall_direction.eq_ignore_ascii_case(wall_direction))
        .collect::<Vec<_>>();

    let mut display_categories = story_display_order(&output.story_order, |story| {
        directional_rows.iter().any(|row| row.story == story)
    });
    if display_categories.is_empty() {
        display_categories = story_display_order(&output.story_order, |_| true);
    }

    let mut pier_names = directional_rows
        .iter()
        .map(|row| row.pier.clone())
        .filter(|label| !is_default_pier_label(label))
        .collect::<Vec<_>>();
    pier_names = ordered_unique(pier_names.into_iter());
    pier_names = normalized_pier_labels(pier_names);

    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#8c564b", "#17becf", "#bcbd22", "#7f7f7f",
    ];
    let mut series = Vec::new();
    for (idx, pier) in pier_names.iter().enumerate() {
        let mut by_story = std::collections::HashMap::new();
        for row in directional_rows.iter().filter(|row| row.pier == *pier) {
            let entry = by_story.entry(row.story.as_str()).or_insert(0.0_f64);
            *entry = entry.max(row.stress_ratio);
        }
        series.push(CartesianSeries {
            name: pier.clone(),
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

    series.push(CartesianSeries {
        name: "Limit (10.0)".to_string(),
        data: vec![output.limit_individual; display_categories.len()],
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
            height: (config.height).max(24 * display_categories.len() as u32 + 100),
            kind: ChartKind::Cartesian {
                categories: display_categories,
                swap_axes: true,
                x_axis_label: Some("Stress Ratio".to_string()),
                y_axis_label: Some("Story".to_string()),
                series,
            },
        },
    }
}

fn build_average_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearStressOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let display_categories = story_display_order(&output.story_order, |story| {
        output.x_average.iter().any(|row| row.story == story)
            || output.y_average.iter().any(|row| row.story == story)
    });

    let x_avg = output
        .x_average
        .iter()
        .map(|row| (row.story.as_str(), row.avg_stress_ratio))
        .collect::<std::collections::HashMap<_, _>>();
    let y_avg = output
        .y_average
        .iter()
        .map(|row| (row.story.as_str(), row.avg_stress_ratio))
        .collect::<std::collections::HashMap<_, _>>();

    let series = vec![
        CartesianSeries {
            name: "X Average Ratio".to_string(),
            data: display_categories
                .iter()
                .map(|story| x_avg.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some("#1f77b4".to_string()),
            line_style: Some(LinePattern::Solid),
            smooth: false,
        },
        CartesianSeries {
            name: "Y Average Ratio".to_string(),
            data: display_categories
                .iter()
                .map(|story| y_avg.get(story.as_str()).copied().unwrap_or(0.0))
                .collect(),
            kind: SeriesType::Line,
            color: Some("#ff7f0e".to_string()),
            line_style: Some(LinePattern::Solid),
            smooth: false,
        },
        CartesianSeries {
            name: "Average Limit (8.0)".to_string(),
            data: vec![output.limit_average; display_categories.len()],
            kind: SeriesType::Line,
            color: Some("#cc0000".to_string()),
            line_style: Some(LinePattern::Dashed),
            smooth: false,
        },
    ];

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: (config.height).max(24 * display_categories.len() as u32 + 100),
            kind: ChartKind::Cartesian {
                categories: display_categories,
                swap_axes: true,
                x_axis_label: Some("Stress Ratio".to_string()),
                y_axis_label: Some("Story".to_string()),
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
    use super::{build_wind, build_wind_average};
    use crate::chart_build::{PIER_SHEAR_STRESS_WIND_X_IMAGE, PIER_SHEAR_STRESS_WIND_Y_IMAGE};
    use crate::chart_types::{ChartKind, RenderConfig};
    use ext_calc::output::{PierShearStressAverageRow, PierShearStressOutput, PierShearStressRow};

    fn make_row(
        story: &str,
        pier: &str,
        direction: &str,
        stress_ratio: f64,
        stress_psi: f64,
    ) -> PierShearStressRow {
        PierShearStressRow {
            story: story.to_string(),
            pier: pier.to_string(),
            combo: "C1".to_string(),
            wall_direction: direction.to_string(),
            acw_in2: 100.0,
            fc_psi: 4000.0,
            sqrt_fc: 4000.0_f64.sqrt(),
            ve_kip: 100.0,
            stress_psi,
            stress_ratio,
            limit_individual: 10.0,
            pass: true,
        }
    }

    fn make_average_row(story: &str, direction: &str, ratio: f64) -> PierShearStressAverageRow {
        PierShearStressAverageRow {
            story: story.to_string(),
            wall_direction: direction.to_string(),
            sum_ve_kip: 100.0,
            sum_acw_in2: 200.0,
            sqrt_fc: 4000.0_f64.sqrt(),
            avg_stress_psi: 250.0,
            avg_stress_ratio: ratio,
            limit_average: 8.0,
            pass: true,
        }
    }

    #[test]
    fn pier_shear_wind_builds_x_and_y_directional_assets() {
        let output = PierShearStressOutput {
            phi_v: 0.75,
            limit_individual: 10.0,
            limit_average: 8.0,
            supported: true,
            support_note: None,
            story_order: vec!["L2".into(), "L1".into()],
            per_pier: vec![
                make_row("L2", "PX1", "X", 2.0, 300.0),
                make_row("L1", "PX1", "X", 1.0, 250.0),
                make_row("L2", "PY1", "Y", 4.0, 450.0),
                make_row("L1", "PY1", "Y", 3.0, 400.0),
            ],
            x_average: vec![],
            y_average: vec![],
            max_individual_ratio: 4.0,
            max_average_ratio: 1.0,
            pass: true,
        };

        let charts = build_wind(&output, &RenderConfig::default());
        assert_eq!(charts.len(), 2);
        assert_eq!(charts[0].logical_name, PIER_SHEAR_STRESS_WIND_X_IMAGE);
        assert_eq!(charts[1].logical_name, PIER_SHEAR_STRESS_WIND_Y_IMAGE);

        let ChartKind::Cartesian {
            series: x_series, ..
        } = &charts[0].spec.kind
        else {
            panic!("expected cartesian chart");
        };
        let ChartKind::Cartesian {
            series: y_series, ..
        } = &charts[1].spec.kind
        else {
            panic!("expected cartesian chart");
        };

        let x_pier = x_series
            .iter()
            .find(|s| s.name == "PX1")
            .expect("x chart should contain PX1 series");
        let y_pier = y_series
            .iter()
            .find(|s| s.name == "PY1")
            .expect("y chart should contain PY1 series");

        assert_eq!(x_pier.data, vec![1.0, 2.0]);
        assert_eq!(y_pier.data, vec![3.0, 4.0]);
    }

    #[test]
    fn pier_shear_chart_reverses_category_order_for_swapped_axis() {
        let output = PierShearStressOutput {
            phi_v: 0.75,
            limit_individual: 10.0,
            limit_average: 8.0,
            supported: true,
            support_note: None,
            story_order: vec!["L2".into(), "L1".into()],
            per_pier: vec![
                make_row("L2", "PX1", "X", 2.0, 300.0),
                make_row("L1", "PX1", "X", 1.0, 250.0),
            ],
            x_average: vec![],
            y_average: vec![],
            max_individual_ratio: 2.0,
            max_average_ratio: 1.0,
            pass: true,
        };

        let charts = build_wind(&output, &RenderConfig::default());
        let ChartKind::Cartesian { categories, .. } = &charts[0].spec.kind else {
            panic!("expected cartesian chart");
        };
        assert_eq!(categories, &vec!["L1", "L2"]);
    }

    #[test]
    fn pier_shear_average_chart_contains_xy_and_limit_series() {
        let output = PierShearStressOutput {
            phi_v: 0.75,
            limit_individual: 10.0,
            limit_average: 8.0,
            supported: true,
            support_note: None,
            story_order: vec!["L2".into(), "L1".into()],
            per_pier: vec![make_row("L2", "PX1", "X", 2.0, 300.0)],
            x_average: vec![
                make_average_row("L2", "X", 0.40),
                make_average_row("L1", "X", 0.20),
            ],
            y_average: vec![
                make_average_row("L2", "Y", 0.55),
                make_average_row("L1", "Y", 0.30),
            ],
            max_individual_ratio: 2.0,
            max_average_ratio: 1.0,
            pass: true,
        };

        let chart = build_wind_average(&output, &RenderConfig::default());
        let ChartKind::Cartesian {
            categories, series, ..
        } = chart.spec.kind
        else {
            panic!("expected cartesian chart");
        };
        assert_eq!(categories, vec!["L1", "L2"]);
        let names = series
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["X Average Ratio", "Y Average Ratio", "Average Limit (8.0)"]
        );
    }
}
