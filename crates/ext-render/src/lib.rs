use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use ext_calc::output::{CalcOutput, DriftOutput};

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 900,
            height: 680,
        }
    }
}

pub fn render_drift_svgs(
    calc: &CalcOutput,
    output_dir: &Path,
    config: &RenderConfig,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)?;
    let mut written = Vec::new();

    if let Some(drift) = calc.drift_wind.as_ref() {
        written.push(write_drift_svg(
            drift,
            output_dir.join("drift_wind.svg"),
            "Story Drift - Wind",
            config,
        )?);
    }

    if let Some(drift) = calc.drift_seismic.as_ref() {
        written.push(write_drift_svg(
            drift,
            output_dir.join("drift_seismic.svg"),
            "Story Drift - Seismic",
            config,
        )?);
    }

    Ok(written)
}

fn write_drift_svg(
    drift: &DriftOutput,
    path: PathBuf,
    title: &str,
    config: &RenderConfig,
) -> Result<PathBuf> {
    let max_ratio = drift
        .stories
        .iter()
        .map(|story| story.drift_ratio)
        .fold(drift.allowable_ratio, f64::max)
        .max(1e-6);

    let count = drift.stories.len().max(1) as f64;
    let mut points = Vec::new();
    for (idx, row) in drift.stories.iter().enumerate() {
        let x = 60.0 + (row.drift_ratio / max_ratio) * ((config.width - 120) as f64);
        let y = 40.0 + (idx as f64 / count) * ((config.height - 80) as f64);
        points.push(format!("{x:.1},{y:.1}"));
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="100%" height="100%" fill="#faf9f7"/>
<text x="24" y="28" font-size="20" font-family="Segoe UI, Arial, sans-serif" fill="#222">{title}</text>
<line x1="60" y1="40" x2="60" y2="{axis_y}" stroke="#888" stroke-width="1"/>
<line x1="60" y1="{axis_y}" x2="{axis_x}" y2="{axis_y}" stroke="#888" stroke-width="1"/>
<line x1="{limit_x}" y1="40" x2="{limit_x}" y2="{axis_y}" stroke="#b92626" stroke-width="2" stroke-dasharray="6 6"/>
<polyline fill="none" stroke="#3a64aa" stroke-width="3" points="{points}"/>
<text x="{limit_label_x}" y="56" font-size="12" font-family="Segoe UI, Arial, sans-serif" fill="#b92626">limit</text>
</svg>"##,
        w = config.width,
        h = config.height,
        axis_y = config.height - 40,
        axis_x = config.width - 40,
        limit_x = 60.0 + (drift.allowable_ratio / max_ratio) * ((config.width - 120) as f64),
        limit_label_x = 66.0 + (drift.allowable_ratio / max_ratio) * ((config.width - 120) as f64),
        points = points.join(" ")
    );

    fs::write(&path, svg)?;
    Ok(path)
}
