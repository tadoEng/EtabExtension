# ext-report Spec — v3
## Full Redesign: Typst-Native Tables, JSON Data Injection, PageTheme System

**Status:** Ready for implementation — Typst layout validated by working demo  
**Target crates:** `ext-report`, `ext-render`, `ext-calc` (compile fixes)  
**Supersedes:** v2 spec, `EXT_RENDER_DESIGN.md` (report pipeline sections)  
**Validated:** Title block + border math confirmed working in Typst playground

---

## The Core Philosophy

> **Data is Rust. Style is Typst. Layout is a theme.**

| Layer | Owns | Does NOT own |
|---|---|---|
| `ext-calc` | `CalcOutput` — all computed values | Any presentation concern |
| `ext-render` | SVG chart strings keyed by logical name | Page layout, tables, PDF |
| `ext-report` (Rust) | Serializing `CalcOutput` to JSON virtual files | Table styling, column widths, colors |
| `ext-report` (Typst template) | All visual styling — fills, strokes, column fr, fonts, layout math | Computation, data transformation |
| `PageTheme` | All measurements that vary by paper size | Content — data stays identical across themes |

**Switching tabloid → A4 = changing one `PageTheme` constant. Zero changes to data, zero changes to template logic.**

---

## Part 1 — The PageTheme System

### 1.1 Validated Layout Math

The demo proved the exact relationship between all page measurements:

```
tb-h          = page-height - margin-top - margin-bottom - content-height
              = 11in - 0.25in - 0.25in - 9.75in  =  0.75in  ✓

text-m-top    = margin-top + content-inset
text-m-left   = margin-left + content-inset
text-m-right  = margin-right + content-inset
text-m-bottom = margin-bottom + tb-h + content-inset   (clears title block + padding)

border-w      = page-width - margin-left - margin-right
```

These relationships are **invariants**. Any `PageTheme` must satisfy them.
The Typst template derives `tb-h` and text margins — Rust does not need these derived values.

### 1.2 `PageTheme` struct

**File:** `crates/ext-report/src/theme.rs`

