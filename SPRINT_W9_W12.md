# Sprint Plan — Weeks 9–12 (Final)
## ext-render · ext-report · ext-tauri

**Revision history:**
- Initial plan: `chart_data` serde structs + `getDataURL()` for report images
- Revision 1: charming SSR discovery — `chart_render` for reports, serde structs for Tauri
- **Revision 2 (final):** One charming `Chart` definition drives both consumers via `HtmlRenderer` (Tauri) and `ImageRenderer` (PDF). `chart_data`, `ts-rs`, `echarts-for-react`, and all TypeScript option builders are eliminated.

---

## Status as of Week 8 End

| Crate | State |
|---|---|
| `ext-error` | ✅ Complete |
| `ext-db` | ✅ Complete |
| `ext-calc` | ✅ Complete — 8 checks, spec, all tests passing |
| `ext-render` | 🟡 Stub — final design written, no implementation |
| `ext-report` | 🟡 Stub — design written, no implementation |
| `ext-tauri` | 🔴 Shell — one `greet` command, no `AppState` |
| `apps/desktop` | 🟡 UI panels with mock data, no real commands wired |

---

## Final Architecture

```
CalcOutput
    │
    └──► ext-render::chart_build::*        ← ONE chart definition per check type
                    │                         (charming::Chart, no feature flag)
                    │
          ┌─────────┴──────────────┐
          │                        │
  HtmlRenderer::render()    ImageRenderer::render()
  (always available)         (feature = "ssr", Deno)
          │                        │
  HTML fragment string        SVG string
          │                        │
  Tauri IPC payload          TypstWorld::image_cache
          │                        │
  webview renders it         typst::compile() → PDF bytes
  (interactive ECharts)
```

**What this eliminates vs. the previous plan:**

| Eliminated | Reason |
|---|---|
| `chart_data::*` serde structs | charming builds the full chart — no parallel data model needed |
| `ts-rs` dependency | No IPC structs to export as TypeScript types |
| `echarts-for-react` npm package | charming HtmlRenderer produces identical interactive output |
| TypeScript ECharts option builders | Chart logic lives in Rust, not duplicated in TypeScript |
| `save_chart_images` Tauri command | Never existed in this plan (removed in Revision 1) |
| `getDataURL()` calls in frontend | Never needed — charming generates both HTML and SVG |

---

## Workspace Dependency Changes

```toml
# Cargo.toml (workspace) — add:
charming        = "0.6.0"
rust_xlsxwriter = "0.84"
walkdir         = "2.5"

# Remove / never add:
# ts-rs is already in workspace but no longer used by ext-render
```

```toml
# crates/ext-render/Cargo.toml
[features]
ssr = ["charming/ssr"]

[dependencies]
anyhow   = { workspace = true }
charming = { workspace = true }
ext-calc = { path = "../ext-calc" }
# NO: serde, ts-rs
```

```toml
# crates/ext-report/Cargo.toml
[features]
ssr = ["ext-render/ssr"]

[dependencies]
anyhow          = { workspace = true }
serde_json      = { workspace = true }
chrono          = { workspace = true }
ext-calc        = { path = "../ext-calc" }
ext-render      = { path = "../ext-render" }
rust_xlsxwriter = { workspace = true }
typst           = { workspace = true }
typst-pdf       = { workspace = true }
walkdir         = { workspace = true }
```

```toml
# crates/ext-tauri/Cargo.toml — add:
[features]
ssr = ["ext-report/ssr"]

ext-calc   = { path = "../ext-calc" }
ext-render = { path = "../ext-render" }
ext-report = { path = "../ext-report" }
```

```
# apps/desktop package.json — REMOVE:
echarts
echarts-for-react
# KEEP: all other dependencies unchanged
```

---

## Week 9 — `ext-render`: chart_build + render_html + render_svg

**Goal:** Build the complete `ext-render` crate. One `chart_build` module per check type feeds both `render_html` (Tauri) and `render_svg` (PDF). `HtmlRenderer` output is interactive ECharts. `ImageRenderer` output is SVG for Typst.

---

### W9-T1 — Module skeleton and ChartConfig (Day 1)

```
crates/ext-render/src/
├── lib.rs
├── chart_build/
│   ├── mod.rs          ← ChartConfig, ChartTheme, color constants
│   ├── drift.rs        ← build(drift, title, cfg) → Chart
│   ├── displacement.rs
│   ├── pier_shear.rs
│   ├── pier_axial.rs
│   └── modal.rs
├── render_html/
│   └── mod.rs          ← render_all_html(calc, w, h) → HashMap<String, String>
│                         render_one_html(chart, id, cfg) → String
└── render_svg/
    └── mod.rs          ← render_all_svg(calc) → HashMap<String, String>  [cfg(ssr)]
                          render_one_svg(chart, w, h) → String             [cfg(ssr)]
```

