# `ext-render` — Rendering Architecture Design (Final)

**Crate:** `crates/ext-render`  
**Revision:** Final architecture — single charming `Chart` definition drives both Tauri frontend and PDF report  
**Purpose:** Convert `CalcOutput` into charming `Chart` objects, then render to whichever format the caller needs.

---

## The Unifying Insight

`charming` has three render modes, all starting from the same `Chart` definition:

| Renderer | Feature flag | Output | Consumer |
|---|---|---|---|
| `HtmlRenderer` | none (always available) | Self-contained HTML fragment | Tauri webview — fully interactive ECharts |
| `ImageRenderer` | `ssr` (bundles Deno) | SVG string | `ext-report` → Typst → PDF |
| `WasmRenderer` | `wasm` | (future, not used now) | — |

**One `Chart` definition. Two render paths. Zero duplication.**

The `HtmlRenderer` output is not a static image — it is a live ECharts instance running in the browser engine, with full hover tooltips, zoom, legend toggle, and responsive layout. This is identical interactivity to `echarts-for-react`, achieved without any TypeScript ECharts option builder.

---

## Why This Eliminates `chart_data`

The previous design had two parallel code paths:

```
OLD (two definitions, two consumers):
  CalcOutput → chart_data::DriftChartData → Tauri IPC → TypeScript option builder → ECharts
  CalcOutput → chart_render::charming::Chart → ImageRenderer → SVG → Typst
```

The new design collapses them:

```
NEW (one definition, two render modes):
  CalcOutput → chart_build::build_drift_chart() → charming::Chart
                                                        │
                              ┌─────────────────────────┴───────────────────────────┐
                              │                                                     │
                    HtmlRenderer::render()                          ImageRenderer::render()
                              │                          (feature = "ssr", Deno bundled)
                    HTML fragment string                                SVG string
                              │                                           │
                      Tauri webview                           TypstWorld::image_cache
                   (interactive ECharts)                                  │
                                                              typst::compile() → PDF bytes
```

`chart_data::*` structs, the `ts-rs` bindings, and all TypeScript ECharts option builders in React are eliminated. The frontend receives HTML fragments over Tauri IPC and renders them in `<iframe>` or `dangerouslySetInnerHTML`. No ECharts npm dependency needed in the frontend at all.

---

## Module Structure

```
crates/ext-render/src/
├── lib.rs
│
├── chart_build/            ← charming::Chart builders (no feature flag)
│   ├── mod.rs              ← pub use drift::build, displacement::build, etc.
│   ├── drift.rs            ← pub fn build(drift: &DriftOutput, cfg: &ChartConfig) -> Chart
│   ├── displacement.rs
│   ├── pier_shear.rs
│   ├── pier_axial.rs
│   └── modal.rs
│
├── render_html/            ← HtmlRenderer path (no feature flag, always available)
│   ├── mod.rs              ← pub fn render_all_html(calc) -> Result<HashMap<String, String>>
│   └── (delegates to chart_build + HtmlRenderer)
│
└── render_svg/             ← ImageRenderer path (feature = "ssr", bundles Deno)
    ├── mod.rs              ← pub fn render_all_svg(calc) -> Result<HashMap<String, String>>
    └── (delegates to chart_build + ImageRenderer)
```

`chart_build` is the single source of truth for chart structure. Both renderer modules consume it.

---

## Feature Flags

```toml
# crates/ext-render/Cargo.toml
[features]
ssr = ["charming/ssr"]   # bundles Deno ~80MB; required for render_svg

[dependencies]
anyhow   = { workspace = true }
charming = { workspace = true }   # always: HtmlRenderer + Chart builders
ext-calc = { path = "../ext-calc" }
```

`ts-rs` and `serde` are **not needed** — no data structs cross the Tauri IPC boundary anymore.

Without `--features ssr`: `chart_build` and `render_html` compile. `render_svg` is gated out.  
With `--features ssr`: full pipeline available.

