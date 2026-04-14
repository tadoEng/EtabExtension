# ext-report Spec — v2
## Full Redesign: Typst-Native Tables, JSON Data Injection, PageTheme System

**Status:** Ready for implementation  
**Target crates:** `ext-report`, `ext-render`, `ext-calc` (compile fixes)  
**Supersedes:** `EXT_RENDER_DESIGN.md` (report pipeline sections only)  
**Depends on:** `EXT_CALC_SPEC_V4.md` (data shapes), `ext-render` chart constants

---

## The Core Philosophy

> **Data is Rust. Style is Typst. Layout is a theme.**

The pipeline has one job per layer:

| Layer | Owns | Does NOT own |
|---|---|---|
| `ext-calc` | `CalcOutput` — all computed values | Any presentation concern |
| `ext-render` | SVG chart strings keyed by logical name | Page layout, tables, PDF |
| `ext-report` (Rust) | Serializing `CalcOutput` to JSON virtual files | Table styling, column widths, colors |
| `ext-report` (Typst template) | All visual styling — fills, strokes, column fr ratios, fonts | Computation, data transformation |
| `PageTheme` | All measurements that vary by paper size | Content — data stays identical |

**Switching tabloid → A4 = changing one `PageTheme` constant. Zero changes to data or template logic.**

---

## Part 1 — The PageTheme System

### 1.1 `PageTheme` struct

**File:** `crates/ext-report/src/theme.rs`

```rust
/// All measurements and style constants that vary between paper formats.
/// Injected into the Typst world as "theme.json".
/// Changing the theme changes the visual layout — data and template logic are unchanged.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PageTheme {
    // ── Page geometry ────────────────────────────────────────────────────────
    pub page_width: &'static str,        // "17in"
    pub page_height: &'static str,       // "11in"
    pub margin_top: &'static str,        // "0.25in"
    pub margin_left: &'static str,       // "0.25in"
    pub margin_right: &'static str,      // "0.25in"
    pub margin_bottom: &'static str,     // "0.25in"

    // ── Content area ─────────────────────────────────────────────────────────
    // Height of the content_rect (page minus margins minus title block)
    pub content_height: &'static str,    // "9.75in"

    // ── Typography ───────────────────────────────────────────────────────────
    pub body_font: &'static str,         // "Libertinus Serif"
    pub body_size: &'static str,         // "9pt"
    pub title_size: &'static str,        // "14pt"
    pub label_size: &'static str,        // "7pt"  (table col headers, annotations)
    pub caption_size: &'static str,      // "8pt"

    // ── Chart heights per layout type ────────────────────────────────────────
    pub chart_single_h: &'static str,          // "6.8in"
    pub chart_two_col_h: &'static str,         // "6.0in"
    pub chart_with_table_chart_h: &'static str, // "5.7in"  (table-emphasis layout)
    pub chart_with_table_normal_h: &'static str, // "6.4in" (chart-emphasis layout)

    // ── Grid column ratios ────────────────────────────────────────────────────
    // Expressed as Typst fraction strings — Typst parses these directly
    pub two_col_ratio: &'static str,              // "(1fr, 1fr)"
    pub chart_table_emphasized: &'static str,     // "(1.08fr, 0.92fr)"
    pub chart_table_normal: &'static str,         // "(0.82fr, 1.18fr)"

    // ── Title block ───────────────────────────────────────────────────────────
    pub title_block_columns: &'static str,
    // Tabloid: "(1.35in, 3.2in, 4.0in, 1.6in, 2.0in, 3.35in)"
    // A4:      "(0.9in, 2.2in, 2.8in, 1.1in, 1.5in, 1.8in)"

    // ── Spacing ───────────────────────────────────────────────────────────────
    pub section_gap: &'static str,    // "10pt"  — gap between title and content
    pub table_inset: &'static str,    // "5pt"   — cell padding in tables
    pub gutter: &'static str,         // "14pt"  — gap between chart and table columns
}

pub const TABLOID_LANDSCAPE: PageTheme = PageTheme {
    page_width:    "17in",
    page_height:   "11in",
    margin_top:    "0.25in",
    margin_left:   "0.25in",
    margin_right:  "0.25in",
    margin_bottom: "0.25in",
    content_height: "9.75in",

    body_font:    "Libertinus Serif",
    body_size:    "9pt",
    title_size:   "14pt",
    label_size:   "7pt",
    caption_size: "8pt",

    chart_single_h:           "6.8in",
    chart_two_col_h:          "6.0in",
    chart_with_table_chart_h: "5.7in",
    chart_with_table_normal_h: "6.4in",

    two_col_ratio:          "(1fr, 1fr)",
    chart_table_emphasized: "(1.08fr, 0.92fr)",
    chart_table_normal:     "(0.82fr, 1.18fr)",

    title_block_columns: "(1.35in, 3.2in, 4.0in, 1.6in, 2.0in, 3.35in)",

    section_gap: "10pt",
    table_inset: "5pt",
    gutter:      "14pt",
};

pub const A4_PORTRAIT: PageTheme = PageTheme {
    page_width:    "8.27in",
    page_height:   "11.69in",
    margin_top:    "0.75in",
    margin_left:   "0.75in",
    margin_right:  "0.75in",
    margin_bottom: "0.75in",
    content_height: "9.5in",

    body_font:    "Libertinus Serif",
    body_size:    "9pt",
    title_size:   "13pt",
    label_size:   "7pt",
    caption_size: "8pt",

    chart_single_h:            "5.8in",
    chart_two_col_h:           "4.8in",
    chart_with_table_chart_h:  "4.5in",
    chart_with_table_normal_h: "5.2in",

    two_col_ratio:          "(1fr, 1fr)",
    chart_table_emphasized: "(1fr, 1fr)",
    chart_table_normal:     "(0.85fr, 1.15fr)",

    title_block_columns: "(0.9in, 2.2in, 2.8in, 1.1in, 1.5in, 1.8in)",

    section_gap: "8pt",
    table_inset: "4pt",
    gutter:      "10pt",
};
```

