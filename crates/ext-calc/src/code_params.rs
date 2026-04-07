use anyhow::{Result, bail};
use ext_db::config::Config;

use crate::checks::CheckSelection;
use crate::unit_convert::UnitContext;

#[derive(Debug, Clone)]
pub struct BaseShearParams {
    pub elf_case_x: String,
    pub elf_case_y: String,
    pub rsa_case_x: String,
    pub rsa_case_y: String,
    pub rsa_scale_min: f64,
}

#[derive(Debug, Clone)]
pub struct DriftParams {
    pub load_cases: Vec<String>,
    pub drift_limit: f64,
}

#[derive(Debug, Clone)]
pub struct DisplacementParams {
    pub load_cases: Vec<String>,
    pub disp_limit_h: u32,
}

/// Parameters for pier shear wall checks (ACI 318-14 §11.5.4.3 / §18.10.4).
/// Wind and seismic checks share the same formula — only ϕ differs:
///   Wind:    ϕ = 0.75  (ACI 318-14 §9.3.2.3)
///   Seismic: ϕ = 0.60  (ACI 318-14 §21.2.4.1)
#[derive(Debug, Clone)]
pub struct PierShearParams {
    pub load_combos: Vec<String>,
    /// Shear strength reduction factor ϕ.
    pub phi_v: f64,
    /// αc coefficient, psi-based form: 2.0 for hw/lw ≥ 2.0.
    /// ACI 318-14 §11.5.4.3.  Must be used with fc_psi, NOT fc_ksi.
    pub alpha_c: f64,
    /// Horizontal reinforcement yield strength (ksi). Grade 60 = 60.0.
    pub fy_ksi: f64,
    /// Horizontal reinforcement ratio ρt. ACI minimum: 0.0025.
    pub rho_t: f64,
    /// Fallback f'c (ksi) when material join yields no match for a pier/story.
    pub fc_default_ksi: f64,
}

/// Parameters for pier axial stress check (ACI 318-14 §22.4).
#[derive(Debug, Clone)]
pub struct PierAxialParams {
    pub load_combos: Vec<String>,
    /// Axial strength reduction factor ϕ.
    /// ACI 318-14 §9.3.2.2 tied: 0.65.
    pub phi_axial: f64,
}

#[derive(Debug, Clone)]
pub struct CodeParams {
    pub code: String,
    pub occupancy_category: String,
    pub modal_case: String,
    pub modal_threshold: f64,
    pub modal_display_limit: usize,
    pub drift_tracking_groups: Vec<String>,
    pub base_shear: BaseShearParams,
    pub drift_wind: DriftParams,
    pub drift_seismic: DriftParams,
    pub displacement_wind: DisplacementParams,
    pub pier_shear_wind: PierShearParams,
    pub pier_shear_seismic: PierShearParams,
    pub pier_axial: PierAxialParams,
    pub check_selection: CheckSelection,
    pub unit_context: UnitContext,
}

