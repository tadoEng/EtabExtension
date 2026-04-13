use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{DriftEnvelopeRow, DriftOutput, DriftWindOutput, StoryDriftResult};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriftDirection {
    X,
    Y,
}

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<DriftWindOutput> {
    Ok(DriftWindOutput {
        x: build_drift_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.drift_wind.x_cases,
            params.drift_wind.drift_limit,
            DriftDirection::X,
        )?,
        y: build_drift_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.drift_wind.y_cases,
            params.drift_wind.drift_limit,
            DriftDirection::Y,
        )?,
    })
}

pub(crate) fn build_drift_output_directional(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    configured_groups: &[String],
    cases: &[String],
    drift_limit: f64,
    direction: DriftDirection,
) -> Result<DriftOutput> {
    let selected_groups = resolve_groups(group_map, configured_groups)?;
    let selected_cases: HashSet<&str> = cases.iter().map(String::as_str).collect();

    // Valid if no cases are supplied for this direction
    if selected_cases.is_empty() {
        return Ok(DriftOutput {
            allowable_ratio: drift_limit,
            rows: vec![],
            governing: StoryDriftResult {
                story: String::new(),
                group_name: String::new(),
                output_case: String::new(),
                direction: if direction == DriftDirection::X { "X".to_string() } else { "Y".to_string() },
                sense: String::new(),
                drift_ratio: 0.0,
                dcr: 0.0,
                pass: true,
            },
            pass: true,
            roof_disp_x: None,
            roof_disp_y: None,
            disp_limit: None,
            disp_pass: None,
        });
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

    let (governing_index, gov_direction, sense, drift_ratio) = rows_out
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let candidates = if direction == DriftDirection::X {
                vec![
                    ("X", "positive", row.max_drift_x_pos.abs()),
                    ("X", "negative", row.max_drift_x_neg.abs()),
                ]
            } else {
                vec![
                    ("Y", "positive", row.max_drift_y_pos.abs()),
                    ("Y", "negative", row.max_drift_y_neg.abs()),
                ]
            };
            
            let (dir, sense, value) = candidates
                .into_iter()
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();
            (idx, dir.to_string(), sense.to_string(), value)
        })
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
        
    let governing_row = rows_out[governing_index].clone();

    let dcr = drift_ratio / drift_limit;
    let pass = dcr <= 1.0;

    Ok(DriftOutput {
        allowable_ratio: drift_limit,
        rows: rows_out,
        governing: StoryDriftResult {
            story: governing_row.story,
            group_name: governing_row.group_name,
            output_case: governing_row.output_case,
            direction: gov_direction,
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
            .ok_or_else(|| anyhow::anyhow!("Configured tracking group '{}' not found", group))?;
        if members.is_empty() {
            bail!("Configured tracking group '{}' has no members", group);
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
