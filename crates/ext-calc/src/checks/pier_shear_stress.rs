use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::code_params::PierShearStressParams;
use crate::output::{PierShearStressAverageRow, PierShearStressOutput, PierShearStressRow};
use crate::tables::{
    pier_forces::PierForceRow, pier_section::PierSectionRow, story_def::StoryDefRow,
};

pub fn run(
    pier_forces: &[PierForceRow],
    pier_sections: &[PierSectionRow],
    stories: &[StoryDefRow],
    pier_fc_map: &HashMap<(String, String), f64>,
    p_params: &PierShearStressParams,
) -> Result<PierShearStressOutput> {
    let phi_v = p_params.phi_v;

    let mut acw_map = HashMap::new();
    let mut axis_map = HashMap::new();

    for section in pier_sections {
        acw_map.insert(
            (section.story.clone(), section.pier.clone()),
            section.acv_in2,
        );
        axis_map.insert(
            (section.story.clone(), section.pier.clone()),
            section.axis_angle,
        );
    }

    let cases_set: HashSet<&str> = p_params.combos.iter().map(|s| s.as_str()).collect();

    let mut envelope: HashMap<(String, String), (String, f64)> = HashMap::new();
    for row in pier_forces {
        if !cases_set.contains(row.output_case.as_str()) {
            continue;
        }

        let key = (row.story.clone(), row.pier.clone());
        let entry = envelope
            .entry(key)
            .or_insert_with(|| (row.output_case.clone(), row.shear_v2_abs_kip));
        if row.shear_v2_abs_kip > entry.1 {
            *entry = (row.output_case.clone(), row.shear_v2_abs_kip);
        }
    }

    let mut out_rows = Vec::new();
    let mut max_ind_ratio = 0.0;

    let is_x_dir = |angle: f64| -> bool {
        let rad = angle.to_radians();
        rad.cos().abs() > rad.sin().abs()
    };

    // Story -> (sum_ve, sum_acw, fc_psi). Use the minimum encountered f'c in
    // each bucket so mixed concrete grades do not accidentally overstate the
    // average-check denominator.
    let mut x_avg_map: HashMap<String, (f64, f64, f64)> = HashMap::new();
    let mut y_avg_map: HashMap<String, (f64, f64, f64)> = HashMap::new();

    for ((story, pier), (combo, ve_kip)) in envelope {
        let key = (story.clone(), pier.clone());
        let acw_in2 = match acw_map.get(&key) {
            Some(&val) => val,
            None => continue,
        };

        let axis_angle = axis_map.get(&key).copied().unwrap_or(0.0);
        let is_x = is_x_dir(axis_angle);
        let orientation = if is_x { "X" } else { "Y" };

        let fc_ksi = pier_fc_map
            .get(&(pier.clone(), story.clone()))
            .copied()
            .unwrap_or(p_params.fc_default_ksi);
        let fc_psi = fc_ksi * 1000.0;
        let fc_sqrt = fc_psi.sqrt();

        let stress_psi = (ve_kip * 1000.0) / (phi_v * acw_in2);
        let limit_individual = 10.0;
        let stress_ratio = stress_psi / fc_sqrt;

        let pass = stress_ratio <= limit_individual;

        out_rows.push(PierShearStressRow {
            story: story.clone(),
            pier: pier.clone(),
            combo,
            wall_direction: orientation.to_string(),
            acw_in2,
            fc_psi,
            sqrt_fc: fc_sqrt,
            ve_kip,
            stress_psi,
            stress_ratio,
            limit_individual,
            pass,
        });

        if stress_ratio > max_ind_ratio {
            max_ind_ratio = stress_ratio;
        }

        let avg_map = if is_x { &mut x_avg_map } else { &mut y_avg_map };
        let entry = avg_map.entry(story.clone()).or_insert((0.0, 0.0, 0.0));
        entry.0 += ve_kip;
        entry.1 += acw_in2;
        if entry.2 == 0.0 || fc_psi < entry.2 {
            entry.2 = fc_psi;
        }
    }

    let mut x_avg_rows = Vec::new();
    let mut y_avg_rows = Vec::new();
    let mut max_avg_ratio = 0.0;
    let limit_average = 8.0;

    for (story, (sum_ve, sum_acw, fc_psi)) in x_avg_map {
        let sqrt_fc = fc_psi.sqrt();
        let avg_stress_psi = (sum_ve * 1000.0) / (phi_v * sum_acw);
        let ratio = avg_stress_psi / sqrt_fc;
        if ratio > max_avg_ratio {
            max_avg_ratio = ratio;
        }

        x_avg_rows.push(PierShearStressAverageRow {
            story,
            wall_direction: "X".to_string(),
            sum_ve_kip: sum_ve,
            sum_acw_in2: sum_acw,
            sqrt_fc,
            avg_stress_ratio: ratio,
            limit_average,
            pass: ratio <= limit_average,
        });
    }

    for (story, (sum_ve, sum_acw, fc_psi)) in y_avg_map {
        let sqrt_fc = fc_psi.sqrt();
        let avg_stress_psi = (sum_ve * 1000.0) / (phi_v * sum_acw);
        let ratio = avg_stress_psi / sqrt_fc;
        if ratio > max_avg_ratio {
            max_avg_ratio = ratio;
        }

        y_avg_rows.push(PierShearStressAverageRow {
            story,
            wall_direction: "Y".to_string(),
            sum_ve_kip: sum_ve,
            sum_acw_in2: sum_acw,
            sqrt_fc,
            avg_stress_ratio: ratio,
            limit_average,
            pass: ratio <= limit_average,
        });
    }

    let pass = max_ind_ratio <= 10.0 && max_avg_ratio <= 8.0;

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
    let story_rank = story_order
        .iter()
        .enumerate()
        .map(|(idx, story)| (story.clone(), idx))
        .collect::<HashMap<_, _>>();

    out_rows.sort_by(|a, b| {
        let a_rank = story_rank.get(&a.story).copied().unwrap_or(usize::MAX);
        let b_rank = story_rank.get(&b.story).copied().unwrap_or(usize::MAX);
        a_rank
            .cmp(&b_rank)
            .then_with(|| a.pier.cmp(&b.pier))
            .then_with(|| a.combo.cmp(&b.combo))
    });
    x_avg_rows.sort_by_key(|row| story_rank.get(&row.story).copied().unwrap_or(usize::MAX));
    y_avg_rows.sort_by_key(|row| story_rank.get(&row.story).copied().unwrap_or(usize::MAX));

    Ok(PierShearStressOutput {
        phi_v,
        limit_individual: 10.0,
        limit_average: 8.0,
        supported: true,
        support_note: None,
        story_order,
        per_pier: out_rows,
        x_average: x_avg_rows,
        y_average: y_avg_rows,
        max_individual_ratio: max_ind_ratio,
        max_average_ratio: max_avg_ratio,
        pass,
    })
}

