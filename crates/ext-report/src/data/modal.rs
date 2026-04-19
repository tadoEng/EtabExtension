use serde::Serialize;

use ext_calc::output::ModalOutput;

// ── Modal ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct ModalReportData {
    pub(super) threshold: f64,
    pub(super) pass: bool,
    pub(super) rows: Vec<ModalReportRow>,
    pub(super) annotations: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct ModalReportRow {
    pub(super) mode: i64,
    pub(super) period: f64,
    pub(super) ux: f64,
    pub(super) uy: f64,
    pub(super) uz: f64,
    pub(super) sum_ux: f64,
    pub(super) sum_uy: f64,
    pub(super) sum_uz: f64,
}

pub(super) fn build_modal(modal: &ModalOutput) -> ModalReportData {
    let mut rows = Vec::with_capacity(modal.rows.len());
    let mut annotations = Vec::with_capacity(modal.rows.len());

    for row in &modal.rows {
        let is_ux = modal.mode_reaching_ux == Some(row.mode);
        let is_uy = modal.mode_reaching_uy == Some(row.mode);
        let annotation = match (is_ux, is_uy) {
            (true, true) => "ux_uy_threshold",
            (true, false) => "ux_threshold",
            (false, true) => "uy_threshold",
            (false, false) => {
                if row.ux >= 0.10 || row.uy >= 0.10 {
                    "high"
                } else {
                    ""
                }
            }
        };
        annotations.push(annotation.to_string());
        rows.push(ModalReportRow {
            mode: row.mode,
            period: row.period,
            ux: row.ux,
            uy: row.uy,
            uz: row.uz,
            sum_ux: row.sum_ux,
            sum_uy: row.sum_uy,
            sum_uz: row.sum_uz,
        });
    }

    ModalReportData {
        threshold: modal.threshold,
        pass: modal.pass,
        rows,
        annotations,
    }
}
