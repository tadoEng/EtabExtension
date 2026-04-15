use ext_calc::output::PierShearStressOutput;

use crate::chart_build::{
    PIER_SHEAR_STRESS_SEISMIC_IMAGE, PIER_SHEAR_STRESS_WIND_IMAGE, is_default_pier_label,
    normalized_pier_labels,
};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(output: &PierShearStressOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_STRESS_WIND_IMAGE,
        "Pier Shear Wind — Vu/Acv vs 8√f'c",
        "All pier shear stresses vs ACI 318 maximum stress limit 8√f’c (psi).",
        output,
        config,
    )
}

pub fn build_seismic(output: &PierShearStressOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_STRESS_SEISMIC_IMAGE,
        "Pier Shear Seismic — Vu/Acv vs 8√f'c",
        "All pier shear stresses vs ACI 318 maximum stress limit 8√f’c (psi).",
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
    let categories = if output.story_order.is_empty() {
        ordered_unique(output.per_pier.iter().map(|row| row.story.clone()))
    } else {
        output.story_order.clone()
    };

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
            *entry = entry.max(row.stress_psi);
        }
        series.push(CartesianSeries {
            name: pier.clone(),
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

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            // Scale height with pier count so bars don't squish on tall buildings.
            height: (config.height).max(24 * categories.len() as u32 + 100),
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
        let names = series.iter().map(|item| item.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, vec!["PX1", "PX2", "PY1", "PY2"]);
    }
}
