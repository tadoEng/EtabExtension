use std::path::Path;

use anyhow::Result;
use chrono::Utc;

pub mod checks;
pub mod code_params;
pub mod output;
pub mod tables;
pub mod unit_convert;

use code_params::CodeParams;
use output::{CalcMeta, CalcOutput, CalcSummary, SummaryLine, UnitLabels};

pub struct CalcRunner;

impl CalcRunner {
    pub fn run_all(
        _version_dir: &Path,
        results_dir: &Path,
        params: &CodeParams,
        version_id: &str,
        branch: &str,
    ) -> Result<CalcOutput> {
        let story_defs = tables::story_def::load_story_definitions(results_dir)?;
        let joint_drifts = tables::joint_drift::load_joint_drifts(results_dir)?;
        let material_props = tables::material_props::load_material_properties(results_dir)?;
        let _material_by_story = tables::material_by_story::load_material_by_story(results_dir)?;
        let modal = tables::modal::load_modal_participating_mass_ratios(results_dir)?;
        let base_reactions = tables::base_reactions::load_base_reactions(results_dir)?;
        let _story_forces = tables::story_forces::load_story_forces(results_dir)?;
        let _pier_forces = tables::pier_forces::load_pier_forces(results_dir)?;
        let _pier_sections = tables::pier_section::load_pier_sections(results_dir)?;
        let group_map = tables::group_assignments::load_group_assignments(results_dir)?;

        let modal_output = if params.check_selection.modal {
            Some(checks::modal::run(&modal, params)?)
        } else {
            None
        };
        let base_shear_output = if params.check_selection.base_shear {
            Some(checks::base_reaction::run(&base_reactions, params)?)
        } else {
            None
        };
        let drift_wind_output = if params.check_selection.drift_wind {
            Some(checks::drift_wind::run(
                &joint_drifts,
                &story_defs,
                &group_map,
                params,
            )?)
        } else {
            None
        };
        let drift_seismic_output = if params.check_selection.drift_seismic {
            Some(checks::drift_seismic::run(
                &joint_drifts,
                &story_defs,
                &group_map,
                params,
            )?)
        } else {
            None
        };
        let displacement_wind_output = if params.check_selection.displacement_wind {
            Some(checks::displacement_wind::run(
                &joint_drifts,
                &story_defs,
                &group_map,
                params,
            )?)
        } else {
            None
        };

        let summary = build_summary(
            modal_output.as_ref(),
            base_shear_output.as_ref(),
            drift_wind_output.as_ref(),
            drift_seismic_output.as_ref(),
            displacement_wind_output.as_ref(),
            material_props.len(),
            group_map.len(),
        );

        Ok(CalcOutput {
            meta: CalcMeta {
                version_id: version_id.to_string(),
                branch: branch.to_string(),
                code: params.code.clone(),
                generated_at: Utc::now(),
                units: UnitLabels {
                    force: params.unit_context.force_label().to_string(),
                    length: params.unit_context.length_label().to_string(),
                    stress: "ksi".to_string(),
                    moment: params.unit_context.moment_label().to_string(),
                },
            },
            summary,
            modal: modal_output,
            base_shear: base_shear_output,
            drift_wind: drift_wind_output,
            drift_seismic: drift_seismic_output,
            displacement_wind: displacement_wind_output,
            torsional: None,
            pier_shear_wind: None,
            pier_shear_seismic: None,
            pier_axial: None,
        })
    }
}

