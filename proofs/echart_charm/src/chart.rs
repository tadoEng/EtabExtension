// FILE: D:\repo\echart_charm\src\chart.rs
// Replace the entire file with this content.

use anyhow::{Context, Result};
use charming::{
    Chart,
    component::{Axis, Grid, Legend, Title},
    element::{AxisType, Color, ItemStyle, LineStyle, LineStyleType, Tooltip, Trigger},
    series::{Bar, Line, Pie},
};

#[cfg(feature = "ssr")]
use charming::ImageRenderer;

#[cfg(feature = "wasm")]
use charming::WasmRenderer;

// ─── Domain ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChartSpec {
    pub title:  String,
    pub width:  u32,
    pub height: u32,
    pub kind:   ChartKind,
}

#[derive(Debug, Clone)]
pub enum ChartKind {
    /// Bar/Line combo — shared x-axis (horizontal) categories, mixed series.
    Cartesian {
        x:      Vec<String>,
        series: Vec<CartesianSeries>,
    },
    /// Swapped-axis: Y = category (stories/levels), X = numeric values.
    /// Used for drift envelopes, pier shear, pier axial — reads bottom-to-top.
    SwappedCartesian {
        y_labels: Vec<String>,  // story/level names, bottom → top
        series:   Vec<CartesianSeries>,
    },
    /// Pie / donut chart
    Pie {
        data: Vec<(f64, String)>, // (value, label)
    },
}

#[derive(Debug, Clone)]
pub struct CartesianSeries {
    pub name:       String,
    pub data:       Vec<f64>,
    pub kind:       SeriesType,
    pub color:      Option<String>,   // hex string e.g. "#1f77b4"
    pub dashed:     bool,             // true → dashed line
}

impl CartesianSeries {
    /// Convenience constructor with defaults (solid, no explicit color).
    pub fn new(name: impl Into<String>, data: Vec<f64>, kind: SeriesType) -> Self {
        Self { name: name.into(), data, kind, color: None, dashed: false }
    }
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into()); self
    }
    pub fn dashed(mut self) -> Self {
        self.dashed = true; self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SeriesType { Bar, Line }

// ─── Adapter: ChartSpec → charming Chart ─────────────────────────────────────

pub fn build_chart(spec: &ChartSpec) -> Chart {
    match &spec.kind {
        ChartKind::Cartesian { x, series }              => build_cartesian(&spec.title, x, series),
        ChartKind::SwappedCartesian { y_labels, series } => build_swapped_cartesian(&spec.title, y_labels, series),
        ChartKind::Pie { data }                          => build_pie(&spec.title, data),
    }
}

fn build_cartesian(title: &str, x: &[String], series: &[CartesianSeries]) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(title).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().top("6%"))
        .grid(Grid::new().left("10%").right("6%").top("18%").bottom("14%"))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(x.iter().map(String::as_str).collect::<Vec<_>>()),
        )
        .y_axis(Axis::new().type_(AxisType::Value));

    for s in series {
        chart = add_series(chart, s, false);
    }
    chart
}

/// Swapped-axis chart: categories on Y, values on X.
/// Used for story-level plots so L01 appears at the bottom and ROOF at the top.
fn build_swapped_cartesian(title: &str, y: &[String], series: &[CartesianSeries]) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(title).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().top("6%"))
        .grid(Grid::new().left("12%").right("8%").top("18%").bottom("8%"))
        // X = value axis
        .x_axis(Axis::new().type_(AxisType::Value))
        // Y = category axis (story labels)
        .y_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(y.iter().map(String::as_str).collect::<Vec<_>>()),
        );

    for s in series {
        chart = add_series(chart, s, true /* swap */);
    }
    chart
}

