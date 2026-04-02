use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{required_f64, required_i64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct ModalParticipationRow {
    pub case_name: String,
    pub mode: i64,
    pub period_sec: f64,
    pub ux: f64,
    pub uy: f64,
    pub uz: f64,
    pub sum_ux: f64,
    pub sum_uy: f64,
    pub rx: f64,
    pub ry: f64,
    pub rz: f64,
    pub sum_rx: f64,
    pub sum_ry: f64,
    pub sum_rz: f64,
}

pub fn load_modal_participating_mass_ratios(
    results_dir: &Path,
) -> Result<Vec<ModalParticipationRow>> {
    let path = results_dir.join("modal_participating_mass_ratios.parquet");
    let file = File::open(&path).with_context(|| {
        format!(
            "Failed to open modal participating mass ratios: {}",
            path.display()
        )
    })?;
    let df = ParquetReader::new(file).finish()?;

    let cases = df.column("Case")?;
    let modes = df.column("Mode")?;
    let periods = df.column("Period")?;
    let ux = df.column("UX")?;
    let uy = df.column("UY")?;
    let uz = df.column("UZ")?;
    let sum_ux = df.column("SumUX")?;
    let sum_uy = df.column("SumUY")?;
    let rx = df.column("RX")?;
    let ry = df.column("RY")?;
    let rz = df.column("RZ")?;
    let sum_rx = df.column("SumRX")?;
    let sum_ry = df.column("SumRY")?;
    let sum_rz = df.column("SumRZ")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        rows.push(ModalParticipationRow {
            case_name: required_string(cases, idx, "Case")?,
            mode: required_i64(modes, idx, "Mode")?,
            period_sec: required_f64(periods, idx, "Period")?,
            ux: required_f64(ux, idx, "UX")?,
            uy: required_f64(uy, idx, "UY")?,
            uz: required_f64(uz, idx, "UZ")?,
            sum_ux: required_f64(sum_ux, idx, "SumUX")?,
            sum_uy: required_f64(sum_uy, idx, "SumUY")?,
            rx: required_f64(rx, idx, "RX")?,
            ry: required_f64(ry, idx, "RY")?,
            rz: required_f64(rz, idx, "RZ")?,
            sum_rx: required_f64(sum_rx, idx, "SumRX")?,
            sum_ry: required_f64(sum_ry, idx, "SumRY")?,
            sum_rz: required_f64(sum_rz, idx, "SumRZ")?,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_modal_participating_mass_ratios;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_modal_participating_mass_ratios_reads_cumulative_columns() {
        let rows = load_modal_participating_mass_ratios(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| row.mode > 0));
        assert!(rows.iter().all(|row| row.period_sec > 0.0));
        assert!(
            rows.iter()
                .any(|row| row.sum_ux >= row.ux && row.sum_uy >= row.uy)
        );
    }
}