`PageTheme` implements `Serialize` so it can be injected as `theme.json` into the Typst world.  
`&'static str` is used throughout — these are compile-time constants, no allocation needed.

---

## Part 2 — JSON Data Injection Architecture

### 2.1 The Bridge Pattern

`json()` in Typst accepts `str | bytes` — including raw bytes injected as a virtual file via `TypstWorld.file()`. This is the mechanism for passing all structured data from Rust to Typst without any string escaping.

**Virtual files injected per render:**

| Virtual path | Content | Typst access |
|---|---|---|
| `theme.json` | `PageTheme` serialized | `#let theme = json("theme.json")` |
| `project.json` | `ReportProjectMeta` serialized | `#let proj = json("project.json")` |
| `modal.json` | `ModalOutput` rows | `#let modal = json("modal.json")` |
| `base_reactions.json` | `BaseReactionsOutput` rows | etc. |
| `story_forces.json` | `StoryForcesOutput` rows | |
| `drift_wind.json` | `DriftWindOutput` (x + y) | |
| `drift_seismic.json` | `DriftSeismicOutput` (x + y) | |
| `displacement_wind.json` | `DisplacementWindOutput` (x + y) | |
| `torsional.json` | `TorsionalOutput` (x + y) | |
| `pier_shear_stress_wind.json` | `PierShearStressOutput` | |
| `pier_shear_stress_seismic.json` | `PierShearStressOutput` | |
| `pier_axial_stress.json` | `PierAxialStressOutput` | |
| `images/*.svg` | SVG bytes from `ext-render` | `image("images/drift_wind_x.svg")` |

### 2.2 `ReportData` — the serialization gateway

**File:** `crates/ext-report/src/data.rs`

```rust
/// Serializes CalcOutput fields into virtual JSON files for Typst.
/// Each field maps to one virtual file path.
pub struct ReportData {
    pub files: HashMap<PathBuf, Bytes>,
}

impl ReportData {
    pub fn from_calc(
        calc: &CalcOutput,
        project: &ReportProjectMeta,
        theme: &PageTheme,
        svg_map: HashMap<String, String>,
    ) -> Result<Self> {
        let mut files = HashMap::new();

        // Theme and project metadata
        files.insert(
            PathBuf::from("theme.json"),
            Bytes::new(serde_json::to_vec(theme)?),
        );
        files.insert(
            PathBuf::from("project.json"),
            Bytes::new(serde_json::to_vec(project)?),
        );

        // Per-check data — only insert if Some
        if let Some(modal) = &calc.modal {
            files.insert(
                PathBuf::from("modal.json"),
                Bytes::new(serde_json::to_vec(modal)?),
            );
        }
        if let Some(base) = &calc.base_reactions {
            files.insert(
                PathBuf::from("base_reactions.json"),
                Bytes::new(serde_json::to_vec(base)?),
            );
        }
        // ... repeat for all checks

        // SVG chart images
        for (name, svg) in svg_map {
            files.insert(
                PathBuf::from(&name),
                Bytes::new(svg.into_bytes()),
            );
        }

        Ok(Self { files })
    }
}
```

