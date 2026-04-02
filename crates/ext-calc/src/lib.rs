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
        let material_by_story = tables::material_by_story::load_material_by_story(results_dir)?;
        let modal = tables::modal::load_modal_participating_mass_ratios(results_dir)?;
        let base_reactions = tables::base_reactions::load_base_reactions(results_dir)?;
        let story_forces = tables::story_forces::load_story_forces(results_dir)?;
        let pier_forces = tables::pier_forces::load_pier_forces(results_dir)?;
        let pier_sections = tables::pier_section::load_pier_sections(results_dir)?;
        let group_map = tables::group_assignments::load_group_assignments(results_dir)?;

        let summary = CalcSummary {
            overall_status: "pending".to_string(),
            check_count: 0,
            pass_count: 0,
            fail_count: 0,
            lines: vec![
                SummaryLine {
                    key: "storyDefinitions".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} story rows", story_defs.len()),
                },
                SummaryLine {
                    key: "jointDrifts".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} drift rows", joint_drifts.len()),
                },
                SummaryLine {
                    key: "materialProperties".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} concrete materials", material_props.len()),
                },
                SummaryLine {
                    key: "materialByStory".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} material-by-story rows", material_by_story.len()),
                },
                SummaryLine {
                    key: "modalParticipation".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} modal rows", modal.len()),
                },
                SummaryLine {
                    key: "baseReactions".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} base reaction rows", base_reactions.len()),
                },
                SummaryLine {
                    key: "storyForces".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} story force rows", story_forces.len()),
                },
                SummaryLine {
                    key: "pierForces".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} pier force rows", pier_forces.len()),
                },
                SummaryLine {
                    key: "pierSections".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} pier section rows", pier_sections.len()),
                },
                SummaryLine {
                    key: "driftGroups".to_string(),
                    status: "loaded".to_string(),
                    message: format!("{} group mappings", group_map.len()),
                },
            ],
        };

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
            modal: None,
            base_shear: None,
            drift_wind: None,
            drift_seismic: None,
            torsional: None,
            pier_shear_wind: None,
            pier_shear_seismic: None,
            pier_axial: None,
        })
    }
}