---

## `chart_build` Module — The Single Definition

### `ChartConfig`

```rust
/// Controls chart appearance and dimensions.
/// Passed to every builder so callers can tune for display vs. report.
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width:  u32,
    pub height: u32,
    pub theme:  ChartTheme,
}

#[derive(Debug, Clone, Copy)]
pub enum ChartTheme {
    /// Dark background for Tauri webview (matches app dark theme)
    Dark,
    /// White background for PDF embedding (print-safe)
    Report,
}

impl ChartConfig {
    pub fn for_display(width: u32, height: u32) -> Self {
        Self { width, height, theme: ChartTheme::Dark }
    }
    pub fn for_report(width: u32, height: u32) -> Self {
        Self { width, height, theme: ChartTheme::Report }
    }
}
```

### Color palette

```rust
// Consistent across both renderers — same ECharts engine processes these
mod colors {
    pub const GOVERNING_FAIL: &str = "#dc2626";  // red — failing governing case
    pub const GOVERNING_PASS: &str = "#ef4444";  // light red — governing but passing
    pub const NORMAL_FAIL:    &str = "#f97316";  // orange — failing non-governing
    pub const NORMAL_PASS:    &str = "#3b82f6";  // blue — normal passing
    pub const LIMIT_LINE:     &str = "#ef4444";  // red dashed — code limit
    pub const SUM_UX:         &str = "#3b82f6";  // blue — modal UX cumulative
    pub const SUM_UY:         &str = "#f59e0b";  // amber — modal UY cumulative
    pub const BG_DARK:        &str = "#0f172a";  // slate-900 for dark theme
    pub const BG_REPORT:      &str = "#ffffff";  // white for report theme
    pub const AXIS_DARK:      &str = "#94a3b8";  // slate-400
    pub const AXIS_REPORT:    &str = "#374151";  // gray-700
}
```

### `chart_build/drift.rs`

```rust
use charming::{Chart, component::*, element::*, series::*};
use ext_calc::output::DriftOutput;
use super::{ChartConfig, ChartTheme, colors};

pub fn build(drift: &DriftOutput, title: &str, cfg: &ChartConfig) -> Chart {
    let stories   = collect_stories(drift);
    let limit     = drift.allowable_ratio;
    let raw       = build_series_raw(drift);
    let axis_color = match cfg.theme {
        ChartTheme::Dark   => colors::AXIS_DARK,
        ChartTheme::Report => colors::AXIS_REPORT,
    };
    let bg_color = match cfg.theme {
        ChartTheme::Dark   => colors::BG_DARK,
        ChartTheme::Report => colors::BG_REPORT,
    };

    let mut chart = Chart::new()
        .background_color(bg_color)
        .title(Title::new()
            .text(title)
            .text_style(TextStyle::new().color(axis_color).font_size(13)))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new()
            .bottom(0)
            .text_style(TextStyle::new().color(axis_color).font_size(11)))
        .grid(Grid::new().left(80).right(120).top(50).bottom(60))
        .x_axis(Axis::new()
            .type_(AxisType::Value)
            .name("Drift Ratio")
            .axis_label(AxisLabel::new().color(axis_color).font_size(11))
            .split_line(SplitLine::new()
                .line_style(LineStyle::new().color(if cfg.theme == ChartTheme::Dark { "#1e293b" } else { "#e5e7eb" }))))
        .y_axis(Axis::new()
            .type_(AxisType::Category)
            .data(stories.clone())
            .axis_label(AxisLabel::new().color(axis_color).font_size(11)))
        // Limit line as a constant-value series
        .series(Line::new()
            .name("Limit")
            .data(stories.iter().map(|_| limit).collect::<Vec<_>>())
            .line_style(LineStyle::new()
                .color(colors::LIMIT_LINE)
                .type_("dashed")
                .width(2))
            .symbol(Symbol::None));

    for s in &raw {
        let color = if s.is_governing && !s.pass { colors::GOVERNING_FAIL }
                    else if s.is_governing        { colors::GOVERNING_PASS }
                    else if !s.pass               { colors::NORMAL_FAIL }
                    else                           { colors::NORMAL_PASS };
        chart = chart.series(Bar::new()
            .name(&s.name)
            .data(s.data.clone())
            .item_style(ItemStyle::new().color(color)));
    }

    chart
}

// ── Internal helpers ──────────────────────────────────────────────────────────

pub(super) struct DriftSeriesRaw {
    pub name:         String,
    pub data:         Vec<f64>,
    pub is_governing: bool,
    pub pass:         bool,
}

pub(super) fn collect_stories(drift: &DriftOutput) -> Vec<String> {
    // Deduplicate, preserve elevation sort order already applied by ext-calc
    let mut seen  = std::collections::HashSet::new();
    let mut result = Vec::new();
    for row in &drift.rows {
        if seen.insert(row.story.clone()) {
            result.push(row.story.clone());
        }
    }
    result
}

pub(super) fn build_series_raw(drift: &DriftOutput) -> Vec<DriftSeriesRaw> {
    // For each unique (group_name, output_case, direction, sense) combination
    // build a flat Vec<f64> aligned to collect_stories() order.
    // Mark is_governing on the combination that matches drift.governing.
    // ... implementation
    todo!()
}
```