`TypstWorld` is updated to use `ReportData.files` as its image/file cache — replacing the current `HashMap<PathBuf, Bytes>` parameter in `render_pdf`. The `World::file()` implementation returns from this map for any path.

### 2.3 Updated `render_pdf` signature

```rust
// crates/ext-report/src/pdf/renderer.rs

pub fn render_pdf(
    calc: &CalcOutput,
    project: &ReportProjectMeta,
    svg_map: HashMap<String, String>,
    theme: &PageTheme,           // NEW — was implicit/hardcoded
) -> Result<Vec<u8>> {
    let data = ReportData::from_calc(calc, project, theme, svg_map)?;
    let source = build_typst_document(calc, theme);  // structural skeleton only
    let world = TypstWorld::new(source, data.files)?;
    // ... compile as before
}
```

The `build_typst_document` function becomes very thin — it only emits:
1. The `#import` / `#let` preamble that loads all JSON files
2. The page `#set` rules (from theme values)
3. The `#show` rules for global styling
4. The page loop with `#pagebreak()` between pages
5. Per-page function calls like `#drift-wind-page()`

All table rendering, column sizing, fill logic, and stroke logic move into the Typst template file.

---

## Part 3 — Typst Template Architecture

### 3.1 Template file structure

The Typst source is now split into logical concerns. `TypstWorld` injects all of them as virtual files:

```
(virtual filesystem injected into TypstWorld)
├── main.typ              ← generated by Rust (thin skeleton)
├── theme.json            ← PageTheme serialized
├── project.json          ← ReportProjectMeta serialized
├── modal.json            ← ModalOutput (if present)
├── base_reactions.json
├── story_forces.json
├── drift_wind.json
├── drift_seismic.json
├── displacement_wind.json
├── torsional.json
├── pier_shear_stress_wind.json
├── pier_shear_stress_seismic.json
├── pier_axial_stress.json
└── images/
    ├── modal.svg
    ├── base_reactions.svg
    ├── story_force_vx.svg
    ├── story_force_my.svg
    ├── story_force_vy.svg
    ├── story_force_mx.svg
    ├── drift_wind_x.svg
    ├── drift_wind_y.svg
    ├── drift_seismic_x.svg
    ├── drift_seismic_y.svg
    ├── disp_wind_x.svg
    ├── disp_wind_y.svg
    ├── torsional_x.svg
    ├── torsional_y.svg
    ├── pier_shear_stress_wind.svg
    ├── pier_shear_stress_seismic.svg
    ├── pier_axial_gravity.svg
    ├── pier_axial_wind.svg
    └── pier_axial_seismic.svg
```

### 3.2 `main.typ` — generated by Rust (skeleton only)

`build_typst_document()` generates this string. It is intentionally minimal:

