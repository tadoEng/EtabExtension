use serde::Serialize;

use ext_calc::output::{TorsionalDirectionOutput, TorsionalOutput};

use super::format::wrap_load_case_label;

// ── Torsional ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct TorsionalReportData {
    pub(super) x: TorsionalDirReport,
    pub(super) y: TorsionalDirReport,
    pub(super) pass: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct TorsionalDirReport {
    pub(super) rows: Vec<TorsionalReportRow>,
    pub(super) annotations: Vec<String>,
    pub(super) governing_story: String,
    pub(super) governing_case: String,
    pub(super) governing_joint_a: String,
    pub(super) governing_joint_b: String,
    pub(super) governing_step: Option<i32>,
    pub(super) governing_drift_a: f64,
    pub(super) governing_drift_b: f64,
    pub(super) governing_delta_max: f64,
    pub(super) governing_delta_avg: f64,
    pub(super) governing_ratio: f64,
    pub(super) governing_ratio_color_value: Option<f64>,
    pub(super) governing_ratio_color_scale_kind: Option<String>,
    pub(super) type_a_threshold: f64,
    pub(super) type_b_threshold: f64,
    pub(super) classification: String,
    pub(super) has_type_a: bool,
    pub(super) has_type_b: bool,
    pub(super) has_rows: bool,
    pub(super) no_data_note: String,
    pub(super) no_data_contexts: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct TorsionalReportRow {
    pub(super) story: String,
    pub(super) case: String,
    pub(super) joint_a: String,
    pub(super) joint_b: String,
    pub(super) governing_step: i32,
    pub(super) drift_a: f64,
    pub(super) drift_b: f64,
    pub(super) delta_max: f64,
    pub(super) delta_avg: f64,
    pub(super) ratio: f64,
    pub(super) ratio_color_value: Option<f64>,
    pub(super) ratio_color_scale_kind: Option<String>,
    pub(super) is_type_a: bool,
    pub(super) is_type_b: bool,
    pub(super) ax: f64,
    pub(super) ecc_ft: f64,
}

pub(super) fn build_torsional(torsional: &TorsionalOutput) -> TorsionalReportData {
    TorsionalReportData {
        x: build_torsional_dir(&torsional.x),
        y: build_torsional_dir(&torsional.y),
        pass: torsional.pass,
    }
}

pub(super) fn default_torsional_report_data() -> TorsionalReportData {
    TorsionalReportData {
        x: default_torsional_dir_report(),
        y: default_torsional_dir_report(),
        pass: true,
    }
}

fn default_torsional_dir_report() -> TorsionalDirReport {
    TorsionalDirReport {
        rows: Vec::new(),
        annotations: Vec::new(),
        governing_story: String::new(),
        governing_case: String::new(),
        governing_joint_a: String::new(),
        governing_joint_b: String::new(),
        governing_step: None,
        governing_drift_a: 0.0,
        governing_drift_b: 0.0,
        governing_delta_max: 0.0,
        governing_delta_avg: 0.0,
        governing_ratio: 0.0,
        governing_ratio_color_value: Some(0.0),
        governing_ratio_color_scale_kind: Some("torsion_thresholds_1_2_1_4".to_string()),
        type_a_threshold: 1.2,
        type_b_threshold: 1.4,
        classification: "No data".to_string(),
        has_type_a: false,
        has_type_b: false,
        has_rows: false,
        no_data_note: "No torsional data available.".to_string(),
        no_data_contexts: Vec::new(),
    }
}

pub(super) fn build_torsional_dir(dir: &TorsionalDirectionOutput) -> TorsionalDirReport {
    let type_a_threshold = 1.2;
    let type_b_threshold = 1.4;
    let mut rows = Vec::with_capacity(dir.rows.len());
    let mut annotations = Vec::with_capacity(dir.rows.len());

    for row in &dir.rows {
        let annotation = if row.is_type_b {
            "fail"
        } else if row.is_type_a {
            "warn"
        } else {
            ""
        };
        annotations.push(annotation.to_string());
        rows.push(TorsionalReportRow {
            story: row.story.clone(),
            case: wrap_load_case_label(&row.case),
            joint_a: row.joint_a.clone(),
            joint_b: row.joint_b.clone(),
            governing_step: row.governing_step,
            drift_a: row.governing_drift_a,
            drift_b: row.governing_drift_b,
            delta_max: row.governing_delta_max,
            delta_avg: row.governing_delta_avg,
            ratio: row.governing_ratio,
            ratio_color_value: Some(row.governing_ratio),
            ratio_color_scale_kind: Some("torsion_thresholds_1_2_1_4".to_string()),
            is_type_a: row.is_type_a,
            is_type_b: row.is_type_b,
            ax: row.ax,
            ecc_ft: row.ecc_ft,
        });
    }

    let governing_row = rows
        .iter()
        .max_by(|left, right| {
            left.ratio
                .partial_cmp(&right.ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();
    let classification = if dir.has_type_b {
        "Type B".to_string()
    } else if dir.has_type_a {
        "Type A".to_string()
    } else if rows.is_empty() {
        "No data".to_string()
    } else {
        "None".to_string()
    };
    let has_rows = !rows.is_empty();

    TorsionalDirReport {
        rows,
        annotations,
        governing_story: dir.governing_story.clone(),
        governing_case: dir.governing_case.clone(),
        governing_joint_a: governing_row
            .as_ref()
            .map(|row| row.joint_a.clone())
            .or_else(|| dir.governing_joints.first().cloned())
            .unwrap_or_default(),
        governing_joint_b: governing_row
            .as_ref()
            .map(|row| row.joint_b.clone())
            .or_else(|| dir.governing_joints.get(1).cloned())
            .unwrap_or_default(),
        governing_step: dir.governing_step,
        governing_drift_a: governing_row
            .as_ref()
            .map(|row| row.drift_a)
            .unwrap_or_default(),
        governing_drift_b: governing_row
            .as_ref()
            .map(|row| row.drift_b)
            .unwrap_or_default(),
        governing_delta_max: governing_row
            .as_ref()
            .map(|row| row.delta_max)
            .unwrap_or_default(),
        governing_delta_avg: governing_row
            .as_ref()
            .map(|row| row.delta_avg)
            .unwrap_or_default(),
        governing_ratio: dir.max_ratio,
        governing_ratio_color_value: Some(dir.max_ratio),
        governing_ratio_color_scale_kind: Some("torsion_thresholds_1_2_1_4".to_string()),
        type_a_threshold,
        type_b_threshold,
        classification,
        has_type_a: dir.has_type_a,
        has_type_b: dir.has_type_b,
        has_rows,
        no_data_note: if has_rows {
            String::new()
        } else {
            "No qualifying rows for configured pairs and cases.".to_string()
        },
        no_data_contexts: dir
            .no_data
            .iter()
            .map(|row| {
                format!(
                    "story={} case={} pair={}/{} step={} missing={}",
                    row.story,
                    row.case,
                    row.joint_a,
                    row.joint_b,
                    row.step,
                    row.missing.join(", ")
                )
            })
            .collect(),
    }
}