### `chart_build/modal.rs`

```rust
pub fn build(modal: &ModalOutput, cfg: &ChartConfig) -> Chart {
    let modes: Vec<i64> = modal.rows.iter().map(|r| r.mode).collect();
    let sum_ux: Vec<f64> = modal.rows.iter().map(|r| r.sum_ux).collect();
    let sum_uy: Vec<f64> = modal.rows.iter().map(|r| r.sum_uy).collect();
    let threshold = modal.threshold;

    Chart::new()
        .background_color(/* by theme */)
        .title(Title::new().text("Modal Mass Participation"))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().bottom(0))
        .x_axis(Axis::new()
            .type_(AxisType::Category)
            .data(modes.iter().map(|m| m.to_string()).collect::<Vec<_>>())
            .name("Mode"))
        .y_axis(Axis::new()
            .type_(AxisType::Value)
            .name("Cumulative Mass Ratio")
            .max(1.0))
        .series(Line::new()
            .name("ΣUX")
            .data(sum_ux)
            .smooth(true)
            .line_style(LineStyle::new().color(colors::SUM_UX).width(2)))
        .series(Line::new()
            .name("ΣUY")
            .data(sum_uy)
            .smooth(true)
            .line_style(LineStyle::new().color(colors::SUM_UY).width(2)))
        // Threshold markLine — horizontal line at modal.threshold
        .series(Line::new()
            .name("Threshold")
            .data(modes.iter().map(|_| threshold).collect::<Vec<_>>())
            .line_style(LineStyle::new()
                .color(colors::LIMIT_LINE)
                .type_("dashed")
                .width(2))
            .symbol(Symbol::None))
}
```

Pier shear, pier axial, and displacement builders follow the same pattern.

---

## `render_html` Module — Tauri Webview Path