impl CodeParams {
    pub fn from_config(config: &Config) -> Result<Self> {
        let modal_case = required_string(config.calc.modal_case.as_deref(), "[calc].modal-case")?;
        let modal_threshold = required_positive_f64(
            config.calc.modal.min_mass_participation,
            "[calc.modal].min-mass-participation",
        )?;
        let modal_display_limit = usize::try_from(required_positive_u32(
            config.calc.modal.display_mode_limit,
            "[calc.modal].display-mode-limit",
        )?)
        .map_err(|_| anyhow::anyhow!("invalid required config: [calc.modal].display-mode-limit"))?;
        let drift_tracking_groups = required_string_list(
            &config.calc.drift_tracking_groups,
            "[calc].drift-tracking-groups",
        )?;
        let elf_case_x = required_string(
            config.calc.base_shear.elf_case_x.as_deref(),
            "[calc.base-shear].elf-case-x",
        )?;
        let elf_case_y = required_string(
            config.calc.base_shear.elf_case_y.as_deref(),
            "[calc.base-shear].elf-case-y",
        )?;
        let rsa_case_x = required_string(
            config.calc.base_shear.rsa_case_x.as_deref(),
            "[calc.base-shear].rsa-case-x",
        )?;
        let rsa_case_y = required_string(
            config.calc.base_shear.rsa_case_y.as_deref(),
            "[calc.base-shear].rsa-case-y",
        )?;
        let rsa_scale_min = required_positive_f64(
            config.calc.base_shear.rsa_scale_min,
            "[calc.base-shear].rsa-scale-min",
        )?;
        let drift_wind_load_cases = required_string_list(
            &config.calc.drift_wind.load_cases,
            "[calc.drift-wind].load-cases",
        )?;
        let drift_wind_limit = required_positive_f64(
            config.calc.drift_wind.drift_limit,
            "[calc.drift-wind].drift-limit",
        )?;
        let drift_seismic_load_cases = required_string_list(
            &config.calc.drift_seismic.load_cases,
            "[calc.drift-seismic].load-cases",
        )?;
        let drift_seismic_limit = required_positive_f64(
            config.calc.drift_seismic.drift_limit,
            "[calc.drift-seismic].drift-limit",
        )?;
        let displacement_wind_load_cases = required_string_list(
            &config.calc.displacement_wind.load_cases,
            "[calc.displacement-wind].load-cases",
        )?;
        let displacement_wind_disp_limit_h = required_positive_u32(
            config.calc.displacement_wind.disp_limit_h,
            "[calc.displacement-wind].disp-limit-h",
        )?;

        let pier_shear_wind_combos = required_string_list(
            &config.calc.pier_shear_wind.load_combos,
            "[calc.pier-shear-wind].load-combos",
        )?;
        let pier_shear_seismic_combos = required_string_list(
            &config.calc.pier_shear_seismic.load_combos,
            "[calc.pier-shear-seismic].load-combos",
        )?;
        let pier_axial_combos = required_string_list(
            &config.calc.pier_axial.load_combos,
            "[calc.pier-axial].load-combos",
        )?;

        Ok(Self {
            code: config.calc.code_or_default().to_string(),
            occupancy_category: config.calc.occupancy_or_default().to_string(),
            modal_case,
            modal_threshold,
            modal_display_limit,
            drift_tracking_groups,
            base_shear: BaseShearParams {
                elf_case_x,
                elf_case_y,
                rsa_case_x,
                rsa_case_y,
                rsa_scale_min,
            },
            drift_wind: DriftParams {
                load_cases: drift_wind_load_cases,
                drift_limit: drift_wind_limit,
            },
            drift_seismic: DriftParams {
                load_cases: drift_seismic_load_cases,
                drift_limit: drift_seismic_limit,
            },
            displacement_wind: DisplacementParams {
                load_cases: displacement_wind_load_cases,
                disp_limit_h: displacement_wind_disp_limit_h,
            },
            pier_shear_wind: PierShearParams {
                load_combos: pier_shear_wind_combos,
                phi_v: config.calc.pier_shear_wind.phi_v(0.75),
                alpha_c: config.calc.pier_shear_wind.alpha_c(),
                fy_ksi: config.calc.pier_shear_wind.fy_ksi(),
                rho_t: config.calc.pier_shear_wind.rho_t(),
                fc_default_ksi: config.calc.pier_shear_wind.fc_default_ksi(),
            },
            pier_shear_seismic: PierShearParams {
                load_combos: pier_shear_seismic_combos,
                phi_v: config.calc.pier_shear_seismic.phi_v(0.60),
                alpha_c: config.calc.pier_shear_seismic.alpha_c(),
                fy_ksi: config.calc.pier_shear_seismic.fy_ksi(),
                rho_t: config.calc.pier_shear_seismic.rho_t(),
                fc_default_ksi: config.calc.pier_shear_seismic.fc_default_ksi(),
            },
            pier_axial: PierAxialParams {
                load_combos: pier_axial_combos,
                phi_axial: config.calc.pier_axial.phi_axial(),
            },
            check_selection: CheckSelection::default(),
            unit_context: UnitContext::from_config(config)?,
        })
    }
}

fn required_string(value: Option<&str>, key: &str) -> Result<String> {
    match value.map(str::trim) {
        Some(value) if !value.is_empty() => Ok(value.to_string()),
        _ => bail!("missing required config: {key}"),
    }
}

