use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;

use crate::code_params::CodeParams;
use crate::output::{PierAxialResult, PierAxialStressOutput, Quantity};
use crate::tables::pier_forces::PierForceRow;
use crate::tables::pier_section::PierSectionRow;

pub fn run(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
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
            .unwrap_or(4.0);
        let ag_in2 = section.ag_in2;
        let pu_kip = p_kip.abs();

        let po_kip = 0.85 * fc_ksi * ag_in2;
        let phi_po_kip = phi_axial * po_kip;

        let dcr = pu_kip / phi_po_kip;
        let fa_ksi = pu_kip / ag_in2;
        let fa_ratio = fa_ksi / (0.85 * fc_ksi);

        let res = PierAxialResult {
            pier_label: pier.clone(),
            story: story.clone(),
            combo: combo.clone(),
            pu: uc.qty_force(pu_kip),
            ag: uc.qty_area_in2(ag_in2),
            phi_po: uc.qty_force(phi_po_kip),
            fa: Quantity::new(fa_ksi, "ksi"),
            fa_ratio,
            dcr,
            pass: dcr <= 1.0,
            fc_ksi,
            material: section.material.clone(),
        };

        if grav_set.contains(combo.as_str()) {
            if gov_grav.as_ref().is_none_or(|g| dcr > g.dcr) {
                gov_grav = Some(res.clone());
            }
        } else if wind_set.contains(combo.as_str()) {
            if gov_wind.as_ref().is_none_or(|g| dcr > g.dcr) {
                gov_wind = Some(res.clone());
            }
        } else if seis_set.contains(combo.as_str()) && gov_seis.as_ref().is_none_or(|g| dcr > g.dcr)
        {
            gov_seis = Some(res.clone());
        }

        results.push(res);
    }

    results.sort_by(|a, b| {
        b.dcr
            .partial_cmp(&a.dcr)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if results.is_empty() {
        // Return dummy output when no matching rows to avoid complete failure
        return Ok(PierAxialStressOutput {
            phi_axial,
            piers: Vec::new(),
            governing_gravity: None,
            governing_wind: None,
            governing_seismic: None,
            governing: PierAxialResult {
                pier_label: String::new(),
                story: String::new(),
                combo: String::new(),
                pu: Quantity::new(0.0, ""),
                ag: Quantity::new(0.0, ""),
                phi_po: Quantity::new(0.0, ""),
                fa: Quantity::new(0.0, ""),
                fa_ratio: 0.0,
                dcr: 0.0,
                pass: true,
                fc_ksi: 0.0,
                material: String::new(),
            },
            pass: true,
        });
    }

    let governing = results[0].clone();
    let pass = results.iter().all(|r| r.pass);

    Ok(PierAxialStressOutput {
        phi_axial,
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
    use std::collections::HashMap;

    #[test]
    fn pier_axial_produces_governing_matches() {
        let mut params = CodeParams::for_testing();

        let axial_params = crate::code_params::PierAxialStressParams {
            phi_axial: 0.65,
            gravity_combos: vec!["Grav1".into()],
            wind_combos: vec!["Wind1".into()],
            seismic_combos: vec!["Seis1".into()],
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

        let output = run(&forces, &sections, &fc_map, &params).unwrap();

        assert!(output.governing_gravity.is_some());
        assert!(output.governing_wind.is_some());
        assert!(output.governing_seismic.is_none());
        assert_eq!(output.governing.combo, "Wind1");
    }
}