```typst
// ── Data loading ─────────────────────────────────────────────────────────────
#let theme    = json("theme.json")
#let proj     = json("project.json")
#let modal    = json("modal.json")         // none if file missing → handle in template
#let base     = json("base_reactions.json")
#let sf       = json("story_forces.json")
#let dw       = json("drift_wind.json")
#let ds       = json("drift_seismic.json")
#let disp     = json("displacement_wind.json")
#let tors     = json("torsional.json")
#let psw      = json("pier_shear_stress_wind.json")
#let pss      = json("pier_shear_stress_seismic.json")
#let pa       = json("pier_axial_stress.json")

// ── Page setup (from theme) ───────────────────────────────────────────────────
#set page(
  width:  eval(theme.page-width,  mode: "code"),
  height: eval(theme.page-height, mode: "code"),
  margin: (
    top:    eval(theme.margin-top,    mode: "code"),
    left:   eval(theme.margin-left,   mode: "code"),
    right:  eval(theme.margin-right,  mode: "code"),
    bottom: eval(theme.margin-bottom, mode: "code"),
  ),
)

// ── Typography (from theme) ───────────────────────────────────────────────────
#set text(font: theme.body-font, size: eval(theme.body-size, mode: "code"))
#set par(justify: false)

// ── Figure numbering — suppress auto-numbering ────────────────────────────────
#set figure(numbering: none, outlined: false)

// ── Global table styling ──────────────────────────────────────────────────────
#set table(
  stroke: 0.5pt + luma(180),
  inset: eval(theme.table-inset, mode: "code"),
)
#show table.cell.where(y: 0): set text(
  weight: "bold",
  size: eval(theme.label-size, mode: "code"),
)

// ── Shared helper functions ───────────────────────────────────────────────────
#let pass-fill  = rgb("#d4edda")
#let warn-fill  = rgb("#fff3cd")
#let fail-fill  = rgb("#f8d7da")
#let stripe-fill = luma(248)
#let header-fill = luma(220)

#let row-fill(annotation, y) = {
  if annotation == "fail"          { fail-fill }
  else if annotation == "warn"     { warn-fill }
  else if annotation == "pass"     { pass-fill }
  else if annotation == "high"     { rgb("#e8f5e9") }
  else if annotation == "ux_threshold"    { rgb("#cfe2ff") }
  else if annotation == "uy_threshold"    { rgb("#fff3cd") }
  else if annotation == "ux_uy_threshold" { rgb("#d1c4e9") }
  else if calc.odd(y)              { stripe-fill }
  else                             { none }
}

// ── Title block ───────────────────────────────────────────────────────────────
#let title-block(sheet) = {
  place(bottom + left)[
    #set text(font: theme.body-font)
    #table(
      columns: eval(theme.title-block-columns, mode: "code"),
      stroke: 1pt + black,
      inset: 5pt,
      [
        #align(center + horizon)[
          #stack(spacing: 0pt,
            text(size: 11pt, weight: "bold")[Thornton],
            text(size: 11pt, weight: "bold")[Tomasetti],
          )
        ]
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[PROJECT],
          text(size: 8pt, weight: "bold")[#proj.project-name],
          text(size: 5.5pt, fill: luma(110))[PROJECT NO.],
          text(size: 7.5pt)[#proj.project-number],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[SUBJECT],
          text(size: 8.5pt, weight: "bold")[#proj.subject],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[REFERENCE],
          text(size: 7.5pt)[#proj.reference],
          text(size: 5.5pt, fill: luma(110))[REVISION],
          text(size: 8pt, weight: "bold")[#proj.revision],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[BY],
          text(size: 8pt, weight: "bold")[#proj.engineer],
          text(size: 5.5pt, fill: luma(110))[CHECKED],
          text(size: 8pt, weight: "bold")[#proj.checker],
        )
      ],
      [
        #stack(spacing: 2pt,
          text(size: 5.5pt, fill: luma(110))[DATE],
          text(size: 7.5pt)[#proj.date],
          text(size: 5.5pt, fill: luma(110))[SHEET],
          text(size: 14pt, weight: "bold")[#sheet],
        )
      ],
    )
  ]
}

// ── Content wrapper ───────────────────────────────────────────────────────────
#let content-rect(body) = block(
  width: 100%,
  height: eval(theme.content-height, mode: "code"),
  stroke: 1.2pt + black,
  inset: 18pt,
  clip: true,
  body,
)

// ── Sheet counter ─────────────────────────────────────────────────────────────
#let sheet-num = counter("sheet")

// ── Page wrapper macro ────────────────────────────────────────────────────────
#let page-wrap(body) = {
  sheet-num.step()
  let sheet = context {
    let n = sheet-num.get().first()
    [#proj.sheet-prefix-#str(n).zero-pad(2)]
  }
  title-block(sheet)
  content-rect(body)
}

// ══════════════════════════════════════════════════════════════════════════════
// PAGES — each section function defined below, called here in order
// ══════════════════════════════════════════════════════════════════════════════
#page-wrap(summary-page())
// ... one #pagebreak() + #page-wrap(section-fn()) per section, emitted by Rust
```

> **Note on `eval()`:** `eval(theme.page-width, mode: "code")` converts the JSON string `"17in"` into a Typst `length` value. This is the one accepted use of `eval()` — converting theme measurement strings into typed Typst values. It is called exactly once per measurement, at the top of the document.

**Alternative (no eval):** Emit the `#set page(...)` block directly from Rust using theme field values, keeping all other styling in Typst. This is safer and avoids `eval()`. Recommended approach:

```rust
// In build_typst_document() — emit page setup directly from theme:
format!(
    "#set page(width: {w}, height: {h}, margin: (top: {mt}, left: {ml}, right: {mr}, bottom: {mb}))\n",
    w = theme.page_width, h = theme.page_height,
    mt = theme.margin_top, ml = theme.margin_left,
    mr = theme.margin_right, mb = theme.margin_bottom,
)
```

Only the `#set page`, `#set text(font:..., size:...)`, `content-rect height`, `title-block columns`, chart heights, and grid ratios are emitted from Rust theme values. Everything else is pure Typst.

### 3.3 The Typst table pattern — no Rust cell emission

