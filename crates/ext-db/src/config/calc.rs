use serde::{Deserialize, Serialize};

/// Shared engineering configuration loaded from .etabs-ext/config.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalcConfig {
    pub code: Option<String>,
    pub occupancy_category: Option<String>,
    pub modal_case: Option<String>,

    #[serde(default)]
    pub drift_tracking_groups: Vec<String>,

    #[serde(default)]
    pub modal: ModalCalcConfig,

    #[serde(default)]
    pub base_shear: BaseShearCalcConfig,

    #[serde(rename = "drift-wind", default)]
    pub drift_wind: DriftCalcConfig,

    #[serde(rename = "drift-seismic", default)]
    pub drift_seismic: DriftCalcConfig,

    #[serde(rename = "pier-shear-wind", default)]
    pub pier_shear_wind: PierShearCalcConfig,

    #[serde(rename = "pier-shear-seismic", default)]
    pub pier_shear_seismic: PierShearCalcConfig,

    #[serde(rename = "pier-axial", default)]
    pub pier_axial: PierAxialCalcConfig,
}

impl CalcConfig {
    pub fn code_or_default(&self) -> &str {
        self.code.as_deref().unwrap_or("ACI318-14")
    }

    pub fn occupancy_or_default(&self) -> &str {
        self.occupancy_category.as_deref().unwrap_or("II")
    }

    pub fn modal_case_or_default(&self) -> &str {
        self.modal_case.as_deref().unwrap_or("Modal-Rizt")
    }

    pub fn drift_groups_or_default(&self) -> Vec<String> {
        if self.drift_tracking_groups.is_empty() {
            vec!["Tracking_Points".to_string()]
        } else {
            self.drift_tracking_groups.clone()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModalCalcConfig {
    pub min_mass_participation: Option<f64>,
}

impl ModalCalcConfig {
    pub fn threshold(&self) -> f64 {
        self.min_mass_participation.unwrap_or(0.90)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BaseShearCalcConfig {
    pub elf_case_x: Option<String>,
    pub elf_case_y: Option<String>,
    pub rsa_case_x: Option<String>,
    pub rsa_case_y: Option<String>,
    pub rsa_scale_min: Option<f64>,
}

impl BaseShearCalcConfig {
    pub fn elf_x(&self) -> &str {
        self.elf_case_x.as_deref().unwrap_or("ELF_X")
    }

    pub fn elf_y(&self) -> &str {
        self.elf_case_y.as_deref().unwrap_or("ELF_Y")
    }

    pub fn rsa_x(&self) -> &str {
        self.rsa_case_x.as_deref().unwrap_or("RSA_X")
    }

    pub fn rsa_y(&self) -> &str {
        self.rsa_case_y.as_deref().unwrap_or("RSA_Y")
    }

    pub fn scale_min(&self) -> f64 {
        self.rsa_scale_min.unwrap_or(1.0)
    }

    pub fn all_cases(&self) -> Vec<String> {
        vec![
            self.elf_x().to_string(),
            self.elf_y().to_string(),
            self.rsa_x().to_string(),
            self.rsa_y().to_string(),
        ]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DriftCalcConfig {
    #[serde(default)]
    pub load_cases: Vec<String>,

    pub drift_limit: Option<f64>,
    pub disp_limit_h: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierShearCalcConfig {
    #[serde(default)]
    pub load_combos: Vec<String>,

    pub phi_v: Option<f64>,
    pub alpha_c: Option<f64>,
    pub fy_ksi: Option<f64>,
    pub rho_t: Option<f64>,
    pub fc_default_ksi: Option<f64>,
}

impl PierShearCalcConfig {
    pub fn phi_v(&self, default: f64) -> f64 {
        self.phi_v.unwrap_or(default)
    }

    pub fn alpha_c(&self) -> f64 {
        self.alpha_c.unwrap_or(2.0)
    }

    pub fn fy_ksi(&self) -> f64 {
        self.fy_ksi.unwrap_or(60.0)
    }

    pub fn rho_t(&self) -> f64 {
        self.rho_t.unwrap_or(0.0025)
    }

    pub fn fc_default_ksi(&self) -> f64 {
        self.fc_default_ksi.unwrap_or(8.0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierAxialCalcConfig {
    #[serde(default)]
    pub load_combos: Vec<String>,
    pub phi_axial: Option<f64>,
}

impl PierAxialCalcConfig {
    pub fn phi_axial(&self) -> f64 {
        self.phi_axial.unwrap_or(0.65)
    }
}
