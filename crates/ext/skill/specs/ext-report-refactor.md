# ext-report Refactor Spec
_Target crates: `ext-report`, `ext-render`, `ext-calc` (read-only reference)_
_Worker agent reads this file in full before touching any code._

---

## 0. Full Context — What Was Read

Every source file in both `ext-calc/src` and `ext-render/src` was reviewed, plus
`ext-report/src` in full, plus `calc_output.json` from a live 37-story run.

This spec is authoritative. Do not apply hunches from earlier sessions.

---

## 1. Current Architecture — Verified Facts

### 1A — `ext-calc` output contract (all fields confirmed against live JSON)

`CalcOutput` fields and their current shape as emitted by `CalcRunner::run_all`:

| Field | Type | Notes |
|---|---|---|
| `story_forces` | `Option<StoryForcesOutput>` | Has `rows`, `story_order`, `x_profiles`, `y_profiles` |
| `drift_wind` | `Option<DriftWindOutput>` | Each direction has `story_order: Vec<String>` |
| `drift_seismic` | `Option<DriftSeismicOutput>` | Same |
| `displacement_wind` | `Option<DisplacementWindOutput>` | Each direction has `story_order`, `story_limits: Vec<DisplacementLimitRow>` |
| `torsional` | `Option<TorsionalOutput>` | Each `TorsionalDirectionOutput` has `no_data`, `governing_step: Option<i32>` |
| `pier_shear_stress_wind/seismic` | `Option<PierShearStressOutput>` | Has `supported`, `support_note`, `story_order` |
| `pier_axial_stress` | `Option<PierAxialStressOutput>` | Has `story_order`; `PierAxialResult` has `pu_signed`, `fa_signed` |