```rust
// render_html/mod.rs
use charming::HtmlRenderer;
use ext_calc::output::CalcOutput;
use anyhow::Result;
use std::collections::HashMap;
use crate::chart_build::{self, ChartConfig};

/// Render all enabled checks to interactive HTML fragments.
/// Each value is a self-contained HTML string with embedded ECharts JS.
/// The Tauri webview renders these in <iframe> or dangerouslySetInnerHTML.
pub fn render_all_html(
    calc: &CalcOutput,
    width: u32,
    height: u32,
) -> Result<HashMap<String, String>> {
    let cfg = ChartConfig::for_display(width, height);
    let mut out = HashMap::new();

    if let Some(drift) = &calc.drift_wind {
        let chart = chart_build::drift::build(drift, "Story Drift — Wind", &cfg);
        out.insert("drift_wind".into(), render_html_chart(&chart, "drift_wind", &cfg)?);
    }
    if let Some(drift) = &calc.drift_seismic {
        let chart = chart_build::drift::build(drift, "Story Drift — Seismic", &cfg);
        out.insert("drift_seismic".into(), render_html_chart(&chart, "drift_seismic", &cfg)?);
    }
    if let Some(disp) = &calc.displacement_wind {
        let chart = chart_build::displacement::build(disp, "Lateral Displacement — Wind", &cfg);
        out.insert("displacement_wind".into(), render_html_chart(&chart, "displacement_wind", &cfg)?);
    }
    if let Some(shear) = &calc.pier_shear_wind {
        let chart = chart_build::pier_shear::build(shear, "Pier Shear — Wind", &cfg);
        out.insert("pier_shear_wind".into(), render_html_chart(&chart, "pier_shear_wind", &cfg)?);
    }
    if let Some(shear) = &calc.pier_shear_seismic {
        let chart = chart_build::pier_shear::build(shear, "Pier Shear — Seismic", &cfg);
        out.insert("pier_shear_seismic".into(), render_html_chart(&chart, "pier_shear_seismic", &cfg)?);
    }
    if let Some(axial) = &calc.pier_axial {
        let chart = chart_build::pier_axial::build(axial, "Pier Axial Stress", &cfg);
        out.insert("pier_axial".into(), render_html_chart(&chart, "pier_axial", &cfg)?);
    }
    if let Some(modal) = &calc.modal {
        let chart = chart_build::modal::build(modal, &cfg);
        out.insert("modal".into(), render_html_chart(&chart, "modal", &cfg)?);
    }

    Ok(out)
}

fn render_html_chart(chart: &charming::Chart, id: &str, cfg: &ChartConfig) -> Result<String> {
    HtmlRenderer::new(id, cfg.width as u64, cfg.height as u64)
        .render(chart)
        .map_err(|e| anyhow::anyhow!("HtmlRenderer failed for {id}: {e}"))
}
```

### What `HtmlRenderer` outputs

The HTML fragment looks like:

```html
<div id="drift_wind" style="width: 900px; height: 600px;"></div>
<script src="https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js"></script>
<script>
  var chart = echarts.init(document.getElementById('drift_wind'));
  chart.setOption({ /* full ECharts option JSON */ });
</script>
```

This is rendered by the Tauri webview (Chromium/WebKit) as a live, interactive ECharts chart — hover, zoom, legend toggle all work. The CDN script load is a one-time cached download; in production, the ECharts JS can be bundled locally instead.

---

## `render_svg` Module — Report Image Path

