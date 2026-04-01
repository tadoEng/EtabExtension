use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

pub fn load_group_assignments(results_dir: &Path) -> Result<HashMap<String, Vec<String>>> {
    let path = results_dir.join("group_assignments.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open group assignments: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let names = df.column("GroupName")?.str()?;
    let unique_names = df.column("UniqueName")?.str()?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for idx in 0..df.height() {
        let group = names
            .get(idx)
            .with_context(|| format!("Missing GroupName at row {idx}"))?;
        let unique_name = unique_names
            .get(idx)
            .with_context(|| format!("Missing UniqueName at row {idx}"))?;
        map.entry(group.to_string())
            .or_default()
            .push(unique_name.to_string());
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_group_assignments;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_group_assignments_collects_members() {
        let groups = load_group_assignments(&fixture_dir()).unwrap();
        assert!(!groups.is_empty());
        assert!(groups.values().any(|members| !members.is_empty()));
    }
}