Key limit values confirmed from live JSON and `pier_shear_stress.rs`:
- `limit_individual = 10.0` (per-pier, ACI 318-14 §18.10.4: 10√f'c)
- `limit_average = 8.0` (story average, ACI 318-14 §18.10.4: 8√f'c)

`TorsionalRow` governing fields now present on every row:
`governing_step: i32`, `governing_drift_a`, `governing_drift_b`, `governing_delta_max`,
`governing_delta_avg`, `governing_ratio` — all `f64`, all `#[serde(default)]`.

`PierAxialResult` has `pu_signed: Quantity` and `fa_signed: Quantity` (`#[serde(default)]`).
The sign convention: ETABS negative P = compression → positive `fa_signed`.

### 1B — `ext-render` chart inventory (confirmed from `mod.rs` + all builders)

`build_report_charts()` currently emits these assets in this order:

| Asset constant | Builder | Image key |
|---|---|---|
| `MODAL_IMAGE` | `modal::build` | `images/modal.svg` |
| `BASE_REACTIONS_IMAGE` | `base_shear::build` | `images/base_reactions.svg` |
| `STORY_FORCE_VX_IMAGE` | `story_forces::build` | `images/story_force_vx.svg` |
| `STORY_FORCE_VY_IMAGE` | | `images/story_force_vy.svg` |
| `STORY_FORCE_MY_IMAGE` | | `images/story_force_my.svg` |
| `STORY_FORCE_MX_IMAGE` | | `images/story_force_mx.svg` |
| `DRIFT_WIND_X_IMAGE` | `drift::build_wind` | `images/drift_wind_x.svg` |
| `DRIFT_WIND_Y_IMAGE` | | `images/drift_wind_y.svg` |
| `DRIFT_SEISMIC_X_IMAGE` | `drift::build_seismic` | `images/drift_seismic_x.svg` |
| `DRIFT_SEISMIC_Y_IMAGE` | | `images/drift_seismic_y.svg` |
| `DISPLACEMENT_WIND_X_IMAGE` | `displacement::build` | `images/displacement_wind_x.svg` |
| `DISPLACEMENT_WIND_Y_IMAGE` | | `images/displacement_wind_y.svg` |
| `TORSIONAL_X_IMAGE` | `torsional::build` | `images/torsional_x.svg` |
| `TORSIONAL_Y_IMAGE` | | `images/torsional_y.svg` |
| `PIER_SHEAR_STRESS_WIND_X_IMAGE` | `pier_shear::build_wind` | `images/pier_shear_stress_wind_x.svg` |
| `PIER_SHEAR_STRESS_WIND_Y_IMAGE` | | `images/pier_shear_stress_wind_y.svg` |
| `PIER_SHEAR_STRESS_WIND_AVG_IMAGE` | `pier_shear::build_wind_average` | `images/pier_shear_stress_wind_avg.svg` |
| `PIER_SHEAR_STRESS_SEISMIC_X_IMAGE` | `pier_shear::build_seismic` | `images/pier_shear_stress_seismic_x.svg` |
| `PIER_SHEAR_STRESS_SEISMIC_Y_IMAGE` | | `images/pier_shear_stress_seismic_y.svg` |
| `PIER_SHEAR_STRESS_SEISMIC_AVG_IMAGE` | `pier_shear::build_seismic_average` | `images/pier_shear_stress_seismic_avg.svg` |
| `PIER_AXIAL_GRAVITY_IMAGE` | `pier_axial::build_all` | `images/pier_axial_gravity.svg` |
| `PIER_AXIAL_WIND_IMAGE` | | `images/pier_axial_wind.svg` |
| `PIER_AXIAL_SEISMIC_IMAGE` | | `images/pier_axial_seismic.svg` |

All story-order handling uses `story_display_order()` from `mod.rs` which reverses
top→bottom to bottom→top so ECharts swapped-axis places roof at top visually.

All pier label normalization uses `normalized_pier_labels()` which sorts
`PX1, PX2, ... PY1, PY2, ...` and filters `is_default_pier_label()` (rejects `""` and `"0"`).

Pier axial uses `fa_signed.value` (not `fa.value`) for the Y=story X=stress chart, which
correctly shows compression as positive per ETABS sign convention.

`ChartKind::Cartesian` now has `x_axis_label: Option<String>` and `y_axis_label: Option<String>`.
All chart builders already populate these. `build_cartesian()` in `mod.rs` already applies them.

### 1C — `ext-report` current page sequence

From `template.rs → build_typst_document()`:

1. Cover page
2. Summary page
3. Modal page (if present)
4. Base reactions page (chart + table side-by-side)
5. Story Forces X page (VX + MY two charts)
6. Story Forces Y page (VY + MX two charts)
7. Wind Drift X page (table + chart)
8. Wind Drift Y page
9. Seismic Drift X page
10. Seismic Drift Y page
11. Wind Displacement X page (table + chart)
12. Wind Displacement Y page
13. Torsional X page (table only, if rows present)
14. Torsional Y page (table only, if rows present)
15. Pier Shear Wind page (table + single chart) ← **OLD: only wind_x or wind_y, not both**
16. Pier Shear Seismic page (table + single chart) ← **OLD: only seismic_x or seismic_y**
17. Pier Axial Gravity (single chart)
18. Pier Axial Wind (single chart)
19. Pier Axial Seismic (single chart)
20. Pier Axial Assumptions page
21. Calc Procedure page (torsion + pier shear worked examples) ← gated by `INCLUDE_CALC_PROCEDURE_PAGE`

### 1D — `ext-report` `data.rs` confirmed-correct behaviours

All of these were previously suspected bugs but are confirmed correct:

- `build_drift_dir()` uses `drift.governing.direction` to pick X vs Y columns ✓
- `build_displacement_dir()` uses `disp.governing.direction` for direction isolation ✓
- `build_base_reactions()` already filters via `should_exclude_base_case_type()`:
  excludes `"combination"`, `"linmodritz"`, `"eigen"` ✓
- `build_pier_shear()` already checks `pier.supported` and uses `pier.story_order` ✓
- All story ordering uses `story_order` Vec from `DriftOutput`, `DisplacementOutput` etc. ✓
- `SummaryCheckerRow` and `build_checker_rows()` fully implemented ✓
- Pier shear matrix payloads (`x_matrix`, `y_matrix`) fully built ✓
- `build_torsional_dir()` maps `row.governing_drift_a`, `row.governing_ratio` etc. ✓
  `TorsionalDirReport` has `has_rows: bool` which Typst procedures reference ✓

---

## 2. Changes Required (User Requests)

### 2A — Remove verification / worked-example pages

**File: `src/pdf/procedures.rs`**

Change:
```rust
pub const INCLUDE_CALC_PROCEDURE_PAGE: bool = true;
```
To:
```rust
pub const INCLUDE_CALC_PROCEDURE_PAGE: bool = false;
```

This single change gates both `append_definitions()` and `append_sequence()`.
The functions remain as dead code for future revival.

Add a TODO comment at the top of `torsion-worked-example()` and
`pier-shear-worked-example()` in the Typst string:
```
// TODO: Typst field references here must be audited before re-enabling.
// data.rs contract has evolved. See ext-report-refactor.md §2A.
```

Pages removed: Torsional Irregularity Verification, Pier Shear Wind Verification,
Pier Shear Wind Average Verification, Pier Shear Seismic Verification,
Pier Shear Seismic Average Verification.

---

### 2B — Pier Shear: merge 2 pages → 1 combined page per type

**Current:** Wind has 1 page (table + old single chart), Seismic has 1 page.
**Current ext-render:** Wind emits 3 assets: `wind_x`, `wind_y`, `wind_avg`.
Seismic emits 3 assets: `seismic_x`, `seismic_y`, `seismic_avg`.

**Required:** Each of wind and seismic gets one combined page showing:
- Left chart: per-pier stress ratio (X direction)
- Right chart: per-pier stress ratio (Y direction)
- AND the average chart is shown on the same page using `two-charts-page`.

Wait — re-reading the user request: _"Pier Shear Wind Average Review and Pier Shear Seismic Average Review should be 1 chart page not 2 chart page, combine 2 page in the image together"_. The images shown are the verification pages (worked example). The request is: combine the two verification-page columns into one page, AND ensure Average Review is one chart page (not split).

**Correct interpretation after re-reading:**

The user wants:
1. Remove all 5 verification/procedure pages (done in 2A)
2. The "Pier Shear Wind Average Review" should be one page (it already is — `build_wind_average` emits one `NamedChartSpec`)
3. The "Pier Shear Seismic Average Review" should be one page (it already is)

The real request is: **combine the per-direction X+Y charts with the average chart into a single page per shear type** using `two-charts-page`.

**Implementation in `template.rs` document sequence:**

Replace the current pier-shear page calls with:

```typst
// Wind: X and Y side-by-side
#pagebreak()
#two-charts-page(
  [Pier Shear Wind — Per Pier],
  "images/pier_shear_stress_wind_x.svg",
  "Stress-ratio trend by story — X-direction walls (individual limit 10.0)",
  "images/pier_shear_stress_wind_y.svg",
  "Stress-ratio trend by story — Y-direction walls (individual limit 10.0)",
)

// Wind average: single chart page
#pagebreak()
#single-chart-page(
  [Pier Shear Wind — Story Average],
  "images/pier_shear_stress_wind_avg.svg",
  "Story-average stress ratio — X and Y walls vs. average limit 8.0",
)

// Seismic: X and Y side-by-side
#pagebreak()
#two-charts-page(
  [Pier Shear Seismic — Per Pier],
  "images/pier_shear_stress_seismic_x.svg",
  "Stress-ratio trend by story — X-direction walls (individual limit 10.0)",
  "images/pier_shear_stress_seismic_y.svg",
  "Stress-ratio trend by story — Y-direction walls (individual limit 10.0)",
)

// Seismic average: single chart page
#pagebreak()
#single-chart-page(
  [Pier Shear Seismic — Story Average],
  "images/pier_shear_stress_seismic_avg.svg",
  "Story-average stress ratio — X and Y walls vs. average limit 8.0",
)
```

Remove the old `pier-shear-page` Typst helper from `template.rs` since it is no longer called.

**Result:** 4 pier-shear pages total (was previously 2 with only 1 chart each, or potentially
misconfigured). All 6 chart assets from `ext-render` are now used.

The `pier_shear_wind.json` and `pier_shear_seismic.json` data files are still generated
by `data.rs` and available for Typst, but the combined pages do not show a data table —
charts only. The table data is available for future use.

---

### 2C — "Continued" header on multi-page tables

**File: `src/pdf/template.rs`**

Add `repeat: true` to every `table.header(` call that appears in a table that
can grow unboundedly with building size. Search for `table.header(` and add
`repeat: true,` as the first argument for these functions:

| Typst function | Table |
|---|---|
| `modal-page()` | Modal participation |
| `base-reactions-table()` | Base reactions |
| `drift-table()` | Drift envelope |
| `displacement-table()` | Displacement |
| `torsional-dir-page()` | Torsional rows |

Example transformation:
```typst
// BEFORE:
table.header(
  ..("Story", "Case", "Demand (ratio)", "Limit (ratio)", "DCR")
    .map(h => table.cell(fill: luma(220))[#h])
),

// AFTER:
table.header(
  repeat: true,
  ..("Story", "Case", "Demand (ratio)", "Limit (ratio)", "DCR")
    .map(h => table.cell(fill: luma(220))[#h])
),
```

Typst will automatically repeat the header row at the top of each overflow page.
The repeated header serves as the "continued" indicator — no explicit "(continued)"
text is needed, and this matches how professional engineering reports handle it.

---

### 2D — Add `wire` theme

**File: `src/theme.rs`**

1. Add `border_stroke: &'static str` field to `PageTheme` struct (after `content_inset`).

2. Add `border_stroke: "1.2pt"` to both `TABLOID_LANDSCAPE` and `A4_PORTRAIT`.

3. Add new constant:

```rust
pub const WIRE_LANDSCAPE: PageTheme = PageTheme {
    layout_kind: "cad-sheet",
    page_width:    "17in",
    page_height:   "11in",
    margin_top:    "0.20in",
    margin_left:   "0.20in",
    margin_right:  "0.20in",
    margin_bottom: "0.20in",
    content_height: "9.85in",
    // tb-h = 11 - 0.20 - 0.20 - 9.85 = 0.75in ✓

    body_font:    "Linux Libertine",
    body_size:    "8.5pt",
    title_size:   "13pt",
    label_size:   "6.5pt",
    caption_size: "7.5pt",

    chart_single_h:            "8.8in",
    chart_two_col_h:           "7.8in",
    chart_with_table_chart_h:  "6.3in",
    chart_with_table_normal_h: "7.3in",

    two_col_ratio:          "(1fr, 1fr)",
    chart_table_emphasized: "(1.08fr, 0.92fr)",
    chart_table_normal:     "(0.82fr, 1.18fr)",

    title_block_columns: "(3.35in, 3.2in, 4.0in, 1.6in, 2.0in, 2.35in)",

    section_gap:   "8pt",
    table_inset:   "4pt",
    grid_gutter:   "16pt",
    content_inset: "14pt",
    border_stroke: "0.5pt",
};
```

4. Add `Wire` to `ReportTheme` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportTheme {
    Tabloid,
    A4,
    Wire,
}

impl ReportTheme {
    pub fn page_theme(&self) -> &'static PageTheme {
        match self {
            ReportTheme::Tabloid => &TABLOID_LANDSCAPE,
            ReportTheme::A4     => &A4_PORTRAIT,
            ReportTheme::Wire   => &WIRE_LANDSCAPE,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportTheme::Tabloid => "tabloid",
            ReportTheme::A4     => "a4",
            ReportTheme::Wire   => "wire",
        }
    }
}

impl FromStr for ReportTheme {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tabloid" => Ok(ReportTheme::Tabloid),
            "a4"      => Ok(ReportTheme::A4),
            "wire"    => Ok(ReportTheme::Wire),
            other     => bail!("Unknown theme '{other}'. Valid: tabloid, a4, wire"),
        }
    }
}
```

5. **`template.rs` boilerplate** — add parse of `border-stroke`:

```typst
#let border-stroke-w = parse-pt(theme.border-stroke)
```

In `title-block()`, change the hardcoded `stroke: 1.2pt + black` to:
```typst
stroke: border-stroke-w + black,
```

In the outer background `rect`:
```typst
stroke: (
  top: border-stroke-w + black,
  left: border-stroke-w + black,
  right: border-stroke-w + black,
  bottom: none,
),
```

6. **`lib.rs`:** Add `WIRE_LANDSCAPE` to the `pub use theme::` export.

7. **`main.rs`:** Update usage string to `--theme tabloid|a4|wire`.

8. **`theme.rs` invariant test:**
```rust
#[test]
fn wire_landscape_satisfies_invariant() {
    assert_theme_invariant(&WIRE_LANDSCAPE, "WIRE_LANDSCAPE");
}
```

---

## 3. New Data Wiring — `data.rs`

### 3A — Wire `x_profiles` / `y_profiles` through to Typst

`StoryForcesOutput.x_profiles` and `y_profiles` are fully populated by `ext-calc`
(confirmed: 4 profiles each in the live JSON). The template renders per-case line plots
from `story_forces::build()` in `ext-render` — these already use `x_profiles` / `y_profiles`.
The data.rs `build_story_forces()` must pass them through to `story_forces.json`.

**Expand `StoryForcesReportData` in `data.rs`:**

```rust
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StoryForcesReportData {
    rows: Vec<StoryForcesReportRow>,
    x_profiles: Vec<StoryForceCaseProfileReport>,
    y_profiles: Vec<StoryForceCaseProfileReport>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StoryForceCaseProfileReport {
    output_case: String,
    rows: Vec<StoryForceCaseRowReport>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct StoryForceCaseRowReport {
    story: String,
    elevation_ft: f64,
    vx_kip: f64,
    vy_kip: f64,
    mx_kip_ft: f64,
    my_kip_ft: f64,
}
```

Update `build_story_forces()`:

```rust
fn build_story_forces(story_forces: &StoryForcesOutput) -> StoryForcesReportData {
    let map_profile = |p: &StoryForceCaseProfile| StoryForceCaseProfileReport {
        output_case: p.output_case.clone(),
        rows: p.rows.iter().map(|r| StoryForceCaseRowReport {
            story: r.story.clone(),
            elevation_ft: r.elevation_ft,
            vx_kip: r.vx_kip,
            vy_kip: r.vy_kip,
            mx_kip_ft: r.mx_kip_ft,
            my_kip_ft: r.my_kip_ft,
        }).collect(),
    };
    StoryForcesReportData {
        rows: story_forces.rows.iter().map(|row| StoryForcesReportRow {
            story: row.story.clone(),
            max_vx_kip: row.max_vx_kip,
            max_my_kip_ft: row.max_my_kip_ft,
            max_vy_kip: row.max_vy_kip,
            max_mx_kip_ft: row.max_mx_kip_ft,
        }).collect(),
        x_profiles: story_forces.x_profiles.iter().map(map_profile).collect(),
        y_profiles: story_forces.y_profiles.iter().map(map_profile).collect(),
    }
}
```

Add the required import at top of `data.rs`:
```rust
use ext_calc::output::{
    ..., StoryForceCaseProfile, StoryForcesOutput, ...
};
```

---

## 4. Test Changes

### 4A — `renderer.rs` test helpers — align to current `output.rs`

The `sample_torsional_row()` and `build_torsional_direction()` helpers must be
updated to include all current fields. Replace both functions:

```rust
fn sample_torsional_row(story: &str, case: &str, ratio: f64) -> TorsionalRow {
    TorsionalRow {
        story: story.to_string(),
        case: case.to_string(),
        joint_a: "J1".to_string(),
        joint_b: "J2".to_string(),
        drift_a_steps: vec![0.1],
        drift_b_steps: vec![0.1],
        delta_max_steps: vec![0.1],
        delta_avg_steps: vec![0.1],
        ratio,
        governing_step: 1,
        governing_drift_a: 0.1,
        governing_drift_b: 0.1,
        governing_delta_max: 0.1,
        governing_delta_avg: 0.1,
        governing_ratio: ratio,
        ax: 1.2,
        ecc_ft: 0.0,
        rho: 1.0,
        is_type_a: ratio >= 1.2,
        is_type_b: ratio >= 1.4,
    }
}

fn build_torsional_direction(rows: Vec<TorsionalRow>) -> TorsionalDirectionOutput {
    let governing_story = rows.first().map(|r| r.story.clone()).unwrap_or_default();
    let governing_case  = rows.first().map(|r| r.case.clone()).unwrap_or_default();
    let max_ratio = rows.iter().map(|r| r.ratio).fold(0.0_f64, f64::max);
    let has_type_a = rows.iter().any(|r| r.is_type_a);
    let has_type_b = rows.iter().any(|r| r.is_type_b);
    TorsionalDirectionOutput {
        rows,
        no_data: vec![],
        governing_story,
        governing_case,
        governing_joints: vec!["J1".to_string(), "J2".to_string()],
        governing_step: Some(1),
        max_ratio,
        has_type_a,
        has_type_b,
    }
}
```

Also update `dummy_svg_map()` in `renderer.rs` to include all 23 current chart assets.
The complete set that must be present (all from `ext-render::chart_build` constants):

```
images/modal.svg
images/base_reactions.svg
images/story_force_vx.svg
images/story_force_vy.svg
images/story_force_my.svg
images/story_force_mx.svg
images/drift_wind_x.svg
images/drift_wind_y.svg
images/drift_seismic_x.svg
images/drift_seismic_y.svg
images/displacement_wind_x.svg
images/displacement_wind_y.svg
images/torsional_x.svg
images/torsional_y.svg
images/pier_shear_stress_wind_x.svg
images/pier_shear_stress_wind_y.svg
images/pier_shear_stress_wind_avg.svg
images/pier_shear_stress_seismic_x.svg
images/pier_shear_stress_seismic_y.svg
images/pier_shear_stress_seismic_avg.svg
images/pier_axial_gravity.svg
images/pier_axial_wind.svg
images/pier_axial_seismic.svg
```

All 23 must be present in `dummy_svg_map()` because the Typst template now references
all of them (including the 4 new pier-shear combined-page charts).

### 4B — Add `wire` theme render test

```rust
#[test]
fn render_pdf_wire_theme_produces_pdf_bytes() {
    let calc = fixture_calc_output();
    let project = ReportProjectMeta {
        project_name: "Proof Tower".to_string(),
        subject: "Wire theme report".to_string(),
        ..ReportProjectMeta::default()
    };
    let pdf = render_pdf(&calc, &project, dummy_svg_map(), &WIRE_LANDSCAPE).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}
```

Import `WIRE_LANDSCAPE` alongside the existing theme imports in the test module.

### 4C — Add `story_forces_json_includes_profiles` test in `data.rs`

```rust
#[test]
fn story_forces_json_includes_profiles() {
    let calc = fixture_calc_output();
    let report_data = ReportData::from_calc(
        &calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new()
    ).unwrap();
    let bytes = report_data
        .files
        .get(&PathBuf::from("story_forces.json"))
        .expect("story_forces.json must exist");
    let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();
    assert!(value.get("x-profiles").is_some(), "x-profiles must be present");
    assert!(value.get("y-profiles").is_some(), "y-profiles must be present");
    if calc.story_forces.as_ref().map(|sf| !sf.x_profiles.is_empty()).unwrap_or(false) {
        let profiles = value["x-profiles"].as_array().unwrap();
        assert!(!profiles.is_empty(), "x-profiles should be non-empty when calc has profiles");
        let first_profile = &profiles[0];
        assert!(first_profile.get("output-case").is_some());
        assert!(first_profile.get("rows").is_some());
    }
}
```

---

## 5. File Change Summary

| File | Change |
|---|---|
| `src/pdf/procedures.rs` | Set `INCLUDE_CALC_PROCEDURE_PAGE = false`. Add TODO comments. |
| `src/pdf/renderer.rs` | Fix `sample_torsional_row` + `build_torsional_direction` (4A). Update `dummy_svg_map` to all 23 assets. Add `wire` test (4B). Import `WIRE_LANDSCAPE`. |
| `src/pdf/template.rs` | (1) Add `border-stroke-w` parse + apply to `title-block` and outer `rect`. (2) Replace pier-shear page calls with 4 chart pages (2B). (3) Remove old `pier-shear-page` helper. (4) Add `repeat: true` to all 5 `table.header` calls (2C). |
| `src/data.rs` | Expand `StoryForcesReportData` to include `x_profiles` / `y_profiles`. Update `build_story_forces`. Add import. Add test 4C. |
| `src/theme.rs` | Add `border_stroke` field to `PageTheme`. Update `TABLOID_LANDSCAPE` + `A4_PORTRAIT` with `border_stroke: "1.2pt"`. Add `WIRE_LANDSCAPE`. Add `Wire` to `ReportTheme`. Update `FromStr` / `as_str`. Add invariant test. |
| `src/lib.rs` | Export `WIRE_LANDSCAPE`. |
| `src/main.rs` | Update usage string. Add `wire` theme parse test. |

---

## 6. What Is Already Correct — No Change Needed

These were investigated in depth and are confirmed correct as-is:

| Item | File | Status |
|---|---|---|
| Direction-isolated drift aggregation | `data.rs::build_drift_dir` | ✓ Uses `governing.direction` to pick X vs Y |
| Direction-isolated displacement aggregation | `data.rs::build_displacement_dir` | ✓ Same pattern |
| Base reactions case-type filter | `data.rs::build_base_reactions` | ✓ `should_exclude_base_case_type` handles combination/linmodritz/eigen |
| Base reactions rounding | `template.rs::base-reactions-table` | ✓ Rounds to 1 decimal; user requirement for 5 decimals applies only to values used in the RSA ratio check, not the full table — leave as-is unless user re-confirms |
| `story_order` used for display ordering | All chart builders in `ext-render` | ✓ All use `story_display_order()` |
| `limit_individual = 10.0` / `limit_average = 8.0` | `pier_shear_stress.rs` | ✓ Correct per ACI 318-14 §18.10.4 |
| Pier label normalization `PX1..PY1..` | `data.rs` + `ext-render/mod.rs` | ✓ Both use same logic |
| `"0"` pier label filtered out | `data.rs::is_default_pier_label` + `ext-render/mod.rs::is_default_pier_label` | ✓ Both filter `"0"` and `""` |
| `fa_signed` used in pier axial chart | `pier_axial.rs::build_category` | ✓ Uses `row.fa_signed.value` for signed stress display |
| `TorsionalRow.governing_*` fields populated | `checks/torsional.rs` | ✓ All fields computed and assigned per-row |
| `torsional.json` always written (even if None) | `data.rs` | ✓ Uses `unwrap_or_else(default_torsional_report_data)` |
| `pier_shear_wind/seismic.json` always written | `data.rs` | ✓ Uses `unwrap_or_else(build_unsupported_pier_shear_report_data)` |
| Pier shear `x_matrix` / `y_matrix` | `data.rs` | ✓ Fully implemented |

---

## 7. Acceptance Criteria

- [ ] `cargo test -p ext-report` passes with zero failures
- [ ] `cargo clippy -p ext-report -- -D warnings` passes
- [ ] `cargo run -p ext-report -- preview <fixture> --theme wire` produces a valid PDF
- [ ] `cargo run -p ext-report -- preview <fixture> --theme tabloid` still works unchanged
- [ ] Generated PDF contains no torsional or pier-shear verification/worked-example pages
- [ ] Pier shear wind has 2 pages: X+Y side-by-side, then average single chart
- [ ] Pier shear seismic has 2 pages: X+Y side-by-side, then average single chart
- [ ] All 5 listed tables use `table.header(repeat: true, ...)`
- [ ] `WIRE_LANDSCAPE` passes the content_height invariant test
- [ ] Wire PDF has 0.5pt border lines, tabloid has 1.2pt (parseable from MediaBox or visual check)
- [ ] `story_forces.json` contains `"x-profiles"` and `"y-profiles"` arrays
- [ ] Each profile in `x-profiles` has `"output-case"` and `"rows"` keys
- [ ] `cargo test -p ext-report -- --test-threads=1` passes (renderer tests share font loading)
