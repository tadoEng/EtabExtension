use std::cmp::Ordering;
use std::collections::HashSet;

use ext_calc::output::BaseReactionsOutput;

use crate::chart_build::BASE_REACTIONS_IMAGE;
use crate::chart_types::{ChartKind, ChartSpec, NamedChartSpec, RenderConfig};

pub fn build(base_shear: &BaseReactionsOutput, config: &RenderConfig) -> NamedChartSpec {
    let data = build_pie_groups(base_shear, config);
    let group_label = config
        .base_reaction_groups
        .first()
        .map(|group| group.label.trim());
    let title = match group_label {
        Some(label) if !label.is_empty() => format!("{label} Load Distribution"),
        _ => "Gravity Load Distribution".to_string(),
    };
    let caption = match group_label {
        Some(label) if !label.is_empty() => {
            format!("{label} load distribution from configured base reaction Fz cases.")
        }
        _ => "Gravity load distribution from configured base reaction Fz groups.".to_string(),
    };

    NamedChartSpec {
        logical_name: BASE_REACTIONS_IMAGE.to_string(),
        caption,
        spec: ChartSpec {
            title,
            width: config.width,
            height: config.height,
            kind: ChartKind::Pie { data },
        },
    }
}

fn build_pie_groups(base_shear: &BaseReactionsOutput, config: &RenderConfig) -> Vec<(f64, String)> {
    if let Some(group) = config.base_reaction_groups.first() {
        let whitelist = group
            .load_cases
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let mut grouped: Vec<(String, f64)> = Vec::new();

        for row in &base_shear.rows {
            if should_exclude_case_type(&row.case_type) {
                continue;
            }
            if !whitelist.contains(row.output_case.as_str()) {
                continue;
            }
            if let Some((_, total)) = grouped
                .iter_mut()
                .find(|(name, _)| name == &row.output_case)
            {
                *total += round5(row.fz_kip.abs());
            } else {
                grouped.push((row.output_case.clone(), round5(row.fz_kip.abs())));
            }
        }

        return sort_pie_data(grouped);
    }

    let mut fallback: Vec<(String, f64)> = Vec::new();
    for row in &base_shear.rows {
        if should_exclude_case_type(&row.case_type) {
            continue;
        }
        if let Some((_, total)) = fallback
            .iter_mut()
            .find(|(name, _)| name == &row.output_case)
        {
            *total += round5(row.fz_kip.abs());
        } else {
            fallback.push((row.output_case.clone(), round5(row.fz_kip.abs())));
        }
    }

    sort_pie_data(fallback)
}

fn sort_pie_data(values: Vec<(String, f64)>) -> Vec<(f64, String)> {
    let mut pie = values
        .into_iter()
        .filter(|(_, total)| *total > 0.0)
        .map(|(label, total)| (round5(total), label))
        .collect::<Vec<_>>();
    pie.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    pie
}

fn round5(value: f64) -> f64 {
    (value * 100_000.0).round() / 100_000.0
}

fn should_exclude_case_type(case_type: &str) -> bool {
    let normalized = case_type.trim().to_ascii_lowercase();
    normalized == "combination" || normalized == "linmodritz" || normalized == "eigen"
}

#[cfg(test)]
mod tests {
    use super::{build, build_pie_groups};
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

