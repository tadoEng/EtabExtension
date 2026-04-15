use crate::tables::{story_def::StoryDefRow, story_forces::StoryForceRow};
use crate::{
    code_params::StoryForcesParams,
    output::{StoryForceCaseProfile, StoryForceCaseRow, StoryForceEnvelopeRow, StoryForcesOutput},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn run(
    rows: &[StoryForceRow],
    stories: &[StoryDefRow],
    params: &StoryForcesParams,
) -> Result<StoryForcesOutput> {
    let mut sorted_stories = stories.to_vec();
    sorted_stories.sort_by(|a, b| {
        b.elevation_ft
            .partial_cmp(&a.elevation_ft)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let story_order = sorted_stories
        .iter()
        .map(|row| row.story.clone())
        .collect::<Vec<_>>();

    let x_profiles = build_profiles(rows, &sorted_stories, &params.x_cases);
    let y_profiles = build_profiles(rows, &sorted_stories, &params.y_cases);

    let mut max_vx: HashMap<&str, f64> = HashMap::new();
    let mut max_my: HashMap<&str, f64> = HashMap::new();
    let mut max_vy: HashMap<&str, f64> = HashMap::new();
    let mut max_mx: HashMap<&str, f64> = HashMap::new();

    for profile in &x_profiles {
        for row in &profile.rows {
            let story_str = row.story.as_str();
            *max_vx.entry(story_str).or_insert(0.0) =
                max_vx.get(story_str).unwrap_or(&0.0).max(row.vx_kip.abs());
            *max_my.entry(story_str).or_insert(0.0) = max_my
                .get(story_str)
                .unwrap_or(&0.0)
                .max(row.my_kip_ft.abs());
        }
    }

    for profile in &y_profiles {
        for row in &profile.rows {
            let story_str = row.story.as_str();
            *max_vy.entry(story_str).or_insert(0.0) =
                max_vy.get(story_str).unwrap_or(&0.0).max(row.vy_kip.abs());
            *max_mx.entry(story_str).or_insert(0.0) = max_mx
                .get(story_str)
                .unwrap_or(&0.0)
                .max(row.mx_kip_ft.abs());
        }
    }

    let mut output_rows = Vec::new();
    for story_def in sorted_stories {
        let story_str = story_def.story.as_str();
        let q_vx = max_vx.get(story_str).copied().unwrap_or(0.0);
        let q_my = max_my.get(story_str).copied().unwrap_or(0.0);
        let q_vy = max_vy.get(story_str).copied().unwrap_or(0.0);
        let q_mx = max_mx.get(story_str).copied().unwrap_or(0.0);

        output_rows.push(StoryForceEnvelopeRow {
            story: story_def.story.clone(),
            max_vx_kip: q_vx,
            max_my_kip_ft: q_my,
            max_vy_kip: q_vy,
            max_mx_kip_ft: q_mx,
        });
    }

    Ok(StoryForcesOutput {
        rows: output_rows,
        story_order,
        x_profiles,
        y_profiles,
    })
}

fn build_profiles(
    rows: &[StoryForceRow],
    stories: &[StoryDefRow],
    cases: &[String],
) -> Vec<StoryForceCaseProfile> {
    let mut profiles = Vec::new();
    for case in cases {
        let mut per_story: HashMap<&str, (f64, f64, f64, f64)> = HashMap::new();
        for row in rows {
            if row.location != "Bottom" || row.output_case != *case {
                continue;
            }
            let entry = per_story
                .entry(row.story.as_str())
                .or_insert((0.0, 0.0, 0.0, 0.0));
            entry.0 = entry.0.max(row.vx_kip.abs());
            entry.1 = entry.1.max(row.vy_kip.abs());
            entry.2 = entry.2.max(row.mx_kip_ft.abs());
            entry.3 = entry.3.max(row.my_kip_ft.abs());
        }

        let mut profile_rows = Vec::with_capacity(stories.len());
        for story in stories {
            let (vx_kip, vy_kip, mx_kip_ft, my_kip_ft) = per_story
                .get(story.story.as_str())
                .copied()
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            profile_rows.push(StoryForceCaseRow {
                story: story.story.clone(),
                elevation_ft: story.elevation_ft,
                vx_kip,
                vy_kip,
                mx_kip_ft,
                my_kip_ft,
            });
        }

        profiles.push(StoryForceCaseProfile {
            output_case: case.clone(),
            rows: profile_rows,
        });
    }
    profiles
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::code_params::CodeParams;
    use crate::tables::{story_def::load_story_definitions, story_forces::load_story_forces};
    use ext_db::config::Config;
    use std::path::PathBuf;

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
            if &row.story == story_tests
                && row.location == "Bottom"
                && sf_params.x_cases.contains(&row.output_case)
            {
                if row.vx_kip.abs() > expected_story1_vx {
                    expected_story1_vx = row.vx_kip.abs();
                }
                if row.my_kip_ft.abs() > expected_story1_my {
                    expected_story1_my = row.my_kip_ft.abs();
                }
            }
        }

        let output = run(&story_forces, &story_defs, sf_params).unwrap();

        // Assert sorting is from largest elevation to smallest
        assert_eq!(
            output.rows.first().unwrap().story,
            story_defs
                .iter()
                .max_by(|a, b| a.elevation_ft.partial_cmp(&b.elevation_ft).unwrap())
                .unwrap()
                .story
        );
        assert_eq!(
            output.rows.last().unwrap().story,
            story_defs
                .iter()
                .min_by(|a, b| a.elevation_ft.partial_cmp(&b.elevation_ft).unwrap())
                .unwrap()
                .story
        );

        // Find test story in the output
        let story1_out = output
            .rows
            .iter()
            .find(|r| r.story == *story_tests)
            .unwrap();

        // Assert numerical match
        assert_eq!(story1_out.max_vx_kip, expected_story1_vx);
        assert_eq!(story1_out.max_my_kip_ft, expected_story1_my);

        assert!(story1_out.max_vx_kip >= 0.0);
        assert_eq!(output.x_profiles.len(), sf_params.x_cases.len());
        assert_eq!(output.y_profiles.len(), sf_params.y_cases.len());
        assert_eq!(
            output
                .x_profiles
                .iter()
                .map(|profile| profile.output_case.as_str())
                .collect::<Vec<_>>(),
            sf_params
                .x_cases
                .iter()
                .map(|case| case.as_str())
                .collect::<Vec<_>>()
        );
    }
}
