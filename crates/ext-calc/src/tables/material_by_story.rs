use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

use super::{optional_f64, required_f64, required_string};

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialByStoryRow {
    pub story: String,
    pub object_type: String,
    pub material: String,
    pub weight_kip: f64,
    pub floor_area_ft2: f64,
    pub unit_weight: f64,
    pub num_pieces: f64,
    pub num_studs: f64,
}

pub fn load_material_by_story(results_dir: &Path) -> Result<Vec<MaterialByStoryRow>> {
    let path = results_dir.join("material_list_by_story.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open material-by-story data: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?;
    let object_types = df.column("ObjectType")?;
    let materials = df.column("Material")?;
    let weights = df.column("Weight")?;
    let floor_areas = df.column("FloorArea")?;
    let unit_weights = df.column("UnitWeight")?;
    let num_pieces = df.column("NumPieces")?;
    let num_studs = df.column("NumStuds")?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        rows.push(MaterialByStoryRow {
            story: required_string(stories, idx, "Story")?,
            object_type: required_string(object_types, idx, "ObjectType")?,
            material: required_string(materials, idx, "Material")?,
            weight_kip: required_f64(weights, idx, "Weight")?,
            floor_area_ft2: required_f64(floor_areas, idx, "FloorArea")?,
            unit_weight: required_f64(unit_weights, idx, "UnitWeight")?,
            num_pieces: optional_f64(num_pieces, idx, "NumPieces")?.unwrap_or(0.0),
            num_studs: optional_f64(num_studs, idx, "NumStuds")?.unwrap_or(0.0),
        });
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_material_by_story;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_material_by_story_reads_weight_breakdown_rows() {
        let rows = load_material_by_story(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| !row.material.is_empty()));
        assert!(rows.iter().all(|row| row.weight_kip.is_finite()));
        assert!(rows.iter().any(|row| row.weight_kip > 0.0));
    }
}