```rust
/// All measurements that vary between paper formats.
/// Injected into TypstWorld as "theme.json".
/// Changing the theme changes visual layout — data and template logic are unchanged.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PageTheme {
    // ── Page geometry ─────────────────────────────────────────────────────────
    pub page_width:    &'static str,   // "17in"
    pub page_height:   &'static str,   // "11in"
    pub margin_top:    &'static str,   // "0.25in"
    pub margin_left:   &'static str,   // "0.25in"
    pub margin_right:  &'static str,   // "0.25in"
    pub margin_bottom: &'static str,   // "0.25in"

    // ── Content area ──────────────────────────────────────────────────────────
    // Invariant: content-height + margin-top + margin-bottom + tb-h = page-height
    // tb-h is derived by the Typst template — not stored here
    pub content_height: &'static str,  // "9.75in"

    // ── Typography ────────────────────────────────────────────────────────────
    pub body_font:    &'static str,    // "Linux Libertine"
    pub body_size:    &'static str,    // "9pt"
    pub title_size:   &'static str,    // "14pt"
    pub label_size:   &'static str,    // "7pt"  (table header row text)
    pub caption_size: &'static str,    // "8pt"

    // ── Chart heights per layout type ─────────────────────────────────────────
    pub chart_single_h:            &'static str,  // "8.5in"
    pub chart_two_col_h:           &'static str,  // "7.5in"
    pub chart_with_table_chart_h:  &'static str,  // "6in"   (chart-emphasis: chart larger)
    pub chart_with_table_normal_h: &'static str,  // "7in"   (table-emphasis: table larger)

    // ── Grid column ratios (Typst fraction strings) ───────────────────────────
    pub two_col_ratio:          &'static str,  // "(1fr, 1fr)"
    pub chart_table_emphasized: &'static str,  // "(1.08fr, 0.92fr)"  chart left, table right
    pub chart_table_normal:     &'static str,  // "(0.82fr, 1.18fr)"  chart left, table right

    // ── Title block ───────────────────────────────────────────────────────────
    // Column widths — must sum to border-w (page-width - margin-left - margin-right)
    pub title_block_columns: &'static str,

    // ── Spacing ───────────────────────────────────────────────────────────────
    pub section_gap:   &'static str,  // "10pt"  space below section heading
    pub table_inset:   &'static str,  // "5pt"   cell padding in tables
    pub grid_gutter:   &'static str,  // "20pt"  gap between chart and table columns
    pub content_inset: &'static str,  // "18pt"  padding inside border rect (drives text margins)
}

pub const TABLOID_LANDSCAPE: PageTheme = PageTheme {
    page_width:    "17in",
    page_height:   "11in",
    margin_top:    "0.25in",
    margin_left:   "0.25in",
    margin_right:  "0.25in",
    margin_bottom: "0.25in",
    content_height: "9.75in",
    // tb-h = 11 - 0.25 - 0.25 - 9.75 = 0.75in ✓

    body_font:    "Linux Libertine",
    body_size:    "9pt",
    title_size:   "14pt",
    label_size:   "7pt",
    caption_size: "8pt",

    chart_single_h:            "8.5in",
    chart_two_col_h:           "7.5in",
    chart_with_table_chart_h:  "6in",
    chart_with_table_normal_h: "7in",

    two_col_ratio:          "(1fr, 1fr)",
    chart_table_emphasized: "(1.08fr, 0.92fr)",
    chart_table_normal:     "(0.82fr, 1.18fr)",

    // Sum = 16.5in = 17in - 0.25in - 0.25in ✓
    title_block_columns: "(3.35in, 3.2in, 4.0in, 1.6in, 2.0in, 2.35in)",

    section_gap:   "10pt",
    table_inset:   "5pt",
    grid_gutter:   "20pt",
    content_inset: "18pt",
};

pub const A4_PORTRAIT: PageTheme = PageTheme {
    page_width:    "8.27in",
    page_height:   "11.69in",
    margin_top:    "0.25in",
    margin_left:   "0.25in",
    margin_right:  "0.25in",
    margin_bottom: "0.25in",
    content_height: "10.44in",
    // tb-h = 11.69 - 0.25 - 0.25 - 10.44 = 0.75in ✓

    body_font:    "Linux Libertine",
    body_size:    "9pt",
    title_size:   "13pt",
    label_size:   "7pt",
    caption_size: "8pt",

    chart_single_h:            "7.5in",
    chart_two_col_h:           "6.5in",
    chart_with_table_chart_h:  "5in",
    chart_with_table_normal_h: "6in",

    two_col_ratio:          "(1fr, 1fr)",
    chart_table_emphasized: "(1fr, 1fr)",
    chart_table_normal:     "(0.85fr, 1.15fr)",

    // Sum = 7.47in ≈ 8.27in - 0.25in - 0.25in ✓
    title_block_columns: "(1.6in, 1.5in, 2.0in, 0.7in, 0.9in, 0.77in)",

    section_gap:   "8pt",
    table_inset:   "4pt",
    grid_gutter:   "14pt",
    content_inset: "14pt",
};
```

---

## Part 2 — JSON Data Injection Architecture

### 2.1 The Bridge Pattern

`json(bytes)` in Typst accepts raw bytes injected via `TypstWorld.file()`.
No string escaping. No Typst literal generation for data. JSON strings are inherently markup-safe.

**Virtual files injected per render:**

| Virtual path | Rust source | Typst `json()` call location |
|---|---|---|
| `theme.json` | `PageTheme` serialized | top of `main.typ` |
| `project.json` | `ReportProjectMeta` serialized | top of `main.typ` |
| `modal.json` | `ModalOutput` | inside `modal-page()` |
| `base_reactions.json` | `BaseReactionsOutput` | inside `base-reactions-page()` |
| `story_forces.json` | `StoryForcesOutput` | inside `story-forces-*-page()` |
| `drift_wind.json` | `DriftWindOutput` | inside `drift-wind-*-page()` |
| `drift_seismic.json` | `DriftSeismicOutput` | inside `drift-seismic-*-page()` |
| `displacement_wind.json` | `DisplacementWindOutput` | inside `displacement-wind-*-page()` |
| `torsional.json` | `TorsionalOutput` | inside `torsional-*-page()` |
| `pier_shear_stress_wind.json` | `PierShearStressOutput` | inside `pier-shear-stress-wind-page()` |
| `pier_shear_stress_seismic.json` | `PierShearStressOutput` | inside `pier-shear-stress-seismic-page()` |
| `pier_axial_stress.json` | `PierAxialStressOutput` | inside `pier-axial-*-page()` |
| `images/*.svg` | from `ext-render::render_all_svg()` | inside each chart section |