```rust
// render_svg/mod.rs
#[cfg(feature = "ssr")]
use charming::ImageRenderer;
use ext_calc::output::CalcOutput;
use anyhow::Result;
use std::collections::HashMap;
use crate::chart_build::{self, ChartConfig};

/// Render all enabled checks to SVG strings for Typst PDF embedding.
/// Dimensions are fixed for A4 report layout.
#[cfg(feature = "ssr")]
pub fn render_all_svg(calc: &CalcOutput) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();

    if let Some(drift) = &calc.drift_wind {
        let cfg   = ChartConfig::for_report(900, 600);
        let chart = chart_build::drift::build(drift, "Story Drift — Wind", &cfg);
        out.insert("images/drift_wind.svg".into(), render_svg_chart(&chart, 900, 600)?);
    }
    if let Some(drift) = &calc.drift_seismic {
        let cfg   = ChartConfig::for_report(900, 600);
        let chart = chart_build::drift::build(drift, "Story Drift — Seismic", &cfg);
        out.insert("images/drift_seismic.svg".into(), render_svg_chart(&chart, 900, 600)?);
    }
    if let Some(disp) = &calc.displacement_wind {
        let cfg   = ChartConfig::for_report(900, 600);
        let chart = chart_build::displacement::build(disp, "Lateral Displacement — Wind", &cfg);
        out.insert("images/displacement_wind.svg".into(), render_svg_chart(&chart, 900, 600)?);
    }
    if let Some(shear) = &calc.pier_shear_wind {
        let cfg   = ChartConfig::for_report(900, 500);
        let chart = chart_build::pier_shear::build(shear, "Pier Shear — Wind", &cfg);
        out.insert("images/pier_shear_wind.svg".into(), render_svg_chart(&chart, 900, 500)?);
    }
    if let Some(shear) = &calc.pier_shear_seismic {
        let cfg   = ChartConfig::for_report(900, 500);
        let chart = chart_build::pier_shear::build(shear, "Pier Shear — Seismic", &cfg);
        out.insert("images/pier_shear_seismic.svg".into(), render_svg_chart(&chart, 900, 500)?);
    }
    if let Some(axial) = &calc.pier_axial {
        let cfg   = ChartConfig::for_report(900, 500);
        let chart = chart_build::pier_axial::build(axial, "Pier Axial Stress", &cfg);
        out.insert("images/pier_axial.svg".into(), render_svg_chart(&chart, 900, 500)?);
    }
    if let Some(modal) = &calc.modal {
        let cfg   = ChartConfig::for_report(900, 500);
        let chart = chart_build::modal::build(modal, &cfg);
        out.insert("images/modal.svg".into(), render_svg_chart(&chart, 900, 500)?);
    }

    Ok(out)
}

#[cfg(feature = "ssr")]
fn render_svg_chart(chart: &charming::Chart, w: u32, h: u32) -> Result<String> {
    ImageRenderer::new(w, h)
        .render(chart)
        .map_err(|e| anyhow::anyhow!("ImageRenderer failed: {e}"))
}
```

---

## Public API Summary

```rust
// lib.rs
pub mod chart_build;
pub mod render_html;
#[cfg(feature = "ssr")]
pub mod render_svg;

// Re-exports for convenience
pub use render_html::render_all_html;
#[cfg(feature = "ssr")]
pub use render_svg::render_all_svg;
pub use chart_build::ChartConfig;
```

---

## Impact on `ext-tauri`

`ext-render` stays a pure Rust library. `ext-api` owns workflow orchestration for every frontend, while `ext-tauri` owns app state, IPC, artifact paths, progress events, and any desktop-specific caching.

### Desktop ownership split

| Layer | Responsibility |
|---|---|
| `ext-render` | build `Chart`, return HTML fragments or SVG strings |
| `ext-report` | compose report content from `CalcOutput` + rendered SVG strings |
| `ext-api` | orchestrate calc/render/report workflows and return app-facing DTOs |
| `ext-tauri` backend | call `ext-api`, manage desktop state, choose artifact paths, bridge IPC |
| `apps/desktop` frontend | request charts/reports, render returned HTML safely, show export progress/errors |

### Required Tauri command matrix

The desktop build should not invent ad hoc commands during implementation. Use this command surface as the baseline:

| Command | Input | Output | Notes |
|---|---|---|---|
| `run_calc_review` | `{ fixtureOrProjectPath }` | `CalcOutputSummaryDto` | optional debug/review path; delegates to `ext-api` |
| `get_rendered_charts` | `{ width, height, theme }` | `HashMap<String, String>` | returns chart key -> HTML fragment from `ext-api` |
| `get_rendered_chart` | `{ chartKey, width, height, theme }` | `String` | fallback for per-tab lazy loading |
| `generate_report_artifacts` | `{ reportName, outputDir }` | `ReportArtifactDto` | delegates to `ext-api` render/report workflow |
| `open_report_artifact` | `{ path }` | `()` | convenience wrapper over shell/open plugin |

Recommended payload DTOs:

