use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::code_params::PierShearStressParams;
use crate::output::{PierShearStressAverageRow, PierShearStressOutput, PierShearStressRow};
use crate::tables::{pier_forces::PierForceRow, pier_section::PierSectionRow};

pub fn run(
    pier_forces: &[PierForceRow],
    pier_sections: &[PierSectionRow],
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

    let mut out_rows = Vec::new();
    let mut max_ind_ratio = 0.0;

    let is_x_dir = |angle: f64| -> bool {
        let rad = angle.to_radians();
        rad.cos().abs() > rad.sin().abs()
    };

    let mut x_avg_map: HashMap<String, (f64, f64, f64)> = HashMap::new(); // Story -> (sum_ve, sum_acw, max_fc)
    let mut y_avg_map: HashMap<String, (f64, f64, f64)> = HashMap::new(); // Story -> (sum_ve, sum_acw, max_fc)

    for row in pier_forces {
        if !cases_set.contains(row.output_case.as_str()) {
            continue;
        }

        let key = (row.story.clone(), row.pier.clone());
        let acw_in2 = match acw_map.get(&key) {
            Some(&val) => val,
            None => continue,
        };

        let axis_angle = axis_map.get(&key).copied().unwrap_or(0.0);
        let is_x = is_x_dir(axis_angle);
        let orientation = if is_x { "X" } else { "Y" };

        let fc_ksi = pier_fc_map
            .get(&(row.pier.clone(), row.story.clone()))
            .copied()
            .unwrap_or(p_params.fc_default_ksi);
        let fc_psi = fc_ksi * 1000.0;
        let fc_sqrt = fc_psi.sqrt();

        let ve_kip = row.shear_v2_abs_kip;
        let stress_psi = (ve_kip * 1000.0) / (phi_v * acw_in2);
        let limit_individual = 8.0;
        let stress_ratio = stress_psi / fc_sqrt;

        let pass = stress_ratio <= limit_individual;

        out_rows.push(PierShearStressRow {
            story: row.story.clone(),
            pier: row.pier.clone(),
            combo: row.output_case.clone(),
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
        let entry = avg_map.entry(row.story.clone()).or_insert((0.0, 0.0, 0.0));
        entry.0 += ve_kip;
        entry.1 += acw_in2;
        if fc_psi > entry.2 {
            entry.2 = fc_psi;
        }
    }

    let mut x_avg_rows = Vec::new();
    let mut y_avg_rows = Vec::new();
    let mut max_avg_ratio = 0.0;
    let limit_average = 10.0;

    for (story, (sum_ve, sum_acw, max_fc)) in x_avg_map {
        let sqrt_fc = max_fc.sqrt();
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

    for (story, (sum_ve, sum_acw, max_fc)) in y_avg_map {
        let sqrt_fc = max_fc.sqrt();
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

    let pass = max_ind_ratio <= 8.0 && max_avg_ratio <= 10.0;

    out_rows.sort_by(|a, b| b.stress_ratio.partial_cmp(&a.stress_ratio).unwrap());

    Ok(PierShearStressOutput {
        phi_v,
        limit_individual: 8.0,
        limit_average: 10.0,
        per_pier: out_rows,
        x_average: x_avg_rows,
        y_average: y_avg_rows,
        max_individual_ratio: max_ind_ratio,
        max_average_ratio: max_avg_ratio,
        pass,
    })
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::code_params::PierShearStressParams;
    use crate::tables::{pier_forces::PierForceRow, pier_section::PierSectionRow};
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

        let output = run(&mock_forces, &mock_sections, &pier_fc_map, &p_params).unwrap();
        let gov = &output.per_pier[0];

        assert_eq!(gov.pier, "P1");

        let expected_stress = 100.0 * 1000.0 / (0.75 * 200.0);
        let expected_ratio = expected_stress / (4000.0f64).sqrt();
        assert!((gov.stress_psi - expected_stress).abs() < 1e-3);
        assert!((gov.stress_ratio - expected_ratio).abs() < 1e-3);
    }
}