fn required_string_list(values: &[String], key: &str) -> Result<Vec<String>> {
    if values.is_empty() {
        bail!("missing required config: {key}");
    }

    let normalized = values
        .iter()
        .map(|value| value.trim())
        .map(|value| {
            if value.is_empty() {
                bail!("missing required config: {key}");
            }
            Ok(value.to_string())
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(normalized)
}

fn required_positive_f64(value: Option<f64>, key: &str) -> Result<f64> {
    match value {
        Some(value) if value > 0.0 => Ok(value),
        Some(_) => bail!("invalid required config: {key} must be greater than zero"),
        None => bail!("missing required config: {key}"),
    }
}

fn required_positive_u32(value: Option<u32>, key: &str) -> Result<u32> {
    match value {
        Some(value) if value > 0 => Ok(value),
        Some(_) => bail!("invalid required config: {key} must be greater than zero"),
        None => bail!("missing required config: {key}"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use super::CodeParams;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    fn base_valid_config() -> Config {
        let mut config = Config::default();
        config.project.units = Some("kip-ft-F".into());
        config.calc.modal_case = Some("Modal-Rizt".into());
        config.calc.drift_tracking_groups = vec!["Tracking_Points".into()];
        config.calc.modal.min_mass_participation = Some(0.90);
        config.calc.modal.display_mode_limit = Some(20);
        config.calc.base_shear.elf_case_x = Some("ELF_X".into());
        config.calc.base_shear.elf_case_y = Some("ELF_Y".into());
        config.calc.base_shear.rsa_case_x = Some("RSA_X".into());
        config.calc.base_shear.rsa_case_y = Some("RSA_Y".into());
        config.calc.base_shear.rsa_scale_min = Some(1.0);
        config.calc.drift_wind.load_cases = vec!["Wind_ASCE_10yr".into()];
        config.calc.drift_wind.drift_limit = Some(0.0025);
        config.calc.drift_seismic.load_cases = vec!["RSA_Y_Drift".into()];
        config.calc.drift_seismic.drift_limit = Some(0.020);
        config.calc.displacement_wind.load_cases = vec!["Wind_10yr_Diagonal".into()];
        config.calc.displacement_wind.disp_limit_h = Some(500);
        config.calc.pier_shear_wind.load_combos = vec!["EVN_LRFD_EQ".into()];
        config.calc.pier_shear_seismic.load_combos = vec!["EVN_LRFD_EQ".into()];
        config.calc.pier_axial.load_combos = vec!["EVN_LRFD_EQ".into()];
        config
    }

    #[test]
    fn code_params_pick_up_shared_calc_and_local_units() {
        let config = Config::load(&fixture_dir()).unwrap();

        let params = CodeParams::from_config(&config).unwrap();
        assert_eq!(params.code, "ACI318-14");
        assert_eq!(params.modal_case, "Modal-Rizt");
        assert!((params.modal_threshold - 0.90).abs() < 1e-9);
        assert_eq!(params.modal_display_limit, 20);
        assert_eq!(params.unit_context.force_label(), "kip");
        assert_eq!(params.displacement_wind.disp_limit_h, 500);
    }

    #[test]
    fn code_params_require_calc_driving_values() {
        let mut config = Config::default();
        config.project.units = Some("kip-ft-F".into());

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc].modal-case")
        );
    }

    #[test]
    fn code_params_reject_zero_wind_displacement_divisor() {
        let mut config = base_valid_config();
        config.calc.displacement_wind.disp_limit_h = Some(0);

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(err.to_string().contains(
            "invalid required config: [calc.displacement-wind].disp-limit-h must be greater than zero"
        ));
    }

    #[test]
    fn code_params_require_drift_groups() {
        let mut config = base_valid_config();
        config.calc.drift_tracking_groups.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc].drift-tracking-groups")
        );
    }

    #[test]
    fn code_params_require_base_shear_cases() {
        let mut config = base_valid_config();
        config.calc.base_shear.elf_case_x = None;

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.base-shear].elf-case-x")
        );
    }

    #[test]
    fn code_params_require_displacement_wind_config() {
        let mut config = base_valid_config();
        config.calc.displacement_wind.load_cases.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.displacement-wind].load-cases")
        );
    }

    #[test]
    fn code_params_require_modal_display_limit() {
        let mut config = base_valid_config();
        config.calc.modal.display_mode_limit = None;

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.modal].display-mode-limit")
        );
    }

    #[test]
    fn code_params_require_pier_shear_wind_combos() {
        let mut config = base_valid_config();
        config.calc.pier_shear_wind.load_combos.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.pier-shear-wind].load-combos")
        );
    }

    #[test]
    fn code_params_require_pier_shear_seismic_combos() {
        let mut config = base_valid_config();
        config.calc.pier_shear_seismic.load_combos.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.pier-shear-seismic].load-combos")
        );
    }

    #[test]
    fn code_params_require_pier_axial_combos() {
        let mut config = base_valid_config();
        config.calc.pier_axial.load_combos.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.pier-axial].load-combos")
        );
    }

    #[test]
    fn code_params_pier_shear_defaults_phi_v_correctly() {
        let config = base_valid_config();
        let params = CodeParams::from_config(&config).unwrap();
        // Wind: default ϕ = 0.75 (ACI 318-14 §9.3.2.3)
        assert!((params.pier_shear_wind.phi_v - 0.75).abs() < 1e-9);
        // Seismic: default ϕ = 0.60 (ACI 318-14 §21.2.4.1)
        assert!((params.pier_shear_seismic.phi_v - 0.60).abs() < 1e-9);
        // Axial: default ϕ = 0.65 (ACI 318-14 §9.3.2.2 tied)
        assert!((params.pier_axial.phi_axial - 0.65).abs() < 1e-9);
    }
}
