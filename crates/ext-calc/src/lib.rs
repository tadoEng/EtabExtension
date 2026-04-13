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
        let story_forces = tables::story_forces::load_story_forces(results_dir)?;
        let pier_forces = tables::pier_forces::load_pier_forces(results_dir)?;
        let pier_sections = tables::pier_section::load_pier_sections(results_dir)?;
        let group_map = tables::group_assignments::load_group_assignments(results_dir)?;

        let mut pier_fc_map = std::collections::HashMap::new();
        // Since we removed checks::pier_shear::build_pier_fc_map, we can quickly reconstruct it here 
        // to map Pier -> fc (ksi) based on section and material_props.
        // For now, we use default fc_default_ksi.
        for sp in &pier_sections {
            let default_fc = params.pier_shear_stress_seismic.as_ref().map(|p| p.fc_default_ksi).unwrap_or(8.0);
            let fc = material_props.get(&sp.material).map(|m| m.fc_ksi).unwrap_or(default_fc);
            pier_fc_map.insert((sp.pier.clone(), sp.story.clone()), fc);
        }

        let modal_output = if params.check_selection.modal {
            Some(checks::modal::run(&modal, params)?)
        } else {
            None
        };
        let base_reactions_output = if params.check_selection.base_reactions {
            Some(checks::base_reaction::run(&base_reactions, params)?) 
        } else {
            None
        };
        let story_forces_output = if params.check_selection.story_forces {
            if let Some(sf_params) = &params.story_forces {
                Some(checks::story_forces::run(&story_forces, &story_defs, sf_params)?)
            } else {
                None
            }
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
        let torsional_output = if params.check_selection.torsional {
            if let Some(tor_params) = &params.torsional {
                Some(checks::torsional::run(&joint_drifts, &story_defs, tor_params)?)
            } else {
                None
            }
        } else {
            None
        };
        
        let pier_shear_stress_wind_output = if params.check_selection.pier_shear_stress_wind {
            match &params.pier_shear_stress_wind {
                Some(p) => {
                    Some(checks::pier_shear_stress::run(&pier_forces, &pier_sections, &pier_fc_map, p)?)
                }
                None => None,
            }
        } else { None };

        let pier_shear_stress_seismic_output = if params.check_selection.pier_shear_stress_seismic {
            match &params.pier_shear_stress_seismic {
                Some(p) => {
                    Some(checks::pier_shear_stress::run(&pier_forces, &pier_sections, &pier_fc_map, p)?)
                }
                None => None,
            }
        } else { None };

        let pier_axial_output = if params.check_selection.pier_axial_stress {
            Some(checks::pier_axial::run(
                &pier_forces,
                &pier_sections,
                &pier_fc_map,
                params,
            )?)
        } else {
            None
        };

        let mut out = CalcOutput {
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
            summary: CalcSummary {
                overall_status: "pending".to_string(),
                check_count: 0, pass_count: 0, fail_count: 0, lines: vec![]
            },
            modal: modal_output,
            base_reactions: base_reactions_output,
            story_forces: story_forces_output,
            drift_wind: drift_wind_output,
            drift_seismic: drift_seismic_output,
            displacement_wind: displacement_wind_output,
            torsional: torsional_output,
            pier_shear_stress_wind: pier_shear_stress_wind_output,
            pier_shear_stress_seismic: pier_shear_stress_seismic_output,
            pier_axial_stress: pier_axial_output,
        };

        out.summary = build_summary(&out, material_props.len(), group_map.len());

        Ok(out)
    }
}