**`chart_build/mod.rs`:**

```rust
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width:  u32,
    pub height: u32,
    pub theme:  ChartTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTheme {
    Dark,    // Tauri dark UI — dark bg, light text
    Report,  // PDF white bg, dark text, print-safe colors
}

impl ChartConfig {
    pub fn for_display(width: u32, height: u32) -> Self {
        Self { width, height, theme: ChartTheme::Dark }
    }
    pub fn for_report(width: u32, height: u32) -> Self {
        Self { width, height, theme: ChartTheme::Report }
    }
}

pub(crate) mod colors {
    // Identical colors used by both HtmlRenderer and ImageRenderer
    // (same ECharts engine processes them)
    pub const GOVERNING_FAIL: &str = "#dc2626";
    pub const GOVERNING_PASS: &str = "#ef4444";
    pub const NORMAL_FAIL:    &str = "#f97316";
    pub const NORMAL_PASS:    &str = "#3b82f6";
    pub const LIMIT_LINE:     &str = "#ef4444";
    pub const SUM_UX:         &str = "#3b82f6";
    pub const SUM_UY:         &str = "#f59e0b";
    pub const BG_DARK:        &str = "#0f172a";
    pub const BG_REPORT:      &str = "#ffffff";
    pub const AXIS_DARK:      &str = "#94a3b8";
    pub const AXIS_REPORT:    &str = "#374151";
    pub const GRID_DARK:      &str = "#1e293b";
    pub const GRID_REPORT:    &str = "#e5e7eb";
}
```

**Acceptance:** `cargo check -p ext-render` passes.

---

### W9-T2 — `chart_build/drift.rs` (Days 1–2)

Builds a `charming::Chart` from `DriftOutput` — used for both wind and seismic drift checks.

**Internal helpers (private to module):**

```rust
struct DriftSeriesRaw {
    name:         String,
    data:         Vec<f64>,    // f64 per story, 0.0 if no data
    is_governing: bool,
    pass:         bool,
}

fn collect_stories(drift: &DriftOutput) -> Vec<String>
// Deduplicate story names preserving elevation order from ext-calc

fn build_series_raw(drift: &DriftOutput) -> Vec<DriftSeriesRaw>
// One series per unique (group_name, output_case, direction, sense)
// Mark is_governing on the combination matching drift.governing
```

**Public function:**

```rust
pub fn build(drift: &DriftOutput, title: &str, cfg: &ChartConfig) -> charming::Chart
```

Chart structure:
- Y axis: category (story names, bottom → top)
- X axis: value (drift ratio)
- Series 1: `Line` (dashed red) at `drift.allowable_ratio` — limit line
- Series 2+: `Bar` per `DriftSeriesRaw`, colored by `is_governing` and `pass`
- `Tooltip::Axis` trigger — hover shows story + all series values

**Tests (no feature flag needed):**

```rust
#[test]
fn drift_chart_has_correct_series_count() {
    let drift = load_fixture_drift_wind();
    let cfg   = ChartConfig::for_display(900, 600);
    let chart = build(&drift, "Test", &cfg);
    // Serialize chart to JSON and count series — should match series count + 1 limit line
    // (charming::Chart implements serde::Serialize)
}

#[test]
fn exactly_one_governing_series() {
    // serialize → parse → count series with is_governing color
}
```

---

### W9-T3 — `chart_build/displacement.rs`, `pier_shear.rs`, `pier_axial.rs` (Day 2–3)

Same pattern as drift. Key differences:

**Displacement:** Y axis = story categories, X axis = displacement in display units. Limit line at `disp_limit.value`. Series per (group, case, direction, sense).

**Pier Shear:** Y axis = `"{pier}/{story}"` labels. X axis = DCR (0.0 → ~1.5). Limit line at `1.0` (DCR = 1.0). One bar per `PierShearResult`. Color: `GOVERNING_FAIL` if `!pass`, `GOVERNING_PASS` if `is_governing && pass`, `NORMAL_PASS` otherwise.

**Pier Axial:** Same as pier shear but using `PierAxialResult` and DCR from axial formula.

---

### W9-T4 — `chart_build/modal.rs` (Day 3)

```rust
pub fn build(modal: &ModalOutput, cfg: &ChartConfig) -> charming::Chart
```

