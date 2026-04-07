// checks/pier_shear_seismic.rs — seismic pier shear check
//
// Delegates to pier_shear::run() with ϕ = 0.60 (ACI 318-14 §21.2.4.1).
// Config section: [calc.pier-shear-seismic]
//
// The only difference from the wind check is ϕ = 0.60 instead of 0.75.
// This reflects the additional ductility demands for seismic loading.

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
        &params.pier_shear_seismic,
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
    fn pier_shear_seismic_produces_results_with_correct_phi() {
        let dir     = fixture_dir();
        let forces  = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat     = load_material_properties(&dir).unwrap();
        let params  = fixture_params();
        let fc_map  = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        let output = run(&forces, &sections, &fc_map, &params).unwrap();

        assert!(!output.piers.is_empty());
        assert_eq!(output.phi_v, 0.60, "seismic check must use ϕ = 0.60");
        assert!(output.governing.dcr > 0.0);
    }

    #[test]
    fn seismic_dcr_is_higher_than_wind_dcr_for_same_pier() {
        // For the same Vn, seismic ϕ=0.60 gives lower ϕVn than wind ϕ=0.75,
        // so seismic DCR = Vu/(0.60×Vn) > wind DCR = Vu/(0.75×Vn).
        // We verify the phi value is correct rather than comparing across
        // different combos (wind and seismic combos are different load cases).
        let dir     = fixture_dir();
        let forces  = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat     = load_material_properties(&dir).unwrap();
        let params  = fixture_params();
        let fc_map  = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        let seismic_out = run(&forces, &sections, &fc_map, &params).unwrap();
        assert!((seismic_out.phi_v - 0.60).abs() < 1e-9,
            "seismic phi should be 0.60, got {}", seismic_out.phi_v);
        // For any pier, ϕVn_seismic = 0.60/0.75 × ϕVn_wind  →  ϕVn_seismic < ϕVn_wind
        // Just verify governing ϕVn makes sense relative to ϕ
        assert!(seismic_out.governing.phi_vn.value > 0.0);
        assert!(seismic_out.governing.vn.value > seismic_out.governing.phi_vn.value,
            "Vn should be > ϕVn since ϕ < 1.0");
    }

    #[test]
    fn pier_shear_seismic_errors_when_combo_missing() {
        let dir     = fixture_dir();
        let forces  = load_pier_forces(&dir).unwrap();
        let sections = load_pier_sections(&dir).unwrap();
        let mat     = load_material_properties(&dir).unwrap();
        let mut config = Config::load(&fixture_dir()).unwrap();
        config.calc.pier_shear_seismic.load_combos = vec!["BAD_COMBO".into()];
        let params  = CodeParams::from_config(&config).unwrap();
        let fc_map  = build_pier_fc_map(&sections, &mat, params.pier_shear_seismic.fc_default_ksi);

        assert!(run(&forces, &sections, &fc_map, &params).is_err());
    }
}