fn build_summary(
    output: &CalcOutput,
    material_count: usize,
    group_count: usize,
) -> CalcSummary {
    let mut lines = Vec::new();
    let mut check_count = 0_u32;
    let mut pass_count = 0_u32;
    let mut fail_count = 0_u32;

    if let Some(modal) = &output.modal {
        check_count += 1;
        if modal.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "modal".to_string(), status: if modal.pass { "pass" } else { "fail" }.to_string(), message: "Modal check".to_string(), });
    }

    if let Some(base_reactions) = &output.base_reactions {
        check_count += 1;
        let pass = base_reactions.direction_x.pass && base_reactions.direction_y.pass;
        if pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine {
            key: "baseReactions".to_string(),
            status: if pass { "pass" } else { "fail" }.to_string(),
            message: format!(
                "Base reactions review (X {:.2}, Y {:.2})",
                base_reactions.direction_x.ratio,
                base_reactions.direction_y.ratio
            ),
        });
    }

    if let Some(_sf) = &output.story_forces {
        check_count += 1; pass_count += 1;
        lines.push(SummaryLine { key: "storyForces".to_string(), status: "pass".to_string(), message: "Story forces extraction".to_string() });
    }

    if let Some(dw) = &output.drift_wind {
        check_count += 1;
        if dw.x.pass && dw.y.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "driftWind".to_string(), status: if dw.x.pass && dw.y.pass { "pass" } else { "fail" }.to_string(), message: "Wind drift check".to_string() });
    }

    if let Some(ds) = &output.drift_seismic {
        check_count += 1;
        if ds.x.pass && ds.y.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "driftSeismic".to_string(), status: if ds.x.pass && ds.y.pass { "pass" } else { "fail" }.to_string(), message: "Seismic drift check".to_string() });
    }

    if let Some(dw) = &output.displacement_wind {
        check_count += 1;
        if dw.x.pass && dw.y.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "displacementWind".to_string(), status: if dw.x.pass && dw.y.pass { "pass" } else { "fail" }.to_string(), message: "Wind displacement check".to_string() });
    }

    if let Some(tor) = &output.torsional {
        check_count += 1; 
        if tor.pass { pass_count += 1; } else { fail_count += 1; }
        
        let status = if !tor.pass {
            "fail".to_string()
        } else if tor.x.has_type_a || tor.y.has_type_a {
            "warn".to_string()
        } else {
            "pass".to_string()
        };
        
        lines.push(SummaryLine { key: "torsional".to_string(), status, message: "Torsional irregularity check".to_string() });
    }

    if let Some(psw) = &output.pier_shear_stress_wind {
        check_count += 1;
        if psw.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "pierShearStressWind".to_string(), status: if psw.pass { "pass" } else { "fail" }.to_string(), message: "Pier shear stress wind".to_string() });
    }

    if let Some(pse) = &output.pier_shear_stress_seismic {
        check_count += 1;
        if pse.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "pierShearStressSeismic".to_string(), status: if pse.pass { "pass" } else { "fail" }.to_string(), message: "Pier shear stress seismic".to_string() });
    }

    if let Some(pa) = &output.pier_axial_stress {
        check_count += 1;
        if pa.pass { pass_count += 1; } else { fail_count += 1; }
        lines.push(SummaryLine { key: "pierAxialStress".to_string(), status: if pa.pass { "pass" } else { "fail" }.to_string(), message: "Pier axial stress".to_string() });
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
    fn calc_runner_populates_all_checks() {
        let results_dir = fixture_dir();
        let mut params = configured_params_from_fixture(&results_dir);
        params.check_selection.torsional = true;
        let output = CalcRunner::run_all(
            results_dir.as_path(),
            results_dir.as_path(),
            &params,
            "v1",
            "main",
        )
        .unwrap();

        assert!(output.modal.is_some());
        assert!(output.base_reactions.is_some());
        assert!(output.story_forces.is_some());
        assert!(output.drift_wind.is_some());
        assert!(output.drift_seismic.is_some());
        assert!(output.displacement_wind.is_some());
        assert!(output.torsional.is_some());
        assert!(output.pier_shear_stress_wind.is_some());
        assert!(output.pier_shear_stress_seismic.is_some());
        assert!(output.pier_axial_stress.is_some());
    }
}