Chart structure:
- X axis: mode numbers (category)
- Y axis: cumulative mass ratio (value, 0.0 → 1.0)
- Series 1: `Line` smooth, `SUM_UX` color — ΣUX
- Series 2: `Line` smooth, `SUM_UY` color — ΣUY
- Series 3: `Line` dashed, `LIMIT_LINE` color — horizontal threshold line
- `markPoint` on the mode where ΣUX and ΣUY each reach threshold

---

### W9-T5 — `render_html` module (Day 4)

```rust
// render_html/mod.rs

/// Render all enabled checks to interactive HTML fragments.
/// Returns HashMap<check_name, html_string>.
/// check_name examples: "drift_wind", "pier_shear_wind", "modal"
pub fn render_all_html(
    calc:   &CalcOutput,
    width:  u32,
    height: u32,
) -> Result<HashMap<String, String>>

/// Render one chart to HTML.
/// The HTML is a self-contained fragment with <div> + ECharts <script>.
pub fn render_one_html(
    chart:  &charming::Chart,
    dom_id: &str,
    width:  u32,
    height: u32,
) -> Result<String> {
    charming::HtmlRenderer::new(dom_id, width as u64, height as u64)
        .render(chart)
        .map_err(|e| anyhow::anyhow!("HtmlRenderer failed for '{dom_id}': {e}"))
}
```

**Local ECharts JS patching** (applied inside `render_one_html`):

```rust
// Replace CDN URL with Tauri local asset protocol
// echarts.min.js must be listed in tauri.conf.json resources
let html = html.replace(
    "https://cdn.jsdelivr.net/npm/echarts/dist/echarts.min.js",
    "asset://localhost/echarts.min.js",
);
```

This is optional but recommended for offline/airgapped use. The CDN fallback works if the machine has internet.

**Tests (no feature flag):**

```rust
#[test]
fn html_output_contains_echarts_init() {
    let drift = load_fixture_drift_wind();
    let cfg   = ChartConfig::for_display(900, 600);
    let chart = chart_build::drift::build(&drift, "Test", &cfg);
    let html  = render_one_html(&chart, "test_chart", 900, 600).unwrap();
    assert!(html.contains("echarts.init"));
    assert!(html.contains("setOption"));
    assert!(html.contains("<div"));
}

#[test]
fn render_all_html_returns_7_charts_from_fixture() {
    let calc    = load_fixture_calc();
    let charts  = render_all_html(&calc, 900, 600).unwrap();
    assert_eq!(charts.len(), 7);
    assert!(charts.contains_key("drift_wind"));
    assert!(charts.contains_key("modal"));
    for (_, html) in &charts {
        assert!(html.contains("echarts.init"));
    }
}
```

---

### W9-T6 — `render_svg` module (Day 4–5)

```rust
// render_svg/mod.rs

/// Render all enabled checks to SVG strings for Typst PDF embedding.
/// Returns HashMap<"images/drift_wind.svg", svg_string>.
/// Requires --features ssr (bundles Deno, ~80MB).
#[cfg(feature = "ssr")]
pub fn render_all_svg(calc: &CalcOutput) -> Result<HashMap<String, String>>

#[cfg(feature = "ssr")]
pub fn render_one_svg(chart: &charming::Chart, width: u32, height: u32) -> Result<String> {
    charming::ImageRenderer::new(width, height)
        .render(chart)
        .map_err(|e| anyhow::anyhow!("ImageRenderer failed: {e}"))
}
```

Fixed report dimensions:

| Check | Width | Height | Typst embed |
|---|---|---|---|
| Drift (wind/seismic) | 900 | 600 | `height: 6in` |
| Displacement | 900 | 600 | `height: 6in` |
| Pier Shear (wind/seismic) | 900 | 500 | `height: 5in` |
| Pier Axial | 900 | 500 | `height: 5in` |
| Modal | 900 | 500 | `height: 5in` |

**Tests (feature = "ssr"):**

```rust
#[cfg(feature = "ssr")]
#[test]
fn svg_output_starts_with_svg_tag() {
    let drift = load_fixture_drift_wind();
    let cfg   = ChartConfig::for_report(900, 600);
    let chart = chart_build::drift::build(&drift, "Test", &cfg);
    let svg   = render_one_svg(&chart, 900, 600).unwrap();
    assert!(svg.trim_start().starts_with("<svg"));
    assert!(svg.len() > 1_000);
}

#[cfg(feature = "ssr")]
#[test]
fn render_all_svg_keys_match_typst_image_paths() {
    let calc = load_fixture_calc();
    let svgs = render_all_svg(&calc).unwrap();
    // Keys must match what Typst template uses in #image("...")
    assert!(svgs.contains_key("images/drift_wind.svg"));
    assert!(svgs.contains_key("images/modal.svg"));
    for svg in svgs.values() {
        assert!(svg.trim_start().starts_with("<svg"));
    }
}
```