The key change: Rust no longer emits individual `table.cell(fill: ...)` strings. Instead, Typst's `fill` function form handles all per-cell styling.

**Pattern for every data table:**

```typst
// Called from Typst template with data loaded from JSON
#let render-drift-table(data, limit) = {
  let rows = data.rows
  let annotations = data.annotations  // parallel array from JSON

  #table(
    columns: (auto, 1fr, auto, auto, auto),
    fill: (x, y) => {
      if y == 0 { header-fill }
      else { row-fill(annotations.at(y - 1, default: ""), y) }
    },
    align: (x, y) => if x >= 2 { right } else { left },
    stroke: (x, y) => if y == 0 { (bottom: 1pt + black) } else { 0.5pt + luma(180) },

    table.header[Story][Case][Demand][Limit][DCR],
    ..rows.map(r => (
      r.story, r.output-case,
      str(calc.round(r.drift-ratio, digits: 5)),
      str(calc.round(limit, digits: 5)),
      str(calc.round(r.dcr, digits: 3)),
    )).flatten()
  )
}
```

**Key Typst patterns used:**
- `fill: (x, y) => ...` — function form for per-cell fill based on position
- `align: (x, y) => ...` — right-align numeric columns (x >= 2), left-align labels
- `stroke: (x, y) => ...` — heavier underline on header row only
- `table.header[...][...][...]` — semantic header for accessibility and repeat-on-page
- `..rows.map(...).flatten()` — spread array into table cells (from CSV guide pattern)
- `calc.round(value, digits: N)` — format numbers in Typst, not Rust

### 3.4 The `annotations` contract

Rust produces a parallel `annotations` array alongside table rows. Each entry is one of:
`"pass"`, `"fail"`, `"warn"`, `"high"`, `"ux_threshold"`, `"uy_threshold"`, `"ux_uy_threshold"`, `""` (no annotation).

This array is serialized into the JSON alongside the rows and indexed by `y - 1` (y=0 is the header row) in the Typst `fill` function.

**Example JSON shape for drift:**
```json
{
  "allowable_ratio": 0.0025,
  "rows": [
    { "story": "L10", "output_case": "W_10YRS", "drift_ratio": 0.00183, "dcr": 0.732 },
    ...
  ],
  "annotations": ["", "warn", "fail", ""],
  "governing": { "story": "L08", "direction": "X", "dcr": 1.02, "pass": false }
}
```

The annotations array is computed in Rust (same logic as current `render_table`) but emitted as data, not as embedded Typst strings.

---

## Part 4 — Page Section Catalogue

Each section is a Typst function defined in `main.typ`. Rust decides which pages to include and emits the call sequence. The Typst function reads its data from the pre-loaded JSON variables.

### Layout types

| Layout | Typst pattern | Used for |
|---|---|---|
| `summary-page()` | Full content-rect, key-value list | Report summary |
| `table-only-page(title, table-fn)` | Title + table spanning content rect | Modal, base reactions |
| `single-chart-page(title, img)` | Title + `figure(image(...))` centered | Pier shear stress wind/seismic |
| `two-charts-page(title, img-a, img-b)` | Title + `grid(1fr 1fr)` of two figures | Story forces X, Story forces Y |
| `chart-and-table-page(title, img, table-fn, emphasis)` | Title + `grid(fr fr)` chart + table | Drift, displacement, torsional, pier axial |
| `calculation-page(title, blocks)` | Title + two-column prose blocks | Pier axial assumptions |

### Section registry (in report page order)

```
Page 01 — Summary
Page 02 — Modal Participation (table-only)
Page 03 — Base Reactions (chart + table, chart-emphasis)
Page 04 — Story Forces X (two-charts: VX + MY)
Page 05 — Story Forces Y (two-charts: VY + MX)
Page 06 — Wind Drift X (chart + table, table-emphasis)
Page 07 — Wind Drift Y (chart + table, table-emphasis)
Page 08 — Seismic Drift X (chart + table, table-emphasis)
Page 09 — Seismic Drift Y (chart + table, table-emphasis)
Page 10 — Wind Displacement X (chart + table, table-emphasis)
Page 11 — Wind Displacement Y (chart + table, table-emphasis)
Page 12 — Torsional X (chart + table, table-emphasis)
Page 13 — Torsional Y (chart + table, table-emphasis)
Page 14 — Pier Shear Stress Wind (single-chart)
Page 15 — Pier Shear Stress Seismic (single-chart)
Page 16 — Pier Axial Gravity (chart + table, chart-emphasis)
Page 17 — Pier Axial Wind (chart + table, chart-emphasis)
Page 18 — Pier Axial Seismic (chart + table, chart-emphasis)
Page 19 — Pier Axial Assumptions (calculation-page)
```

