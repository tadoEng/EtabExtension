use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{DisplacementEnvelopeRow, DisplacementOutput, JointDisplacementResult};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

use super::drift_wind::{max_negative, max_positive, resolve_groups, sort_rows_by_story};

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<DisplacementOutput> {
    let selected_groups = resolve_groups(group_map, &params.drift_tracking_groups)?;

    for case in &params.displacement_wind.load_cases {
        if !rows.iter().any(|row| row.output_case == *case) {
            bail!("Configured displacement load case '{}' not found", case);
        }
    }

    let selected_cases = params
        .displacement_wind
        .load_cases
        .iter()
        .map(String::as_str)
        .collect::<std::collections::HashSet<_>>();

    let mut grouped: BTreeMap<(String, String, String), Vec<&JointDriftRow>> = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|row| selected_cases.contains(row.output_case.as_str()))
    {
        for (group_name, members) in &selected_groups {
            if members.contains(row.unique_name.as_str()) {
                grouped
                    .entry((
                        row.story.clone(),
                        (*group_name).to_string(),
                        row.output_case.clone(),
                    ))
                    .or_default()
                    .push(row);
            }
        }
    }

    for case in &params.displacement_wind.load_cases {
        for group in &params.drift_tracking_groups {
            if !grouped
                .keys()
                .any(|(_, group_name, output_case)| group_name == group && output_case == case)
            {
                bail!(
                    "No displacement rows found for group '{}' and case '{}'",
                    group,
                    case
                );
            }
        }
    }

    let mut rows_out = Vec::with_capacity(grouped.len());
    for ((story, group_name, output_case), group_rows) in grouped {
        rows_out.push(DisplacementEnvelopeRow {
            story,
            group_name,
            output_case,
            max_disp_x_pos_ft: max_positive(group_rows.iter().map(|row| row.disp_x_ft)),
            max_disp_x_neg_ft: max_negative(group_rows.iter().map(|row| row.disp_x_ft)),
            max_disp_y_pos_ft: max_positive(group_rows.iter().map(|row| row.disp_y_ft)),
            max_disp_y_neg_ft: max_negative(group_rows.iter().map(|row| row.disp_y_ft)),
        });
    }

    sort_rows_by_story(stories, &mut rows_out, |row| &row.story);

    if rows_out.is_empty() {
        bail!("No displacement envelope rows generated");
    }

    let total_height_ft = stories
        .iter()
        .map(|row| row.elevation_ft)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);
    let disp_limit_ft = total_height_ft / f64::from(params.displacement_wind.disp_limit_h);

    let (governing_index, direction, sense, displacement_ft) = rows_out
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let candidates = [
                ("X", "positive", row.max_disp_x_pos_ft.abs()),
                ("X", "negative", row.max_disp_x_neg_ft.abs()),
                ("Y", "positive", row.max_disp_y_pos_ft.abs()),
                ("Y", "negative", row.max_disp_y_neg_ft.abs()),
            ];
            let (direction, sense, value) = candidates
                .into_iter()
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();
            (idx, direction.to_string(), sense.to_string(), value)
        })
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let governing_row = rows_out[governing_index].clone();

    let dcr = displacement_ft / disp_limit_ft;
    let pass = dcr <= 1.0;

    Ok(DisplacementOutput {
        rows: rows_out,
        governing: JointDisplacementResult {
            story: governing_row.story,
            group_name: governing_row.group_name,
            output_case: governing_row.output_case,
            direction,
            sense,
            displacement: params.unit_context.qty_length_disp(displacement_ft),
            dcr,
            pass,
        },
        disp_limit: params.unit_context.qty_length_disp(disp_limit_ft),
        pass,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::code_params::CodeParams;
    use crate::tables::group_assignments::load_group_assignments;
    use crate::tables::joint_drift::load_joint_drifts;
    use crate::tables::story_def::load_story_definitions;

    use super::*;

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
    fn displacement_wind_produces_group_envelopes() {
        let dir = fixture_dir();
        let drifts = load_joint_drifts(&dir).unwrap();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();

        let output = run(&drifts, &stories, &groups, &params).unwrap();
        assert_eq!(output.governing.story, "ROOF");
        assert_eq!(output.governing.group_name, "Joint48");
        assert_eq!(output.governing.output_case, "Wind_10yr_Diagonal");
        assert_eq!(output.governing.direction, "Y");
        assert_eq!(output.governing.sense, "positive");
        assert!((output.governing.displacement.value - 3.944_208).abs() < 1e-6);
        assert!((output.governing.dcr - 0.351_158_119_658_119_6).abs() < 1e-12);
        assert!((output.disp_limit.value - 11.232).abs() < 1e-9);
        assert_eq!(output.rows.first().map(|row| row.story.as_str()), Some("L01"));
        assert_eq!(output.rows.last().map(|row| row.story.as_str()), Some("ROOF"));
    }

    #[test]
    fn displacement_wind_errors_when_case_missing() {
        let dir = fixture_dir();
        let drifts = load_joint_drifts(&dir).unwrap();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let mut config = fixture_config();
        config.calc.displacement_wind.load_cases = vec!["missing-case".into()];
        let params = CodeParams::from_config(&config).unwrap();

        assert!(run(&drifts, &stories, &groups, &params).is_err());
    }
}