```rust
#[derive(Serialize)]
pub struct CalcOutputSummaryDto {
    pub overall_status: String,
    pub available_checks: Vec<String>,
}

#[derive(Deserialize)]
pub struct ChartRequest {
    pub width: u32,
    pub height: u32,
    pub theme: String, // "dark" | "report"
}

#[derive(Serialize)]
pub struct ReportArtifactDto {
    pub pdf_path: String,
    pub typ_path: Option<String>,
    pub asset_dir: String,
    pub asset_files: Vec<String>,
}
```

### Required backend behavior

`ext-tauri` should hold app-facing state after a successful `ext-api` workflow call. It should not reimplement render/report orchestration itself.

The canonical render input is the persisted calc artifact loaded through `ext-api`. In-memory desktop caches are allowed only as a performance optimization layered on top of that persisted contract.

Report generation follows this sequence:

1. validate a persisted `calc_output.json` is available for the active project/version
2. call `ext-api` report/render workflow
3. `ext-api` loads `CalcOutput` from the persisted artifact
4. `ext-api` calls `ext_render::render_all_svg(calc)` with report dimensions
5. `ext-api` passes the returned SVG map into `ext-report`
6. `ext-api` returns artifact paths and chart HTML to `ext-tauri`
7. `ext-tauri` may cache those payloads in memory and returns them to the frontend

### Required frontend behavior

- chart tabs request `get_rendered_charts` once per calc result + viewport size and cache by chart key
- report UI calls `generate_report_artifacts`, then offers `View` / `Open folder`
- loading and error states are owned by the frontend store, not hidden in chart components
- ownership for report UI stays in `apps/desktop/src/components/reports/*`; ownership for chart display stays in dedicated analysis/chart components rather than generic sandbox code

---

## Impact on `ext-tauri` Frontend

The React components no longer build ECharts option objects. Instead each chart component renders an HTML fragment:

```typescript
// components/charts/DriftChart.tsx
interface DriftChartProps {
    html:    string;   // from invoke('get_drift_wind_chart')
    height?: number;
}

export function DriftChart({ html, height = 500 }: DriftChartProps) {
    return (
        <div
            style={{ width: '100%', height }}
            dangerouslySetInnerHTML={{ __html: html }}
        />
    );
}
```

Or with an `<iframe>` for stricter isolation:

```typescript
export function DriftChart({ html, height = 500 }: DriftChartProps) {
    const blob = new Blob([html], { type: 'text/html' });
    const url  = URL.createObjectURL(blob);
    return <iframe src={url} style={{ width: '100%', height, border: 'none' }} />;
}
```

The ECharts interactivity (hover, zoom, legend toggle) is fully preserved — it runs inside the browser engine regardless of whether the HTML was built in TypeScript or by charming.

**Eliminated from the frontend:**
- `echarts` npm package
- `echarts-for-react` npm package
- All ECharts option builders (`option: EChartsOption = { xAxis, yAxis, series, ... }`)
- All `chart_data::*` TypeScript type imports from `bindings/`
- `ts-rs` dependency from `ext-render`

---

## What Changes vs. Previous Design

| Concern | Previous | Final |
|---|---|---|
| Chart definition count | 2 (charming + TS option) | **1** (charming only) |
| `chart_data` module | Exists (serde structs + ts-rs) | **Eliminated** |
| TypeScript option builders | In each React component | **Eliminated** |
| `ts-rs` dependency | Required | **Removed** |
| `echarts-for-react` | Required | **Removed** |
| ECharts interactivity in Tauri | Via echarts-for-react | **Via HtmlRenderer (identical)** |
| Report image source | charming ImageRenderer | **Same** |
| Frontend bundle size | Includes ECharts (~1MB) | **ECharts loads from CDN or local file** |
| `ChartTheme::Dark` | Not possible in old arch | **Now supported via ChartConfig** |

---

## CDN vs. Local ECharts JS

The `HtmlRenderer` embeds a `<script src="https://cdn.jsdelivr.net/...">` by default.