Pages are skipped if their check output is `None` in `CalcOutput`. Rust determines which pages to emit.

### Page skip logic in Rust

```rust
// In build_typst_document() — emit page calls conditionally:
let mut pages: Vec<&str> = vec!["summary-page()"];

if calc.modal.is_some()                   { pages.push("modal-page()"); }
if calc.base_reactions.is_some()          { pages.push("base-reactions-page()"); }
if calc.story_forces.is_some() {
    pages.push("story-forces-x-page()");
    pages.push("story-forces-y-page()");
}
if calc.drift_wind.is_some() {
    pages.push("drift-wind-x-page()");
    pages.push("drift-wind-y-page()");
}
// ... etc.

// Emit:
let body = pages.iter()
    .enumerate()
    .map(|(i, call)| {
        if i == 0 { format!("#page-wrap({call})")  }
        else       { format!("\n#pagebreak()\n#page-wrap({call})") }
    })
    .collect::<String>();
```

---

## Part 5 — `ReportProjectMeta` Serialization

`ReportProjectMeta` gets `Serialize` derived so it can be emitted as `project.json`:

```rust
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ReportProjectMeta {
    pub project_name: String,
    pub project_number: String,
    pub reference: String,
    pub engineer: String,
    pub checker: String,
    pub date: String,
    pub subject: String,
    pub scale: String,
    pub revision: String,
    pub sheet_prefix: String,
}
```

Fields in Typst are accessed as `proj.project-name`, `proj.sheet-prefix`, etc. (JSON kebab-case keys).

**No `escape_text()` needed in Rust.** Typst reads field values from JSON — JSON strings are natively safe from Typst markup injection. The `escape_text()` function is deleted entirely.

---

## Part 6 — `ext-render` Integration Contract

### 6.1 SVG map keys

`render_all_svg()` returns `HashMap<String, String>` keyed by logical name. These keys must exactly match the virtual paths used in `image()` calls in the Typst template:

```rust
// ext-render/src/render_svg/mod.rs — authoritative key list:
pub const MODAL_SVG:                  &str = "images/modal.svg";
pub const BASE_REACTIONS_SVG:         &str = "images/base_reactions.svg";
pub const STORY_FORCE_VX_SVG:         &str = "images/story_force_vx.svg";
pub const STORY_FORCE_MY_SVG:         &str = "images/story_force_my.svg";
pub const STORY_FORCE_VY_SVG:         &str = "images/story_force_vy.svg";
pub const STORY_FORCE_MX_SVG:         &str = "images/story_force_mx.svg";
pub const DRIFT_WIND_X_SVG:           &str = "images/drift_wind_x.svg";
pub const DRIFT_WIND_Y_SVG:           &str = "images/drift_wind_y.svg";
pub const DRIFT_SEISMIC_X_SVG:        &str = "images/drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_SVG:        &str = "images/drift_seismic_y.svg";
pub const DISP_WIND_X_SVG:            &str = "images/disp_wind_x.svg";
pub const DISP_WIND_Y_SVG:            &str = "images/disp_wind_y.svg";
pub const TORSIONAL_X_SVG:            &str = "images/torsional_x.svg";
pub const TORSIONAL_Y_SVG:            &str = "images/torsional_y.svg";
pub const PIER_SHEAR_STRESS_WIND_SVG: &str = "images/pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEIS_SVG: &str = "images/pier_shear_stress_seismic.svg";
pub const PIER_AXIAL_GRAVITY_SVG:     &str = "images/pier_axial_gravity.svg";
pub const PIER_AXIAL_WIND_SVG:        &str = "images/pier_axial_wind.svg";
pub const PIER_AXIAL_SEISMIC_SVG:     &str = "images/pier_axial_seismic.svg";
```

These replace `BASE_SHEAR_IMAGE`, `DRIFT_WIND_IMAGE`, etc. from the old `chart_build/mod.rs`.

### 6.2 `render_all_svg` signature (updated)

```rust
// ext-render/src/render_svg/mod.rs
#[cfg(feature = "ssr")]
pub fn render_all_svg(
    calc: &CalcOutput,
    theme: &PageTheme,   // drives chart width/height dimensions
) -> Result<HashMap<String, String>> {
    // Uses theme.chart_single_h etc. to set ImageRenderer dimensions
}
```

