use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Result, bail};

use crate::code_params::{CodeParams, DriftParams};
use crate::output::{DriftEnvelopeRow, DriftOutput, StoryDriftResult};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<DriftOutput> {
    build_drift_output(
        rows,
        stories,
        group_map,
        &params.drift_tracking_groups,
        &params.drift_wind,
    )
}

pub(crate) fn build_drift_output(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    configured_groups: &[String],
    params: &DriftParams,
) -> Result<DriftOutput> {
    let selected_groups = resolve_groups(group_map, configured_groups)?;
    let selected_cases: HashSet<&str> = params.load_cases.iter().map(String::as_str).collect();

    if selected_cases.is_empty() {
        bail!("No load cases configured for drift check");
    }

    for case in &params.load_cases {
        if !rows.iter().any(|row| row.output_case == *case) {
            bail!("Configured drift load case '{}' not found", case);
        }
    }

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

    for case in &params.load_cases {
        for group in configured_groups {
            if !grouped
                .keys()
                .any(|(_, group_name, output_case)| group_name == group && output_case == case)
            {
                bail!(
                    "No drift rows found for group '{}' and case '{}'",
                    group,
                    case
                );
            }
        }
    }

    let mut rows_out = Vec::with_capacity(grouped.len());
    for ((story, group_name, output_case), group_rows) in grouped {
        rows_out.push(DriftEnvelopeRow {
            story,
            group_name,
            output_case,
            max_disp_x_pos_ft: max_positive(group_rows.iter().map(|row| row.disp_x_ft)),
            max_disp_x_neg_ft: max_negative(group_rows.iter().map(|row| row.disp_x_ft)),
            max_disp_y_pos_ft: max_positive(group_rows.iter().map(|row| row.disp_y_ft)),
            max_disp_y_neg_ft: max_negative(group_rows.iter().map(|row| row.disp_y_ft)),
            max_drift_x_pos: max_positive(group_rows.iter().map(|row| row.drift_x)),
            max_drift_x_neg: max_negative(group_rows.iter().map(|row| row.drift_x)),
            max_drift_y_pos: max_positive(group_rows.iter().map(|row| row.drift_y)),
            max_drift_y_neg: max_negative(group_rows.iter().map(|row| row.drift_y)),
        });
    }

    sort_rows_by_story(stories, &mut rows_out, |row| &row.story);

    if rows_out.is_empty() {
        bail!("No drift envelope rows generated");
    }

    let (governing_index, direction, sense, drift_ratio) = rows_out
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let candidates = [
                ("X", "positive", row.max_drift_x_pos.abs()),
                ("X", "negative", row.max_drift_x_neg.abs()),
                ("Y", "positive", row.max_drift_y_pos.abs()),
                ("Y", "negative", row.max_drift_y_neg.abs()),
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

    let dcr = drift_ratio / params.drift_limit;
    let pass = dcr <= 1.0;

    Ok(DriftOutput {
        allowable_ratio: params.drift_limit,
        rows: rows_out,
        governing: StoryDriftResult {
            story: governing_row.story,
            group_name: governing_row.group_name,
            output_case: governing_row.output_case,
            direction,
            sense,
            drift_ratio,
            dcr,
            pass,
        },
        pass,
        roof_disp_x: None,
        roof_disp_y: None,
        disp_limit: None,
        disp_pass: None,
    })
}

pub(crate) fn story_order_lookup(stories: &[StoryDefRow]) -> HashMap<String, usize> {
    let mut ordered = stories.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| {
        a.elevation_ft
            .partial_cmp(&b.elevation_ft)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ordered
        .into_iter()
        .enumerate()
        .map(|(index, row)| (row.story.clone(), index))
        .collect()
}

pub(crate) fn sort_rows_by_story<T, F>(stories: &[StoryDefRow], rows: &mut [T], story_name: F)
where
    F: Fn(&T) -> &str,
{
    let order = story_order_lookup(stories);
    rows.sort_by_key(|row| order.get(story_name(row)).copied().unwrap_or(usize::MAX));
}

pub(crate) fn resolve_groups<'a>(
    group_map: &'a HashMap<String, Vec<String>>,
    configured_groups: &'a [String],
) -> Result<HashMap<&'a str, HashSet<&'a str>>> {
    if configured_groups.is_empty() {
        bail!("No drift tracking groups configured");
    }

    let mut selected = HashMap::new();
    for group in configured_groups {
        let members = group_map
            .get(group)
            .ok_or_else(|| anyhow::anyhow!("Configured drift group '{}' not found", group))?;
        if members.is_empty() {
            bail!("Configured drift group '{}' has no members", group);
        }
        selected.insert(
            group.as_str(),
            members.iter().map(String::as_str).collect::<HashSet<_>>(),
        );
    }
    Ok(selected)
}

pub(crate) fn max_positive(values: impl Iterator<Item = f64>) -> f64 {
    values.filter(|value| *value > 0.0).fold(0.0_f64, f64::max)
}

pub(crate) fn max_negative(values: impl Iterator<Item = f64>) -> f64 {
    values.filter(|value| *value < 0.0).fold(0.0_f64, f64::min)
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
    fn drift_wind_produces_sorted_group_envelopes() {
        let dir = fixture_dir();
        let drifts = load_joint_drifts(&dir).unwrap();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();

        let output = run(&drifts, &stories, &groups, &params).unwrap();
        assert_eq!(output.governing.story, "L35");
        assert_eq!(output.governing.group_name, "Joint48");
        assert_eq!(output.governing.output_case, "Wind_10yr_Diagonal");
        assert_eq!(output.governing.direction, "Y");
        assert_eq!(output.governing.sense, "positive");
        assert!((output.governing.dcr - 0.4028).abs() < 1e-9);
        assert_eq!(
            output.rows.first().map(|row| row.story.as_str()),
            Some("L01")
        );
        assert_eq!(
            output.rows.last().map(|row| row.story.as_str()),
            Some("ROOF")
        );
        assert!(output.roof_disp_x.is_none());
        assert!(output.disp_limit.is_none());
    }

    #[test]
    fn drift_wind_errors_when_group_missing() {
        let dir = fixture_dir();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let drifts = load_joint_drifts(&dir).unwrap();
        let mut config = fixture_config();
        config.calc.drift_tracking_groups = vec!["missing-group".into()];
        let params = CodeParams::from_config(&config).unwrap();

        assert!(run(&drifts, &stories, &groups, &params).is_err());
    }

    #[test]
    fn drift_wind_requires_explicit_load_cases() {
        let mut config = fixture_config();
        config.calc.drift_wind.drift_limit = Some(0.0025);
        config.calc.drift_wind.load_cases.clear();

        let err = CodeParams::from_config(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required config: [calc.drift-wind].load-cases")
        );
    }

    #[test]
    fn drift_wind_errors_when_group_and_case_exist_but_have_no_matching_rows() {
        let dir = fixture_dir();
        let groups = load_group_assignments(&dir).unwrap();
        let stories = load_story_definitions(&dir).unwrap();
        let drifts = load_joint_drifts(&dir).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();
        let joint48 = groups.get("Joint48").unwrap();
        let joint48_members = joint48.iter().collect::<std::collections::HashSet<_>>();
        let filtered = drifts
            .into_iter()
            .filter(|row| {
                !(row.output_case == "Wind_10yr_Diagonal"
                    && joint48_members.contains(&row.unique_name))
            })
            .collect::<Vec<_>>();

        let err = run(&filtered, &stories, &groups, &params).unwrap_err();
        assert!(
            err.to_string()
                .contains("No drift rows found for group 'Joint48' and case 'Wind_10yr_Diagonal'")
        );
    }
}
