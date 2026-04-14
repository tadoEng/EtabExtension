use ext_calc::output::BaseReactionsOutput;

use crate::chart_build::BASE_REACTIONS_IMAGE;
use crate::chart_types::{ChartKind, ChartSpec, NamedChartSpec, RenderConfig};

pub fn build(base_shear: &BaseReactionsOutput, config: &RenderConfig) -> NamedChartSpec {
    let data = build_pie_groups(base_shear, config);

    NamedChartSpec {
        logical_name: BASE_REACTIONS_IMAGE.to_string(),
        caption: "Gravity load distribution from configured base reaction Fz groups.".to_string(),
        spec: ChartSpec {
            title: "Gravity Load Distribution".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Pie { data },
        },
    }
}

fn build_pie_groups(base_shear: &BaseReactionsOutput, config: &RenderConfig) -> Vec<(f64, String)> {
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

#[cfg(test)]
mod tests {
    use super::build_pie_groups;
    use crate::chart_types::{BaseReactionGroup, RenderConfig};
    use ext_calc::output::{BaseReactionCheckRow, BaseReactionDir, BaseReactionsOutput, Quantity};

    fn sample_dir() -> BaseReactionDir {
        BaseReactionDir {
            rsa_case: "DBE_X".to_string(),
            elf_case: "ELF_X".to_string(),
            v_rsa: Quantity::new(1.0, "kip"),
            v_elf: Quantity::new(1.0, "kip"),
            ratio: 1.0,
            pass: true,
        }
    }

    fn sample_row(case_name: &str, fz_kip: f64) -> BaseReactionCheckRow {
        BaseReactionCheckRow {
            output_case: case_name.to_string(),
            case_type: "Combo".to_string(),
            step_type: "Max".to_string(),
            step_number: None,
            fx_kip: 0.0,
            fy_kip: 0.0,
            fz_kip,
            mx_kip_ft: 0.0,
            my_kip_ft: 0.0,
            mz_kip_ft: 0.0,
        }
    }

    fn sample_output(rows: Vec<BaseReactionCheckRow>) -> BaseReactionsOutput {
        BaseReactionsOutput {
            rows,
            direction_x: sample_dir(),
            direction_y: sample_dir(),
        }
    }

    #[test]
    fn pie_groups_sum_configured_fz_for_gravity_group() {
        let output = sample_output(vec![
            sample_row("Dead", 100.0),
            sample_row("SDL", -50.0),
            sample_row("Live (red)", 200.0),
            sample_row("Live (non-red)", 150.0),
            sample_row("W_10YRS", 300.0),
        ]);
        let config = RenderConfig {
            base_reaction_groups: vec![BaseReactionGroup {
                label: "Gravity".to_string(),
                load_cases: vec![
                    "Dead".to_string(),
                    "SDL".to_string(),
                    "Live (red)".to_string(),
                    "Live (non-red)".to_string(),
                    "Live (roof)".to_string(),
                ],
            }],
            ..RenderConfig::default()
        };

        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].1, "Gravity");
        assert!((grouped[0].0 - 500.0).abs() < 1e-9);
    }

    #[test]
    fn pie_groups_respect_multiple_named_group_totals() {
        let output = sample_output(vec![
            sample_row("Dead", 90.0),
            sample_row("SDL", 10.0),
            sample_row("W_10YRS", -80.0),
            sample_row("DBE_X", 120.0),
            sample_row("DBE_Y", 60.0),
        ]);
        let config = RenderConfig {
            base_reaction_groups: vec![
                BaseReactionGroup {
                    label: "Gravity".to_string(),
                    load_cases: vec!["Dead".to_string(), "SDL".to_string()],
                },
                BaseReactionGroup {
                    label: "Wind".to_string(),
                    load_cases: vec!["W_10YRS".to_string()],
                },
                BaseReactionGroup {
                    label: "Seismic".to_string(),
                    load_cases: vec!["DBE_X".to_string(), "DBE_Y".to_string()],
                },
            ],
            ..RenderConfig::default()
        };

        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0], (100.0, "Gravity".to_string()));
        assert_eq!(grouped[1], (80.0, "Wind".to_string()));
        assert_eq!(grouped[2], (180.0, "Seismic".to_string()));
    }
}
