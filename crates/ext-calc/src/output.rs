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
pub struct BaseReactionDir {
    pub rsa_case: String,
    pub elf_case: String,
    pub v_rsa: Quantity,
    pub v_elf: Quantity,
    pub ratio: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseReactionsOutput {
    pub rows: Vec<BaseReactionCheckRow>,
    pub direction_x: BaseReactionDir,
    pub direction_y: BaseReactionDir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForceEnvelopeRow {
    pub story: String,
    pub max_vx_kip: f64,
    pub max_my_kip_ft: f64,
    pub max_vy_kip: f64,
    pub max_mx_kip_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForcesOutput {
    pub rows: Vec<StoryForceEnvelopeRow>,
    #[serde(default)]
    pub story_order: Vec<String>,
    #[serde(default)]
    pub x_profiles: Vec<StoryForceCaseProfile>,
    #[serde(default)]
    pub y_profiles: Vec<StoryForceCaseProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForceCaseProfile {
    pub output_case: String,
    pub rows: Vec<StoryForceCaseRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForceCaseRow {
    pub story: String,
    pub elevation_ft: f64,
    pub vx_kip: f64,
    pub vy_kip: f64,
    pub mx_kip_ft: f64,
    pub my_kip_ft: f64,
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
pub struct DriftOutput {
    pub allowable_ratio: f64,
    pub rows: Vec<DriftEnvelopeRow>,
    #[serde(default)]
    pub story_order: Vec<String>,
    pub governing: StoryDriftResult,
    pub pass: bool,
    pub roof_disp_x: Option<Quantity>,
    pub roof_disp_y: Option<Quantity>,
    pub disp_limit: Option<Quantity>,
    pub disp_pass: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftWindOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftSeismicOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
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
    #[serde(default)]
    pub story_order: Vec<String>,
    #[serde(default)]
    pub story_limits: Vec<DisplacementLimitRow>,
    pub governing: JointDisplacementResult,
    pub disp_limit: Quantity,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementLimitRow {
    pub story: String,
    pub elevation_ft: f64,
    pub limit_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementWindOutput {
    pub x: DisplacementOutput,
    pub y: DisplacementOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalRow {
    pub story: String,
    pub case: String,
    pub joint_a: String,
    pub joint_b: String,
    pub drift_a_steps: Vec<f64>,
    pub drift_b_steps: Vec<f64>,
    pub delta_max_steps: Vec<f64>,
    pub delta_avg_steps: Vec<f64>,
    pub ratio: f64,
    pub ax: f64,
    pub ecc_ft: f64,
    pub rho: f64,
    pub is_type_a: bool,
    pub is_type_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalDirectionOutput {
    pub rows: Vec<TorsionalRow>,
    pub governing_story: String,
    pub governing_case: String,
    pub governing_joints: Vec<String>,
    pub max_ratio: f64,
    pub has_type_a: bool,
    pub has_type_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalOutput {
    pub x: TorsionalDirectionOutput,
    pub y: TorsionalDirectionOutput,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressRow {
    pub story: String,
    pub pier: String,
    pub combo: String,
    pub wall_direction: String,
    pub acw_in2: f64,
    pub fc_psi: f64,
    pub sqrt_fc: f64,
    pub ve_kip: f64,
    pub stress_psi: f64,
    pub stress_ratio: f64,
    pub limit_individual: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressAverageRow {
    pub story: String,
    pub wall_direction: String,
    pub sum_ve_kip: f64,
    pub sum_acw_in2: f64,
    pub sqrt_fc: f64,
    pub avg_stress_ratio: f64,
    pub limit_average: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressOutput {
    pub phi_v: f64,
    pub limit_individual: f64,
    pub limit_average: f64,
    #[serde(default)]
    pub story_order: Vec<String>,
    pub per_pier: Vec<PierShearStressRow>,
    pub x_average: Vec<PierShearStressAverageRow>,
    pub y_average: Vec<PierShearStressAverageRow>,
    pub max_individual_ratio: f64,
    pub max_average_ratio: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierAxialResult {
    pub pier_label: String,
    pub story: String,
    pub combo: String,
    /// Load category: "gravity", "wind", or "seismic".
    #[serde(default)]
    pub category: String,
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
pub struct PierAxialStressOutput {
    pub phi_axial: f64,
    pub piers: Vec<PierAxialResult>,
    pub governing_gravity: Option<PierAxialResult>,
    pub governing_wind: Option<PierAxialResult>,
    pub governing_seismic: Option<PierAxialResult>,
    pub governing: PierAxialResult,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcOutput {
    pub meta: CalcMeta,
    pub summary: CalcSummary,
    pub modal: Option<ModalOutput>,
    pub base_reactions: Option<BaseReactionsOutput>,
    pub story_forces: Option<StoryForcesOutput>,
    pub drift_wind: Option<DriftWindOutput>,
    pub drift_seismic: Option<DriftSeismicOutput>,
    pub displacement_wind: Option<DisplacementWindOutput>,
    pub torsional: Option<TorsionalOutput>,
    pub pier_shear_stress_wind: Option<PierShearStressOutput>,
    pub pier_shear_stress_seismic: Option<PierShearStressOutput>,
    pub pier_axial_stress: Option<PierAxialStressOutput>,
}
