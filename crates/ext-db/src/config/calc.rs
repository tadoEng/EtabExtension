use serde::{Deserialize, Serialize};

/// Shared engineering configuration loaded from .etabs-ext/config.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalcConfig {
    pub code: Option<String>,
    pub occupancy_category: Option<String>,
    pub modal_case: Option<String>,

    #[serde(default)]
    pub joint_tracking_groups: Vec<String>,

    #[serde(default)]
    pub modal: ModalCalcConfig,

    #[serde(rename = "base-reactions", default)]
    pub base_reactions: BaseReactionsCalcConfig,

    #[serde(rename = "story-forces", default)]
    pub story_forces: StoryForcesCalcConfig,

    #[serde(rename = "drift-wind", default)]
    pub drift_wind: DriftDirectionalCalcConfig,

    #[serde(rename = "drift-seismic", default)]
    pub drift_seismic: DriftDirectionalCalcConfig,

    #[serde(rename = "displacement-wind", default)]
    pub displacement_wind: DisplacementDirectionalCalcConfig,

    #[serde(rename = "torsional", default)]
    pub torsional: TorsionalCalcConfig,

    #[serde(rename = "pier-shear-stress-wind", default)]
    pub pier_shear_stress_wind: PierShearStressCalcConfig,

    #[serde(rename = "pier-shear-stress-seismic", default)]
    pub pier_shear_stress_seismic: PierShearStressCalcConfig,

    #[serde(rename = "pier-axial-stress", default)]
    pub pier_axial_stress: PierAxialStressCalcConfig,
}

impl CalcConfig {
    pub fn code_or_default(&self) -> &str {
        self.code.as_deref().unwrap_or("ACI318-14")
    }

    pub fn occupancy_or_default(&self) -> &str {
        self.occupancy_category.as_deref().unwrap_or("II")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModalCalcConfig {
    pub min_mass_participation: Option<f64>,
    pub display_mode_limit: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BaseReactionsCalcConfig {
    pub elf_case_x: Option<String>,
    pub elf_case_y: Option<String>,
    pub rsa_case_x: Option<String>,
    pub rsa_case_y: Option<String>,
    pub rsa_scale_min: Option<f64>,
    #[serde(default)]
    pub pie_groups: Vec<BaseReactionPieGroupConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BaseReactionPieGroupConfig {
    pub label: String,
    #[serde(default)]
    pub load_cases: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoryForcesCalcConfig {
    #[serde(default)]
    pub story_force_x_cases: Vec<String>,
    #[serde(default)]
    pub story_force_y_cases: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DriftDirectionalCalcConfig {
    #[serde(default)]
    pub drift_x_cases: Vec<String>,
    #[serde(default)]
    pub drift_y_cases: Vec<String>,
    pub drift_limit: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DisplacementDirectionalCalcConfig {
    #[serde(default)]
    pub disp_x_cases: Vec<String>,
    #[serde(default)]
    pub disp_y_cases: Vec<String>,
    pub disp_limit_h: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TorsionalCalcConfig {
    #[serde(default)]
    pub torsional_x_case: Vec<String>,
    #[serde(default)]
    pub torsional_y_case: Vec<String>,
    #[serde(default)]
    pub x_joints: Vec<Vec<String>>,
    #[serde(default)]
    pub y_joints: Vec<Vec<String>>,
    pub ecc_ratio: Option<f64>,
    pub building_dim_x_ft: Option<f64>,
    pub building_dim_y_ft: Option<f64>,
}

impl TorsionalCalcConfig {
    pub fn ecc_ratio(&self) -> f64 {
        self.ecc_ratio.unwrap_or(0.05)
    }

    pub fn is_configured(&self) -> bool {
        !self.torsional_x_case.is_empty()
            && !self.torsional_y_case.is_empty()
            && !self.x_joints.is_empty()
            && !self.y_joints.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierShearStressCalcConfig {
    #[serde(default)]
    pub stress_combos: Vec<String>,
    pub phi_v: Option<f64>,
    pub fc_default_ksi: Option<f64>,
}

impl PierShearStressCalcConfig {
    pub fn phi_v(&self) -> f64 {
        self.phi_v.unwrap_or(0.75)
    }

    pub fn fc_default_ksi(&self) -> f64 {
        self.fc_default_ksi.unwrap_or(8.0)
    }

    pub fn is_configured(&self) -> bool {
        !self.stress_combos.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierAxialStressCalcConfig {
    #[serde(default)]
    pub stress_gravity_combos: Vec<String>,
    #[serde(default)]
    pub stress_wind_combos: Vec<String>,
    #[serde(default)]
    pub stress_seismic_combos: Vec<String>,
    pub phi_axial: Option<f64>,
    pub fc_default_ksi: Option<f64>,
}

impl PierAxialStressCalcConfig {
    pub fn phi_axial(&self) -> f64 {
        self.phi_axial.unwrap_or(0.65)
    }

    pub fn all_combos(&self) -> Vec<&str> {
        self.stress_gravity_combos
            .iter()
            .chain(&self.stress_wind_combos)
            .chain(&self.stress_seismic_combos)
            .map(String::as_str)
            .collect()
    }

    pub fn is_configured(&self) -> bool {
        !self.all_combos().is_empty()
    }
}
