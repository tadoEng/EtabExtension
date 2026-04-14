use anyhow::Result;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::code_params::{TorsionalParams, TorsionalJointPair};
use crate::output::{TorsionalDirectionOutput, TorsionalOutput, TorsionalRow};
use crate::tables::joint_drift::JointDriftRow;
use crate::tables::story_def::StoryDefRow;

pub fn run(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    params: &TorsionalParams,
) -> Result<TorsionalOutput> {
    
    // Sort stories bottom-up
    let mut sorted_stories = stories.to_vec();
    sorted_stories.sort_by(|a, b| a.elevation_ft.partial_cmp(&b.elevation_ft).unwrap());
    
    let x_out = process_direction(rows, &sorted_stories, &params.x_cases, &params.x_pairs, params.ecc_ratio, params.building_dim_y_ft, true)?;
    let y_out = process_direction(rows, &sorted_stories, &params.y_cases, &params.y_pairs, params.ecc_ratio, params.building_dim_x_ft, false)?;

    let pass = !x_out.has_type_b && !y_out.has_type_b;

    Ok(TorsionalOutput {
        x: x_out,
        y: y_out,
        pass,
    })
}

fn process_direction(
    rows: &[JointDriftRow],
    stories: &[StoryDefRow],
    cases: &[String],
    pairs: &[TorsionalJointPair],
    ecc_ratio: f64,
    perp_dim_ft: f64,
    is_x: bool,
) -> Result<TorsionalDirectionOutput> {
    if cases.is_empty() || pairs.is_empty() {
        return Ok(TorsionalDirectionOutput {
            rows: vec![],
            governing_story: String::new(),
            governing_case: String::new(),
            governing_joints: vec![],
            max_ratio: 0.0,
            has_type_a: false,
            has_type_b: false,
        });
    }

    let mut out_rows = Vec::new();
    let selected_cases: HashSet<&str> = cases.iter().map(|s| s.as_str()).collect();
    let mut available_steps_by_case: HashMap<&str, BTreeSet<i32>> = HashMap::new();

    // Optimize by creating a lookup map: (UniqueName, Story, OutputCase, StepNumber) -> Disp
    let mut disp_map: HashMap<(&str, &str, &str, i32), f64> = HashMap::new();
    
    for r in rows {
        if selected_cases.contains(r.output_case.as_str()) {
            let step = r.step_number.unwrap_or(1.0).round() as i32;
            let disp = if is_x { r.disp_x_ft } else { r.disp_y_ft };
            disp_map.insert((r.unique_name.as_str(), r.story.as_str(), r.output_case.as_str(), step), disp);
            available_steps_by_case
                .entry(r.output_case.as_str())
                .or_default()
                .insert(step);
        }
    }

    for case in cases {
        let steps = available_steps_by_case
            .get(case.as_str())
            .map(|values| values.iter().copied().collect::<Vec<_>>())
            .unwrap_or_else(|| vec![1]);

        for pair in pairs {
            let j_a = pair.joint_a.as_str();
            let j_b = pair.joint_b.as_str();

            for i in 1..stories.len() {
                let story_bot = &stories[i - 1];
                let story_top = &stories[i];

                let mut drift_a_steps = Vec::with_capacity(steps.len());
                let mut drift_b_steps = Vec::with_capacity(steps.len());
                let mut delta_max_steps = Vec::with_capacity(steps.len());
                let mut delta_avg_steps = Vec::with_capacity(steps.len());
                let mut has_data = true;

                for step in &steps {
                    let a_top = disp_map.get(&(j_a, &story_top.story, case, *step));
                    let a_bot = disp_map.get(&(j_a, &story_bot.story, case, *step));
                    let b_top = disp_map.get(&(j_b, &story_top.story, case, *step));
                    let b_bot = disp_map.get(&(j_b, &story_bot.story, case, *step));

                    if let (Some(&at), Some(&ab), Some(&bt), Some(&bb)) = (a_top, a_bot, b_top, b_bot) {
                        // Drift is |DispTop - DispBot|. We multiply by 12.0 to get Inches.
                        let d_a = (at - ab).abs() * 12.0;
                        let d_b = (bt - bb).abs() * 12.0;
                        
                        drift_a_steps.push(d_a);
                        drift_b_steps.push(d_b);
                        delta_max_steps.push(d_a.max(d_b));
                        delta_avg_steps.push((d_a + d_b) / 2.0);
                    } else {
                        has_data = false;
                        break;
                    }
                }

                if !has_data {
                    continue; // Skip if this story doesn't have all 3 steps for both joints
                }

                let mut max_ratio = 0.0;
                let mut max_ax_base = 0.0;

                for idx in 0..delta_avg_steps.len() {
                    let avg = delta_avg_steps[idx];
                    let max = delta_max_steps[idx];
                    if avg > 1e-6 {
                        let ratio = max / avg;
                        if ratio > max_ratio {
                            max_ratio = ratio;
                        }
                        
                        let ax_val = (max / (1.2 * avg)).powi(2);
                        if ax_val > max_ax_base {
                            max_ax_base = ax_val;
                        }
                    } else {
                        // If avg is 0, ratio is essentially 1.0 (no torsion)
                        if 1.0 > max_ratio { max_ratio = 1.0; }
                    }
                }

                let ax = max_ax_base.min(3.0).max(1.0);
                
                let ecc_ft = ecc_ratio * perp_dim_ft;
                
                let is_type_a = max_ratio > 1.2;
                let is_type_b = max_ratio > 1.4;
                let rho = if is_type_b { 1.3 } else { 1.0 };

                out_rows.push(TorsionalRow {
                    story: story_top.story.clone(),
                    case: case.clone(),
                    joint_a: j_a.to_string(),
                    joint_b: j_b.to_string(),
                    drift_a_steps,
                    drift_b_steps,
                    delta_max_steps,
                    delta_avg_steps,
                    ratio: max_ratio,
                    ax,
                    ecc_ft,
                    rho,
                    is_type_a,
                    is_type_b,
                });
            }
        }
    }

    if out_rows.is_empty() {
        return Ok(TorsionalDirectionOutput {
            rows: vec![],
            governing_story: String::new(),
            governing_case: String::new(),
            governing_joints: vec![],
            max_ratio: 0.0,
            has_type_a: false,
            has_type_b: false,
        });
    }

    // Gov is max ratio
    let gov = out_rows.iter().max_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap()).unwrap().clone();
    
    // Spec says sort by story elevation descending
    let story_map: HashMap<&str, f64> = stories.iter().map(|s| (s.story.as_str(), s.elevation_ft)).collect();
    out_rows.sort_by(|a, b| {
        let elev_a = story_map.get(a.story.as_str()).unwrap_or(&0.0);
        let elev_b = story_map.get(b.story.as_str()).unwrap_or(&0.0);
        elev_b.partial_cmp(elev_a).unwrap()
    });

    let has_type_a = out_rows.iter().any(|r| r.is_type_a);
    let has_type_b = out_rows.iter().any(|r| r.is_type_b);

    Ok(TorsionalDirectionOutput {
        governing_story: gov.story.clone(),
        governing_case: gov.case.clone(),
        governing_joints: vec![gov.joint_a.clone(), gov.joint_b.clone()],
        max_ratio: gov.ratio,
        has_type_a,
        has_type_b,
        rows: out_rows,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use ext_db::config::Config;
    use crate::code_params::CodeParams;
    use crate::tables::{joint_drift::load_joint_drifts, story_def::load_story_definitions};
    use super::run;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn torsional_3step_iteration_hand_calc_checks() {
        let results_dir = fixture_dir();
        let config = Config::load(&results_dir).unwrap();
        let params = CodeParams::from_config(&config).unwrap();
        
        let drifts = load_joint_drifts(&results_dir).unwrap();
        let stories = load_story_definitions(&results_dir).unwrap();
        
        // Let's artificially get the raw data inside the test and run a hand-calc loop!
        let t_params = params.torsional.as_ref().unwrap();
        let _output = run(&drifts, &stories, t_params).unwrap();

        // Let's create an explicit hand-calc trace to prove the math works isolated from the parquet
        use crate::tables::joint_drift::JointDriftRow;
        
        let mut mock_drifts = vec![];
        let cases = t_params.x_cases.clone();
        let target_case = cases[0].clone();
        
        // Define Top and Bottom Story Displacements (X direction)
        // Joint47 Top Step 1=1.5, Step 2=1.6, Step 3=1.4  / Bot = 0.5, 0.6, 0.4. Drift = 1.0 everywhere.
        // Joint50 Top Step 1=2.5, Step 2=2.8, Step 3=2.2  / Bot = 1.0, 1.3, 0.7. Drift = 1.5 everywhere.
        
        for step in 1..=3 {
            let s = step as f64;
            let top = stories[1].story.clone();
            // Floor 2 (Top)
            mock_drifts.push(JointDriftRow {
                story: top.clone(), unique_name: "Joint47".into(), output_case: target_case.clone(), step_type: "Step".into(),
                step_number: Some(s), disp_x_ft: if step==1{1.5}else if step==2{1.6}else{1.4}, disp_y_ft: 0.0, drift_x: 0.0, drift_y: 0.0,
                case_type: "".into(), label: 0,
            });
            mock_drifts.push(JointDriftRow {
                story: top.clone(), unique_name: "Joint50".into(), output_case: target_case.clone(), step_type: "Step".into(),
                step_number: Some(s), disp_x_ft: if step==1{2.5}else if step==2{2.8}else{2.2}, disp_y_ft: 0.0, drift_x: 0.0, drift_y: 0.0,
                case_type: "".into(), label: 0,
            });
            
            let bot = stories[0].story.clone();
            // Floor 1 (Bottom)
            mock_drifts.push(JointDriftRow {
                story: bot.clone(), unique_name: "Joint47".into(), output_case: target_case.clone(), step_type: "Step".into(),
                step_number: Some(s), disp_x_ft: if step==1{0.5}else if step==2{0.6}else{0.4}, disp_y_ft: 0.0, drift_x: 0.0, drift_y: 0.0,
                case_type: "".into(), label: 0,
            });
            mock_drifts.push(JointDriftRow {
                story: bot.clone(), unique_name: "Joint50".into(), output_case: target_case.clone(), step_type: "Step".into(),
                step_number: Some(s), disp_x_ft: if step==1{1.0}else if step==2{1.3}else{0.7}, disp_y_ft: 0.0, drift_x: 0.0, drift_y: 0.0,
                case_type: "".into(), label: 0,
            });
        }
        
        let mut t_custom = t_params.clone();
        t_custom.x_pairs = vec![crate::code_params::TorsionalJointPair { joint_a: "Joint47".into(), joint_b: "Joint50".into() }];
        
        let output = run(&mock_drifts, &stories, &t_custom).unwrap();
        
        let gov = &output.x.rows[0];
        assert_eq!(gov.joint_a, "Joint47");
        assert_eq!(gov.joint_b, "Joint50");
        
        // Drift A should be |1.5-0.5|*12 = 12.0 inches for step 1
        assert!((gov.drift_a_steps[0] - 12.0).abs() < 1e-4);
        // Drift B should be |2.5-1.0|*12 = 18.0 inches for step 1
        assert!((gov.drift_b_steps[0] - 18.0).abs() < 1e-4);
        
        // Max = 18.0, Avg = 15.0
        assert!((gov.delta_max_steps[0] - 18.0).abs() < 1e-4);
        assert!((gov.delta_avg_steps[0] - 15.0).abs() < 1e-4);
        
        // Ratio = 18.0 / 15.0 = 1.2
        assert!((gov.ratio - 1.2).abs() < 1e-4);
        
        assert!(!output.x.rows.is_empty(), "X Torsion rows must be populated");
    }

    #[test]
    fn torsional_accepts_single_step_cases() {
        use crate::code_params::{TorsionalJointPair, TorsionalParams};
        use crate::tables::joint_drift::JointDriftRow;
        use crate::tables::story_def::StoryDefRow;

        let stories = vec![
            StoryDefRow {
                story: "L1".into(),
                height_ft: 10.0,
                elevation_ft: 10.0,
            },
            StoryDefRow {
                story: "L2".into(),
                height_ft: 10.0,
                elevation_ft: 20.0,
            },
        ];

        let rows = vec![
            JointDriftRow {
                story: "L2".into(),
                unique_name: "J1".into(),
                output_case: "ELF_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: Some(1.0),
                disp_x_ft: 0.20,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 1,
            },
            JointDriftRow {
                story: "L1".into(),
                unique_name: "J1".into(),
                output_case: "ELF_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: Some(1.0),
                disp_x_ft: 0.05,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 1,
            },
            JointDriftRow {
                story: "L2".into(),
                unique_name: "J2".into(),
                output_case: "ELF_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: Some(1.0),
                disp_x_ft: 0.18,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 2,
            },
            JointDriftRow {
                story: "L1".into(),
                unique_name: "J2".into(),
                output_case: "ELF_X".into(),
                case_type: "LinStatic".into(),
                step_type: String::new(),
                step_number: Some(1.0),
                disp_x_ft: 0.04,
                disp_y_ft: 0.0,
                drift_x: 0.0,
                drift_y: 0.0,
                label: 2,
            },
        ];

        let params = TorsionalParams {
            x_cases: vec!["ELF_X".into()],
            y_cases: vec![],
            x_pairs: vec![TorsionalJointPair {
                joint_a: "J1".into(),
                joint_b: "J2".into(),
            }],
            y_pairs: vec![],
            ecc_ratio: 0.05,
            building_dim_x_ft: 100.0,
            building_dim_y_ft: 50.0,
        };

        let output = run(&rows, &stories, &params).unwrap();

        assert_eq!(output.x.rows.len(), 1);
        assert_eq!(output.x.rows[0].delta_avg_steps.len(), 1);
        assert!(output.x.rows[0].ratio >= 1.0);
    }
}