The `PageTheme` drives chart pixel dimensions in `ext-render`. Report and chart dimensions stay synchronized by sharing the same theme instance.

### 6.3 Chart pixel dimensions from theme

Theme stores dimensions as Typst strings (`"6.8in"`). For `ImageRenderer`, convert to pixels at 96dpi:

```rust
fn parse_theme_height_px(s: &str) -> u32 {
    // "6.8in" → 6.8 * 96.0 as u32 = 652
    let inches: f64 = s.trim_end_matches("in").parse().unwrap_or(6.0);
    (inches * 96.0) as u32
}
```

Width is fixed at 900px for all report charts (matches the content rect width at tabloid scale).

---

## Part 7 — Compile Fixes Required Before This Spec Applies

These must be done first as they are blocking compilation:

### 7.1 `ext-report/src/report_document.rs`

**Delete entirely.** Replace with `data.rs` (the `ReportData` builder from Part 2). The old file imports `BaseShearOutput` (deleted type) and uses stale field names on `CalcOutput`.

### 7.2 `ext-report/src/pdf/template.rs`

**Gut and rewrite.** Keep `build_typst_document()` signature but replace body with the new skeleton emitter from Part 3.2. **Delete `escape_text()`** — no longer needed. Update tests to reflect new structure.

### 7.3 `ext-report/src/pdf/sections/`

**Delete all files.** The section rendering logic moves into the Typst template. The `sections/` directory becomes empty and is removed. `KeyValueTable`, `ChartRef`, `ReportSection`, `ReportDocument` types in `report_types.rs` are also deleted — they are replaced by the JSON data contract.

### 7.4 `ext-render/src/lib.rs`

Replace stale exports:
```rust
// DELETE all of:
pub use chart_build::{BASE_SHEAR_IMAGE, DRIFT_WIND_IMAGE, ...};

// REPLACE with:
pub use render_svg::{
    MODAL_SVG, BASE_REACTIONS_SVG, DRIFT_WIND_X_SVG, DRIFT_WIND_Y_SVG,
    // ... all 19 constants
};
```

### 7.5 `ext-render/src/chart_build/mod.rs`

Remove `build_report_charts()` function and all references to deleted `CalcOutput` fields (`base_shear`, `pier_shear_wind`, `pier_shear_seismic`). Replace with updated field names from v4 spec.

### 7.6 `ext-calc` — `pier_shear_stress.rs`

Fix field names: `phi_shear` → `phi_v`, `load_combos` → `combos`.  
Fix stress ratio formula: `stress_psi / fc_sqrt` (not `stress_psi / (limit * fc_sqrt)`).  
Fix `fc_map` key to `(pier_name, story_name)` tuple.

### 7.7 `ext-calc` — `displacement_wind.rs`

Fix displacement limit calculation: use `max_elev` (roof height) not `story_height` (governing story elevation).

### 7.8 `ext-calc/tests/fixtures/results_realistic/calc_output.json`

Regenerate after compile fixes pass. The fixture has stale field names (`base_shear` → `base_reactions`, etc.) and will fail deserialization tests.

---

## Part 8 — Module Structure After Refactor

```
crates/ext-report/src/
├── lib.rs                    ← pub fn render_pdf(..., theme: &PageTheme) -> Result<Vec<u8>>
├── main.rs                   ← CLI entry point (unchanged interface)
├── theme.rs                  ← PageTheme + TABLOID_LANDSCAPE + A4_PORTRAIT
├── data.rs                   ← ReportData::from_calc() → HashMap<PathBuf, Bytes>
└── pdf/
    ├── mod.rs                ← pub use renderer::{render_pdf, write_pdf}
    ├── renderer.rs           ← TypstWorld + compile pipeline (updated to use ReportData)
    └── template.rs           ← build_typst_document() — thin skeleton emitter only

crates/ext-render/src/
├── lib.rs                    ← updated exports
├── chart_build/
│   ├── mod.rs                ← ChartConfig, colors, build_report_charts (updated)
│   ├── drift.rs
│   ├── displacement.rs
│   ├── story_forces.rs       ← NEW
│   ├── torsional.rs          ← NEW
│   ├── pier_shear_stress.rs  ← NEW (replaces pier_shear.rs)
│   ├── pier_axial.rs         ← UPDATED (3 categories)
│   ├── base_reaction.rs      ← RENAMED from base_shear.rs
│   └── modal.rs
├── render_html/mod.rs
└── render_svg/mod.rs         ← updated with all 19 SVG key constants
```

---

## Part 9 — Public API of `ext-report`

