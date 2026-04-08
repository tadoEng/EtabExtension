use ext_calc::output::BaseShearOutput;

use crate::chart_build::BASE_SHEAR_IMAGE;
use crate::chart_types::{ChartKind, ChartSpec, NamedChartSpec, RenderConfig};

pub fn build(base_shear: &BaseShearOutput, config: &RenderConfig) -> NamedChartSpec {
    let data = build_pie_groups(base_shear, config);

    NamedChartSpec {
        logical_name: BASE_SHEAR_IMAGE.to_string(),
        caption: "Gravity load distribution from configured base reaction Fz groups.".to_string(),
        spec: ChartSpec {
            title: "Gravity Load Distribution".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Pie { data },
        },
    }
}

fn build_pie_groups(base_shear: &BaseShearOutput, config: &RenderConfig) -> Vec<(f64, String)> {
    if !config.base_reaction_groups.is_empty() {
        let mut grouped = Vec::new();
        for group in &config.base_reaction_groups {
            let total = base_shear
                .rows
                .iter()
                .filter(|row| {
                    group
                        .load_cases
                        .iter()
                        .any(|case_name| case_name == &row.output_case)
                })
                .map(|row| row.fz_kip.abs())
                .sum::<f64>();
            if total > 0.0 {
                grouped.push((total, group.label.clone()));
            }
        }
        if !grouped.is_empty() {
            return grouped;
        }
    }

    let mut fallback: Vec<(String, f64)> = Vec::new();
    for row in &base_shear.rows {
        if let Some((_, total)) = fallback
            .iter_mut()
            .find(|(name, _)| name == &row.output_case)
        {
            *total += row.fz_kip.abs();
        } else {
            fallback.push((row.output_case.clone(), row.fz_kip.abs()));
        }
    }

    fallback
        .into_iter()
        .filter(|(_, total)| *total > 0.0)
        .map(|(label, total)| (total, label))
        .collect()
}