Only files for present checks are injected. `json()` calls live **inside section functions** so no file is accessed without a corresponding Rust call emitting that section.

### 2.2 `annotations` contract

Each check's JSON includes a parallel `annotations` array alongside `rows`.
Values: `"pass"`, `"fail"`, `"warn"`, `"high"`, `"ux_threshold"`, `"uy_threshold"`, `"ux_uy_threshold"`, `""`.

Rust computes annotations:
- `dcr >= 1.0` → `"fail"`
- `dcr >= 0.85` → `"warn"`
- governing row (passing) → `"pass"`
- else → `""`

Accessed in Typst as `data.annotations.at(y - 1, default: "")` where `y=0` is the header.

### 2.3 `ReportData` — the serialization gateway

**File:** `crates/ext-report/src/data.rs`

```rust
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

        // Always present
        files.insert(pb("theme.json"),   to_json(theme)?);
        files.insert(pb("project.json"), to_json(project)?);

        // Per-check — only inject if Some
        if let Some(v) = &calc.modal                     { files.insert(pb("modal.json"),                    to_json(v)?); }
        if let Some(v) = &calc.base_reactions            { files.insert(pb("base_reactions.json"),            to_json(v)?); }
        if let Some(v) = &calc.story_forces              { files.insert(pb("story_forces.json"),              to_json(v)?); }
        if let Some(v) = &calc.drift_wind                { files.insert(pb("drift_wind.json"),                to_json(v)?); }
        if let Some(v) = &calc.drift_seismic             { files.insert(pb("drift_seismic.json"),             to_json(v)?); }
        if let Some(v) = &calc.displacement_wind         { files.insert(pb("displacement_wind.json"),         to_json(v)?); }
        if let Some(v) = &calc.torsional                 { files.insert(pb("torsional.json"),                 to_json(v)?); }
        if let Some(v) = &calc.pier_shear_stress_wind    { files.insert(pb("pier_shear_stress_wind.json"),    to_json(v)?); }
        if let Some(v) = &calc.pier_shear_stress_seismic { files.insert(pb("pier_shear_stress_seismic.json"), to_json(v)?); }
        if let Some(v) = &calc.pier_axial_stress         { files.insert(pb("pier_axial_stress.json"),         to_json(v)?); }

        // SVG charts from ext-render
        for (name, svg) in svg_map {
            files.insert(PathBuf::from(&name), Bytes::new(svg.into_bytes()));
        }

        Ok(Self { files })
    }
}

fn pb(s: &str) -> PathBuf { PathBuf::from(s) }

fn to_json<T: Serialize>(v: &T) -> Result<Bytes> {
    Ok(Bytes::new(serde_json::to_vec(v)?))
}
```

### 2.4 Updated `render_pdf`

```rust
// crates/ext-report/src/pdf/renderer.rs
pub fn render_pdf(
    calc: &CalcOutput,
    project: &ReportProjectMeta,
    svg_map: HashMap<String, String>,
    theme: &PageTheme,
) -> Result<Vec<u8>> {
    let data   = ReportData::from_calc(calc, project, theme, svg_map)?;
    let source = build_typst_document(calc, theme);
    let world  = TypstWorld::new(source, data.files)?;
    let result = typst::compile(&world);
    let compiled = result.output.map_err(|errors| {
        anyhow::anyhow!("typst failed:\n{}",
            errors.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join("\n"))
    })?;
    typst_pdf::pdf(&compiled, &PdfOptions::default())
        .map_err(|e| anyhow::anyhow!("PDF render failed: {e:?}"))
}
```

`TypstWorld.file()` serves all keys from `data.files` — SVG images and JSON data share the same virtual filesystem.

---

## Part 3 — Validated Typst Template

### 3.1 Page setup — exact validated pattern

This is the pattern from the working demo. **Do not deviate from this structure.**

