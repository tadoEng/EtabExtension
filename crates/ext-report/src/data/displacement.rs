use std::collections::HashMap;

use serde::Serialize;

use ext_calc::output::DisplacementOutput;

use super::ordering::ordered_unique;

// ── Displacement ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct DisplacementReportData {
    pub(super) x: DisplacementDirReport,
    pub(super) y: DisplacementDirReport,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct DisplacementDirReport {
    pub(super) levels: Vec<String>,
    pub(super) groups: Vec<String>,
    pub(super) matrix_in: Vec<Vec<Option<f64>>>,
    pub(super) level_elevations_ft: Vec<f64>,
    pub(super) level_limits_in: Vec<f64>,
    pub(super) level_max_demand_in: Vec<f64>,
    pub(super) level_utilization: Vec<f64>,
    pub(super) governing_limit_in: f64,
    pub(super) governing_utilization: f64,
    pub(super) governing_margin: f64,
    pub(super) governing_story: String,
    pub(super) governing_direction: String,
    pub(super) governing_case: String,
    pub(super) pass: bool,
}

pub(super) fn build_displacement_dir(disp: &DisplacementOutput) -> DisplacementDirReport {
    let to_in = |ft: f64| ft * 12.0;
    let levels = disp
        .story_order
        .iter()
        .filter(|story| disp.rows.iter().any(|row| &row.story == *story))
        .cloned()
        .collect::<Vec<_>>();
    let groups = ordered_unique(disp.rows.iter().map(|row| row.group_name.clone()));
    let is_x = disp.governing.direction.eq_ignore_ascii_case("X");

    let mut values: HashMap<(String, String), f64> = HashMap::new();
    for row in &disp.rows {
        let demand_ft = if is_x {
            row.max_disp_x_pos_ft.abs().max(row.max_disp_x_neg_ft.abs())
        } else {
            row.max_disp_y_pos_ft.abs().max(row.max_disp_y_neg_ft.abs())
        };
        let key = (row.story.clone(), row.group_name.clone());
        let entry = values.entry(key).or_insert(0.0);
        *entry = entry.max(demand_ft);
    }

    let mut limits_by_level = HashMap::new();
    let mut elevations_by_level = HashMap::new();
    for row in &disp.story_limits {
        limits_by_level.insert(row.story.clone(), to_in(row.limit_ft));
        elevations_by_level.insert(row.story.clone(), row.elevation_ft);
    }

    let mut matrix_in = Vec::with_capacity(levels.len());
    let mut level_max_demand_in = Vec::with_capacity(levels.len());
    let mut level_utilization = Vec::with_capacity(levels.len());
    let level_elevations_ft = levels
        .iter()
        .map(|level| elevations_by_level.get(level).copied().unwrap_or(0.0))
        .collect::<Vec<_>>();
    for level in &levels {
        let mut row_values = Vec::with_capacity(groups.len());
        for group in &groups {
            row_values.push(
                values
                    .get(&(level.clone(), group.clone()))
                    .copied()
                    .map(to_in),
            );
        }
        let max_demand = row_values.iter().flatten().copied().fold(0.0_f64, f64::max);
        let limit_in = limits_by_level
            .get(level)
            .copied()
            .unwrap_or(to_in(disp.disp_limit.value));
        let utilization = if limit_in > 1e-9 {
            max_demand / limit_in
        } else {
            0.0
        };
        matrix_in.push(row_values);
        level_max_demand_in.push(max_demand);
        level_utilization.push(utilization);
    }
    let level_limits_in = levels
        .iter()
        .map(|level| {
            limits_by_level
                .get(level)
                .copied()
                .unwrap_or(to_in(disp.disp_limit.value))
        })
        .collect();
    let governing_limit_in = limits_by_level
        .get(&disp.governing.story)
        .copied()
        .unwrap_or(to_in(disp.disp_limit.value));

    DisplacementDirReport {
        levels,
        groups,
        matrix_in,
        level_elevations_ft,
        level_limits_in,
        level_max_demand_in,
        level_utilization,
        governing_limit_in,
        governing_utilization: disp.governing.dcr,
        governing_margin: 1.0 - disp.governing.dcr,
        governing_story: disp.governing.story.clone(),
        governing_direction: disp.governing.direction.clone(),
        governing_case: disp.governing.output_case.clone(),
        pass: disp.pass,
    }
}
