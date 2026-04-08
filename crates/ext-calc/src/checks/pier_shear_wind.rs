// checks/pier_shear_wind.rs — wind pier shear check
//
// Delegates to pier_shear::run() with ϕ = 0.75 (ACI 318-14 §9.3.2.3).
// Config section: [calc.pier-shear-wind]

use std::collections::HashMap;

use anyhow::Result;

use crate::code_params::CodeParams;
use crate::output::PierShearOutput;
use crate::tables::pier_forces::PierForceRow;
use crate::tables::pier_section::PierSectionRow;

use super::pier_shear;

pub fn run(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    fc_map: &HashMap<(String, String), f64>,
    params: &CodeParams,
) -> Result<PierShearOutput> {
    pier_shear::run(
        forces,
        sections,
        fc_map,
        &params.pier_shear_wind,
        &params.unit_context,
    )
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
    fn pier_shear_wind_produces_results_for_all_configured_combos() {
        let dir = fixture_dir();
        let forces = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat = load_material_properties(&dir).unwrap();
        let params = fixture_params();
        let fc_map = build_pier_fc_map(&sections, &mat, params.pier_shear_wind.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();

        assert!(
            !output.piers.is_empty(),
            "expected at least one pier result"
        );
        assert!(
            output.governing.dcr > 0.0,
            "governing DCR should be positive"
        );
        assert_eq!(output.phi_v, 0.75, "wind check must use ϕ = 0.75");
        // Every result that passes must have DCR ≤ 1.0
        for r in &output.piers {
            if r.pass {
                assert!(r.dcr <= 1.0, "passing result has DCR > 1.0: {:?}", r.dcr);
            }
        }
        // Governing is the row with the highest DCR
        let max_dcr = output
            .piers
            .iter()
            .map(|r| r.dcr)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((output.governing.dcr - max_dcr).abs() < 1e-9);
    }

    #[test]
    fn pier_shear_wind_errors_when_combo_missing() {
        let dir = fixture_dir();
        let forces = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat = load_material_properties(&dir).unwrap();
        let mut config = Config::load(&fixture_dir()).unwrap();
        config.calc.pier_shear_wind.load_combos = vec!["NONEXISTENT_COMBO".into()];
        let params = CodeParams::from_config(&config).unwrap();
        let fc_map = build_pier_fc_map(&sections, &mat, params.pier_shear_wind.fc_default_ksi);

        assert!(run(&forces, &sections, &fc_map, &params).is_err());
    }

    #[test]
    fn pier_shear_wind_acv_and_capacity_are_positive() {
        let dir = fixture_dir();
        let forces = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat = load_material_properties(&dir).unwrap();
        let params = fixture_params();
        let fc_map = build_pier_fc_map(&sections, &mat, params.pier_shear_wind.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();
        for r in &output.piers {
            assert!(
                r.acv.value > 0.0,
                "Acv must be positive for pier {}/{}",
                r.pier_label,
                r.story
            );
            assert!(
                r.phi_vn.value > 0.0,
                "ϕVn must be positive for pier {}/{}",
                r.pier_label,
                r.story
            );
            assert!(
                r.vu.value >= 0.0,
                "Vu must be non-negative for pier {}/{}",
                r.pier_label,
                r.story
            );
        }
    }

    #[test]
    fn pier_shear_wind_hand_check_c1y1_l20() {
        // C1Y1 at L20 (EVN_LRFD_EQ combo, reused as wind fixture):
        //   Section: WidthBot=22 ft, ThickBot=2 ft → Acv = 22×2×144 = 6336 in²
        //   Material: 8000Psi → fc_ksi=8.0 → fc_psi=8000
        //   Vu = max(|159.533|, |145.439|) = 159.533 kip  (from Parquet)
        //   Vn = 6336 × (2.0×√8000 + 0.0025×60000) / 1000 ≈ 2083.9 kip
        //   ϕVn (wind ϕ=0.75) = 0.75 × 2083.9 ≈ 1562.9 kip
        //   DCR = 159.533 / 1562.9 ≈ 0.102
        let dir = fixture_dir();
        let forces = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat = load_material_properties(&dir).unwrap();
        let params = fixture_params();
        let fc_map = build_pier_fc_map(&sections, &mat, params.pier_shear_wind.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();

        let r = output
            .piers
            .iter()
            .find(|r| r.pier_label == "C1Y1" && r.story == "L20")
            .expect("C1Y1 at L20 should be in output");

        // Acv check
        // qty_area_in2 converts 6336 in² back to ft²: 6336/144 = 44.0 ft²
        assert!(
            (r.acv.value - 44.0).abs() < 0.01,
            "Acv = {:.3} ft², expected 44.0 ft²",
            r.acv.value
        );
        assert_eq!(r.acv.unit, "ft²");

        // fc_ksi check
        assert!(
            (r.fc_ksi - 8.0).abs() < 1e-9,
            "fc_ksi = {:.3}, expected 8.0",
            r.fc_ksi
        );

        // ϕVn check: 0.75 × 2083.9 ≈ 1562.9 kip, displayed in kip
        assert!(
            (r.phi_vn.value - 1562.9).abs() < 1.5,
            "ϕVn = {:.1} kip, expected ≈1562.9 kip",
            r.phi_vn.value
        );
        assert_eq!(r.phi_vn.unit, "kip");

        // DCR check: 159.533 / 1562.9 ≈ 0.102
        assert!(
            (r.dcr - 0.102).abs() < 0.005,
            "DCR = {:.4}, expected ≈0.102",
            r.dcr
        );
        assert!(r.pass, "C1Y1/L20 should pass at DCR ≈0.102");
    }
}
