use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct StoryDefRow {
    pub story: String,
    pub height_ft: f64,
    pub elevation_ft: f64,
}

pub fn load_story_definitions(results_dir: &Path) -> Result<Vec<StoryDefRow>> {
    let path = results_dir.join("story_definitions.parquet");
    let file = File::open(&path)
        .with_context(|| format!("Failed to open story definitions: {}", path.display()))?;
    let df = ParquetReader::new(file).finish()?;

    let stories = df.column("Story")?.str()?;
    let heights = df.column("Height")?.f64()?;

    let mut rows = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let story = stories
            .get(idx)
            .with_context(|| format!("Missing Story at row {idx}"))?;
        let height_ft = heights
            .get(idx)
            .with_context(|| format!("Missing Height at row {idx}"))?;
        rows.push(StoryDefRow {
            story: story.to_string(),
            height_ft,
            elevation_ft: 0.0,
        });
    }

    let mut cumulative = 0.0;
    for row in rows.iter_mut().rev() {
        cumulative += row.height_ft;
        row.elevation_ft = cumulative;
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::load_story_definitions;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    #[test]
    fn load_story_definitions_computes_elevations() {
        let rows = load_story_definitions(&fixture_dir()).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| row.height_ft > 0.0));
        assert!(rows.iter().all(|row| row.elevation_ft > 0.0));
        assert!(rows.first().unwrap().elevation_ft > rows.last().unwrap().elevation_ft);
    }
}
