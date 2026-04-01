use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialProp {
    pub name: String,
    pub fc_kipsft2: f64,
    pub fc_ksi: f64,
    pub fc_psi: f64,
    pub is_lightweight: bool,
}

pub fn load_material_properties(results_dir: &Path) -> Result<HashMap<String, MaterialProp>> {
    let path = results_dir.join("material_properties_concrete_data.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open material properties: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let names = df.column("Material")?.str()?;
    let fc = df.column("Fc")?.f64()?;
    let lightweight = df.column("LtWtConc")?.str()?;

    let mut map = HashMap::with_capacity(df.height());
    for idx in 0..df.height() {
        let name = names
            .get(idx)
            .with_context(|| format!("Missing Material at row {idx}"))?;
        let fc_kipsft2 = fc
            .get(idx)
            .with_context(|| format!("Missing Fc at row {idx}"))?;
        let is_lightweight = matches!(lightweight.get(idx), Some("Yes"));
        let fc_ksi = fc_kipsft2 / 144.0;

        map.insert(
            name.to_string(),
            MaterialProp {
                name: name.to_string(),
                fc_kipsft2,
                fc_ksi,
                fc_psi: fc_ksi * 1000.0,
                is_lightweight,
            },
        );
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_material_properties;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_material_properties_derives_ksi_and_psi() {
        let rows = load_material_properties(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        let sample = rows.values().next().unwrap();
        assert!((sample.fc_psi - sample.fc_ksi * 1000.0).abs() < 1e-6);
        assert!(sample.fc_ksi > 0.0);
    }
}