---

### W9 End-of-Week Definition of Done

- [ ] `cargo test -p ext-render` passes (no ssr — tests `chart_build` and `render_html`)
- [ ] `cargo test -p ext-render --features ssr` passes (tests `render_svg`)
- [ ] `render_all_html` returns 7 HTML strings from fixture `CalcOutput`
- [ ] `render_all_svg` returns 7 SVG strings, all starting with `<svg`
- [ ] Zero `serde`, `ts-rs`, or `echarts-for-react` references in `ext-render`
- [ ] `chart_build` has one function per check type — no per-consumer duplication
- [ ] `ChartTheme::Dark` and `ChartTheme::Report` both compile and produce different colors

---

## Week 10 — `ext-report`: Excel + PDF Pipelines

**Goal:** Full working `ext-report`. `render_pdf` calls `ext_render::render_svg::render_all_svg()` internally — no `assets_dir`, no disk images, no Tauri dependency. `render_excel` produces complete `.xlsx`. Both compile from the same `CalcOutput`.

---

### W10-T1 — Crate skeleton and public API (Day 1)

```
crates/ext-report/src/
├── lib.rs
├── excel/
│   ├── mod.rs          ← pub fn render(calc, path) -> Result<PathBuf>
│   ├── styles.rs       ← shared Format objects
│   ├── summary.rs
│   ├── modal.rs
│   ├── base_shear.rs
│   ├── drift_wind.rs
│   ├── drift_seismic.rs
│   ├── displacement.rs
│   ├── pier_shear_wind.rs
│   ├── pier_shear_seismic.rs
│   └── pier_axial.rs
└── pdf/
    ├── mod.rs          ← pub fn render(calc, path) -> Result<PathBuf>  [cfg(ssr)]
    ├── compiler.rs     ← TypstWorld + compile_to_pdf()
    ├── template.rs     ← build_typst_source(calc) -> String
    └── sections/
        ├── cover.rs, summary.rs, modal.rs, base_shear.rs
        ├── drift.rs, displacement.rs, piers.rs
```

**Public API:**

```rust
pub struct ReportPaths { pub excel: PathBuf, pub pdf: PathBuf }

pub fn render_excel(calc: &CalcOutput, path: &Path) -> Result<PathBuf>;

#[cfg(feature = "ssr")]
pub fn render_pdf(calc: &CalcOutput, path: &Path) -> Result<PathBuf>;

#[cfg(feature = "ssr")]
pub fn render_all(calc: &CalcOutput, output_dir: &Path, name: &str) -> Result<ReportPaths>;
```

---

### W10-T2 — Excel: styles + Summary sheet (Days 1–2)

**`excel/styles.rs`** — `rust_xlsxwriter::Format` objects:

| Name | Appearance |
|---|---|
| `title` | Bold, 14pt |
| `subheader` | Bold 11pt, bg `#4472C4`, white text |
| `header` | Bold 10pt, bg `#D9E1F2` |
| `pass` | bg `#C6EFCE`, text `#276221` |
| `fail` | bg `#FFC7CE`, text `#9C0006` |
| `pending` | bg `#FFEB9C`, text `#9C6500` |
| `governing` | Bold, bg `#FCE4D6` (light orange) |
| `num_3dp` / `num_4dp` | Right-aligned, decimal places |

**Summary sheet structure:**

```
Row 0:  "EtabExtension — Structural Check Summary"      [title, A:C merged]
Row 1:  "Code: ACI318-14 | Version: v3 | Branch: main | 2026-04-05"  [subheader]
Row 2:  [blank]
Row 3:  Check | Status | Message                        [header]
Row 4+: one per SummaryLine — col B colored by status
[blank]
Last:   "Overall: PASS" / "Overall: FAIL"               [colored, large]
```

Key-to-display-label map: `modal→"Modal Mass Participation"`, `baseShear→"Base Shear (RSA/ELF)"`, `driftWind→"Story Drift — Wind"`, `driftSeismic→"Story Drift — Seismic"`, `displacementWind→"Lateral Displacement — Wind"`, `pierShearWind→"Pier Shear — Wind"`, `pierShearSeismic→"Pier Shear — Seismic"`, `pierAxial→"Pier Axial Stress"`.

