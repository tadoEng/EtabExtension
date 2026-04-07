use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{required_f64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct PierForceRow {
    pub story: String,
    pub pier: String,
    pub output_case: String,
    pub case_type: String,
    pub step_type: String,
    pub location: String,
    pub axial_p_kip: f64,
    pub shear_v2_kip: f64,
    pub shear_v2_abs_kip: f64,
    pub shear_v3_kip: f64,
    pub torsion_t_kip_ft: f64,
    pub moment_m2_kip_ft: f64,
    pub moment_m3_kip_ft: f64,
}

pub fn load_pier_forces(results_dir: &Path) -> Result<Vec<PierForceRow>> {
    let path = results_dir.join("pier_forces.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open pier forces: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?;
    let piers = df.column("Pier")?;
    let output_cases = df.column("OutputCase")?;
    let case_types = df.column("CaseType")?;
    let step_types = df.column("StepType")?;
    let locations = df.column("Location")?;
    let axial = df.column("P")?;
    let shear_v2 = df.column("V2")?;
    let shear_v3 = df.column("V3")?;
    let torsion = df.column("T")?;
    let moment_m2 = df.column("M2")?;
    let moment_m3 = df.column("M3")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let shear_v2_kip = required_f64(shear_v2, idx, "V2")?;
        rows.push(PierForceRow {
            story: required_string(stories, idx, "Story")?,
            pier: required_string(piers, idx, "Pier")?,
            output_case: required_string(output_cases, idx, "OutputCase")?,
            case_type: required_string(case_types, idx, "CaseType")?,
            step_type: required_string(step_types, idx, "StepType")?,
            location: required_string(locations, idx, "Location")?,
            axial_p_kip: required_f64(axial, idx, "P")?,
            shear_v2_kip,
            shear_v2_abs_kip: shear_v2_kip.abs(),
            shear_v3_kip: required_f64(shear_v3, idx, "V3")?,
            torsion_t_kip_ft: required_f64(torsion, idx, "T")?,
            moment_m2_kip_ft: required_f64(moment_m2, idx, "M2")?,
            moment_m3_kip_ft: required_f64(moment_m3, idx, "M3")?,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_pier_forces;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_pier_forces_derives_absolute_in_plane_shear() {
        let rows = load_pier_forces(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row.shear_v2_abs_kip == row.shear_v2_kip.abs())
        );
        assert!(rows.iter().any(|row| row.shear_v2_abs_kip > 0.0));
        assert!(rows.iter().all(|row| !row.location.is_empty()));
    }
}
