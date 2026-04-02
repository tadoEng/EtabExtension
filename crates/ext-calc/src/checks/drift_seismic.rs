use std::collections::HashMap;

use anyhow::Result;

use crate::code_params::CodeParams;
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

use super::drift_wind::build_drift_output;

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<crate::output::DriftOutput> {
    let _ = stories;
    build_drift_output(
        rows,
        stories,
        group_map,
        &params.drift_tracking_groups,
        &params.drift_seismic,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::code_params::CodeParams;
    use crate::tables::group_assignments::load_group_assignments;
    use crate::tables::joint_drift::load_joint_drifts;
    use crate::tables::story_def::load_story_definitions;

    use super::run;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    fn fixture_config() -> Config {
        Config::load(&fixture_dir()).unwrap()
    }

    #[test]
    fn drift_seismic_produces_group_envelopes() {
        let dir = fixture_dir();
        let drifts = load_joint_drifts(&dir).unwrap();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();

        let output = run(&drifts, &stories, &groups, &params).unwrap();
        assert!(!output.rows.is_empty());
        assert!(output.roof_disp_x.is_none());
        assert!(output.disp_limit.is_none());
    }
}
