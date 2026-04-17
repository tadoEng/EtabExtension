use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;

use crate::code_params::CodeParams;
use crate::output::{PierAxialResult, PierAxialStressOutput, Quantity};
use crate::tables::pier_forces::PierForceRow;
use crate::tables::pier_section::PierSectionRow;
use crate::tables::story_def::StoryDefRow;

pub fn run(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    stories: &[StoryDefRow],
    fc_map: &HashMap<(String, String), f64>,
    params: &CodeParams,
) -> Result<PierAxialStressOutput> {
    let axial_params = params.pier_axial_stress.as_ref().unwrap();
    let uc = &params.unit_context;

    let grav_set: HashSet<&str> = axial_params
        .gravity_combos
        .iter()
        .map(|s| s.as_str())
        .collect();
    let wind_set: HashSet<&str> = axial_params
        .wind_combos
        .iter()
        .map(|s| s.as_str())
        .collect();
    let seis_set: HashSet<&str> = axial_params
        .seismic_combos
        .iter()
        .map(|s| s.as_str())
        .collect();

    let section_map: HashMap<(&str, &str), &PierSectionRow> = sections
        .iter()
        .map(|s| ((s.pier.as_str(), s.story.as_str()), s))
        .collect();

    let mut sorted_stories = stories.to_vec();
    sorted_stories.sort_by(|a, b| {
        b.elevation_ft
            .partial_cmp(&a.elevation_ft)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let story_order = sorted_stories
        .iter()
        .map(|story| story.story.clone())
        .collect::<Vec<_>>();
    let story_rank = story_order
        .iter()
        .enumerate()
        .map(|(idx, story)| (story.clone(), idx))
        .collect::<HashMap<_, _>>();

    let mut grouped: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for row in forces {
        let is_grav = grav_set.contains(row.output_case.as_str());
        let is_wind = wind_set.contains(row.output_case.as_str());
        let is_seis = seis_set.contains(row.output_case.as_str());

        if !is_grav && !is_wind && !is_seis {
            continue;
        }

        let key = (row.story.clone(), row.pier.clone(), row.output_case.clone());
        let entry = grouped.entry(key).or_insert(0.0_f64);

        if row.axial_p_kip.abs() > entry.abs() {
            *entry = row.axial_p_kip;
        }
    }

    let phi_axial = axial_params.phi_axial;
    let mut results = Vec::with_capacity(grouped.len());

    let mut gov_grav: Option<PierAxialResult> = None;
    let mut gov_wind: Option<PierAxialResult> = None;
    let mut gov_seis: Option<PierAxialResult> = None;

    for ((story, pier, combo), p_kip) in grouped {
        let sec_key = (pier.as_str(), story.as_str());
        let section = match section_map.get(&sec_key) {
            Some(s) => s,
            None => {
                continue;
            }
        };

        let fc_ksi = fc_map
            .get(&(pier.clone(), story.clone()))
            .copied()
            .unwrap_or(axial_params.fc_default_ksi);
        let ag_in2 = section.ag_in2;
        let pu_kip = p_kip.abs();
        let fa_signed_ksi = -p_kip / ag_in2;

        let po_kip = 0.85 * fc_ksi * ag_in2;
        let phi_po_kip = phi_axial * po_kip;

        let dcr = pu_kip / phi_po_kip;
        let fa_ksi = pu_kip / ag_in2;
        let fa_ratio = fa_ksi / (0.85 * fc_ksi);

        let category = if grav_set.contains(combo.as_str()) {
            "gravity"
        } else if wind_set.contains(combo.as_str()) {
            "wind"
        } else {
            "seismic"
        };

        let res = PierAxialResult {
            pier_label: pier.clone(),
            story: story.clone(),
            combo: combo.clone(),
            category: category.to_string(),
            pu: uc.qty_force(pu_kip),
            pu_signed: uc.qty_force(p_kip),
            ag: uc.qty_area_in2(ag_in2),
            phi_po: uc.qty_force(phi_po_kip),
            fa: Quantity::new(fa_ksi, "ksi"),
            fa_signed: Quantity::new(fa_signed_ksi, "ksi"),
            fa_ratio,
            dcr,
            pass: dcr <= 1.0,
            fc_ksi,
            material: section.material.clone(),
        };

        if grav_set.contains(combo.as_str()) {
            if gov_grav.as_ref().map_or(true, |g| dcr > g.dcr) {
                gov_grav = Some(res.clone());
            }
        } else if wind_set.contains(combo.as_str()) {
            if gov_wind.as_ref().map_or(true, |g| dcr > g.dcr) {
                gov_wind = Some(res.clone());
            }
        } else if seis_set.contains(combo.as_str()) {
            if gov_seis.as_ref().map_or(true, |g| dcr > g.dcr) {
                gov_seis = Some(res.clone());
            }
        }

        results.push(res);
    }

    if results.is_empty() {
        // Return dummy output when no matching rows to avoid complete failure
        return Ok(PierAxialStressOutput {
            phi_axial,
            story_order,
            piers: Vec::new(),
            governing_gravity: None,
            governing_wind: None,
            governing_seismic: None,
            governing: PierAxialResult {
                pier_label: String::new(),
                story: String::new(),
                combo: String::new(),
                category: String::new(),
                pu: Quantity::new(0.0, ""),
                pu_signed: Quantity::new(0.0, ""),
                ag: Quantity::new(0.0, ""),
                phi_po: Quantity::new(0.0, ""),
                fa: Quantity::new(0.0, ""),
                fa_signed: Quantity::new(0.0, ""),
                fa_ratio: 0.0,
                dcr: 0.0,
                pass: true,
                fc_ksi: 0.0,
                material: String::new(),
            },
            pass: true,
        });
    }

    let governing = results
        .iter()
        .max_by(|a, b| {
            a.dcr
                .partial_cmp(&b.dcr)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap();
    results.sort_by(|a, b| {
        let a_rank = story_rank.get(&a.story).copied().unwrap_or(usize::MAX);
        let b_rank = story_rank.get(&b.story).copied().unwrap_or(usize::MAX);
        a_rank
            .cmp(&b_rank)
            .then_with(|| a.pier_label.cmp(&b.pier_label))
            .then_with(|| a.combo.cmp(&b.combo))
    });
    let pass = results.iter().all(|r| r.pass);

    Ok(PierAxialStressOutput {
        phi_axial,
        story_order,
        piers: results,
        governing_gravity: gov_grav,
        governing_wind: gov_wind,
        governing_seismic: gov_seis,
        governing,
        pass,
    })
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::code_params::CodeParams;
    use crate::tables::pier_forces::PierForceRow;
    use crate::tables::pier_section::PierSectionRow;
    use crate::tables::story_def::StoryDefRow;
    use std::collections::HashMap;

    #[test]
    fn pier_axial_produces_governing_matches() {
        let mut params = CodeParams::for_testing();

        let axial_params = crate::code_params::PierAxialStressParams {
            phi_axial: 0.65,
            gravity_combos: vec!["Grav1".into()],
            wind_combos: vec!["Wind1".into()],
            seismic_combos: vec!["Seis1".into()],
            fc_default_ksi: 4.0,
        };
        params.pier_axial_stress = Some(axial_params);

        let mut sections = vec![];
        sections.push(PierSectionRow {
            story: "L1".into(),
            pier: "P1".into(),
            axis_angle: 0.0,
            width_bot_ft: 10.0,
            thick_bot_ft: 1.0,
            width_top_ft: 10.0,
            thick_top_ft: 1.0,
            material: "C4000".into(),
            acv_in2: 100.0,
            ag_in2: 100.0,
        });

        let mut forces = vec![];
        // Grav: 100 kips  => DCR = 100 / (0.65 * 0.85 * 4 * 100) = 100 / 221 = 0.45
        forces.push(PierForceRow {
            story: "L1".into(),
            pier: "P1".into(),
            output_case: "Grav1".into(),
            case_type: "".into(),
            step_type: "".into(),
            location: "".into(),
            axial_p_kip: 100.0,
            shear_v2_kip: 0.0,
            shear_v2_abs_kip: 0.0,
            shear_v3_kip: 0.0,
            torsion_t_kip_ft: 0.0,
            moment_m2_kip_ft: 0.0,
            moment_m3_kip_ft: 0.0,
        });

        // Wind: 150 kips
        forces.push(PierForceRow {
            story: "L1".into(),
            pier: "P1".into(),
            output_case: "Wind1".into(),
            case_type: "".into(),
            step_type: "".into(),
            location: "".into(),
            axial_p_kip: 150.0,
            shear_v2_kip: 0.0,
            shear_v2_abs_kip: 0.0,
            shear_v3_kip: 0.0,
            torsion_t_kip_ft: 0.0,
            moment_m2_kip_ft: 0.0,
            moment_m3_kip_ft: 0.0,
        });

        let mut fc_map = HashMap::new();
        fc_map.insert(("P1".into(), "L1".into()), 4.0);

        let stories = vec![StoryDefRow {
            story: "L1".into(),
            height_ft: 10.0,
            elevation_ft: 10.0,
        }];

        let output = run(&forces, &sections, &stories, &fc_map, &params).unwrap();

        assert!(output.governing_gravity.is_some());
        assert!(output.governing_wind.is_some());
        assert!(output.governing_seismic.is_none());
        assert_eq!(output.governing.combo, "Wind1");
    }

    #[test]
    fn pier_axial_preserves_top_to_bottom_story_order() {
        let mut params = CodeParams::for_testing();
        params.pier_axial_stress = Some(crate::code_params::PierAxialStressParams {
            phi_axial: 0.65,
            gravity_combos: vec!["Grav1".into()],
            wind_combos: vec![],
            seismic_combos: vec![],
            fc_default_ksi: 4.0,
        });

        let sections = vec![
            PierSectionRow {
                story: "L2".into(),
                pier: "P1".into(),
                axis_angle: 0.0,
                width_bot_ft: 10.0,
                thick_bot_ft: 1.0,
                width_top_ft: 10.0,
                thick_top_ft: 1.0,
                material: "C4000".into(),
                acv_in2: 100.0,
                ag_in2: 100.0,
            },
            PierSectionRow {
                story: "L1".into(),
                pier: "P1".into(),
                axis_angle: 0.0,
                width_bot_ft: 10.0,
                thick_bot_ft: 1.0,
                width_top_ft: 10.0,
                thick_top_ft: 1.0,
                material: "C4000".into(),
                acv_in2: 100.0,
                ag_in2: 100.0,
            },
        ];
        let forces = vec![
            PierForceRow {
                story: "L1".into(),
                pier: "P1".into(),
                output_case: "Grav1".into(),
                case_type: "".into(),
                step_type: "".into(),
                location: "".into(),
                axial_p_kip: 100.0,
                shear_v2_kip: 0.0,
                shear_v2_abs_kip: 0.0,
                shear_v3_kip: 0.0,
                torsion_t_kip_ft: 0.0,
                moment_m2_kip_ft: 0.0,
                moment_m3_kip_ft: 0.0,
            },
            PierForceRow {
                story: "L2".into(),
                pier: "P1".into(),
                output_case: "Grav1".into(),
                case_type: "".into(),
                step_type: "".into(),
                location: "".into(),
                axial_p_kip: 80.0,
                shear_v2_kip: 0.0,
                shear_v2_abs_kip: 0.0,
                shear_v3_kip: 0.0,
                torsion_t_kip_ft: 0.0,
                moment_m2_kip_ft: 0.0,
                moment_m3_kip_ft: 0.0,
            },
        ];
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
        let mut fc_map = HashMap::new();
        fc_map.insert(("P1".into(), "L1".into()), 4.0);
        fc_map.insert(("P1".into(), "L2".into()), 4.0);

        let output = run(&forces, &sections, &stories, &fc_map, &params).unwrap();
        assert_eq!(output.story_order, vec!["L2", "L1"]);
        assert_eq!(
            output
                .piers
                .iter()
                .map(|row| row.story.as_str())
                .collect::<Vec<_>>(),
            vec!["L2", "L1"]
        );
    }

    #[test]
    fn pier_axial_reports_signed_stress_with_etabs_sign_convention() {
        let mut params = CodeParams::for_testing();
        params.pier_axial_stress = Some(crate::code_params::PierAxialStressParams {
            phi_axial: 0.65,
            gravity_combos: vec!["Grav1".into()],
            wind_combos: vec![],
            seismic_combos: vec![],
            fc_default_ksi: 4.0,
        });

        let sections = vec![PierSectionRow {
            story: "L1".into(),
            pier: "P1".into(),
            axis_angle: 0.0,
            width_bot_ft: 10.0,
            thick_bot_ft: 1.0,
            width_top_ft: 10.0,
            thick_top_ft: 1.0,
            material: "C4000".into(),
            acv_in2: 100.0,
            ag_in2: 100.0,
        }];
        let stories = vec![StoryDefRow {
            story: "L1".into(),
            height_ft: 10.0,
            elevation_ft: 10.0,
        }];
        let mut fc_map = HashMap::new();
        fc_map.insert(("P1".into(), "L1".into()), 4.0);

        // ETABS: negative P = compression => positive fa_signed.
        let compression = vec![PierForceRow {
            story: "L1".into(),
            pier: "P1".into(),
            output_case: "Grav1".into(),
            case_type: "".into(),
            step_type: "".into(),
            location: "".into(),
            axial_p_kip: -120.0,
            shear_v2_kip: 0.0,
            shear_v2_abs_kip: 0.0,
            shear_v3_kip: 0.0,
            torsion_t_kip_ft: 0.0,
            moment_m2_kip_ft: 0.0,
            moment_m3_kip_ft: 0.0,
        }];
        let out_c = run(&compression, &sections, &stories, &fc_map, &params).unwrap();
        assert!((out_c.piers[0].fa_signed.value - 1.2).abs() < 1e-9);

        // ETABS: positive P = tension => negative fa_signed.
        let tension = vec![PierForceRow {
            story: "L1".into(),
            pier: "P1".into(),
            output_case: "Grav1".into(),
            case_type: "".into(),
            step_type: "".into(),
            location: "".into(),
            axial_p_kip: 80.0,
            shear_v2_kip: 0.0,
            shear_v2_abs_kip: 0.0,
            shear_v3_kip: 0.0,
            torsion_t_kip_ft: 0.0,
            moment_m2_kip_ft: 0.0,
            moment_m3_kip_ft: 0.0,
        }];
        let out_t = run(&tension, &sections, &stories, &fc_map, &params).unwrap();
        assert!((out_t.piers[0].fa_signed.value + 0.8).abs() < 1e-9);
    }
}