---

### W10-T3 — Excel: Modal and Base Shear sheets (Day 2)

**Modal:** Mode | Period(s) | UX | UY | ΣUX | ΣUY | Rz | ΣRz.  
Governing-style rows where ΣUX and ΣUY first cross threshold. Footer: mode numbers + PASS/FAIL.

**Base Shear:** Direction block (X then Y): RSA Case | V_RSA | ELF Case | V_ELF | Ratio | Status.  
Full review table below from `output.rows`.

---

### W10-T4 — Excel: Drift, Displacement, Pier sheets (Day 3)

**DriftWind / DriftSeismic:**  
Header: limit, load cases. Data: Story | Group | Case | X+ | X- | Y+ | Y- | Max DCR | Status.  
DCR column red if > 1.0. Governing row: governing style. Footer: governing detail + PASS/FAIL.

**DispWind:**  
Header: H/divisor limit. Data: Story | Group | Case | Disp X+ | X- | Y+ | Y-.  
Footer: governing displacement / allowable / DCR / PASS/FAIL.

**PierShearWind / PierShearSeismic:**  
Header: ϕ, αc, fy, ρt, fc_default. Data: Pier | Story | Combo | Vu | Acv | fc'(ksi) | Vn | ϕVn | DCR | Status | Material | Section.  
DCR red if > 1.0.

**PierAxial:**  
Header: ϕ, formula note. Data: Pier | Story | Combo | Pu | Ag | ϕPo | fa(ksi) | fa/f'c | DCR | Status | fc'(ksi) | Material.

---

### W10-T5 — PDF: TypstWorld (Days 3–4)

Direct port from `echart_charm/src/typst.rs` — already proven on Windows:

```rust
// pdf/compiler.rs
struct TypstWorld {
    library:     LazyHash<Library>,
    book:        LazyHash<FontBook>,
    fonts:       Vec<Font>,
    main:        Source,
    image_cache: HashMap<PathBuf, Bytes>,  // SVGs from render_all_svg() injected here
}

impl World for TypstWorld { /* port verbatim from echart_charm */ }

pub fn compile_to_pdf(
    source: String,
    svgs:   HashMap<String, String>,   // from ext_render::render_all_svg()
) -> Result<Vec<u8>> {
    let image_cache = svgs.into_iter()
        .map(|(k, v)| (PathBuf::from(k), Bytes::new(v.into_bytes())))
        .collect();
    let world = TypstWorld::new(source, image_cache)?;
    let doc   = typst::compile(&world).output.map_err(|e| anyhow!("{e:?}"))?;
    typst_pdf::pdf(&doc, &PdfOptions::default()).map_err(|e| anyhow!("{e:?}"))
}
```

Font loading: `C:\Windows\Fonts` → `fonts/` directory fallback (same as `echart_charm`).

---

### W10-T6 — PDF: Template builder (Day 4)

`pdf/template.rs` — `build_typst_source(calc: &CalcOutput) -> String`

Generates complete Typst markup. Section builders emit `#image("images/{name}.svg")` with heights from the dimension table.

Thornton Tomasetti title block from `echart_charm/src/typst.rs` adapted for A4 portrait.

---

### W10-T7 — Integration tests (Day 5)

```rust
#[test]
fn render_excel_produces_valid_workbook() {
    let calc = load_fixture_calc();
    let dir  = tempfile::tempdir().unwrap();
    let path = dir.path().join("report.xlsx");
    render_excel(&calc, &path).unwrap();
    assert!(path.metadata().unwrap().len() > 5_000);
    let mut wb = calamine::open_workbook_auto(&path).unwrap();
    let names = wb.sheet_names().to_vec();
    assert!(names.contains(&"Summary".to_string()));
    assert!(names.contains(&"PierShearWind".to_string()));
}

#[cfg(feature = "ssr")]
#[test]
fn render_pdf_produces_valid_pdf() {
    let calc  = load_fixture_calc();
    let dir   = tempfile::tempdir().unwrap();
    let path  = dir.path().join("report.pdf");
    render_pdf(&calc, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"%PDF");
    assert!(bytes.len() > 50_000);
    // No .svg files should exist in temp dir — all images were in-memory
    assert!(std::fs::read_dir(dir.path()).unwrap()
        .filter_map(|e| e.ok())
        .all(|e| e.path().extension().map_or(true, |ext| ext != "svg")));
}
```

### W10 End-of-Week Definition of Done

