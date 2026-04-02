use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{optional_f64, required_f64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct StoryForceRow {
    pub story: String,
    pub output_case: String,
    pub case_type: String,
    pub step_type: String,
    pub step_number: Option<f64>,
    pub location: String,
    pub axial_p_kip: f64,
    pub vx_kip: f64,
    pub vy_kip: f64,
    pub torsion_t_kip_ft: f64,
    pub mx_kip_ft: f64,
    pub my_kip_ft: f64,
}

pub fn load_story_forces(results_dir: &Path) -> Result<Vec<StoryForceRow>> {
    let path = results_dir.join("story_forces.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open story forces: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?;
    let output_cases = df.column("OutputCase")?;
    let case_types = df.column("CaseType")?;
    let step_types = df.column("StepType")?;
    let step_numbers = df.column("StepNumber")?;
    let locations = df.column("Location")?;
    let axial = df.column("P")?;
    let vx = df.column("VX")?;
    let vy = df.column("VY")?;
    let torsion = df.column("T")?;
    let mx = df.column("MX")?;
    let my = df.column("MY")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        rows.push(StoryForceRow {
            story: required_string(stories, idx, "Story")?,
            output_case: required_string(output_cases, idx, "OutputCase")?,
            case_type: required_string(case_types, idx, "CaseType")?,
            step_type: required_string(step_types, idx, "StepType")?,
            step_number: optional_f64(step_numbers, idx, "StepNumber")?,
            location: required_string(locations, idx, "Location")?,
            axial_p_kip: required_f64(axial, idx, "P")?,
            vx_kip: required_f64(vx, idx, "VX")?,
            vy_kip: required_f64(vy, idx, "VY")?,
            torsion_t_kip_ft: required_f64(torsion, idx, "T")?,
            mx_kip_ft: required_f64(mx, idx, "MX")?,
            my_kip_ft: required_f64(my, idx, "MY")?,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_story_forces;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_story_forces_reads_story_shear_rows() {
        let rows = load_story_forces(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| !row.story.is_empty()));
        assert!(
            rows.iter()
                .all(|row| row.vx_kip.is_finite() && row.vy_kip.is_finite())
        );
        assert!(
            rows.iter()
                .any(|row| row.vx_kip.abs() > 0.0 || row.vy_kip.abs() > 0.0)
        );
    }
}
