use serde::Serialize;

use ext_calc::output::BaseReactionsOutput;

use super::format::wrap_load_case_label;

// ── Base Reactions ───────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct BaseReactionsReportData {
    pub(super) rows: Vec<BaseReactionsReportRow>,
    pub(super) annotations: Vec<String>,
    pub(super) pass: bool,
    pub(super) ratio_x: f64,
    pub(super) ratio_y: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct BaseReactionsReportRow {
    pub(super) output_case: String,
    pub(super) case_type: String,
    pub(super) step_type: String,
    pub(super) step_number: Option<f64>,
    pub(super) fx_kip: f64,
    pub(super) fy_kip: f64,
    pub(super) fz_kip: f64,
    pub(super) mx_kip_ft: f64,
    pub(super) my_kip_ft: f64,
    pub(super) mz_kip_ft: f64,
}

pub(super) fn build_base_reactions(base: &BaseReactionsOutput) -> BaseReactionsReportData {
    let rows = base
        .rows
        .iter()
        .filter(|row| !should_exclude_base_case_type(&row.case_type))
        .map(|row| BaseReactionsReportRow {
            output_case: wrap_load_case_label(&row.output_case),
            case_type: row.case_type.clone(),
            step_type: row.step_type.clone(),
            step_number: row.step_number,
            fx_kip: row.fx_kip.abs(),
            fy_kip: row.fy_kip.abs(),
            fz_kip: row.fz_kip.abs(),
            mx_kip_ft: row.mx_kip_ft.abs(),
            my_kip_ft: row.my_kip_ft.abs(),
            mz_kip_ft: row.mz_kip_ft.abs(),
        })
        .collect::<Vec<_>>();
    let row_count = rows.len();

    BaseReactionsReportData {
        rows,
        annotations: vec![String::new(); row_count],
        pass: base.direction_x.pass && base.direction_y.pass,
        ratio_x: base.direction_x.ratio,
        ratio_y: base.direction_y.ratio,
    }
}

fn should_exclude_base_case_type(case_type: &str) -> bool {
    let normalized = case_type.trim().to_ascii_lowercase();
    normalized == "combination" || normalized == "linmodritz" || normalized == "eigen"
}
