use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use charming::{
    Chart,
    component::{Axis, Grid, Legend, Title},
    element::{AxisType, Tooltip, Trigger},
    series::{Bar, Line, Pie},
};
use ext_calc::output::{
    BaseShearOutput, CalcOutput, DisplacementOutput, DriftOutput, ModalOutput, PierAxialOutput,
    PierShearOutput,
};

#[cfg(feature = "ssr")]
use charming::ImageRenderer;

pub const MODAL_IMAGE: &str = "images/modal.svg";
pub const BASE_SHEAR_IMAGE: &str = "images/base_shear.svg";
pub const DRIFT_WIND_IMAGE: &str = "images/drift_wind.svg";
pub const DRIFT_SEISMIC_IMAGE: &str = "images/drift_seismic.svg";
pub const DISPLACEMENT_WIND_IMAGE: &str = "images/displacement_wind.svg";
pub const PIER_SHEAR_WIND_IMAGE: &str = "images/pier_shear_wind.svg";
pub const PIER_SHEAR_SEISMIC_IMAGE: &str = "images/pier_shear_seismic.svg";
pub const PIER_AXIAL_IMAGE: &str = "images/pier_axial.svg";

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 900,
            height: 620,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartSpec {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub kind: ChartKind,
}

#[derive(Debug, Clone)]
pub enum ChartKind {
    Cartesian {
        categories: Vec<String>,
        series: Vec<CartesianSeries>,
    },
    Pie {
        data: Vec<(f64, String)>,
    },
}

#[derive(Debug, Clone)]
pub struct CartesianSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub kind: SeriesType,
}

#[derive(Debug, Clone, Copy)]
pub enum SeriesType {
    Bar,
    Line,
}

#[derive(Debug, Clone)]
pub struct NamedChartSpec {
    pub logical_name: String,
    pub caption: String,
    pub spec: ChartSpec,
}

#[derive(Debug, Clone)]
pub struct RenderedAsset {
    pub logical_name: String,
    pub caption: String,
    pub svg: String,
}

#[derive(Debug, Clone, Default)]
pub struct RenderedCharts {
    pub assets: Vec<RenderedAsset>,
}

pub fn build_report_charts(calc: &CalcOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    let mut charts = Vec::new();

    if let Some(modal) = calc.modal.as_ref() {
        charts.push(build_modal_chart(modal, config));
    }

    if let Some(base_shear) = calc.base_shear.as_ref() {
        charts.push(build_base_shear_chart(base_shear, config));
    }

    if let Some(drift) = calc.drift_wind.as_ref() {
        charts.push(build_drift_chart(
            DRIFT_WIND_IMAGE,
            "Wind Drift Envelope",
            "Maximum drift ratio per story under wind loading.",
            drift,
            config,
        ));
    }

    if let Some(drift) = calc.drift_seismic.as_ref() {
        charts.push(build_drift_chart(
            DRIFT_SEISMIC_IMAGE,
            "Seismic Drift Envelope",
            "Maximum drift ratio per story under seismic loading.",
            drift,
            config,
        ));
    }

    if let Some(displacement) = calc.displacement_wind.as_ref() {
        charts.push(build_displacement_chart(displacement, config));
    }

    if let Some(output) = calc.pier_shear_wind.as_ref() {
        charts.push(build_pier_shear_chart(
            PIER_SHEAR_WIND_IMAGE,
            "Pier Shear Wind DCR",
            "Top governing pier shear DCR values for wind combinations.",
            output,
            config,
        ));
    }

    if let Some(output) = calc.pier_shear_seismic.as_ref() {
        charts.push(build_pier_shear_chart(
            PIER_SHEAR_SEISMIC_IMAGE,
            "Pier Shear Seismic DCR",
            "Top governing pier shear DCR values for seismic combinations.",
            output,
            config,
        ));
    }

    if let Some(output) = calc.pier_axial.as_ref() {
        charts.push(build_pier_axial_chart(output, config));
    }

    charts
}

pub fn build_chart(spec: &ChartSpec) -> Chart {
    match &spec.kind {
        ChartKind::Cartesian { categories, series } => build_cartesian(spec, categories, series),
        ChartKind::Pie { data } => build_pie(spec, data),
    }
}