- [ ] `cargo test -p ext-report` passes (Excel, no ssr)
- [ ] `cargo test -p ext-report --features ssr` passes (full pipeline)
- [ ] `report.xlsx` opens: 9 sheets, styled headers, red DCR cells, governing row highlight
- [ ] `report.pdf` opens: cover + summary + sections with embedded charming SVGs
- [ ] `render_pdf` signature is `(calc, path)` — no `assets_dir` parameter
- [ ] Zero `.svg` files written to disk during PDF generation

---

## Week 11 — `ext-tauri`: AppState and All Commands

**Goal:** Wire `ext-tauri`. Every `invoke()` in the frontend has a real Rust handler. Chart commands return HTML strings. Report generation is a single command.

---

### W11-T1 — AppState (Day 1)

```rust
// src/state.rs
pub struct AppState {
    pub project_path:  Mutex<Option<PathBuf>>,
    pub calc_output:   Mutex<Option<CalcOutput>>,
    pub code_params:   Mutex<Option<CodeParams>>,
    /// Cached chart HTML — set after run_calculations, read by chart commands.
    /// Keyed by check name: "drift_wind", "modal", etc.
    pub chart_cache:   Mutex<HashMap<String, String>>,
}
```

**Locking rule:** Never hold two `AppState` Mutex locks in the same command. Lock → use → drop before taking the next.

The `chart_cache` is populated eagerly after `run_calculations` succeeds — all 7 charts rendered to HTML and cached. Subsequent `get_*_chart` commands read from cache without re-rendering.

---

### W11-T2 — TypeScript types (Day 1)

Minimal `types.rs` — no chart data structs needed since charts are HTML strings:

```rust
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct TauriResult<T: TS + Serialize> {
    pub success: bool,
    pub data:    Option<T>,
    pub error:   Option<String>,
}

// ProjectState, BranchInfo, VersionInfo, WorkingFileInfo,
// EtabsStatus, CalcSummaryResponse, ReportPathsResponse
// — all derive TS for the project/version management UI
```

Chart commands return `TauriResult<String>` — just a string, no TS struct needed.

---

### W11-T3 — Project commands (Day 2)

`open_project`, `get_project_state`, `init_project` — delegate to `ext_api`.

---

### W11-T4 — Branch + Version commands (Days 2–3)

`create_branch`, `switch_branch`, `delete_branch`, `save_version`, `checkout_version`.

---

### W11-T5 — ETABS commands (Day 3)

`open_in_etabs`, `close_etabs`, `get_etabs_status`.

---

### W11-T6 — Calc commands + chart cache population (Day 4)

```rust
#[tauri::command]
pub async fn run_calculations(
    version_id: String,
    branch:     String,
    state: State<'_, AppState>,
) -> Result<TauriResult<CalcSummaryResponse>, String> {
    // 1. Read project_path (lock → clone → drop)
    // 2. results_dir = project_path/.ext/{branch}/{version_id}/results
    // 3. Config::load + CodeParams::from_config
    // 4. CalcRunner::run_all(...)
    // 5. Store CalcOutput (lock → store → drop)
    // 6. Store CodeParams (lock → store → drop)
    // 7. Render all HTML charts eagerly (no ssr needed — HtmlRenderer always available)
    //    let charts = ext_render::render_all_html(&calc, 900, 600)?;
    //    { state.chart_cache.lock().extend(charts); }
    // 8. Return CalcSummaryResponse
}
```

Step 7 is key: chart rendering happens **inside `run_calculations`**, not in a separate command. The frontend calls `run_calculations` once and then all chart commands return instantly from cache.

---

### W11-T7 — Chart commands (Day 4)

```rust
/// Returns pre-rendered HTML for one chart check.
/// All charts are computed and cached during run_calculations.
#[tauri::command]
pub fn get_chart(check: String, state: State<'_, AppState>) -> Result<TauriResult<String>, String> {
    let cache = state.chart_cache.lock();
    match cache.get(&check) {
        Some(html) => Ok(TauriResult::ok(html.clone())),
        None => Ok(TauriResult::err(format!("Chart '{}' not available", check))),
    }
}

/// Returns all chart HTML in one call.
#[tauri::command]
pub fn get_all_charts(state: State<'_, AppState>) -> Result<TauriResult<HashMap<String, String>>, String> {
    let cache = state.chart_cache.lock();
    Ok(TauriResult::ok(cache.clone()))
}
```

One generic `get_chart(check: String)` command serves all 7 chart types. The frontend passes the check name as a string.

---