```rust
// crates/ext-report/src/lib.rs

pub use theme::{PageTheme, TABLOID_LANDSCAPE, A4_PORTRAIT};
pub use data::ReportData;
pub use pdf::{render_pdf, write_pdf};

/// Primary entry point.
/// svg_map comes from ext_render::render_all_svg(calc, theme).
pub fn render_pdf(
    calc: &CalcOutput,
    project: &ReportProjectMeta,
    svg_map: HashMap<String, String>,
    theme: &PageTheme,
) -> Result<Vec<u8>>;

/// Write PDF bytes to disk.
pub fn write_pdf(path: &Path, pdf_bytes: &[u8]) -> Result<()>;
```

The caller (ext-api or ext-tauri) orchestrates:
```rust
let svg_map = ext_render::render_all_svg(&calc, &TABLOID_LANDSCAPE)?;
let pdf     = ext_report::render_pdf(&calc, &project, svg_map, &TABLOID_LANDSCAPE)?;
ext_report::write_pdf(&output_path, &pdf)?;
```

---

## Part 10 — Implementation Order

Work in this sequence. Each step must compile before proceeding.

1. **`ext-calc` compile fixes** (Part 7.6, 7.7) — fix `pier_shear_stress.rs` field names, ratio formula, `fc_map` key, displacement limit. Run `cargo check -p ext-calc`.

2. **`ext-report/src/theme.rs`** — add `PageTheme` struct + two constants. No dependencies.

3. **`ext-render/src/render_svg/mod.rs`** — add 19 SVG key constants. Update `lib.rs` exports (Part 7.4).

4. **`ext-render/src/chart_build/mod.rs`** — fix `build_report_charts()` field references (Part 7.5). Run `cargo check -p ext-render`.

5. **`ext-report/src/data.rs`** — implement `ReportData::from_calc()`. Derive `Serialize` on `ReportProjectMeta`. Run `cargo check -p ext-report`.

6. **`ext-report/src/pdf/renderer.rs`** — update `TypstWorld` to use `ReportData.files`. Update `render_pdf` signature to accept `theme: &PageTheme`.

7. **`ext-report/src/pdf/template.rs`** — rewrite `build_typst_document()` to emit the new skeleton. Delete `escape_text()`. Delete `report_document.rs` and `pdf/sections/`. Delete `report_types.rs`.

8. **Typst template tables** — implement all section functions inside `main.typ`. Start with summary, modal, and one drift page as the working proof.

9. **Regenerate `calc_output.json` fixture** — run `cargo test -p ext-calc` to produce a fresh fixture via the integration test.

10. **`ext-report` tests** — update `render_pdf_returns_pdf_bytes` and `typst_document_uses_tabloid_landscape` for new API. Add `render_pdf_a4_produces_pdf_bytes` for theme switching proof.

11. **`cargo test --workspace`** — all tests pass.

---

## Part 11 — What Is Explicitly Deleted

| Deleted | Reason |
|---|---|
| `report_types.rs` — `ReportSection`, `ReportDocument`, `KeyValueTable`, `ChartRef` | Replaced by JSON data contract |
| `report_document.rs` — `build_report_document()` | Replaced by `ReportData::from_calc()` |
| `pdf/sections/` — all 5 files | Table rendering moves to Typst |
| `escape_text()` | JSON strings are markup-safe by nature |
| `BASE_SHEAR_IMAGE`, `DRIFT_WIND_IMAGE`, etc. (7 old constants) | Replaced by 19 typed SVG key constants |
| Per-cell `table.cell(fill: ...)` emission in Rust | Replaced by Typst `fill: (x, y) => ...` |
| Hardcoded measurement strings in section renderers | Replaced by `PageTheme` fields |

---

## Invariants

- `PageTheme` is the only place paper-size-specific measurements live. Any hardcoded `"6.8in"` string outside `theme.rs` is a violation.
- All numeric formatting (rounding, decimal places) happens in Typst via `calc.round()` and `str()`. Rust emits raw floats in JSON.
- `escape_text()` must not be re-introduced. If a string needs to go into Typst markup, it goes via JSON — never via string interpolation.
- `render_all_svg()` and `render_pdf()` must receive the same `PageTheme` instance. Chart dimensions and page layout are coupled through the theme.
- The Typst `#set figure(numbering: none, outlined: false)` rule must appear at document level. No `figure()` call in the template should produce auto-numbering.
- `table.header(...)` must wrap the header row of every data table — required for Typst accessibility and for `repeat: true` behavior on multi-page tables.