```typst
// ── Load theme and project (always present) ────────────────────────────────
#let theme = json("theme.json")
#let proj  = json("project.json")

// ── Parse theme measurements ───────────────────────────────────────────────
#let pg-w     = eval(theme.page-width,     mode: "code")
#let pg-h     = eval(theme.page-height,    mode: "code")
#let m-top    = eval(theme.margin-top,     mode: "code")
#let m-left   = eval(theme.margin-left,    mode: "code")
#let m-right  = eval(theme.margin-right,   mode: "code")
#let m-bottom = eval(theme.margin-bottom,  mode: "code")
#let c-h      = eval(theme.content-height, mode: "code")
#let inset    = eval(theme.content-inset,  mode: "code")

// ── Derive layout (Typst computes, not Rust) ───────────────────────────────
#let border-w  = pg-w - m-left - m-right
#let tb-h      = pg-h - m-top - m-bottom - c-h   // title block exact height

// ── Text area margins ──────────────────────────────────────────────────────
#let text-m-top    = m-top + inset
#let text-m-left   = m-left + inset
#let text-m-right  = m-right + inset
#let text-m-bottom = m-bottom + tb-h + inset   // clears title block + inset padding

// ── Title block ────────────────────────────────────────────────────────────
#let title-block(sheet) = table(
  columns: eval(theme.title-block-columns, mode: "code"),
  rows: tb-h,           // forces exact height — does not grow with content
  stroke: 1.2pt + black,
  inset: 8pt,
  align: top + left,

  align(center + horizon)[
    #stack(spacing: 5pt,
      text(size: 15pt, weight: "bold")[Thornton],
      text(size: 15pt, weight: "bold")[Tomasetti],
    )
  ],
  stack(spacing: 4pt,
    text(size: 8pt, fill: luma(110))[PROJECT],
    text(size: 10pt, weight: "bold")[#proj.project-name],
    v(4pt),
    text(size: 8pt, fill: luma(110))[PROJECT NO.],
    text(size: 10pt)[#proj.project-number],
  ),
  stack(spacing: 4pt,
    text(size: 8pt, fill: luma(110))[SUBJECT],
    text(size: 10pt, weight: "bold")[#proj.subject],
  ),
  stack(spacing: 4pt,
    text(size: 5.5pt, fill: luma(110))[REFERENCE],
    text(size: 7.5pt)[#proj.reference],
    v(4pt),
    text(size: 5.5pt, fill: luma(110))[REVISION],
    text(size: 8pt, weight: "bold")[#proj.revision],
  ),
  stack(spacing: 4pt,
    text(size: 5.5pt, fill: luma(110))[BY],
    text(size: 8pt, weight: "bold")[#proj.engineer],
    v(4pt),
    text(size: 5.5pt, fill: luma(110))[CHECKED],
    text(size: 8pt, weight: "bold")[#proj.checker],
  ),
  stack(spacing: 4pt,
    text(size: 5.5pt, fill: luma(110))[DATE],
    text(size: 7.5pt)[#proj.date],
    v(4pt),
    text(size: 5.5pt, fill: luma(110))[SHEET],
    text(size: 14pt, weight: "bold")[#sheet],
  ),
)

// ── Page setup ─────────────────────────────────────────────────────────────
// background: carries the border frame + title block as one atomic unit.
// This is the validated approach — not place(), not foreground.
#set page(
  width:  pg-w,
  height: pg-h,
  margin: (
    top:    text-m-top,
    bottom: text-m-bottom,
    left:   text-m-left,
    right:  text-m-right,
  ),
  background: pad(
    top: m-top, bottom: m-bottom,
    left: m-left, right: m-right,
    align(top)[
      #stack(spacing: 0pt,
        // Content border — no bottom stroke; title block provides it
        rect(
          width: border-w,
          height: c-h,
          stroke: (top: 1.2pt + black, left: 1.2pt + black,
                   right: 1.2pt + black, bottom: none),
        ),
        // Title block — exactly tb-h tall, zero gap between rect and table
        context {
          let n = counter(page).get().first()
          let sheet = proj.sheet-prefix + str(n)
          title-block(sheet)
        }
      )
    ]
  )
)
```

**Why `background:` not `place()`:**
- `background:` renders behind all content — border and title block are never occluded by data
- `stack(spacing: 0pt, rect, table)` guarantees zero gap between content border and title block
- `context { counter(page).get().first() }` gives correct per-page sheet numbers
- `pad()` with exact outer margins clamps the entire frame to page edges precisely
- `rows: tb-h` on the table forces exact height regardless of content

