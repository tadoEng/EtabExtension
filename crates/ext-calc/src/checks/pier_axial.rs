// checks/pier_axial.rs — pier axial stress check (ACI 318-14 §22.4)
//
// Checks compression demand against the nominal squash load of each wall pier.
//
// Formula:
//   Pu   = |P|                    [kip]  (axial demand — compression is negative in ETABS)
//   Ag   = lw × t × 144          [in²]  (gross section area = Acv for solid walls)
//   Po   = 0.85 × fc' × Ag       [kip]  (nominal squash load, simplified — rebar omitted)
//   ϕPo  = ϕ × Po                [kip]  (design squash load; ϕ=0.65 for tied ACI §9.3.2.2)
//   DCR  = Pu / ϕPo
//   fa   = Pu / Ag               [ksi]  (computed axial stress)
//   fa_ratio = fa / (0.85 × fc') [-]    (utilisation vs squash stress)
//
// Note: Po omits the rebar contribution (0.85 f'c Ag vs the full 0.85 f'c(Ag-Ast)+Ast·fy).
// This is the conservative simplified form standard for preliminary wall checks.
// A Phase 2 enhancement can add Ast when rebar area data is available.

use std::collections::{BTreeMap, HashMap};

use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{PierAxialOutput, PierAxialResult};
use crate::tables::pier_forces::PierForceRow;
use crate::tables::pier_section::PierSectionRow;

