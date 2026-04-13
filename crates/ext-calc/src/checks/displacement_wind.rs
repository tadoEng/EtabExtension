use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{DisplacementEnvelopeRow, DisplacementOutput, DisplacementWindOutput, JointDisplacementResult, Quantity};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;
use super::drift_wind::{max_negative, max_positive, resolve_groups, sort_rows_by_story, DriftDirection};

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    params: &CodeParams,
) -> Result<DisplacementWindOutput> {
    Ok(DisplacementWindOutput {
        x: build_displacement_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.displacement_wind.x_cases,
            params.displacement_wind.disp_limit_h,
            DriftDirection::X,
            &params.unit_context.length_label().to_string(),
        )?,
        y: build_displacement_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.displacement_wind.y_cases,
            params.displacement_wind.disp_limit_h,
            DriftDirection::Y,
            &params.unit_context.length_label().to_string(),
        )?,
    })
}

pub(crate) fn build_displacement_output_directional(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    group_map: &HashMap<String, Vec<String>>,
    configured_groups: &[String],
    cases: &[String],
    disp_limit_h: u32,
    direction: DriftDirection,
    length_label: &str,
) -> Result<DisplacementOutput> {
    let selected_groups = resolve_groups(group_map, configured_groups)?;
    let selected_cases: HashSet<&str> = cases.iter().map(String::as_str).collect();

    // Valid if no cases are supplied for this direction
    if selected_cases.is_empty() {
        return Ok(DisplacementOutput {
            rows: vec![],
            governing: JointDisplacementResult {
                story: String::new(),
                group_name: String::new(),
                output_case: String::new(),
                direction: if direction == DriftDirection::X { "X".to_string() } else { "Y".to_string() },
                sense: String::new(),
                displacement: Quantity::new(0.0, length_label),
                dcr: 0.0,
                pass: true,
            },
            disp_limit: Quantity::new(0.0, length_label),
            pass: true,
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
        rows_out.push(DisplacementEnvelopeRow {
            story: story.clone(),
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

    let (governing_index, gov_direction, sense, max_disp_val) = rows_out
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let candidates = if direction == DriftDirection::X {
                vec![
                    ("X", "positive", row.max_disp_x_pos_ft.abs()),
                    ("X", "negative", row.max_disp_x_neg_ft.abs()),
                ]
            } else {
                vec![
                    ("Y", "positive", row.max_disp_y_pos_ft.abs()),
                    ("Y", "negative", row.max_disp_y_neg_ft.abs()),
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
    
    // Find the height of the governing story.
    // Stories are ordered internally by sort_rows_by_story which uses order map.
    // The closest to story_idx height should be looked up.
    let story_height = stories.iter()
        .find(|s| s.story == governing_row.story)
        .map(|s| s.elevation_ft)
        .unwrap_or(0.0);
        
    // Roof elevation is max elevation.
    let mut max_elev = 0.0;
    for s in stories {
        if s.elevation_ft > max_elev {
            max_elev = s.elevation_ft;
        }
    }

    // Usually limit is Height / divisor at that specific story. 
    // Wait, the specification logic usually calculates `allowable = elev / limit`.
    let limit_val = story_height / (disp_limit_h as f64);
    let dcr = if limit_val > 1e-9 { max_disp_val / limit_val } else { 0.0 };
    let pass = dcr <= 1.0;

    Ok(DisplacementOutput {
        rows: rows_out,
        governing: JointDisplacementResult {
            story: governing_row.story,
            group_name: governing_row.group_name,
            output_case: governing_row.output_case,
            direction: gov_direction,
            sense,
            displacement: Quantity::new(max_disp_val, length_label),
            dcr,
            pass,
        },
        disp_limit: Quantity::new(limit_val, length_label),
        pass,
    })
}