/// Add a single series to a chart, handling bar vs line, color, dash style,
/// and the coordinate swap needed for SwappedCartesian.
fn add_series(chart: Chart, s: &CartesianSeries, swapped: bool) -> Chart {
    // For swapped axes charming needs [value, category] data points.
    // For normal axes just pass values directly.
    match s.kind {
        SeriesType::Bar => {
            let mut b = Bar::new().name(s.name.as_str());
            if swapped {
                // charming Bar on swapped axis: pass (value, category) tuples
                let pts: Vec<(f64, &str)> = s.data.iter().zip(
                    // We don't have the y_labels here, so we pass values only
                    // and rely on the axis order matching the series data order.
                    std::iter::repeat("").take(s.data.len())
                ).map(|(v, _)| (*v, "")).collect();
                // Simpler: just pass raw values; charming matches by index to y-axis categories.
                b = b.data(s.data.clone());
            } else {
                b = b.data(s.data.clone());
            }
            if let Some(c) = &s.color {
                b = b.item_style(ItemStyle::new().color(Color::Value(c.clone())));
            }
            chart.series(b)
        }
        SeriesType::Line => {
            let mut l = Line::new()
                .name(s.name.as_str())
                .data(s.data.clone())
                .show_symbol(!s.dashed);

            // Build line style (color + dash)
            let mut ls = LineStyle::new();
            if let Some(c) = &s.color {
                ls = ls.color(Color::Value(c.clone()));
            }
            if s.dashed {
                ls = ls.type_(LineStyleType::Dashed);
            }
            l = l.line_style(ls);

            chart.series(l)
        }
    }
}