### 3.2 Global document styling

```typst
#set text(font: theme.body-font, size: eval(theme.body-size, mode: "code"))
#set par(justify: false)
#set figure(numbering: none, outlined: false)
#show heading: set block(sticky: true)   // prevents orphaned section titles

#set table(
  stroke: 0.5pt + luma(180),
  inset: eval(theme.table-inset, mode: "code"),
)
// Bold + smaller text for every table header row, document-wide
#show table.cell.where(y: 0): set text(
  weight: "bold",
  size: eval(theme.label-size, mode: "code"),
)
```

### 3.3 Shared style helpers

```typst
// ── Row annotation fills ───────────────────────────────────────────────────
#let row-fill(annotation, y) = {
  if annotation == "fail"            { rgb("#f8d7da") }
  else if annotation == "warn"       { rgb("#fff3cd") }
  else if annotation == "pass"       { rgb("#d4edda") }
  else if annotation == "high"       { rgb("#e8f5e9") }
  else if annotation == "ux_threshold"    { rgb("#cfe2ff") }
  else if annotation == "uy_threshold"    { rgb("#fff3cd") }
  else if annotation == "ux_uy_threshold" { rgb("#d1c4e9") }
  else if calc.odd(y)                { luma(248) }
  else                               { none }
}

#let header-fill = luma(220)

// Standard stroke: heavy bottom on header, light grid on body
#let standard-stroke(x, y) = {
  if y == 0 { (bottom: 1pt + black) }
  else       { 0.5pt + luma(180) }
}

// Standard alignment: right for numeric columns (x >= 2), left for labels
#let standard-align(x, y) = if x >= 2 { right } else { left }
```

### 3.4 Validated drift table — production pattern

Confirmed working with 85-row dataset in demo:

```typst
#let render-drift-table(data, limit) = {
  table(
    columns: (auto, 1fr, auto, auto, auto),
    fill:   (x, y) => if y == 0 { header-fill }
                      else { row-fill(data.annotations.at(y - 1, default: ""), y) },
    align:  standard-align,
    stroke: standard-stroke,

    table.header[*Story*][*Case*][*Demand*][*Limit*][*DCR*],

    ..data.rows.map(r => (
      r.story,
      r.output-case,
      str(calc.round(r.drift-ratio, digits: 5)),
      str(calc.round(limit,         digits: 5)),
      str(calc.round(r.dcr,         digits: 3)),
    )).flatten()
  )
}
```

Key points:
- `table.header[...]` — semantic header; repeats on multi-page tables automatically
- `..data.rows.map(...).flatten()` — spread pattern; no manual per-cell Rust emission
- `str(calc.round(...))` — number formatting in Typst; raw `f64` values in JSON
- `data.annotations.at(y - 1, default: "")` — `y=0` is header row, body starts at `y=1`

### 3.5 Layout macros

```typst
// ── Section title + body wrapper ─────────────────────────────────────────────
#let page-content(title: "", body) = {
  text(size: eval(theme.title-size, mode: "code"), weight: "bold")[#title]
  v(eval(theme.section-gap, mode: "code"))
  body
}

// ── Chart left + table right ──────────────────────────────────────────────────
// emphasized: true  → chart-table-emphasized ratio (chart bigger)
// emphasized: false → chart-table-normal ratio (table bigger)
#let chart-and-table(img-path, table-body, emphasized: true) = {
  let cols    = eval(if emphasized { theme.chart-table-emphasized }
                     else          { theme.chart-table-normal }, mode: "code")
  let chart-h = eval(if emphasized { theme.chart-with-table-chart-h }
                     else          { theme.chart-with-table-normal-h }, mode: "code")
  grid(
    columns: cols,
    gutter: eval(theme.grid-gutter, mode: "code"),
    figure(image(img-path, height: chart-h)),
    table-body,
  )
}

// ── Two charts side by side ───────────────────────────────────────────────────
#let two-charts(img-a, img-b) = {
  grid(
    columns: eval(theme.two-col-ratio, mode: "code"),
    gutter: eval(theme.grid-gutter, mode: "code"),
    figure(image(img-a, height: eval(theme.chart-two-col-h, mode: "code"))),
    figure(image(img-b, height: eval(theme.chart-two-col-h, mode: "code"))),
  )
}

// ── Single full-width chart ────────────────────────────────────────────────────
#let single-chart(img-path) = {
  align(center)[
    #figure(image(img-path, height: eval(theme.chart-single-h, mode: "code")))
  ]
}
```

