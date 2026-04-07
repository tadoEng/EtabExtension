use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::chart_build::{build_chart, build_report_charts};
use crate::chart_types::{ChartSpec, RenderConfig, RenderedAsset, RenderedCharts};

#[cfg(feature = "ssr")]
use charming::ImageRenderer;

#[cfg(feature = "ssr")]
pub fn render_svg(spec: &ChartSpec) -> Result<String> {
    ImageRenderer::new(spec.width, spec.height)
        .render(&build_chart(spec))
        .context("charming SVG render failed")
}

#[cfg(feature = "ssr")]
pub fn render_all_svg(
    calc: &ext_calc::output::CalcOutput,
    config: &RenderConfig,
) -> Result<RenderedCharts> {
    let assets = build_report_charts(calc, config)
        .into_iter()
        .map(|entry| {
            Ok(RenderedAsset {
                logical_name: entry.logical_name,
                caption: entry.caption,
                svg: render_svg(&entry.spec)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(RenderedCharts { assets })
}

pub fn write_svg_assets(rendered: &RenderedCharts, output_dir: &Path) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create asset dir {}", output_dir.display()))?;

    rendered
        .assets
        .iter()
        .map(|asset| {
            let file_name = Path::new(&asset.logical_name)
                .file_name()
                .with_context(|| format!("Invalid logical image name '{}'", asset.logical_name))?;
            let path = output_dir.join(file_name);
            fs::write(&path, &asset.svg)
                .with_context(|| format!("Failed to write {}", path.display()))?;
            Ok(path)
        })
        .collect()
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use crate::chart_build::{BASE_SHEAR_IMAGE, DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE, MODAL_IMAGE};
    use crate::chart_types::RenderConfig;
    use crate::render_svg::render_all_svg;
    use ext_calc::output::CalcOutput;
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn render_report_svgs_returns_expected_assets() {
        let calc = fixture_calc_output();
        let rendered = render_all_svg(&calc, &RenderConfig::default()).unwrap();
        let names = rendered
            .assets
            .iter()
            .map(|asset| asset.logical_name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&MODAL_IMAGE));
        assert!(names.contains(&BASE_SHEAR_IMAGE));
        assert!(names.contains(&DRIFT_WIND_IMAGE));
        assert!(names.contains(&DRIFT_SEISMIC_IMAGE));
        assert!(rendered.assets.iter().all(|asset| asset.svg.contains("<svg")));
    }
}