fn build_summary(
    modal: Option<&output::ModalOutput>,
    base_shear: Option<&output::BaseShearOutput>,
    drift_wind: Option<&output::DriftOutput>,
    drift_seismic: Option<&output::DriftOutput>,
    displacement_wind: Option<&output::DisplacementOutput>,
    material_count: usize,
    group_count: usize,
) -> CalcSummary {
    let mut lines = Vec::new();
    let mut check_count = 0_u32;
    let mut pass_count = 0_u32;
    let mut fail_count = 0_u32;

    if let Some(modal) = modal {
        check_count += 1;
        if modal.pass {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(SummaryLine {
            key: "modal".to_string(),
            status: if modal.pass { "pass" } else { "fail" }.to_string(),
            message: format!(
                "threshold {:.2} reached at UX mode {}, UY mode {}",
                modal.threshold,
                modal
                    .mode_reaching_ux
                    .map_or_else(|| "not reached".to_string(), |mode| mode.to_string()),
                modal
                    .mode_reaching_uy
                    .map_or_else(|| "not reached".to_string(), |mode| mode.to_string())
            ),
        });
    }

    if let Some(base_shear) = base_shear {
        check_count += 1;
        let pass = base_shear.direction_x.pass && base_shear.direction_y.pass;
        if pass {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(SummaryLine {
            key: "baseShear".to_string(),
            status: if pass { "pass" } else { "fail" }.to_string(),
            message: format!(
                "X ratio {:.3}, Y ratio {:.3}",
                base_shear.direction_x.ratio, base_shear.direction_y.ratio
            ),
        });
    }

    if let Some(drift_wind) = drift_wind {
        check_count += 1;
        if drift_wind.pass {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(SummaryLine {
            key: "driftWind".to_string(),
            status: if drift_wind.pass { "pass" } else { "fail" }.to_string(),
            message: format!(
                "{} / {} / {} {} {} DCR={:.3}",
                drift_wind.governing.story,
                drift_wind.governing.group_name,
                drift_wind.governing.output_case,
                drift_wind.governing.direction,
                drift_wind.governing.sense,
                drift_wind.governing.dcr
            ),
        });
    }

    if let Some(drift_seismic) = drift_seismic {
        check_count += 1;
        if drift_seismic.pass {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(SummaryLine {
            key: "driftSeismic".to_string(),
            status: if drift_seismic.pass { "pass" } else { "fail" }.to_string(),
            message: format!(
                "{} / {} / {} {} {} DCR={:.3}",
                drift_seismic.governing.story,
                drift_seismic.governing.group_name,
                drift_seismic.governing.output_case,
                drift_seismic.governing.direction,
                drift_seismic.governing.sense,
                drift_seismic.governing.dcr
            ),
        });
    }

    if let Some(displacement_wind) = displacement_wind {
        check_count += 1;
        if displacement_wind.pass {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(SummaryLine {
            key: "displacementWind".to_string(),
            status: if displacement_wind.pass {
                "pass"
            } else {
                "fail"
            }
            .to_string(),
            message: format!(
                "{} / {} / {} {} {} DCR={:.3}",
                displacement_wind.governing.story,
                displacement_wind.governing.group_name,
                displacement_wind.governing.output_case,
                displacement_wind.governing.direction,
                displacement_wind.governing.sense,
                displacement_wind.governing.dcr
            ),
        });
    }

    lines.push(SummaryLine {
        key: "materials".to_string(),
        status: "loaded".to_string(),
        message: format!("{material_count} concrete materials available"),
    });
    lines.push(SummaryLine {
        key: "driftGroups".to_string(),
        status: "loaded".to_string(),
        message: format!("{group_count} group mappings available"),
    });
    lines.push(SummaryLine {
        key: "torsional".to_string(),
        status: "pending".to_string(),
        message: "torsional irregularity check not implemented yet".to_string(),
    });
    lines.push(SummaryLine {
        key: "pierShearWind".to_string(),
        status: "pending".to_string(),
        message: "pier shear wind check not implemented yet".to_string(),
    });
    lines.push(SummaryLine {
        key: "pierShearSeismic".to_string(),
        status: "pending".to_string(),
        message: "pier shear seismic check not implemented yet".to_string(),
    });
    lines.push(SummaryLine {
        key: "pierAxial".to_string(),
        status: "pending".to_string(),
        message: "pier axial check not implemented yet".to_string(),
    });

    CalcSummary {
        overall_status: if fail_count > 0 {
            "fail".to_string()
        } else if pass_count > 0 {
            "pass".to_string()
        } else {
            "pending".to_string()
        },
        check_count,
        pass_count,
        fail_count,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::{CalcRunner, code_params::CodeParams};

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    fn configured_params_from_fixture(results_dir: &std::path::Path) -> CodeParams {
        let config = Config::load(results_dir).unwrap();
        CodeParams::from_config(&config).unwrap()
    }

    #[test]
    fn calc_runner_populates_checks_1_to_4() {
        let results_dir = fixture_dir();
        let params = configured_params_from_fixture(&results_dir);
        let output = CalcRunner::run_all(
            results_dir.as_path(),
            results_dir.as_path(),
            &params,
            "v1",
            "main",
        )
        .unwrap();

        assert!(output.modal.is_some());
        assert!(output.base_shear.is_some());
        assert!(output.drift_wind.is_some());
        assert!(output.drift_seismic.is_some());
        assert!(output.displacement_wind.is_some());
        assert!(output.torsional.is_none());
        assert_eq!(output.summary.check_count, 5);
        assert_ne!(output.summary.overall_status, "pending");
        assert_eq!(output.summary.overall_status, "pass");
        assert!(
            output
                .summary
                .lines
                .iter()
                .find(|line| line.key == "modal")
                .unwrap()
                .message
                .contains("UX mode 12, UY mode 23")
        );
    }
}