fn build_cartesian(spec: &ChartSpec, categories: &[String], series: &[CartesianSeries]) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(spec.title.as_str()).left("center"))
        .grid(
            Grid::new()
                .left("10%")
                .right("6%")
                .top("16%")
                .bottom("16%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().top("6%"))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(categories.iter().map(String::as_str).collect::<Vec<_>>()),
        )
        .y_axis(Axis::new().type_(AxisType::Value));

    for entry in series {
        chart = match entry.kind {
            SeriesType::Bar => chart.series(
                Bar::new()
                    .name(entry.name.as_str())
                    .data(entry.data.clone()),
            ),
            SeriesType::Line => chart.series(
                Line::new()
                    .name(entry.name.as_str())
                    .smooth(true)
                    .data(entry.data.clone()),
            ),
        };
    }

    chart
}

fn build_pie(spec: &ChartSpec, data: &[(f64, String)]) -> Chart {
    Chart::new()
        .title(Title::new().text(spec.title.as_str()).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .legend(Legend::new().bottom("3%").left("center"))
        .series(
            Pie::new()
                .name(spec.title.as_str())
                .radius(vec!["35%", "65%"])
                .center(vec!["50%", "48%"])
                .data(
                    data.iter()
                        .map(|(value, label)| (*value, label.as_str()))
                        .collect::<Vec<_>>(),
                ),
        )
}

#[cfg(feature = "ssr")]
pub fn render_svg(spec: &ChartSpec) -> Result<String> {
    ImageRenderer::new(spec.width, spec.height)
        .render(&build_chart(spec))
        .context("charming SVG render failed")
}

#[cfg(feature = "ssr")]
pub fn render_report_svgs(calc: &CalcOutput, config: &RenderConfig) -> Result<RenderedCharts> {
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

pub fn render_html(spec: &ChartSpec, chart_id: &str) -> Result<String> {
    charming::HtmlRenderer::new(chart_id, spec.width as u64, spec.height as u64)
        .render(&build_chart(spec))
        .context("charming HTML render failed")
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

fn build_modal_chart(modal: &ModalOutput, config: &RenderConfig) -> NamedChartSpec {
    NamedChartSpec {
        logical_name: MODAL_IMAGE.to_string(),
        caption: "Cumulative modal mass participation in the principal directions.".to_string(),
        spec: ChartSpec {
            title: "Modal Participation".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: modal
                    .rows
                    .iter()
                    .map(|row| format!("Mode {}", row.mode))
                    .collect(),
                series: vec![
                    CartesianSeries {
                        name: "Sum UX".to_string(),
                        data: modal.rows.iter().map(|row| row.sum_ux).collect(),
                        kind: SeriesType::Line,
                    },
                    CartesianSeries {
                        name: "Sum UY".to_string(),
                        data: modal.rows.iter().map(|row| row.sum_uy).collect(),
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}

fn build_base_shear_chart(base_shear: &BaseShearOutput, config: &RenderConfig) -> NamedChartSpec {
    NamedChartSpec {
        logical_name: BASE_SHEAR_IMAGE.to_string(),
        caption: "Equivalent static and response spectrum base shear comparison.".to_string(),
        spec: ChartSpec {
            title: "Base Shear Comparison".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: vec!["X".to_string(), "Y".to_string()],
                series: vec![
                    CartesianSeries {
                        name: "RSA".to_string(),
                        data: vec![
                            base_shear.direction_x.v_rsa.value,
                            base_shear.direction_y.v_rsa.value,
                        ],
                        kind: SeriesType::Bar,
                    },
                    CartesianSeries {
                        name: "ELF".to_string(),
                        data: vec![
                            base_shear.direction_x.v_elf.value,
                            base_shear.direction_y.v_elf.value,
                        ],
                        kind: SeriesType::Bar,
                    },
                    CartesianSeries {
                        name: "Ratio".to_string(),
                        data: vec![base_shear.direction_x.ratio, base_shear.direction_y.ratio],
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}

fn build_drift_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    drift: &DriftOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let story_values = aggregate_story_max(drift.rows.iter().map(|row| {
        let value = [
            row.max_drift_x_pos.abs(),
            row.max_drift_x_neg.abs(),
            row.max_drift_y_pos.abs(),
            row.max_drift_y_neg.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        (row.story.clone(), value)
    }));

    let categories = story_values.iter().map(|(story, _)| story.clone()).collect();
    let values = story_values.iter().map(|(_, value)| *value).collect();
    let limits = vec![drift.allowable_ratio; story_values.len()];

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories,
                series: vec![
                    CartesianSeries {
                        name: "Demand".to_string(),
                        data: values,
                        kind: SeriesType::Line,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: limits,
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}

fn build_displacement_chart(
    displacement: &DisplacementOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let story_values = aggregate_story_max(displacement.rows.iter().map(|row| {
        let value = [
            row.max_disp_x_pos_ft.abs(),
            row.max_disp_x_neg_ft.abs(),
            row.max_disp_y_pos_ft.abs(),
            row.max_disp_y_neg_ft.abs(),
        ]
        .into_iter()
        .fold(0.0_f64, f64::max);
        (row.story.clone(), value)
    }));

    NamedChartSpec {
        logical_name: DISPLACEMENT_WIND_IMAGE.to_string(),
        caption: "Maximum roof and story displacement demand under wind loading.".to_string(),
        spec: ChartSpec {
            title: "Wind Displacement Envelope".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: story_values.iter().map(|(story, _)| story.clone()).collect(),
                series: vec![
                    CartesianSeries {
                        name: "Demand".to_string(),
                        data: story_values.iter().map(|(_, value)| *value).collect(),
                        kind: SeriesType::Line,
                    },
                    CartesianSeries {
                        name: "Limit".to_string(),
                        data: vec![displacement.disp_limit.value; story_values.len()],
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}

fn build_pier_shear_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    let governing = top_pier_values(
        output
            .piers
            .iter()
            .map(|row| (format!("{} {}", row.story, row.pier_label), row.dcr)),
    );

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: governing.iter().map(|(label, _)| label.clone()).collect(),
                series: vec![CartesianSeries {
                    name: "DCR".to_string(),
                    data: governing.iter().map(|(_, value)| *value).collect(),
                    kind: SeriesType::Bar,
                }],
            },
        },
    }
}

fn build_pier_axial_chart(output: &PierAxialOutput, config: &RenderConfig) -> NamedChartSpec {
    let governing = top_pier_values(
        output
            .piers
            .iter()
            .map(|row| (format!("{} {}", row.story, row.pier_label), row.dcr)),
    );

    NamedChartSpec {
        logical_name: PIER_AXIAL_IMAGE.to_string(),
        caption: "Top governing pier axial demand-capacity ratios.".to_string(),
        spec: ChartSpec {
            title: "Pier Axial DCR".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: governing.iter().map(|(label, _)| label.clone()).collect(),
                series: vec![CartesianSeries {
                    name: "DCR".to_string(),
                    data: governing.iter().map(|(_, value)| *value).collect(),
                    kind: SeriesType::Bar,
                }],
            },
        },
    }
}

fn aggregate_story_max(iter: impl Iterator<Item = (String, f64)>) -> Vec<(String, f64)> {
    let mut values: Vec<(String, f64)> = Vec::new();

    for (story, value) in iter {
        if let Some((_, existing)) = values.iter_mut().find(|(name, _)| *name == story) {
            *existing = existing.max(value);
        } else {
            values.push((story, value));
        }
    }

    values
}

fn top_pier_values(iter: impl Iterator<Item = (String, f64)>) -> Vec<(String, f64)> {
    let mut values = iter.collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.total_cmp(&left.1));
    values.truncate(8);
    values.reverse();
    values
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::{
        BASE_SHEAR_IMAGE, DRIFT_SEISMIC_IMAGE, DRIFT_WIND_IMAGE, MODAL_IMAGE, RenderConfig,
        render_report_svgs,
    };
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
        let rendered = render_report_svgs(&calc, &RenderConfig::default()).unwrap();
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
