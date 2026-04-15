use anyhow::{Result, bail};
use ext_db::config::Config;

use crate::checks::CheckSelection;
use crate::unit_convert::UnitContext;

#[derive(Debug, Clone)]
pub struct BaseReactionsParams {
    pub elf_case_x: String,
    pub elf_case_y: String,
    pub rsa_case_x: String,
    pub rsa_case_y: String,
    pub rsa_scale_min: f64,
}

#[derive(Debug, Clone)]
pub struct StoryForcesParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DriftDirectionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub drift_limit: f64,
}

#[derive(Debug, Clone)]
pub struct DisplacementDirectionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub disp_limit_h: u32,
}

#[derive(Debug, Clone)]
pub struct TorsionalJointPair {
    pub joint_a: String,
    pub joint_b: String,
}

#[derive(Debug, Clone)]
pub struct TorsionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub x_pairs: Vec<TorsionalJointPair>,
    pub y_pairs: Vec<TorsionalJointPair>,
    pub ecc_ratio: f64,
    pub building_dim_x_ft: f64,
    pub building_dim_y_ft: f64,
}

#[derive(Debug, Clone)]
pub struct PierShearStressParams {
    pub combos: Vec<String>,
    pub phi_v: f64,
    pub fc_default_ksi: f64,
}

#[derive(Debug, Clone)]
pub struct PierAxialStressParams {
    pub gravity_combos: Vec<String>,
    pub wind_combos: Vec<String>,
    pub seismic_combos: Vec<String>,
    pub phi_axial: f64,
    pub fc_default_ksi: f64,
}

