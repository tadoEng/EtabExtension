use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

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

    let stories = df.column("Story")?.str()?;
    let piers = df.column("Pier")?.str()?;
    let axis_angles = df.column("AxisAngle")?.f64()?;
    let width_bot = df.column("WidthBot")?.f64()?;
    let thick_bot = df.column("ThickBot")?.f64()?;
    let width_top = df.column("WidthTop")?.f64()?;
    let thick_top = df.column("ThickTop")?.f64()?;
    let materials = df.column("Material")?.str()?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let width_bot_ft = width_bot
            .get(idx)
            .with_context(|| format!("Missing WidthBot at row {idx}"))?;
        let thick_bot_ft = thick_bot
            .get(idx)
            .with_context(|| format!("Missing ThickBot at row {idx}"))?;
        let acv_in2 = width_bot_ft * thick_bot_ft * 144.0;
        rows.push(PierSectionRow {
            story: stories
                .get(idx)
                .with_context(|| format!("Missing Story at row {idx}"))?
                .to_string(),
            pier: piers
                .get(idx)
                .with_context(|| format!("Missing Pier at row {idx}"))?
                .to_string(),
            axis_angle: axis_angles
                .get(idx)
                .with_context(|| format!("Missing AxisAngle at row {idx}"))?,
            width_bot_ft,
            thick_bot_ft,
            width_top_ft: width_top
                .get(idx)
                .with_context(|| format!("Missing WidthTop at row {idx}"))?,
            thick_top_ft: thick_top
                .get(idx)
                .with_context(|| format!("Missing ThickTop at row {idx}"))?,
            material: materials
                .get(idx)
                .with_context(|| format!("Missing Material at row {idx}"))?
                .to_string(),
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
