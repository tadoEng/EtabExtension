// checks/pier_shear.rs — shared ACI 318-14 §11.5.4.3 shear wall formula
//
// Both wind (ϕ=0.75) and seismic (ϕ=0.60) checks delegate here.
// The caller supplies PierShearParams with the correct ϕ value.
//
// Formula (psi-based US customary, ACI 318-14 §11.5.4.3):
//   Acv  = lw × t × 144                  [in²]  (WidthBot × ThickBot × 144)
//   fc'  = fc_ksi × 1000                 [psi]
//   Vn   = Acv × (αc × √fc' + ρt × fy)  [lb]  → ÷ 1000 → [kip]
//   ϕVn  = ϕ × Vn                        [kip]
//   Vu   = max |V2| across Top+Bottom+Max+Min for each (pier, story, combo)
//   DCR  = Vu / ϕVn
//
// αc = 2.0 is the psi-based coefficient for hw/lw ≥ 2.0 (ACI 318-14 §11.5.4.3).
// fy in the formula is in psi (fy_ksi × 1000).
// Do NOT use αc = 0.17 here — that is the MPa-based form.

use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, bail};

use crate::code_params::PierShearParams;
use crate::output::{PierShearOutput, PierShearResult};
use crate::tables::material_props::MaterialProp;
use crate::tables::pier_forces::PierForceRow;
use crate::tables::pier_section::PierSectionRow;
use crate::unit_convert::UnitContext;

/// Build a (pier_label, story) → fc_ksi lookup from the section-material join.
///
/// Call once per `CalcRunner::run_all` invocation and share across all three
/// pier checks (wind shear, seismic shear, axial).
///
/// Falls back to `default_fc_ksi` when a section's material name is missing
/// from the material properties table. A warning is printed to stderr so the
/// engineer can see which piers/stories are using the fallback.
pub fn build_pier_fc_map(
    sections: &[PierSectionRow],
    material_props: &HashMap<String, MaterialProp>,
    default_fc_ksi: f64,
) -> HashMap<(String, String), f64> {
    sections
        .iter()
        .map(|s| {
            let fc = material_props
                .get(&s.material)
                .map(|m| m.fc_ksi)
                .unwrap_or_else(|| {
                    eprintln!(
                        "[ext-calc] warn: material '{}' not found for pier '{}' at story '{}'; \
                         using fc_default = {:.1} ksi",
                        s.material, s.pier, s.story, default_fc_ksi
                    );
                    default_fc_ksi
                });
            ((s.pier.clone(), s.story.clone()), fc)
        })
        .collect()
}