pub fn unsupported_output(note: impl Into<String>) -> PierShearStressOutput {
    PierShearStressOutput {
        phi_v: 0.75,
        limit_individual: 10.0,
        limit_average: 8.0,
        supported: false,
        support_note: Some(note.into()),
        story_order: Vec::new(),
        per_pier: Vec::new(),
        x_average: Vec::new(),
        y_average: Vec::new(),
        max_individual_ratio: 0.0,
        max_average_ratio: 0.0,
        pass: true,
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::code_params::PierShearStressParams;
    use crate::tables::{
        pier_forces::PierForceRow, pier_section::PierSectionRow, story_def::StoryDefRow,
    };
    use std::collections::HashMap;

    #[test]
    fn pier_shear_stress_hand_calc_checks() {
        let mut pier_fc_map = HashMap::new();
        let p_params = PierShearStressParams {
            phi_v: 0.75,
            combos: vec!["Combo1".to_string()],
            fc_default_ksi: 4.0,
        };

        let mut mock_sections = vec![];
        mock_sections.push(PierSectionRow {
            story: "L01".into(),
            pier: "P1".into(),
            axis_angle: 0.0,
            width_bot_ft: 10.0,
            thick_bot_ft: 1.0,
            width_top_ft: 10.0,
            thick_top_ft: 1.0,
            material: "C4000".into(),
            acv_in2: 200.0,
            ag_in2: 200.0,
        });

        let mut mock_forces = vec![];
        mock_forces.push(PierForceRow {
            story: "L01".into(),
            pier: "P1".into(),
            output_case: "Combo1".into(),
            case_type: "Combo".into(),
            step_type: "".into(),
            location: "Bottom".into(),
            axial_p_kip: 0.0,
            shear_v2_kip: 100.0,
            shear_v2_abs_kip: 100.0,
            shear_v3_kip: 0.0,
            torsion_t_kip_ft: 0.0,
            moment_m2_kip_ft: 0.0,
            moment_m3_kip_ft: 0.0,
        });

        pier_fc_map.insert(("P1".into(), "L01".into()), 4.0);
        let stories = vec![StoryDefRow {
            story: "L01".into(),
            height_ft: 10.0,
            elevation_ft: 10.0,
        }];

        let output = run(
            &mock_forces,
            &mock_sections,
            &stories,
            &pier_fc_map,
            &p_params,
        )
        .unwrap();
        let gov = &output.per_pier[0];

        assert_eq!(gov.pier, "P1");
        assert_eq!(output.limit_individual, 10.0);
        assert_eq!(output.limit_average, 8.0);

        let expected_stress = 100.0 * 1000.0 / (0.75 * 200.0);
        let expected_ratio = expected_stress / (4000.0f64).sqrt();
        assert!((gov.stress_psi - expected_stress).abs() < 1e-3);
        assert!((gov.stress_ratio - expected_ratio).abs() < 1e-3);
    }

    #[test]
    fn pier_shear_stress_envelopes_duplicate_rows_per_pier() {
        let mut pier_fc_map = HashMap::new();
        pier_fc_map.insert(("P1".into(), "L01".into()), 4.0);

        let p_params = PierShearStressParams {
            phi_v: 0.75,
            combos: vec!["Combo1".to_string()],
            fc_default_ksi: 4.0,
        };

        let mock_sections = vec![PierSectionRow {
            story: "L01".into(),
            pier: "P1".into(),
            axis_angle: 0.0,
            width_bot_ft: 10.0,
            thick_bot_ft: 1.0,
            width_top_ft: 10.0,
            thick_top_ft: 1.0,
            material: "C4000".into(),
            acv_in2: 200.0,
            ag_in2: 200.0,
        }];

        let mock_forces = vec![
            PierForceRow {
                story: "L01".into(),
                pier: "P1".into(),
                output_case: "Combo1".into(),
                case_type: "Combo".into(),
                step_type: "".into(),
                location: "Top".into(),
                axial_p_kip: 0.0,
                shear_v2_kip: 80.0,
                shear_v2_abs_kip: 80.0,
                shear_v3_kip: 0.0,
                torsion_t_kip_ft: 0.0,
                moment_m2_kip_ft: 0.0,
                moment_m3_kip_ft: 0.0,
            },
            PierForceRow {
                story: "L01".into(),
                pier: "P1".into(),
                output_case: "Combo1".into(),
                case_type: "Combo".into(),
                step_type: "".into(),
                location: "Bottom".into(),
                axial_p_kip: 0.0,
                shear_v2_kip: 100.0,
                shear_v2_abs_kip: 100.0,
                shear_v3_kip: 0.0,
                torsion_t_kip_ft: 0.0,
                moment_m2_kip_ft: 0.0,
                moment_m3_kip_ft: 0.0,
            },
        ];

        let stories = vec![StoryDefRow {
            story: "L01".into(),
            height_ft: 10.0,
            elevation_ft: 10.0,
        }];

        let output = run(
            &mock_forces,
            &mock_sections,
            &stories,
            &pier_fc_map,
            &p_params,
        )
        .unwrap();

        assert_eq!(output.per_pier.len(), 1);
        assert_eq!(output.per_pier[0].combo, "Combo1");
        assert!((output.per_pier[0].ve_kip - 100.0).abs() < 1e-9);
    }
}
