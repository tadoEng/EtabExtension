use std::collections::HashMap;

use serde::Serialize;

use ext_calc::output::DriftOutput;

use super::ordering::ordered_unique;

// ── Drift ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct DriftReportData {
    pub(super) x: DriftDirReport,
    pub(super) y: DriftDirReport,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct DriftDirReport {
    pub(super) levels: Vec<String>,
    pub(super) groups: Vec<String>,
    pub(super) matrix: Vec<Vec<Option<f64>>>,
    pub(super) allowable_ratio: f64,
    pub(super) governing_demand_ratio: f64,
    pub(super) governing_utilization: f64,
    pub(super) governing_margin_ratio: f64,
    pub(super) governing_story: String,
    pub(super) governing_direction: String,
    pub(super) governing_case: String,
    pub(super) pass: bool,
}

pub(super) fn build_drift_dir(drift: &DriftOutput) -> DriftDirReport {
    let levels = drift
        .story_order
        .iter()
        .filter(|story| drift.rows.iter().any(|row| &row.story == *story))
        .cloned()
        .collect::<Vec<_>>();
    let groups = ordered_unique(drift.rows.iter().map(|row| row.group_name.clone()));
    let is_x = drift.governing.direction.eq_ignore_ascii_case("X");

    let mut values: HashMap<(String, String), f64> = HashMap::new();
    for row in &drift.rows {
        let demand = if is_x {
            row.max_drift_x_pos.abs().max(row.max_drift_x_neg.abs())
        } else {
            row.max_drift_y_pos.abs().max(row.max_drift_y_neg.abs())
        };
        let key = (row.story.clone(), row.group_name.clone());
        let entry = values.entry(key).or_insert(0.0);
        *entry = entry.max(demand);
    }

    let mut matrix = Vec::with_capacity(levels.len());
    for level in &levels {
        let mut row_values = Vec::with_capacity(groups.len());
        for group in &groups {
            row_values.push(values.get(&(level.clone(), group.clone())).copied());
        }
        matrix.push(row_values);
    }

    DriftDirReport {
        levels,
        groups,
        matrix,
        allowable_ratio: drift.allowable_ratio,
        governing_demand_ratio: drift.governing.drift_ratio,
        governing_utilization: drift.governing.dcr,
        governing_margin_ratio: drift.allowable_ratio - drift.governing.drift_ratio,
        governing_story: drift.governing.story.clone(),
        governing_direction: drift.governing.direction.clone(),
        governing_case: drift.governing.output_case.clone(),
        pass: drift.pass,
    }
}