impl PierAxialStressParams {
    pub fn all_combos(&self) -> impl Iterator<Item = &str> {
        self.gravity_combos
            .iter()
            .chain(&self.wind_combos)
            .chain(&self.seismic_combos)
            .map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub struct CodeParams {
    pub code: String,
    pub occupancy_category: String,
    pub modal_case: String,
    pub modal_threshold: f64,
    pub modal_display_limit: usize,
    pub joint_tracking_groups: Vec<String>,
    pub base_reactions: BaseReactionsParams,
    pub story_forces: Option<StoryForcesParams>,
    pub drift_wind: DriftDirectionalParams,
    pub drift_seismic: DriftDirectionalParams,
    pub displacement_wind: DisplacementDirectionalParams,
    pub torsional: Option<TorsionalParams>,
    pub pier_shear_stress_wind: Option<PierShearStressParams>,
    pub pier_shear_stress_seismic: Option<PierShearStressParams>,
    pub pier_axial_stress: Option<PierAxialStressParams>,
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
        let joint_tracking_groups = required_string_list(
            &config.calc.joint_tracking_groups,
            "[calc].joint-tracking-groups",
        )?;
        let elf_case_x = required_string(
            config.calc.base_reactions.elf_case_x.as_deref(),
            "[calc.base-reactions].elf-case-x",
        )?;
        let elf_case_y = required_string(
            config.calc.base_reactions.elf_case_y.as_deref(),
            "[calc.base-reactions].elf-case-y",
        )?;
        let rsa_case_x = required_string(
            config.calc.base_reactions.rsa_case_x.as_deref(),
            "[calc.base-reactions].rsa-case-x",
        )?;
        let rsa_case_y = required_string(
            config.calc.base_reactions.rsa_case_y.as_deref(),
            "[calc.base-reactions].rsa-case-y",
        )?;
        let rsa_scale_min = required_positive_f64(
            config.calc.base_reactions.rsa_scale_min,
            "[calc.base-reactions].rsa-scale-min",
        )?;

        // Drift wind
        let drift_wind_x = required_string_list(
            &config.calc.drift_wind.drift_x_cases,
            "[calc.drift-wind].drift-x-cases",
        )?;
        let drift_wind_y = required_string_list(
            &config.calc.drift_wind.drift_y_cases,
            "[calc.drift-wind].drift-y-cases",
        )?;
        let drift_wind_limit = required_positive_f64(
            config.calc.drift_wind.drift_limit,
            "[calc.drift-wind].drift-limit",
        )?;

        // Drift seismic
        let drift_seismic_x = required_string_list(
            &config.calc.drift_seismic.drift_x_cases,
            "[calc.drift-seismic].drift-x-cases",
        )?;
        let drift_seismic_y = required_string_list(
            &config.calc.drift_seismic.drift_y_cases,
            "[calc.drift-seismic].drift-y-cases",
        )?;
        let drift_seismic_limit = required_positive_f64(
            config.calc.drift_seismic.drift_limit,
            "[calc.drift-seismic].drift-limit",
        )?;

        // Displacement wind
        let disp_wind_x = required_string_list(
            &config.calc.displacement_wind.disp_x_cases,
            "[calc.displacement-wind].disp-x-cases",
        )?;
        let disp_wind_y = required_string_list(
            &config.calc.displacement_wind.disp_y_cases,
            "[calc.displacement-wind].disp-y-cases",
        )?;
        let disp_limit_h = required_positive_u32(
            config.calc.displacement_wind.disp_limit_h,
            "[calc.displacement-wind].disp-limit-h",
        )?;

        // Optionals
        let story_forces = if config.calc.story_forces.story_force_x_cases.is_empty()
            && config.calc.story_forces.story_force_y_cases.is_empty()
        {
            None
        } else {
            Some(StoryForcesParams {
                x_cases: config.calc.story_forces.story_force_x_cases.clone(),
                y_cases: config.calc.story_forces.story_force_y_cases.clone(),
            })
        };

        let torsional = if config.calc.torsional.is_configured() {
            let mut x_pairs = Vec::new();
            for pair in &config.calc.torsional.x_joints {
                if pair.len() != 2 {
                    bail!(
                        "[calc.torsional].x-joints: each pair must have exactly 2 joint names, found {}",
                        pair.len()
                    );
                }
                x_pairs.push(TorsionalJointPair {
                    joint_a: pair[0].clone(),
                    joint_b: pair[1].clone(),
                });
            }

            let mut y_pairs = Vec::new();
            for pair in &config.calc.torsional.y_joints {
                if pair.len() != 2 {
                    bail!(
                        "[calc.torsional].y-joints: each pair must have exactly 2 joint names, found {}",
                        pair.len()
                    );
                }
                y_pairs.push(TorsionalJointPair {
                    joint_a: pair[0].clone(),
                    joint_b: pair[1].clone(),
                });
            }

            Some(TorsionalParams {
                x_cases: config.calc.torsional.torsional_x_case.clone(),
                y_cases: config.calc.torsional.torsional_y_case.clone(),
                x_pairs,
                y_pairs,
                ecc_ratio: config.calc.torsional.ecc_ratio(),
                building_dim_x_ft: config.calc.torsional.building_dim_x_ft.unwrap_or(0.0),
                building_dim_y_ft: config.calc.torsional.building_dim_y_ft.unwrap_or(0.0),
            })
        } else {
            None
        };

        let pier_shear_stress_wind = if config.calc.pier_shear_stress_wind.is_configured() {
            Some(PierShearStressParams {
                combos: config.calc.pier_shear_stress_wind.stress_combos.clone(),
                phi_v: config.calc.pier_shear_stress_wind.phi_v(),
                fc_default_ksi: config.calc.pier_shear_stress_wind.fc_default_ksi(),
            })
        } else {
            None
        };

        let pier_shear_stress_seismic = if config.calc.pier_shear_stress_seismic.is_configured() {
            Some(PierShearStressParams {
                combos: config.calc.pier_shear_stress_seismic.stress_combos.clone(),
                phi_v: config.calc.pier_shear_stress_seismic.phi_v(),
                fc_default_ksi: config.calc.pier_shear_stress_seismic.fc_default_ksi(),
            })
        } else {
            None
        };

        let pier_axial_stress = if config.calc.pier_axial_stress.is_configured() {
            let fc_default_ksi = config
                .calc
                .pier_axial_stress
                .fc_default_ksi
                .or(config.calc.pier_shear_stress_seismic.fc_default_ksi)
                .or(config.calc.pier_shear_stress_wind.fc_default_ksi)
                .unwrap_or(8.0);
            Some(PierAxialStressParams {
                gravity_combos: config.calc.pier_axial_stress.stress_gravity_combos.clone(),
                wind_combos: config.calc.pier_axial_stress.stress_wind_combos.clone(),
                seismic_combos: config.calc.pier_axial_stress.stress_seismic_combos.clone(),
                phi_axial: config.calc.pier_axial_stress.phi_axial(),
                fc_default_ksi,
            })
        } else {
            None
        };

        let mut check_selection = CheckSelection::default();
        check_selection.torsional = torsional.is_some();

        Ok(Self {
            code: config.calc.code_or_default().to_string(),
            occupancy_category: config.calc.occupancy_or_default().to_string(),
            modal_case,
            modal_threshold,
            modal_display_limit,
            joint_tracking_groups,
            base_reactions: BaseReactionsParams {
                elf_case_x,
                elf_case_y,
                rsa_case_x,
                rsa_case_y,
                rsa_scale_min,
            },
            drift_wind: DriftDirectionalParams {
                x_cases: drift_wind_x,
                y_cases: drift_wind_y,
                drift_limit: drift_wind_limit,
            },
            drift_seismic: DriftDirectionalParams {
                x_cases: drift_seismic_x,
                y_cases: drift_seismic_y,
                drift_limit: drift_seismic_limit,
            },
            displacement_wind: DisplacementDirectionalParams {
                x_cases: disp_wind_x,
                y_cases: disp_wind_y,
                disp_limit_h,
            },
            story_forces,
            torsional,
            pier_shear_stress_wind,
            pier_shear_stress_seismic,
            pier_axial_stress,
            check_selection,
            unit_context: UnitContext::from_config(config)?,
        })
    }

    #[cfg(test)]
    pub fn for_testing() -> Self {
        let mut config = Config::default();
        config.project.units = Some("kip-ft-F".into());
        config.calc.modal_case = Some("Modal-Rizt".into());
        config.calc.joint_tracking_groups = vec!["Tracking_Points".into()];
        config.calc.modal.min_mass_participation = Some(0.90);
        config.calc.modal.display_mode_limit = Some(20);

        config.calc.base_reactions.elf_case_x = Some("ELF_X".into());
        config.calc.base_reactions.elf_case_y = Some("ELF_Y".into());
        config.calc.base_reactions.rsa_case_x = Some("RSA_X".into());
        config.calc.base_reactions.rsa_case_y = Some("RSA_Y".into());
        config.calc.base_reactions.rsa_scale_min = Some(1.0);

        config.calc.drift_wind.drift_x_cases = vec!["Wind_ASCE_10yr_X".into()];
        config.calc.drift_wind.drift_y_cases = vec!["Wind_ASCE_10yr_Y".into()];
        config.calc.drift_wind.drift_limit = Some(0.0025);

        config.calc.drift_seismic.drift_x_cases = vec!["RSA_X_Drift".into()];
        config.calc.drift_seismic.drift_y_cases = vec!["RSA_Y_Drift".into()];
        config.calc.drift_seismic.drift_limit = Some(0.020);

        config.calc.displacement_wind.disp_x_cases = vec!["Wind_10yr_X".into()];
        config.calc.displacement_wind.disp_y_cases = vec!["Wind_10yr_Y".into()];
        config.calc.displacement_wind.disp_limit_h = Some(500);

        config.calc.pier_shear_stress_wind.stress_combos = vec!["EVN_LRFD_WIND".into()];
        config.calc.pier_shear_stress_seismic.stress_combos = vec!["EVN_LRFD_EQ".into()];

        config.calc.pier_axial_stress.stress_gravity_combos = vec!["LC1_Grav".into()];
        config.calc.pier_axial_stress.stress_wind_combos = vec!["LC2_Wind".into()];
        config.calc.pier_axial_stress.stress_seismic_combos = vec!["LC3_EQ".into()];

        Self::from_config(&config).unwrap()
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
    use ext_db::config::Config;

    use super::CodeParams;
    fn base_valid_config() -> Config {
        // Bootstrap from the for_testing() builder so all required fields are already set,
        // then override specific fields where individual tests need different values.
        let _p = CodeParams::for_testing();
        let mut config = Config::default(); // Dummy for tests that specifically modify config
        config.project.units = Some("kip-ft-F".into());
        config.calc.modal_case = Some("Modal-Rizt".into());
        config.calc.joint_tracking_groups = vec!["Tracking_Points".into()];
        config.calc.modal.min_mass_participation = Some(0.90);
        config.calc.modal.display_mode_limit = Some(20);
        config.calc.base_reactions.elf_case_x = Some("ELF_X".into());
        config.calc.base_reactions.elf_case_y = Some("ELF_Y".into());
        config.calc.base_reactions.rsa_case_x = Some("RSA_X".into());
        config.calc.base_reactions.rsa_case_y = Some("RSA_Y".into());
        config.calc.base_reactions.rsa_scale_min = Some(1.0);
        config.calc.drift_wind.drift_x_cases = vec!["Wind_ASCE_10yr_X".into()];
        config.calc.drift_wind.drift_y_cases = vec!["Wind_ASCE_10yr_Y".into()];
        config.calc.drift_wind.drift_limit = Some(0.0025);
        config.calc.drift_seismic.drift_x_cases = vec!["RSA_X_Drift".into()];
        config.calc.drift_seismic.drift_y_cases = vec!["RSA_Y_Drift".into()];
        config.calc.drift_seismic.drift_limit = Some(0.020);
        config.calc.displacement_wind.disp_x_cases = vec!["Wind_10yr_X".into()];
        config.calc.displacement_wind.disp_y_cases = vec!["Wind_10yr_Y".into()];
        config.calc.displacement_wind.disp_limit_h = Some(500);
        config
    }

    #[test]
    fn code_params_pick_up_shared_calc_and_local_units() {
        let config = base_valid_config();

        let params = CodeParams::from_config(&config).unwrap();
        assert_eq!(params.code, "ACI318-14");
        assert_eq!(params.modal_case, "Modal-Rizt");
        assert!((params.modal_threshold - 0.90).abs() < 1e-9);
        assert_eq!(params.modal_display_limit, 20);
        assert_eq!(params.unit_context.force_label(), "kip");
        assert_eq!(params.displacement_wind.disp_limit_h, 500);
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
        config.calc.joint_tracking_groups.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc].joint-tracking-groups")
        );
    }

    #[test]
    fn code_params_require_base_reactions_cases() {
        let mut config = base_valid_config();
        config.calc.base_reactions.elf_case_x = None;

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.base-reactions].elf-case-x")
        );
    }

    #[test]
    fn code_params_require_displacement_wind_config() {
        let mut config = base_valid_config();
        config.calc.displacement_wind.disp_x_cases.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.displacement-wind].disp-x-cases")
        );
    }

    #[test]
    fn code_params_enable_torsional_when_configured() {
        let mut config = base_valid_config();
        config.calc.torsional.torsional_x_case = vec!["ELF_X".into()];
        config.calc.torsional.torsional_y_case = vec!["ELF_Y".into()];
        config.calc.torsional.x_joints = vec![vec!["J1".into(), "J2".into()]];
        config.calc.torsional.y_joints = vec![vec!["J3".into(), "J4".into()]];

        let params = CodeParams::from_config(&config).unwrap();
        assert!(params.torsional.is_some());
        assert!(params.check_selection.torsional);
    }
}
