use std::collections::HashMap;

use serde::Serialize;

use super::format::wrap_load_case_label;
use super::ordering::{compare_pier_labels, is_default_pier_label, ordered_unique};

// ── Pier Shear ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PierShearReportData {
    pub(super) supported: bool,
    pub(super) support_note: String,
    pub(super) phi_v: f64,
    pub(super) limit_individual_ratio: f64,
    pub(super) limit_average_ratio: f64,
    pub(super) max_individual_ratio: f64,
    pub(super) max_average_ratio: f64,
    pub(super) pass: bool,
    pub(super) x_rows: Vec<PierShearDirectionalReportRow>,
    pub(super) y_rows: Vec<PierShearDirectionalReportRow>,
    pub(super) x_matrix: PierShearMatrixReportData,
    pub(super) y_matrix: PierShearMatrixReportData,
    pub(super) x_average_rows: Vec<PierShearAverageDetailReportRow>,
    pub(super) y_average_rows: Vec<PierShearAverageDetailReportRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PierShearMatrixReportData {
    pub(super) levels: Vec<String>,
    pub(super) piers: Vec<String>,
    pub(super) matrix_ratio: Vec<Vec<Option<f64>>>,
    pub(super) individual_ratio_scale_kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PierShearDirectionalReportRow {
    pub(super) story: String,
    pub(super) pier: String,
    pub(super) combo: String,
    pub(super) limit: f64,
    pub(super) stress_ratio: f64,
    pub(super) stress_psi: f64,
    pub(super) ve_kip: f64,
    pub(super) acw_in2: f64,
    pub(super) fc_psi: f64,
    pub(super) ratio_color_value: Option<f64>,
    pub(super) ratio_color_scale_kind: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PierShearAverageDetailReportRow {
    pub(super) story: String,
    pub(super) limit: f64,
    pub(super) avg_stress_psi: f64,
    pub(super) sum_area_in2: f64,
    pub(super) sum_shear_kip: f64,
    pub(super) avg_ratio: f64,
    pub(super) ratio_color_value: Option<f64>,
    pub(super) ratio_color_scale_kind: Option<String>,
}

pub(super) fn build_pier_shear(
    pier: &ext_calc::output::PierShearStressOutput,
) -> PierShearReportData {
    if !pier.supported {
        return build_unsupported_pier_shear_report_data(
            pier.support_note
                .clone()
                .unwrap_or_else(|| "Pier shear check is unavailable.".to_string()),
        );
    }

    let story_rank = pier
        .story_order
        .iter()
        .enumerate()
        .map(|(idx, story)| (story.clone(), idx))
        .collect::<HashMap<_, _>>();

    let mut filtered = pier
        .per_pier
        .iter()
        .filter(|row| !is_default_pier_label(&row.pier))
        .cloned()
        .collect::<Vec<_>>();

    filtered.sort_by(|a, b| {
        let a_rank = story_rank.get(&a.story).copied().unwrap_or(usize::MAX);
        let b_rank = story_rank.get(&b.story).copied().unwrap_or(usize::MAX);
        a_rank
            .cmp(&b_rank)
            .then_with(|| compare_pier_labels(a.pier.as_str(), b.pier.as_str()))
            .then_with(|| a.combo.cmp(&b.combo))
    });

    let map_direction_rows = |direction: &str| {
        filtered
            .iter()
            .filter(|row| row.wall_direction.eq_ignore_ascii_case(direction))
            .map(|row| PierShearDirectionalReportRow {
                story: row.story.clone(),
                pier: row.pier.clone(),
                combo: wrap_load_case_label(&row.combo),
                limit: row.limit_individual,
                stress_ratio: row.stress_ratio,
                stress_psi: row.stress_psi,
                ve_kip: row.ve_kip,
                acw_in2: row.acw_in2,
                fc_psi: row.fc_psi,
                ratio_color_value: Some(row.stress_ratio),
                ratio_color_scale_kind: Some("shear_individual_0_10".to_string()),
            })
            .collect::<Vec<_>>()
    };

    let map_direction_matrix = |direction: &str| {
        let direction_rows = filtered
            .iter()
            .filter(|row| row.wall_direction.eq_ignore_ascii_case(direction))
            .collect::<Vec<_>>();

        let levels = pier
            .story_order
            .iter()
            .filter(|story| direction_rows.iter().any(|row| row.story == **story))
            .cloned()
            .collect::<Vec<_>>();

        let mut piers = ordered_unique(direction_rows.iter().map(|row| row.pier.clone()));
        piers.sort_by(|a, b| compare_pier_labels(a, b));

        let mut values: HashMap<(String, String), f64> = HashMap::new();
        for row in &direction_rows {
            let key = (row.story.clone(), row.pier.clone());
            let entry = values.entry(key).or_insert(0.0);
            *entry = entry.max(row.stress_ratio);
        }

        let mut matrix_ratio = Vec::with_capacity(levels.len());
        for level in &levels {
            let mut row_values = Vec::with_capacity(piers.len());
            for pier_name in &piers {
                row_values.push(values.get(&(level.clone(), pier_name.clone())).copied());
            }
            matrix_ratio.push(row_values);
        }

        PierShearMatrixReportData {
            levels,
            piers,
            matrix_ratio,
            individual_ratio_scale_kind: "shear_individual_0_10".to_string(),
        }
    };

    let map_average_rows = |rows: &[ext_calc::output::PierShearStressAverageRow]| {
        let mut out = rows
            .iter()
            .map(|row| {
                let avg_stress_psi = if row.avg_stress_psi > 0.0 {
                    row.avg_stress_psi
                } else {
                    row.avg_stress_ratio * row.sqrt_fc
                };
                PierShearAverageDetailReportRow {
                    story: row.story.clone(),
                    limit: row.limit_average,
                    avg_stress_psi,
                    sum_area_in2: row.sum_acw_in2,
                    sum_shear_kip: row.sum_ve_kip,
                    avg_ratio: row.avg_stress_ratio,
                    ratio_color_value: Some(row.avg_stress_ratio),
                    ratio_color_scale_kind: Some("shear_average_0_8".to_string()),
                }
            })
            .collect::<Vec<_>>();
        out.sort_by_key(|row| story_rank.get(&row.story).copied().unwrap_or(usize::MAX));
        out
    };

    PierShearReportData {
        supported: true,
        support_note: String::new(),
        phi_v: pier.phi_v,
        limit_individual_ratio: pier.limit_individual,
        limit_average_ratio: pier.limit_average,
        max_individual_ratio: pier.max_individual_ratio,
        max_average_ratio: pier.max_average_ratio,
        pass: pier.pass,
        x_rows: map_direction_rows("X"),
        y_rows: map_direction_rows("Y"),
        x_matrix: map_direction_matrix("X"),
        y_matrix: map_direction_matrix("Y"),
        x_average_rows: map_average_rows(&pier.x_average),
        y_average_rows: map_average_rows(&pier.y_average),
    }
}

pub(super) fn build_unsupported_pier_shear_report_data(
    note: impl Into<String>,
) -> PierShearReportData {
    PierShearReportData {
        supported: false,
        support_note: note.into(),
        phi_v: 0.75,
        limit_individual_ratio: 10.0,
        limit_average_ratio: 8.0,
        max_individual_ratio: 0.0,
        max_average_ratio: 0.0,
        pass: true,
        x_rows: Vec::new(),
        y_rows: Vec::new(),
        x_matrix: PierShearMatrixReportData {
            levels: Vec::new(),
            piers: Vec::new(),
            matrix_ratio: Vec::new(),
            individual_ratio_scale_kind: "shear_individual_0_10".to_string(),
        },
        y_matrix: PierShearMatrixReportData {
            levels: Vec::new(),
            piers: Vec::new(),
            matrix_ratio: Vec::new(),
            individual_ratio_scale_kind: "shear_individual_0_10".to_string(),
        },
        x_average_rows: Vec::new(),
        y_average_rows: Vec::new(),
    }
}
