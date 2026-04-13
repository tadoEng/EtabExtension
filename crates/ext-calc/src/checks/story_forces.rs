use anyhow::Result;
use std::collections::HashMap;
use crate::{code_params::StoryForcesParams, output::{StoryForcesOutput, StoryForceEnvelopeRow}};
use crate::tables::{story_def::StoryDefRow, story_forces::StoryForceRow};

pub fn run(
    rows: &[StoryForceRow],
    stories: &[StoryDefRow],
    params: &StoryForcesParams,
) -> Result<StoryForcesOutput> {
    let mut max_vx: HashMap<&str, f64> = HashMap::new();
    let mut max_my: HashMap<&str, f64> = HashMap::new();
    let mut max_vy: HashMap<&str, f64> = HashMap::new();
    let mut max_mx: HashMap<&str, f64> = HashMap::new();

    // X Direction: Filter matching output_case && location == Bottom
    for row in rows {
        if row.location == "Bottom" && params.x_cases.contains(&row.output_case) {
            let story_str = row.story.as_str();
            let vx_abs = row.vx_kip.abs();
            let my_abs = row.my_kip_ft.abs();
            *max_vx.entry(story_str).or_insert(0.0) = max_vx.get(story_str).unwrap_or(&0.0).max(vx_abs);
            *max_my.entry(story_str).or_insert(0.0) = max_my.get(story_str).unwrap_or(&0.0).max(my_abs);
        }
    }

    // Y Direction: Filter matching output_case && location == Bottom
    for row in rows {
        if row.location == "Bottom" && params.y_cases.contains(&row.output_case) {
            let story_str = row.story.as_str();
            let vy_abs = row.vy_kip.abs();
            let mx_abs = row.mx_kip_ft.abs();
            *max_vy.entry(story_str).or_insert(0.0) = max_vy.get(story_str).unwrap_or(&0.0).max(vy_abs);
            *max_mx.entry(story_str).or_insert(0.0) = max_mx.get(story_str).unwrap_or(&0.0).max(mx_abs);
        }
    }

    let mut output_rows = Vec::new();
    let mut sorted_stories = stories.to_vec();
    sorted_stories.sort_by(|a, b| b.elevation_ft.partial_cmp(&a.elevation_ft).unwrap()); // Top-down

    for story_def in sorted_stories {
        let story_str = story_def.story.as_str();
        let q_vx = max_vx.get(story_str).copied().unwrap_or(0.0);
        let q_my = max_my.get(story_str).copied().unwrap_or(0.0);
        let q_vy = max_vy.get(story_str).copied().unwrap_or(0.0);
        let q_mx = max_mx.get(story_str).copied().unwrap_or(0.0);
        
        // Output row even if values are zero (since we show all floors)
        output_rows.push(StoryForceEnvelopeRow {
            story: story_def.story.clone(),
            max_vx_kip: q_vx,
            max_my_kip_ft: q_my,
            max_vy_kip: q_vy,
            max_mx_kip_ft: q_mx,
        });
    }

    Ok(StoryForcesOutput { rows: output_rows })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use ext_db::config::Config;
    use crate::code_params::CodeParams;
    use crate::tables::{story_def::load_story_definitions, story_forces::load_story_forces};
    use super::run;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn story_forces_envelopes_with_hand_calc_checks() {
        let results_dir = fixture_dir();
        let config = Config::load(&results_dir).unwrap();
        let params = CodeParams::from_config(&config).unwrap();
        
        let story_defs = load_story_definitions(&results_dir).unwrap();
        let story_forces = load_story_forces(&results_dir).unwrap();

        // 1. Manually identify what SHOULD happen using hand filters
        let mut expected_story1_vx = 0.0;
        let mut expected_story1_my = 0.0;
        let sf_params = params.story_forces.as_ref().unwrap();

        let story_tests = &story_defs[0].story;
        
        for row in &story_forces {
            if &row.story == story_tests && row.location == "Bottom" && sf_params.x_cases.contains(&row.output_case) {
                if row.vx_kip.abs() > expected_story1_vx { expected_story1_vx = row.vx_kip.abs(); }
                if row.my_kip_ft.abs() > expected_story1_my { expected_story1_my = row.my_kip_ft.abs(); }
            }
        }

        let output = run(&story_forces, &story_defs, sf_params).unwrap();
        
        // Assert sorting is from largest elevation to smallest
        assert_eq!(output.rows.first().unwrap().story, story_defs.iter().max_by(|a,b| a.elevation_ft.partial_cmp(&b.elevation_ft).unwrap()).unwrap().story);
        assert_eq!(output.rows.last().unwrap().story, story_defs.iter().min_by(|a,b| a.elevation_ft.partial_cmp(&b.elevation_ft).unwrap()).unwrap().story);

        // Find test story in the output
        let story1_out = output.rows.iter().find(|r| r.story == *story_tests).unwrap();
        
        // Assert numerical match
        assert_eq!(story1_out.max_vx_kip, expected_story1_vx);
        assert_eq!(story1_out.max_my_kip_ft, expected_story1_my);

        assert!(story1_out.max_vx_kip >= 0.0);
    }
}