pub fn run(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    fc_map: &HashMap<(String, String), f64>,
    params: &CodeParams,
) -> Result<PierAxialOutput> {
    let axial_params = &params.pier_axial;
    let uc = &params.unit_context;

    // Validate combos exist in the force table.
    let available_cases: std::collections::HashSet<&str> =
        forces.iter().map(|r| r.output_case.as_str()).collect();
    for combo in &axial_params.load_combos {
        if !available_cases.contains(combo.as_str()) {
            bail!(
                "Configured pier axial combo '{}' not found in pier_forces table",
                combo
            );
        }
    }

    // Build section lookup: (pier, story) → PierSectionRow.
    let section_map: HashMap<(&str, &str), &PierSectionRow> = sections
        .iter()
        .map(|s| ((s.pier.as_str(), s.story.as_str()), s))
        .collect();

    // Group forces by (story, pier, combo) and find the maximum compression demand.
    // Compression is negative per ETABS convention, so minimum P = largest compression.
    let selected_combos: std::collections::HashSet<&str> =
        axial_params.load_combos.iter().map(String::as_str).collect();

    // Store (max_abs_p, raw_p_for_min) per group.
    // We want the most compressive (most negative) P value as the demand.
    let mut grouped: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for row in forces
        .iter()
        .filter(|r| selected_combos.contains(r.output_case.as_str()))
    {
        let key = (row.story.clone(), row.pier.clone(), row.output_case.clone());
        let entry = grouped.entry(key).or_insert(0.0_f64);
        // Take the value with the largest absolute axial force.
        // For compression-dominated walls this is the most negative P.
        if row.axial_p_kip.abs() > entry.abs() {
            *entry = row.axial_p_kip;
        }
    }

    if grouped.is_empty() {
        bail!("No pier force rows matched the configured axial combos");
    }

    let phi_axial = axial_params.phi_axial;
    let mut results: Vec<PierAxialResult> = Vec::with_capacity(grouped.len());

    for ((story, pier, combo), p_kip) in &grouped {
        let sec_key = (pier.as_str(), story.as_str());
        let section = match section_map.get(&sec_key) {
            Some(s) => s,
            None => {
                eprintln!(
                    "[ext-calc] warn: no section properties for pier '{}' at story '{}'; \
                     axial row skipped",
                    pier, story
                );
                continue;
            }
        };

        let fc_ksi = fc_map
            .get(&(pier.clone(), story.clone()))
            .copied()
            .unwrap_or(params.pier_shear_seismic.fc_default_ksi);

        // Ag = lw × t × 144 [in²] — gross section, same as Acv for a solid wall.
        let ag_in2 = section.ag_in2;

        // Demand: use |P| (absolute value) — handles both compression and tension.
        let pu_kip = p_kip.abs();

        // Nominal squash load (simplified, no rebar contribution):
        //   Po [kip] = 0.85 × fc' [ksi] × Ag [in²]
        let po_kip     = 0.85 * fc_ksi * ag_in2;
        let phi_po_kip = phi_axial * po_kip;

        let dcr      = pu_kip / phi_po_kip;
        let fa_ksi   = pu_kip / ag_in2;
        let fa_ratio = fa_ksi / (0.85 * fc_ksi);

        results.push(PierAxialResult {
            pier_label: pier.clone(),
            story:      story.clone(),
            combo:      combo.clone(),
            pu:         uc.qty_force(pu_kip),
            ag:         uc.qty_area_in2(ag_in2),
            phi_po:     uc.qty_force(phi_po_kip),
            fa:         crate::output::Quantity::new(fa_ksi, "ksi"),
            fa_ratio,
            dcr,
            pass:       dcr <= 1.0,
            fc_ksi,
            material:   section.material.clone(),
        });
    }

    if results.is_empty() {
        bail!("No pier axial results produced — check section/force table alignment");
    }

    let governing = results
        .iter()
        .max_by(|a, b| a.dcr.partial_cmp(&b.dcr).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
        .expect("results is non-empty");

    let pass = results.iter().all(|r| r.pass);

    Ok(PierAxialOutput { piers: results, governing, pass })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::code_params::CodeParams;
    use crate::tables::material_props::load_material_properties;
    use crate::tables::pier_forces::load_pier_forces;
    use crate::tables::pier_section::load_pier_sections;

    use super::super::pier_shear::build_pier_fc_map;
    use super::run;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    fn fixture_params() -> CodeParams {
        let config = Config::load(&fixture_dir()).unwrap();
        CodeParams::from_config(&config).unwrap()
    }

    #[test]
    fn pier_axial_produces_results_and_governing_is_max_dcr() {
        let dir      = fixture_dir();
        let forces   = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat      = load_material_properties(&dir).unwrap();
        let params   = fixture_params();
        let fc_map   = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();

        assert!(!output.piers.is_empty());
        assert!(output.governing.dcr > 0.0);
        // Governing must be the maximum DCR across all results.
        let max_dcr = output.piers.iter().map(|r| r.dcr).fold(f64::NEG_INFINITY, f64::max);
        assert!((output.governing.dcr - max_dcr).abs() < 1e-9);
    }

    #[test]
    fn pier_axial_all_pu_and_phi_po_are_positive() {
        let dir      = fixture_dir();
        let forces   = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat      = load_material_properties(&dir).unwrap();
        let params   = fixture_params();
        let fc_map   = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();
        for r in &output.piers {
            assert!(r.pu.value >= 0.0, "Pu must be non-negative: {}/{}", r.pier_label, r.story);
            assert!(r.phi_po.value > 0.0, "ϕPo must be positive: {}/{}", r.pier_label, r.story);
            assert!(r.ag.value > 0.0, "Ag must be positive: {}/{}", r.pier_label, r.story);
            assert!(r.fa_ratio >= 0.0, "fa_ratio must be non-negative: {}/{}", r.pier_label, r.story);
        }
    }

    #[test]
    fn pier_axial_hand_check_phi_po_formula() {
        // Po = 0.85 × fc' × Ag; for C1Y1 at L01: Ag = 42×2×144 = 12096 in², fc=8.0 ksi
        // Po = 0.85 × 8.0 × 12096 = 82,252.8 kip
        // ϕPo = 0.65 × 82252.8 = 53,464.3 kip
        let ag_in2 = 42.0_f64 * 2.0 * 144.0; // 12096
        let fc_ksi = 8.0_f64;
        let po = 0.85 * fc_ksi * ag_in2;
        let phi_po = 0.65 * po;
        assert!((po - 82_252.8).abs() < 1.0, "Po = {po:.1}");
        assert!((phi_po - 53_464.3).abs() < 1.0, "ϕPo = {phi_po:.1}");
    }

    #[test]
    fn pier_axial_errors_when_combo_missing() {
        let dir      = fixture_dir();
        let forces   = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat      = load_material_properties(&dir).unwrap();
        let mut config = Config::load(&fixture_dir()).unwrap();
        config.calc.pier_axial.load_combos = vec!["BAD_COMBO".into()];
        let params   = CodeParams::from_config(&config).unwrap();
        let fc_map   = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        assert!(run(&forces, &sections, &fc_map, &params).is_err());
    }
}