---

## Part 4 — Section Functions

Each section function loads its own JSON data, so no file is accessed without Rust having emitted the call.

```typst
// ── Drift wind ────────────────────────────────────────────────────────────────
#let drift-wind-x-page() = {
  let dw = json("drift_wind.json")
  page-content(title: "Wind Drift — X Direction")[
    #chart-and-table("images/drift_wind_x.svg",
      render-drift-table(dw.x, dw.x.allowable-ratio),
      emphasized: true)
  ]
}

#let drift-wind-y-page() = {
  let dw = json("drift_wind.json")
  page-content(title: "Wind Drift — Y Direction")[
    #chart-and-table("images/drift_wind_y.svg",
      render-drift-table(dw.y, dw.y.allowable-ratio),
      emphasized: true)
  ]
}

// ── Story forces (two charts per page) ───────────────────────────────────────
#let story-forces-x-page() = {
  page-content(title: "Story Forces — X Direction")[
    #two-charts("images/story_force_vx.svg", "images/story_force_my.svg")
  ]
}

#let story-forces-y-page() = {
  page-content(title: "Story Forces — Y Direction")[
    #two-charts("images/story_force_vy.svg", "images/story_force_mx.svg")
  ]
}

// ── Pier shear stress (single chart, no table) ────────────────────────────────
#let pier-shear-stress-wind-page() = {
  page-content(title: "Pier Shear Stress — Wind")[
    #single-chart("images/pier_shear_stress_wind.svg")
  ]
}

// (drift-seismic, displacement-wind, torsional, pier-axial follow the same pattern)
```

### Page order and Rust skip logic

```
Page 01 — summary-page()
Page 02 — modal-page()                       if calc.modal.is_some()
Page 03 — base-reactions-page()              if calc.base_reactions.is_some()
Page 04 — story-forces-x-page()             if calc.story_forces.is_some()
Page 05 — story-forces-y-page()             (same guard, both emitted together)
Page 06 — drift-wind-x-page()               if calc.drift_wind.is_some()
Page 07 — drift-wind-y-page()
Page 08 — drift-seismic-x-page()            if calc.drift_seismic.is_some()
Page 09 — drift-seismic-y-page()
Page 10 — displacement-wind-x-page()        if calc.displacement_wind.is_some()
Page 11 — displacement-wind-y-page()
Page 12 — torsional-x-page()               if calc.torsional.is_some()
Page 13 — torsional-y-page()
Page 14 — pier-shear-stress-wind-page()     if calc.pier_shear_stress_wind.is_some()
Page 15 — pier-shear-stress-seismic-page()  if calc.pier_shear_stress_seismic.is_some()
Page 16 — pier-axial-gravity-page()         if calc.pier_axial_stress.is_some()
Page 17 — pier-axial-wind-page()
Page 18 — pier-axial-seismic-page()
Page 19 — pier-axial-assumptions-page()
```

Rust emits `#section-fn()` for page 1, then `\n#pagebreak()\n#section-fn()` for each subsequent page.

---

## Part 5 — `ext-render` Integration Contract

### SVG key constants

```rust
// crates/ext-render/src/render_svg/mod.rs — replace all old constants
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

### `render_all_svg` signature

```rust
#[cfg(feature = "ssr")]
pub fn render_all_svg(
    calc: &CalcOutput,
    theme: &PageTheme,    // chart height driven by theme
) -> Result<HashMap<String, String>>;

// Chart pixel height from theme string:
fn theme_h_px(s: &str) -> u32 {
    let inches: f64 = s.trim_end_matches("in").parse().unwrap_or(6.0);
    (inches * 96.0) as u32
}
// Width fixed at 900px for all report charts
```

---

## Part 6 — Public API

```rust
// crates/ext-report/src/lib.rs
pub use theme::{PageTheme, TABLOID_LANDSCAPE, A4_PORTRAIT};
pub use pdf::{render_pdf, write_pdf};