For a desktop Tauri app, **bundle ECharts locally** to avoid network dependency:

```rust
// In HtmlRenderer output, replace the CDN URL with a local asset
// Option A: Copy echarts.min.js into Tauri's resource dir, serve via asset protocol
// Option B: Patch the HTML string after render to replace CDN URL
let html = render_html_chart(&chart, id, cfg)?
    .replace(
        "https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js",
        "asset://localhost/echarts.min.js"  // Tauri asset protocol
    );
```

This is a one-line patch applied in `render_html_chart()`. The ECharts JS file is added to Tauri resources and loaded locally in both dev and release desktop builds.

### Required Tauri wiring for local assets

`ext-tauri` must declare the bundled asset explicitly. The design is not complete until these two pieces are present:

1. add `echarts.min.js` to a checked-in desktop resource location, for example:

```text
crates/ext-tauri/resources/echarts/echarts.min.js
```

2. add that resource path to `crates/ext-tauri/tauri.conf.json`

```json
{
  "bundle": {
    "resources": [
      "resources/echarts/echarts.min.js"
    ]
  }
}
```

The desktop frontend must not depend on a live CDN for chart rendering.

---

## ChartConfig Dimensions

| Check | Display (HtmlRenderer) | Report (ImageRenderer) | Typst height |
|---|---|---|---|
| Drift wind | 100% × window | 900 × 600 | `height: 6in` |
| Drift seismic | 100% × window | 900 × 600 | `height: 6in` |
| Displacement wind | 100% × window | 900 × 600 | `height: 6in` |
| Pier Shear wind | 100% × window | 900 × 500 | `height: 5in` |
| Pier Shear seismic | 100% × window | 900 × 500 | `height: 5in` |
| Pier Axial | 100% × window | 900 × 500 | `height: 5in` |
| Modal | 100% × window | 900 × 500 | `height: 5in` |

For display, `HtmlRenderer` uses pixel dimensions but the container div is `width: 100%` via CSS — ECharts auto-resizes on `window.resize`.

Report-facing dimensions are sized for the tabloid landscape report locked by the Week 7-8 spec, not A4. `ext-report` owns final page composition, but `ext-render` should treat the 900px-wide report variants above as the baseline image contract for that tabloid layout.

---

## Binary Size and Feature Wiring

| Build target | Features | Deno bundled | Approx delta |
|---|---|---|---|
| `cargo check` | none | No | 0 MB |
| `cargo test` | none | No | 0 MB |
| `ext` CLI release | `ssr` | Yes | +80 MB |
| `ext-tauri` release | `ssr` | Yes | +80 MB |
| `ext-tauri` dev build | none | No | 0 MB (HTML charts work without SSR) |

Dev builds use `HtmlRenderer` only — instant startup, no Deno. Release builds add `--features ssr` and bundle Deno for the PDF pipeline.

### Required workspace wiring

The release story is only implementation-ready when these are wired together:

1. `crates/ext-tauri/Cargo.toml` depends on `ext-render` and `ext-report`
2. `ext-render` exposes `ssr` and `ext-tauri` forwards a matching feature, for example:

```toml
[features]
default = []
report-ssr = ["ext-render/ssr"]
```

3. release build commands enable that feature:

```text
cargo tauri build --features report-ssr
```

4. dev builds stay on HTML-only chart rendering and do not require Deno

Without those four steps, the SSR/PDF path is still design-only.

---

## What `ext-render` Does NOT Do

- Does not write files — all output is in-memory strings
- Does not import Tauri — `ext-api` and `ext-tauri` import from `ext-render`
- Does not call Typst — that is `ext-report`
- Does not produce Excel — that is `ext-report`
- Does not have serde structs for IPC — the HTML string is the IPC payload

---

*Last updated: 2026-04-06*  
*Related: `EXT_REPORT_DESIGN.md` · `EXT_CALC_SPEC.md` · `SPRINT_W9_W12.md`*
