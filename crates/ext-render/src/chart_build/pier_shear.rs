use ext_calc::output::PierShearStressOutput;

use crate::chart_build::{
    PIER_SHEAR_STRESS_SEISMIC_IMAGE, PIER_SHEAR_STRESS_WIND_IMAGE, is_default_pier_label,
    normalized_pier_labels, story_display_order,
};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(output: &PierShearStressOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_STRESS_WIND_IMAGE,
        "Pier Shear Wind — Stress Ratio by Pier",
        "Pier shear stress-ratio trends by level with individual limit line (10.0).",
        output,
        config,
    )
}

pub fn build_seismic(output: &PierShearStressOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_STRESS_SEISMIC_IMAGE,
        "Pier Shear Seismic — Stress Ratio by Pier",
        "Pier shear stress-ratio trends by level with individual limit line (10.0).",
        output,
        config,
    )
}

fn build_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearStressOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let display_categories = story_display_order(&output.story_order, |story| {
        output.per_pier.iter().any(|row| row.story == story)
    });

    let mut pier_names = output
        .per_pier
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
        for row in output.per_pier.iter().filter(|row| row.pier == *pier) {
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
        name: "Individual Limit (10.0)".to_string(),
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
            // Scale height with pier count so bars don't squish on tall buildings.
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
    use super::build_wind;
    use crate::chart_types::{ChartKind, RenderConfig};
    use ext_calc::output::{PierShearStressOutput, PierShearStressRow};

    fn make_row(story: &str, pier: &str, stress_psi: f64) -> PierShearStressRow {
        PierShearStressRow {
            story: story.to_string(),
            pier: pier.to_string(),
            combo: "C1".to_string(),
            wall_direction: "X".to_string(),
            acw_in2: 100.0,
            fc_psi: 4000.0,
            sqrt_fc: 4000.0_f64.sqrt(),
            ve_kip: 100.0,
            stress_psi,
            stress_ratio: 1.0,
            limit_individual: 10.0,
            pass: true,
        }
    }

    #[test]
    fn pier_shear_filters_default_labels_and_sorts_legend() {
        let output = PierShearStressOutput {
            phi_v: 0.75,
            limit_individual: 10.0,
            limit_average: 8.0,
            supported: true,
            support_note: None,
            story_order: vec!["L2".into(), "L1".into()],
            per_pier: vec![
                make_row("L2", "PX2", 12.0),
                make_row("L2", "PY2", 11.0),
                make_row("L2", "PX1", 10.0),
                make_row("L2", "PY1", 9.0),
                make_row("L2", "0", 8.0),
                make_row("L2", "", 7.0),
            ],
            x_average: vec![],
            y_average: vec![],
            max_individual_ratio: 1.0,
            max_average_ratio: 1.0,
            pass: true,
        };

        let chart = build_wind(&output, &RenderConfig::default());
        let ChartKind::Cartesian { series, .. } = chart.spec.kind else {
            panic!("expected cartesian chart");
        };
        let names = series
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["PX1", "PX2", "PY1", "PY2", "Individual Limit (10.0)"]
        );
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
            per_pier: vec![make_row("L2", "PX1", 10.0), make_row("L1", "PX1", 9.0)],
            x_average: vec![],
            y_average: vec![],
            max_individual_ratio: 1.0,
            max_average_ratio: 1.0,
            pass: true,
        };

        let chart = build_wind(&output, &RenderConfig::default());
        let ChartKind::Cartesian { categories, .. } = chart.spec.kind else {
            panic!("expected cartesian chart");
        };
        assert_eq!(categories, vec!["L1", "L2"]);
    }
}
