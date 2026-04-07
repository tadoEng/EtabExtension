use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{required_f64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct PierSectionRow {
    pub story: String,
    pub pier: String,
    pub axis_angle: f64,
    pub width_bot_ft: f64,
    pub thick_bot_ft: f64,
    pub width_top_ft: f64,
    pub thick_top_ft: f64,
    pub material: String,
    pub acv_in2: f64,
    pub ag_in2: f64,
}

pub fn load_pier_sections(results_dir: &Path) -> Result<Vec<PierSectionRow>> {
    let path = results_dir.join("pier_section_properties.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open pier sections: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?;
    let piers = df.column("Pier")?;
    let axis_angles = df.column("AxisAngle")?;
    let width_bot = df.column("WidthBot")?;
    let thick_bot = df.column("ThickBot")?;
    let width_top = df.column("WidthTop")?;
    let thick_top = df.column("ThickTop")?;
    let materials = df.column("Material")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let width_bot_ft = required_f64(width_bot, idx, "WidthBot")?;
        let thick_bot_ft = required_f64(thick_bot, idx, "ThickBot")?;
        let acv_in2 = width_bot_ft * thick_bot_ft * 144.0;
        rows.push(PierSectionRow {
            story: required_string(stories, idx, "Story")?,
            pier: required_string(piers, idx, "Pier")?,
            axis_angle: required_f64(axis_angles, idx, "AxisAngle")?,
            width_bot_ft,
            thick_bot_ft,
            width_top_ft: required_f64(width_top, idx, "WidthTop")?,
            thick_top_ft: required_f64(thick_top, idx, "ThickTop")?,
            material: required_string(materials, idx, "Material")?,
            acv_in2,
            ag_in2: acv_in2,
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_pier_sections;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_pier_sections_derives_acv_and_ag() {
        let rows = load_pier_sections(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        let sample = rows.first().unwrap();
        assert!((sample.acv_in2 - sample.width_bot_ft * sample.thick_bot_ft * 144.0).abs() < 1e-6);
        assert_eq!(sample.acv_in2, sample.ag_in2);
    }
}
