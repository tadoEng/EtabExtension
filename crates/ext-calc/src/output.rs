use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

impl Quantity {
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitLabels {
    pub force: String,
    pub length: String,
    pub stress: String,
    pub moment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcMeta {
    pub version_id: String,
    pub branch: String,
    pub code: String,
    pub generated_at: DateTime<Utc>,
    pub units: UnitLabels,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryLine {
    pub key: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcSummary {
    pub overall_status: String,
    pub check_count: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub lines: Vec<SummaryLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryDriftResult {
    pub story: String,
    pub group_name: String,
    pub output_case: String,
    pub direction: String,
    pub sense: String,
    pub drift_ratio: f64,
    pub dcr: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftEnvelopeRow {
    pub story: String,
    pub group_name: String,
    pub output_case: String,
    pub max_disp_x_pos_ft: f64,
    pub max_disp_x_neg_ft: f64,
    pub max_disp_y_pos_ft: f64,
    pub max_disp_y_neg_ft: f64,
    pub max_drift_x_pos: f64,
    pub max_drift_x_neg: f64,
    pub max_drift_y_pos: f64,
    pub max_drift_y_neg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftOutput {
    pub allowable_ratio: f64,
    pub rows: Vec<DriftEnvelopeRow>,
    pub governing: StoryDriftResult,
    pub pass: bool,
    pub roof_disp_x: Option<Quantity>,
    pub roof_disp_y: Option<Quantity>,
    pub disp_limit: Option<Quantity>,
    pub disp_pass: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementEnvelopeRow {
    pub story: String,
    pub group_name: String,
    pub output_case: String,
    pub max_disp_x_pos_ft: f64,
    pub max_disp_x_neg_ft: f64,
    pub max_disp_y_pos_ft: f64,
    pub max_disp_y_neg_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointDisplacementResult {
    pub story: String,
    pub group_name: String,
    pub output_case: String,
    pub direction: String,
    pub sense: String,
    pub displacement: Quantity,
    pub dcr: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementOutput {
    pub rows: Vec<DisplacementEnvelopeRow>,
    pub governing: JointDisplacementResult,
    pub disp_limit: Quantity,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseShearDir {
    pub rsa_case: String,
    pub elf_case: String,
    pub v_rsa: Quantity,
    pub v_elf: Quantity,
    pub ratio: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseReactionCheckRow {
    pub output_case: String,
    pub case_type: String,
    pub step_type: String,
    pub step_number: Option<f64>,
    pub fx_kip: f64,
    pub fy_kip: f64,
    pub fz_kip: f64,
    pub mx_kip_ft: f64,
    pub my_kip_ft: f64,
    pub mz_kip_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseShearOutput {
    /// Review table: carries filtered base-reaction rows for reporting, while the
    /// actual design summary is computed only from configured ELF/RSA cases.
    pub rows: Vec<BaseReactionCheckRow>,
    pub direction_x: BaseShearDir,
    pub direction_y: BaseShearDir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalModeRow {
    pub case: String,
    pub mode: i64,
    pub period: f64,
    pub ux: f64,
    pub uy: f64,
    pub sum_ux: f64,
    pub sum_uy: f64,
    pub rz: f64,
    pub sum_rz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalOutput {
    pub rows: Vec<ModalModeRow>,
    pub threshold: f64,
    pub mode_reaching_ux: Option<i64>,
    pub mode_reaching_uy: Option<i64>,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalOutput {
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearResult {
    pub pier_label: String,
    pub story: String,
    pub combo: String,
    pub location: String,
    pub vu: Quantity,
    pub acv: Quantity,
    pub fc_ksi: f64,
    pub vn: Quantity,
    pub phi_vn: Quantity,
    pub dcr: f64,
    pub pass: bool,
    pub section_id: String,
    pub material: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearOutput {
    pub phi_v: f64,
    pub piers: Vec<PierShearResult>,
    pub governing: PierShearResult,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierAxialResult {
    pub pier_label: String,
    pub story: String,
    pub combo: String,
    pub pu: Quantity,
    pub ag: Quantity,
    pub phi_po: Quantity,
    pub fa: Quantity,
    pub fa_ratio: f64,
    pub dcr: f64,
    pub pass: bool,
    pub fc_ksi: f64,
    pub material: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierAxialOutput {
    pub piers: Vec<PierAxialResult>,
    pub governing: PierAxialResult,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcOutput {
    pub meta: CalcMeta,
    pub summary: CalcSummary,
    pub modal: Option<ModalOutput>,
    pub base_shear: Option<BaseShearOutput>,
    pub drift_wind: Option<DriftOutput>,
    pub drift_seismic: Option<DriftOutput>,
    pub displacement_wind: Option<DisplacementOutput>,
    pub torsional: Option<TorsionalOutput>,
    pub pier_shear_wind: Option<PierShearOutput>,
    pub pier_shear_seismic: Option<PierShearOutput>,
    pub pier_axial: Option<PierAxialOutput>,
}