// Caller (ext-api or ext-tauri):
let svg_map = ext_render::render_all_svg(&calc, &TABLOID_LANDSCAPE)?;
let pdf     = ext_report::render_pdf(&calc, &project, svg_map, &TABLOID_LANDSCAPE)?;
ext_report::write_pdf(&output_path, &pdf)?;
```

---

## Part 7 — Compile Fixes Required First

Must complete before starting report work.

| # | File | Fix |
|---|---|---|
| 1 | `ext-calc/checks/pier_shear_stress.rs` | `phi_shear`→`phi_v`, `load_combos`→`combos`, fix stress ratio formula, fix `fc_map` key to `(pier, story)` |
| 2 | `ext-calc/checks/displacement_wind.rs` | Use `max_elev` (roof height) not `story_height` for limit |
| 3 | `ext-render/chart_build/mod.rs` | Remove refs to deleted `CalcOutput` fields (`base_shear`, `pier_shear_wind`, `pier_shear_seismic`) |
| 4 | `ext-render/src/lib.rs` | Replace 7 stale exports with 19 new SVG key constants |
| 5 | `ext-report/report_document.rs` | Delete entirely |
| 6 | `ext-report/pdf/sections/` | Delete all 5 files |
| 7 | `ext-report/report_types.rs` | Delete `ReportSection`, `ReportDocument`, `KeyValueTable`, `ChartRef` |
| 8 | `ext-report/pdf/template.rs` | Rewrite `build_typst_document()`, delete `escape_text()` |
| 9 | `ext-calc` fixture `calc_output.json` | Regenerate after fixes — stale field names break deserialization |

---

## Part 8 — Module Structure After Refactor

```
crates/ext-report/src/
├── lib.rs           ← render_pdf(), write_pdf(), pub re-exports
├── main.rs          ← CLI (unchanged interface)
├── theme.rs         ← PageTheme + TABLOID_LANDSCAPE + A4_PORTRAIT
├── data.rs          ← ReportData::from_calc()
└── pdf/
    ├── mod.rs
    ├── renderer.rs  ← TypstWorld, render_pdf(), write_pdf()
    └── template.rs  ← build_typst_document() — skeleton emitter only

crates/ext-render/src/
├── lib.rs           ← updated exports
├── chart_build/
│   ├── mod.rs
│   ├── modal.rs
│   ├── base_reaction.rs      ← renamed from base_shear.rs
│   ├── drift.rs
│   ├── displacement.rs
│   ├── story_forces.rs       ← NEW
│   ├── torsional.rs          ← NEW
│   ├── pier_shear_stress.rs  ← NEW (replaces pier_shear.rs)
│   └── pier_axial.rs         ← updated (3 load categories)
├── render_html/mod.rs
└── render_svg/mod.rs         ← 19 SVG key constants + render_all_svg()
```

---

## Part 9 — Implementation Order

1. **Compile fixes** (Part 7, items 1–4) — `cargo check -p ext-calc && cargo check -p ext-render` green.
2. **`theme.rs`** — `PageTheme` + two constants. `cargo check -p ext-report`.
3. **`data.rs`** — `ReportData::from_calc()`. Add `Serialize` to `ReportProjectMeta`.
4. **`renderer.rs`** — update `TypstWorld`, update `render_pdf` signature.
5. **Delete** `report_document.rs`, `report_types.rs`, `pdf/sections/`.
6. **`template.rs`** — rewrite `build_typst_document()` to emit the validated skeleton.
7. **Typst template** — page setup (3.1), global styles (3.2), helpers (3.3), drift-wind page as proof of concept.
8. **Regenerate fixture** — fresh `calc_output.json` via integration test.
9. **Tests** — update existing, add `theme_switch_a4_produces_pdf`.
10. **`cargo test --workspace`** — all green.

---

## Invariants

- `tb-h + content-height + margin-top + margin-bottom == page-height` for every `PageTheme`. Write a `#[test]` that parses strings and verifies this.
- `background:` is the only acceptable pattern for border + title block. Never `place()`.
- `json()` calls live **inside** section functions — never at document top level for check data.
- `table.header(...)` wraps every data table's header row.
- `str(calc.round(value, digits: N))` is the only number formatter — raw `f64` in JSON.
- `escape_text()` must never be reintroduced.
- `render_all_svg()` and `render_pdf()` must receive the **same** `PageTheme` instance.
- `#set figure(numbering: none, outlined: false)` at document level.
- `#show heading: set block(sticky: true)` at document level.