fn build_pie(title: &str, data: &[(f64, String)]) -> Chart {
    Chart::new()
        .title(Title::new().text(title).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .legend(Legend::new().bottom("5%").left("center"))
        .series(
            Pie::new()
                .name(title)
                .radius(vec!["35%", "65%"])
                .center(vec!["50%", "48%"])
                .data(data.iter().map(|(v, l)| (*v, l.as_str())).collect::<Vec<_>>()),
        )
}

// ─── Preset chart constructors ────────────────────────────────────────────────

/// Base reaction pie chart — Dead / Live / SDL / Wind / Seismic
pub fn base_reaction_pie(dead: f64, live: f64, sdl: f64, wind: f64, seismic: f64) -> ChartSpec {
    ChartSpec {
        title:  "Base Reactions by Load Case".into(),
        width:  700,
        height: 500,
        kind:   ChartKind::Pie {
            data: vec![
                (dead,    "Dead (D)".into()),
                (live,    "Live (L)".into()),
                (sdl,     "Super. Dead (SDL)".into()),
                (wind,    "Wind (W)".into()),
                (seismic, "Seismic (E)".into()),
            ],
        },
    }
}

/// Story shear line chart — X axis = story labels, Y = shear (kips).
pub fn story_shear_chart(
    story_labels: Vec<String>,
    x_shear:      Vec<f64>,
    y_shear:      Vec<f64>,
) -> ChartSpec {
    ChartSpec {
        title:  "Story Shear — Lateral Loads".into(),
        width:  700,
        height: 450,
        kind:   ChartKind::Cartesian {
            x: story_labels,
            series: vec![
                CartesianSeries::new("Shear X (kips)", x_shear, SeriesType::Line)
                    .with_color("#1f77b4"),
                CartesianSeries::new("Shear Y (kips)", y_shear, SeriesType::Line)
                    .with_color("#ff7f0e"),
            ],
        },
    }
}

/// Force vs displacement bar+line combo.
pub fn force_displacement_chart(
    labels:       Vec<String>,
    force:        Vec<f64>,
    displacement: Vec<f64>,
) -> ChartSpec {
    ChartSpec {
        title:  "Force vs Displacement".into(),
        width:  700,
        height: 450,
        kind:   ChartKind::Cartesian {
            x: labels,
            series: vec![
                CartesianSeries::new("Force (kips)", force, SeriesType::Bar)
                    .with_color("#4c78a8"),
                CartesianSeries::new("Displacement (in)", displacement, SeriesType::Line)
                    .with_color("#e45756"),
            ],
        },
    }
}

/// Drift envelope — Y = story level, X = drift ratio.
/// Demand as a solid line; limit as a dashed red vertical reference line.
pub fn drift_envelope_chart(
    story_labels:    Vec<String>,  // L01 … L35, bottom → top
    demand_per_story: Vec<f64>,   // drift ratio per story (same order)
    allowable_ratio:  f64,
) -> ChartSpec {
    let n = story_labels.len();
    let height = (n as u32 * 18 + 100).max(500);
    ChartSpec {
        title:  "Drift Envelope".into(),
        width:  800,
        height,
        kind:   ChartKind::SwappedCartesian {
            y_labels: story_labels,
            series: vec![
                CartesianSeries::new("Demand", demand_per_story, SeriesType::Line)
                    .with_color("#1f77b4"),
                CartesianSeries::new("Limit", vec![allowable_ratio; n], SeriesType::Line)
                    .with_color("#cc0000")
                    .dashed(),
            ],
        },
    }
}

/// Pier shear line chart.
///
/// Y axis = story/pier label (one entry per pier × story combination).
/// X axis = shear stress Vu/Acv (psi).
/// Limit line = 8√f'c (psi) — ACI 318-19 §18.10.4.4 absolute maximum.
///
/// # Arguments
/// * `labels`   – one label per data point, e.g. `"C1Y1 / L10"`, sorted by
///               demand descending so the highest-stressed entry is at the top.
/// * `vu_acv`   – corresponding Vu/Acv values (psi), same order as `labels`.
/// * `fc_psi`   – concrete compressive strength in psi (e.g. 5000.0).
pub fn pier_shear_chart(
    labels:  Vec<String>,
    vu_acv:  Vec<f64>,
    fc_psi:  f64,
) -> ChartSpec {
    let limit = 8.0 * fc_psi.sqrt();
    let n     = labels.len();
    let height = (n as u32 * 22 + 100).max(500);

    ChartSpec {
        title:  format!("Pier Shear — Vu/Acv vs 8√f'c ({:.0} psi)", limit),
        width:  900,
        height,
        kind:   ChartKind::SwappedCartesian {
            y_labels: labels,
            series: vec![
                CartesianSeries::new("Vu/Acv (psi)", vu_acv, SeriesType::Line)
                    .with_color("#1f77b4"),
                CartesianSeries::new(
                    format!("8√f'c = {:.0} psi", limit),
                    vec![limit; n],
                    SeriesType::Line,
                )
                .with_color("#cc0000")
                .dashed(),
            ],
        },
    }
}

/// Pier axial stress profile — one line per pier label.
///
/// Y axis = story level (bottom → top).
/// X axis = axial stress fa (ksi), negative = compression, positive = tension.
/// A grey dashed zero-reference line is always included.
///
/// # Arguments
/// * `story_labels` – all story labels in bottom-to-top order.
/// * `pier_series`  – vec of (pier_label, values_per_story) pairs.
///                    Values should be NaN for stories where the pier doesn't exist.
pub fn pier_axial_chart(
    story_labels: Vec<String>,
    pier_series:  Vec<(String, Vec<f64>)>,
) -> ChartSpec {
    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728",
        "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    ];
    let n = story_labels.len();
    let height = (n as u32 * 18 + 100).max(500);

    let mut series: Vec<CartesianSeries> = pier_series
        .into_iter()
        .enumerate()
        .map(|(i, (name, data))| {
            CartesianSeries::new(name, data, SeriesType::Line)
                .with_color(palette[i % palette.len()])
        })
        .collect();

    // Zero reference
    series.push(
        CartesianSeries::new("Zero", vec![0.0; n], SeriesType::Line)
            .with_color("#aaaaaa")
            .dashed(),
    );

    ChartSpec {
        title:  "Pier Axial Stress (ksi)".into(),
        width:  800,
        height,
        kind:   ChartKind::SwappedCartesian { y_labels: story_labels, series },
    }
}

// ─── Renderers ────────────────────────────────────────────────────────────────

/// SSR — returns SVG string. No disk I/O. Caller decides where it goes.
#[cfg(feature = "ssr")]
pub fn render_svg(spec: &ChartSpec) -> Result<String> {
    ImageRenderer::new(spec.width, spec.height)
        .render(&build_chart(spec))
        .context("charming SVG render failed")
}

/// HTML fragment — always available, no feature flag.
/// Inject into a Tauri webview div for interactive ECharts.
pub fn render_html(spec: &ChartSpec, chart_id: &str) -> Result<String> {
    charming::HtmlRenderer::new(chart_id, spec.width as u64, spec.height as u64)
        .render(&build_chart(spec))
        .context("charming HTML render failed")
}

/// WASM renderer — Tauri frontend (--features wasm, exclusive with ssr).
#[cfg(feature = "wasm")]
pub fn render_wasm(spec: &ChartSpec, dom_id: &str) -> Result<()> {
    WasmRenderer::new(spec.width, spec.height)
        .render(dom_id, &build_chart(spec))
        .context("charming WASM render failed")
}