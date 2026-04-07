use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{optional_f64, required_f64, required_i64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct JointDriftRow {
    pub story: String,
    pub label: i64,
    pub unique_name: String,
    pub output_case: String,
    pub case_type: String,
    pub step_type: String,
    pub step_number: Option<f64>,
    pub disp_x_ft: f64,
    pub disp_y_ft: f64,
    pub drift_x: f64,
    pub drift_y: f64,
}

pub fn load_joint_drifts(results_dir: &Path) -> Result<Vec<JointDriftRow>> {
    let path = results_dir.join("joint_drifts.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open joint drifts: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?;
    let labels = df.column("Label")?;
    let unique_names = df.column("UniqueName")?;
    let output_cases = df.column("OutputCase")?;
    let case_types = df.column("CaseType")?;
    let step_types = df.column("StepType")?;
    let step_numbers = df.column("StepNumber")?;
    let disp_x = df.column("DispX")?;
    let disp_y = df.column("DispY")?;
    let drift_x = df.column("DriftX")?;
    let drift_y = df.column("DriftY")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        rows.push(JointDriftRow {
            story: required_string(stories, idx, "Story")?,
            label: required_i64(labels, idx, "Label")?,
            unique_name: required_string(unique_names, idx, "UniqueName")?,
            output_case: required_string(output_cases, idx, "OutputCase")?,
            case_type: required_string(case_types, idx, "CaseType")?,
            step_type: required_string(step_types, idx, "StepType")?,
            step_number: optional_f64(step_numbers, idx, "StepNumber")?,
            disp_x_ft: required_f64(disp_x, idx, "DispX")?,
            disp_y_ft: required_f64(disp_y, idx, "DispY")?,
            drift_x: required_f64(drift_x, idx, "DriftX")?,
            drift_y: required_f64(drift_y, idx, "DriftY")?,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_joint_drifts;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_joint_drifts_reads_ratios_without_recomputing() {
        let rows = load_joint_drifts(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| !row.unique_name.is_empty()));
        assert!(
            rows.iter()
                .all(|row| row.drift_x.is_finite() && row.drift_y.is_finite())
        );
        assert!(
            rows.iter()
                .any(|row| row.drift_x.abs() > 0.0 || row.drift_y.abs() > 0.0)
        );
    }
}