### W11-T8 — Report commands (Days 4–5)

```rust
#[tauri::command]
pub async fn generate_excel(output_dir: String, name: String, state: State<'_, AppState>)
    -> Result<TauriResult<String>, String>

#[cfg(feature = "ssr")]
#[tauri::command]
pub async fn generate_report(output_dir: String, name: String, state: State<'_, AppState>)
    -> Result<TauriResult<ReportPathsResponse>, String>
```

`generate_report`: reads `CalcOutput` from state → `ext_report::render_all()` → PDF (charming SSR) + Excel → return paths.

---

### W11-T9 — Wire frontend (Day 5)

Update `projectStore.ts` invoke payloads. Remove mock auto-load. Add `calcStore.ts`.

Update `App.tsx`: disable Results tab until `run_calculations` succeeds.

### W11 End-of-Week Definition of Done

- [ ] All `invoke()` in project/branch/version/ETABS stores have Rust handlers
- [ ] `run_calculations` → stores `CalcOutput` + populates `chart_cache` in one call
- [ ] `get_chart("drift_wind")` → returns HTML string from cache, instant
- [ ] `generate_excel` → real `.xlsx`
- [ ] `generate_report` → real `.pdf` + `.xlsx` (no chart refs from frontend)
- [ ] Zero `echarts-for-react` imports in frontend after this week

---

## Week 12 — `ext-tauri`: Results Panel and Report UI

**Goal:** Replace mock ECharts sandbox with real Results panel driven by HTML chart cache. Wire complete report generation flow.

---

### W12-T1 — Chart display components (Days 1–2)

**`components/charts/ChartFrame.tsx`** — universal chart component:

```typescript
interface ChartFrameProps {
    html:    string;    // from invoke('get_chart', { check: 'drift_wind' })
    height?: number;
}

export function ChartFrame({ html, height = 500 }: ChartFrameProps) {
    // Option A: dangerouslySetInnerHTML (simplest)
    return (
        <div
            style={{ width: '100%', height, overflow: 'hidden' }}
            dangerouslySetInnerHTML={{ __html: html }}
        />
    );

    // Option B: srcdoc iframe (better isolation)
    return (
        <iframe
            srcDoc={html}
            style={{ width: '100%', height, border: 'none', background: 'transparent' }}
            sandbox="allow-scripts"
        />
    );
}
```

The `sandbox="allow-scripts"` attribute on the `<iframe>` allows ECharts to initialize while preventing cross-frame navigation. Both options preserve full ECharts interactivity (hover, zoom, legend).

**`hooks/useCharts.ts`:**

```typescript
export function useCharts() {
    const hasCalc = useCalcStore(s => s.calcSummary !== null);
    return useQuery({
        queryKey: ['charts'],
        queryFn:  () => invoke<TauriResult<Record<string, string>>>('get_all_charts'),
        enabled:  hasCalc,
        staleTime: Infinity,
    });
}

export function useChart(check: string) {
    const hasCalc = useCalcStore(s => s.calcSummary !== null);
    return useQuery({
        queryKey: ['chart', check],
        queryFn:  () => invoke<TauriResult<string>>('get_chart', { check }),
        enabled:  hasCalc,
        staleTime: Infinity,
    });
}
```

---

### W12-T2 — Results panel (Days 2–3)

**New sidebar entry:** "Results" (`BarChart3` icon). Visible after `run_calculations` succeeds.

**`components/results/ResultsPanel.tsx`:**

```
SummaryStatusBar         ← overall PASS/FAIL + check count (from calcStore)
Tabs:
  "Summary"              ← table of lines (key/status/message)
  "Drift"                ← ChartFrame for drift_wind | toggle | ChartFrame for drift_seismic
  "Displacement"         ← ChartFrame for displacement_wind
  "Piers"                ← tabs: Shear Wind | Shear Seismic | Axial
  "Modal"                ← ChartFrame for modal
```

Each `ChartFrame` pulls from the `useChart(check)` hook. Loading skeleton while query is in flight.

**Direction toggle for drift:** Both `drift_wind` and `drift_seismic` charts are pre-built with all directions (X+, X-, Y+, Y-) as separate series. ECharts legend toggle handles direction filtering natively — the user clicks the legend to hide/show X or Y direction series. No additional toggle state needed in React.

---

### W12-T3 — Replace EChartsPanel sandbox (Day 3)

Remove `EChartsPanel.tsx` and the `sandbox` sidebar entry. Replace with a pointer to the real Results panel. The sandbox served its purpose during development; it's no longer needed.

---

