use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Result, bail};

use super::drift_wind::{
    DriftDirection, max_negative, max_positive, resolve_groups, sort_rows_by_story,
};
use crate::code_params::CodeParams;
use crate::output::{
    DisplacementEnvelopeRow, DisplacementOutput, DisplacementWindOutput, JointDisplacementResult,
    Quantity,
};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

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
            params.unit_context.length_label(),
        )?,
        y: build_displacement_output_directional(
            rows,
            stories,
            group_map,
            &params.joint_tracking_groups,
            &params.displacement_wind.y_cases,
            params.displacement_wind.disp_limit_h,
            DriftDirection::Y,
            params.unit_context.length_label(),
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
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
                direction: if direction == DriftDirection::X {
                    "X".to_string()
                } else {
                    "Y".to_string()
                },
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

    // Wind displacement limit is based on total building height H, not the
    // elevation of the governing story.
    let mut max_elev = 0.0;
    for s in stories {
        if s.elevation_ft > max_elev {
            max_elev = s.elevation_ft;
        }
    }

    let limit_val = max_elev / (disp_limit_h as f64);
    let dcr = if limit_val > 1e-9 {
        max_disp_val / limit_val
    } else {
        0.0
    };
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{DriftDirection, build_displacement_output_directional};
    use crate::tables::{joint_drift::JointDriftRow, story_def::StoryDefRow};

    #[test]
    fn displacement_limit_uses_total_building_height() {
        let stories = vec![
            StoryDefRow {
                story: "L2".into(),
                height_ft: 10.0,
                elevation_ft: 20.0,
            },
            StoryDefRow {
                story: "L1".into(),
                height_ft: 10.0,
                elevation_ft: 10.0,
            },
        ];

        let rows = vec![
            JointDriftRow {
                story: "L2".into(),
                unique_name: "J1".into(),
                output_case: "WIND_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: None,
                disp_x_ft: 0.20,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 1,
            },
            JointDriftRow {
                story: "L1".into(),
                unique_name: "J1".into(),
                output_case: "WIND_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: None,
                disp_x_ft: 0.05,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 1,
            },
        ];

        let mut group_map = HashMap::new();
        group_map.insert("Tracking".to_string(), vec!["J1".to_string()]);

        let output = build_displacement_output_directional(
            &rows,
            &stories,
            &group_map,
            &["Tracking".to_string()],
            &["WIND_X".to_string()],
            400,
            DriftDirection::X,
            "ft",
        )
        .unwrap();

        assert!((output.disp_limit.value - 0.05).abs() < 1e-9);
    }
}
