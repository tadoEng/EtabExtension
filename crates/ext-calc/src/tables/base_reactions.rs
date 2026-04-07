use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{optional_f64, required_f64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct BaseReactionRow {
    pub output_case: String,
    pub case_type: String,
    pub step_type: String,
    pub step_number: Option<f64>,
    pub fx_kip: f64,
    pub fy_kip: f64,
    pub fz_kip: f64,
    pub mx_kip_ft: f64,
    pub my_kip_ft: f64,
    pub mz_kip_ft: f64,
    pub x_ft: f64,
    pub y_ft: f64,
    pub z_ft: f64,
}

pub fn load_base_reactions(results_dir: &Path) -> Result<Vec<BaseReactionRow>> {
    let path = results_dir.join("base_reactions.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open base reactions: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let output_cases = df.column("OutputCase")?;
    let case_types = df.column("CaseType")?;
    let step_types = df.column("StepType")?;
    let step_numbers = df.column("StepNumber")?;
    let fx = df.column("FX")?;
    let fy = df.column("FY")?;
    let fz = df.column("FZ")?;
    let mx = df.column("MX")?;
    let my = df.column("MY")?;
    let mz = df.column("MZ")?;
    let x = df.column("X")?;
    let y = df.column("Y")?;
    let z = df.column("Z")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        rows.push(BaseReactionRow {
            output_case: required_string(output_cases, idx, "OutputCase")?,
            case_type: required_string(case_types, idx, "CaseType")?,
            step_type: required_string(step_types, idx, "StepType")?,
            step_number: optional_f64(step_numbers, idx, "StepNumber")?,
            fx_kip: required_f64(fx, idx, "FX")?,
            fy_kip: required_f64(fy, idx, "FY")?,
            fz_kip: required_f64(fz, idx, "FZ")?,
            mx_kip_ft: required_f64(mx, idx, "MX")?,
            my_kip_ft: required_f64(my, idx, "MY")?,
            mz_kip_ft: required_f64(mz, idx, "MZ")?,
            x_ft: required_f64(x, idx, "X")?,
            y_ft: required_f64(y, idx, "Y")?,
            z_ft: required_f64(z, idx, "Z")?,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_base_reactions;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_base_reactions_reads_force_components() {
        let rows = load_base_reactions(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row.fx_kip.is_finite() && row.fy_kip.is_finite())
        );
        assert!(
            rows.iter()
                .any(|row| row.fx_kip.abs() > 0.0 || row.fy_kip.abs() > 0.0)
        );
        assert!(rows.iter().all(|row| !row.output_case.is_empty()));
    }
}
