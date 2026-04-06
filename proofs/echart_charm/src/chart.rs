use anyhow::{Context, Result};
use charming::{
    Chart,
    component::{Axis, Legend, Title},
    element::{AxisType, Tooltip, Trigger},
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
    /// Bar/Line combo — shared x-axis categories, mixed series
    Cartesian {
        x:      Vec<String>,
        series: Vec<CartesianSeries>,
    },
    /// Pie / donut chart
    Pie {
        data: Vec<(f64, String)>, // (value, label)
    },
}

#[derive(Debug, Clone)]
pub struct CartesianSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub kind: SeriesType,
}

#[derive(Debug, Clone)]
pub enum SeriesType { Bar, Line }

// ─── Adapter: ChartSpec → charming Chart ─────────────────────────────────────

pub fn build_chart(spec: &ChartSpec) -> Chart {
    match &spec.kind {
        ChartKind::Cartesian { x, series } => build_cartesian(&spec.title, x, series),
        ChartKind::Pie { data }            => build_pie(&spec.title, data),
    }
}

fn build_cartesian(title: &str, x: &[String], series: &[CartesianSeries]) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(title))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new())
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(x.iter().map(String::as_str).collect::<Vec<_>>()),
        )
        .y_axis(Axis::new().type_(AxisType::Value));

    for s in series {
        chart = match s.kind {
            SeriesType::Bar  => chart.series(Bar::new().name(s.name.as_str()).data(s.data.clone())),
            SeriesType::Line => chart.series(Line::new().name(s.name.as_str()).data(s.data.clone())),
        };
    }
    chart
}

fn build_pie(title: &str, data: &[(f64, String)]) -> Chart {
    Chart::new()
        .title(Title::new().text(title).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .legend(Legend::new().bottom("5%").left("center"))
        .series(
            Pie::new()
                .name(title)
                .radius(vec!["35%", "65%"])   // donut style
                .center(vec!["50%", "48%"])
                .data(data.iter().map(|(v, l)| (*v, l.as_str())).collect::<Vec<_>>()),
        )
}

// ─── Preset chart constructors ────────────────────────────────────────────────
// These return ready-to-render ChartSpec values for common structural
// engineering chart types. Callers substitute real data.

/// Base reaction pie chart — Dead / Live / SDL / Wind / Seismic
pub fn base_reaction_pie(
    dead: f64,
    live: f64,
    sdl:  f64,
    wind: f64,
    seismic: f64,
) -> ChartSpec {
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

/// Story shear line chart — multiple lateral load directions per story level
pub fn story_shear_chart(
    story_labels: Vec<String>,    // ["Roof", "L10", "L09", ...]
    x_shear:      Vec<f64>,       // kips, X-direction
    y_shear:      Vec<f64>,       // kips, Y-direction
) -> ChartSpec {
    ChartSpec {
        title:  "Story Shear — Lateral Loads".into(),
        width:  700,
        height: 450,
        kind:   ChartKind::Cartesian {
            x: story_labels,
            series: vec![
                CartesianSeries { name: "Shear X (kips)".into(), data: x_shear, kind: SeriesType::Line },
                CartesianSeries { name: "Shear Y (kips)".into(), data: y_shear, kind: SeriesType::Line },
            ],
        },
    }
}

/// Force vs displacement bar+line combo
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
                CartesianSeries { name: "Force (kips)".into(),      data: force,        kind: SeriesType::Bar  },
                CartesianSeries { name: "Displacement (in)".into(), data: displacement, kind: SeriesType::Line },
            ],
        },
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