    fn sample_row_with_type(case_name: &str, case_type: &str, fz_kip: f64) -> BaseReactionCheckRow {
        BaseReactionCheckRow {
            output_case: case_name.to_string(),
            case_type: case_type.to_string(),
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

    fn gravity_config(load_cases: Vec<&str>) -> RenderConfig {
        RenderConfig {
            base_reaction_groups: vec![BaseReactionGroup {
                label: "Gravity".to_string(),
                load_cases: load_cases.into_iter().map(str::to_string).collect(),
            }],
            ..RenderConfig::default()
        }
    }

    fn slice_total(slices: &[(f64, String)], label: &str) -> f64 {
        slices
            .iter()
            .find(|(_, name)| name == label)
            .map(|(value, _)| *value)
            .unwrap_or_default()
    }

    #[test]
    fn pie_groups_keep_each_configured_case_as_its_own_slice() {
        let output = sample_output(vec![
            sample_row("Dead", 100.0),
            sample_row("SDL", -50.0),
            sample_row("Live (red)", 200.0),
            sample_row("W_10YRS", 300.0),
        ]);
        let config = gravity_config(vec!["Dead", "SDL", "Live (red)"]);
        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 3);
        assert!((slice_total(&grouped, "Dead") - 100.0).abs() < 1e-9);
        assert!((slice_total(&grouped, "SDL") - 50.0).abs() < 1e-9);
        assert!((slice_total(&grouped, "Live (red)") - 200.0).abs() < 1e-9);
    }

    #[test]
    fn pie_groups_sum_duplicate_rows_per_case() {
        let output = sample_output(vec![
            sample_row("Dead", 100.0),
            sample_row("Dead", -25.0),
            sample_row("SDL", 10.0),
        ]);
        let config = gravity_config(vec!["Dead", "SDL"]);

        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 2);
        assert!((slice_total(&grouped, "Dead") - 125.0).abs() < 1e-9);
        assert!((slice_total(&grouped, "SDL") - 10.0).abs() < 1e-9);
    }

    #[test]
    fn pie_groups_exclude_cases_not_in_whitelist() {
        let output = sample_output(vec![
            sample_row("Dead", 100.0),
            sample_row("SDL", 50.0),
            sample_row("W_10YRS", 300.0),
        ]);
        let config = gravity_config(vec!["Dead"]);

        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 1);
        assert!((slice_total(&grouped, "Dead") - 100.0).abs() < 1e-9);
        assert!(grouped.iter().all(|(_, label)| label != "SDL"));
        assert!(grouped.iter().all(|(_, label)| label != "W_10YRS"));
    }

    #[test]
    fn pie_groups_fallback_to_all_cases_when_no_config() {
        let output = sample_output(vec![
            sample_row("Dead", 100.0),
            sample_row("W_10YRS", 80.0),
            sample_row("Dead", 20.0),
        ]);

        let grouped = build_pie_groups(&output, &RenderConfig::default());
        assert_eq!(grouped.len(), 2);
        assert!((slice_total(&grouped, "Dead") - 120.0).abs() < 1e-9);
        assert!((slice_total(&grouped, "W_10YRS") - 80.0).abs() < 1e-9);
    }

    #[test]
    fn build_uses_first_group_label_for_title_and_caption() {
        let output = sample_output(vec![sample_row("W_10YRS", 80.0), sample_row("Dead", 100.0)]);
        let config = RenderConfig {
            base_reaction_groups: vec![
                BaseReactionGroup {
                    label: "Lateral".to_string(),
                    load_cases: vec!["W_10YRS".to_string()],
                },
                BaseReactionGroup {
                    label: "Gravity".to_string(),
                    load_cases: vec!["Dead".to_string()],
                },
            ],
            ..RenderConfig::default()
        };

        let chart = build(&output, &config);
        assert_eq!(chart.spec.title, "Lateral Load Distribution");
        assert!(chart.caption.contains("Lateral"));
    }

    #[test]
    fn pie_groups_skip_combination_case_type_rows() {
        let output = sample_output(vec![
            sample_row_with_type("Dead", "Combo", 100.0),
            sample_row_with_type("Dead", "Combination", 200.0),
        ]);
        let config = gravity_config(vec!["Dead"]);
        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 1);
        assert!((slice_total(&grouped, "Dead") - 100.0).abs() < 1e-9);
    }

    #[test]
    fn pie_groups_skip_linmodritz_and_eigen_case_types() {
        let output = sample_output(vec![
            sample_row_with_type("Dead", "LinStatic", 100.0),
            sample_row_with_type("Dead", "LinModRitz", 200.0),
            sample_row_with_type("Dead", "Eigen", 300.0),
        ]);
        let config = gravity_config(vec!["Dead"]);
        let grouped = build_pie_groups(&output, &config);
        assert_eq!(grouped.len(), 1);
        assert!((slice_total(&grouped, "Dead") - 100.0).abs() < 1e-9);
    }
}