### W12-T4 — Report panel wiring (Day 4)

**`components/reports/ReportsPanel.tsx`** — replace mock flow:

```typescript
const handleGenerateReport = async () => {
    setIsGenerating(true);
    const dir = await open({ directory: true, title: 'Select output folder' });
    if (!dir) { setIsGenerating(false); return; }

    setProgress('Generating PDF + Excel (may take ~10 seconds)...');
    const result = await invoke<TauriResult<ReportPathsResponse>>('generate_report', {
        outputDir: dir,
        name: 'structural_check',
    });

    if (result.success && result.data) {
        setProgress(`Done! PDF: ${result.data.pdf}`);
        await invoke('open_path', { path: dir });
    } else {
        setProgress(`Error: ${result.error}`);
    }
    setIsGenerating(false);
};

const handleExcelOnly = async () => {
    const dir = await open({ directory: true });
    if (!dir) return;
    setIsGenerating(true);
    await invoke('generate_excel', { outputDir: dir, name: 'structural_check' });
    setIsGenerating(false);
};
```

No chart refs. No `getDataURL()`. No `save_chart_images`. Charts are in `AppState.chart_cache`.

---

### W12-T5 — ECharts local asset bundling (Day 4)

Add `echarts.min.js` to Tauri resources:

```json
// tauri.conf.json
{
  "resources": ["echarts/echarts.min.js"]
}
```

The `render_one_html` function in `ext-render` patches the CDN URL to `asset://localhost/echarts.min.js`. This makes charts work fully offline.

---

### W12-T6 — End-to-end smoke test (Day 5)

1. Open project → checkout version
2. "Run Analysis" → all 7 charts cached in `AppState.chart_cache`
3. Results panel: Summary tab shows real check statuses
4. Drift tab: interactive ECharts chart — hover shows story name, DCR, load case
5. Piers tab: DCR bars colored pass/fail — failing bars orange/red
6. Modal tab: ΣUX and ΣUY lines crossing threshold line visible
7. "Generate Report" → select folder → wait ~10s
8. Open PDF: cover, summary, drift section with embedded chart image (charming SVG)
9. Open Excel: 9 sheets, styled, red DCR cells visible

---

### W12 End-of-Sprint Definition of Done

**Functional:**
- [ ] Results panel shows 5 tabs with real interactive charts
- [ ] Charts are ECharts running in webview — hover/zoom/legend toggle all work
- [ ] Drift chart: X+ Y+ X- Y- series individually toggleable via legend
- [ ] "Generate Report" → PDF + Excel, no frontend chart export required
- [ ] "Excel only" button → fast, ~1s
- [ ] PDF contains charming-rendered SVG charts (visually consistent with interactive HTML)
- [ ] Charts work offline (local echarts.min.js, no CDN dependency)

**Code quality:**
- [ ] `cargo test` and `cargo test --features ssr` both pass across all crates
- [ ] Zero `echarts-for-react` or `echarts` npm packages in `package.json`
- [ ] Zero TypeScript ECharts option builders anywhere in frontend
- [ ] Zero `chart_data::*` or `ts-rs` usage in `ext-render`
- [ ] `AppState.chart_cache` populated by `run_calculations`, read by `get_chart`
- [ ] `charming::Chart` is the single chart definition — `chart_build` is the only place chart logic lives

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `HtmlRenderer` output HTML incompatible with Tauri webview sandbox | Low | High | Test W9-T5 early; `iframe sandbox="allow-scripts"` is the safe fallback |
| ECharts CDN unreachable during demo/field use | Medium | Medium | Bundle `echarts.min.js` locally (W12-T5); CDN only for dev |
| charming `Chart` API changes between 0.6.x patch versions | Low | Low | Pin exact version `"=0.6.0"` in workspace |
| Typst font loading fails on Windows without system fonts | Low | High | Port `echart_charm/src/typst.rs` verbatim — already proven on Windows |
| PDF compile + charming SSR sequential → > 15s | Medium | Medium | Profile after W10; parallelize `render_all_svg` with `rayon` if needed |
| `AppState` Mutex deadlock | Low | High | Single-lock rule documented; lint with `clippy::await_holding_lock` |
| `HtmlRenderer` dark theme not applying correctly | Low | Low | `ChartTheme::Dark` colors set on `chart.background_color()` and all axis labels |

---

*Written: 2026-04-06 (Revision 2 — final)*
*Related: `EXT_CALC_SPEC.md` · `EXT_RENDER_DESIGN.md` · `EXT_REPORT_DESIGN.md`*
