use std::collections::HashMap;

use anyhow::Result;

use crate::code_params::CodeParams;
use crate::output::DriftSeismicOutput;
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

use super::drift_wind::{DriftDirection, build_drift_output_directional};

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<DriftSeismicOutput> {
    Ok(DriftSeismicOutput {
        x: build_drift_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.drift_seismic.x_cases,
            params.drift_seismic.drift_limit,
            DriftDirection::X,
        )?,
        y: build_drift_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.drift_seismic.y_cases,
            params.drift_seismic.drift_limit,
            DriftDirection::Y,
        )?,
    })
}