/// Run the pier shear check for one combo set (wind or seismic).
///
/// Groups pier_forces rows by (story, pier, combo) and computes the ACI 318-14
/// §11.5.4.3 capacity for each group using section geometry and material
/// strength from the pre-built lookup maps.
pub fn run(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    fc_map: &HashMap<(String, String), f64>,
    shear_params: &PierShearParams,
    uc: &UnitContext,
) -> Result<PierShearOutput> {
    // Validate that every configured combo is present in the force table.
    let available_cases: std::collections::HashSet<&str> =
        forces.iter().map(|r| r.output_case.as_str()).collect();
    for combo in &shear_params.load_combos {
        if !available_cases.contains(combo.as_str()) {
            bail!(
                "Configured pier shear combo '{}' not found in pier_forces table",
                combo
            );
        }
    }

    // Build section lookup: (pier, story) → PierSectionRow.
    // ETABS aggregates multi-element piers into one row per (pier, story) in
    // pier_section_properties, so WidthBot/ThickBot represent the full combined section.
    let section_map: HashMap<(&str, &str), &PierSectionRow> = sections
        .iter()
        .map(|s| ((s.pier.as_str(), s.story.as_str()), s))
        .collect();

    // Group forces by (story, pier, combo) and envelope across all Location
    // (Top/Bottom) and StepType (Max/Min/Step By Step) variants.
    // This ensures the governing shear is found regardless of how ETABS
    // reports the combination results.
    let selected_combos: std::collections::HashSet<&str> = shear_params
        .load_combos
        .iter()
        .map(String::as_str)
        .collect();

    let mut grouped: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for row in forces
        .iter()
        .filter(|r| selected_combos.contains(r.output_case.as_str()))
    {
        // Key order: (story, pier, combo) — produces elevation-ordered output.
        let key = (row.story.clone(), row.pier.clone(), row.output_case.clone());
        let entry = grouped.entry(key).or_insert(0.0_f64);
        // V2 is the in-plane (strong-axis) shear for wall piers.
        // shear_v2_abs_kip is pre-computed at load time as |V2|.
        *entry = entry.max(row.shear_v2_abs_kip);
    }

    if grouped.is_empty() {
        bail!("No pier force rows matched the configured shear combos");
    }

    let phi_v = shear_params.phi_v;
    let alpha_c = shear_params.alpha_c; // 2.0 psi-based
    let fy_psi = shear_params.fy_ksi * 1_000.0;
    let rho_t = shear_params.rho_t;

    let mut results: Vec<PierShearResult> = Vec::with_capacity(grouped.len());

    for ((story, pier, combo), vu_kip) in &grouped {
        let sec_key = (pier.as_str(), story.as_str());
        let section = match section_map.get(&sec_key) {
            Some(s) => s,
            None => {
                // Pier/story present in forces but absent from section properties.
                // Skip with a warning rather than aborting the entire check.
                eprintln!(
                    "[ext-calc] warn: no section properties for pier '{}' at story '{}'; \
                     row skipped",
                    pier, story
                );
                continue;
            }
        };

        let fc_ksi = fc_map
            .get(&(pier.clone(), story.clone()))
            .copied()
            .unwrap_or(shear_params.fc_default_ksi);
        let fc_psi = fc_ksi * 1_000.0;

        // Acv = lw × t × 144  [in²] — already computed in PierSectionRow.
        let acv_in2 = section.acv_in2;

        // ACI 318-14 §11.5.4.3 (psi-based):
        //   Vn [lb] = Acv [in²] × (αc × √f'c [psi] + ρt × fy [psi])
        //   Vn [kip] = Vn [lb] / 1000
        let vn_kip = acv_in2 * (alpha_c * fc_psi.sqrt() + rho_t * fy_psi) / 1_000.0;
        let phi_vn_kip = phi_v * vn_kip;
        let dcr = vu_kip / phi_vn_kip;

        results.push(PierShearResult {
            pier_label: pier.clone(),
            story: story.clone(),
            combo: combo.clone(),
            // "envelope" because we already took max across Top/Bottom/Max/Min.
            location: "envelope".to_string(),
            vu: uc.qty_force(*vu_kip),
            acv: uc.qty_area_in2(acv_in2),
            fc_ksi,
            vn: uc.qty_force(vn_kip),
            phi_vn: uc.qty_force(phi_vn_kip),
            dcr,
            pass: dcr <= 1.0,
            section_id: format!("{:.0}x{:.0}", section.width_bot_ft, section.thick_bot_ft),
            material: section.material.clone(),
        });
    }

    if results.is_empty() {
        bail!("No pier shear results produced — check section/force table alignment");
    }

    let governing = results
        .iter()
        .max_by(|a, b| {
            a.dcr
                .partial_cmp(&b.dcr)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .expect("results is non-empty");

    let pass = results.iter().all(|r| r.pass);

    Ok(PierShearOutput {
        phi_v,
        piers: results,
        governing,
        pass,
    })
}

// ── Hand-check reference values for C1Y1 at L20 ─────────────────────────────
// section : WidthBot=22 ft, ThickBot=2 ft → Acv = 22×2×144 = 6336 in²
// material: 8000Psi → fc_ksi = 8.0 → fc_psi = 8000
// Vn  = 6336 × (2.0×√8000 + 0.0025×60000) / 1000
//      = 6336 × (178.885 + 150.0) / 1000
//      ≈ 2083.9 kip
// ϕVn (wind)    = 0.75 × 2083.9 ≈ 1562.9 kip
// ϕVn (seismic) = 0.60 × 2083.9 ≈ 1250.3 kip
pub const C1Y1_L20_ACV_IN2: f64 = 22.0 * 2.0 * 144.0; // 6336.0
pub const C1Y1_L20_FC_PSI: f64 = 8_000.0;
pub const C1Y1_L20_VN_KIP: f64 = 2_083.9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acv_for_22x2_ft_section_is_6336_in2() {
        assert!((C1Y1_L20_ACV_IN2 - 6_336.0).abs() < 1e-6);
    }

    #[test]
    fn vn_formula_produces_correct_value_for_8000psi() {
        let vn = C1Y1_L20_ACV_IN2 * (2.0 * C1Y1_L20_FC_PSI.sqrt() + 0.0025 * 60_000.0) / 1_000.0;
        assert!(
            (vn - C1Y1_L20_VN_KIP).abs() < 0.2,
            "Vn = {vn:.3} kip, expected ≈ {C1Y1_L20_VN_KIP} kip"
        );
    }

    #[test]
    fn phi_vn_wind_is_75_percent_of_vn() {
        assert!((0.75 * C1Y1_L20_VN_KIP - 1_562.9).abs() < 0.5);
    }

    #[test]
    fn phi_vn_seismic_is_60_percent_of_vn() {
        assert!((0.60 * C1Y1_L20_VN_KIP - 1_250.3).abs() < 0.5);
    }

    #[test]
    fn build_pier_fc_map_uses_fallback_for_unknown_material() {
        let sections = vec![PierSectionRow {
            story: "L01".into(),
            pier: "P99".into(),
            axis_angle: 0.0,
            width_bot_ft: 10.0,
            thick_bot_ft: 1.0,
            width_top_ft: 10.0,
            thick_top_ft: 1.0,
            material: "UNKNOWN".into(),
            acv_in2: 10.0 * 1.0 * 144.0,
            ag_in2: 10.0 * 1.0 * 144.0,
        }];
        let map = build_pier_fc_map(&sections, &HashMap::new(), 5.0);
        assert_eq!(
            map.get(&("P99".to_string(), "L01".to_string())).copied(),
            Some(5.0)
        );
    }

    #[test]
    fn build_pier_fc_map_uses_material_table_when_found() {
        let sections = vec![PierSectionRow {
            story: "L20".into(),
            pier: "C1Y1".into(),
            axis_angle: 90.0,
            width_bot_ft: 22.0,
            thick_bot_ft: 2.0,
            width_top_ft: 22.0,
            thick_top_ft: 2.0,
            material: "8000Psi".into(),
            acv_in2: 22.0 * 2.0 * 144.0,
            ag_in2: 22.0 * 2.0 * 144.0,
        }];
        let mut mat_props: HashMap<String, MaterialProp> = HashMap::new();
        mat_props.insert(
            "8000Psi".to_string(),
            MaterialProp {
                name: "8000Psi".into(),
                fc_kipsft2: 1152.0,
                fc_ksi: 8.0,
                fc_psi: 8_000.0,
                is_lightweight: false,
            },
        );
        let map = build_pier_fc_map(&sections, &mat_props, 4.0);
        assert!((map[&("C1Y1".to_string(), "L20".to_string())] - 8.0).abs() < 1e-9);
    }
}
