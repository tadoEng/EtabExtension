# ext-render Deep Dive: Calculation Logic, Chart Outputs, and Table Interfaces

## 1) What this review covers
This is a code-accurate deep dive of `crates/ext-render` with cross-reference to `ext-report` so you can decide:

- calculation/display logic used in each rendered chart
- exact chart types and series
- what data is output to charts vs report tables
- where logic is transformed, aggregated, or simplified
- key decision points and potential mismatches

Code sources reviewed:

- `crates/ext-render/src/chart_types.rs`
- `crates/ext-render/src/chart_build/*.rs`
- `crates/ext-render/src/render_html/mod.rs`
- `crates/ext-render/src/render_svg/mod.rs`
- `crates/ext-report/src/data.rs` (table JSON shaping for report)
- `crates/ext-report/src/pdf/template.rs` (table/chart page wiring)

## 2) Architecture boundary (important)

## 2.1 ext-render responsibilities

`ext-render`:

- reads `ext_calc::output::CalcOutput`
- builds chart specifications (`ChartSpec`)
- renders to:
  - HTML fragments (`HtmlRenderer`)
  - SVG strings (`ImageRenderer`, `ssr` feature)

It does **not** build report tables.

## 2.2 Where table data is actually produced

Table-shaped output is built in `ext-report`, not `ext-render`:

- `crates/ext-report/src/data.rs` converts `CalcOutput` into JSON files like:
  - `modal.json`
  - `base_reactions.json`
  - `story_forces.json`
  - `drift_wind.json`, `drift_seismic.json`
  - `displacement_wind.json`
  - `torsional.json`
  - `pier_shear_wind.json`, `pier_shear_seismic.json`
  - `pier_axial_stress.json`

Then `template.rs` uses those JSON files for report tables, and uses `images/*.svg` for charts.

## 3) Render model and data flow

High-level flow:

1. `ext-calc` computes `CalcOutput`.
2. `build_report_charts(calc, config)` builds a list of `NamedChartSpec`.
3. Rendering path:
   - HTML: `render_all_html` => `HashMap<logical_name, html_fragment>`
   - SVG: `render_all_svg` => `RenderedCharts { assets: [{ logical_name, caption, svg }] }`
4. `ext-report` receives SVG map and report JSON data, then composes PDF.

`RenderConfig` defaults:

- width: `900`
- height: `620`
- base reaction grouping: empty unless provided from config

## 4) Chart output inventory

Maximum chart assets emitted (when all relevant checks exist): **17**

- `images/modal.svg`
- `images/base_reactions.svg`
- `images/story_force_vx.svg`
- `images/story_force_vy.svg`
- `images/story_force_my.svg`
- `images/story_force_mx.svg`
- `images/drift_wind_x.svg`
- `images/drift_wind_y.svg`
- `images/drift_seismic_x.svg`
- `images/drift_seismic_y.svg`
- `images/displacement_wind_x.svg`
- `images/displacement_wind_y.svg`
- `images/pier_shear_stress_wind.svg`
- `images/pier_shear_stress_seismic.svg`
- `images/pier_axial_gravity.svg`
- `images/pier_axial_wind.svg`
- `images/pier_axial_seismic.svg`

No torsional chart is built in `ext-render` (torsional is table-only in current report).

## 5) Per-chart logic breakdown

## 5.1 Modal (`chart_build/modal.rs`)

- Chart type: Cartesian, vertical axes (`swap_axes = false`)
- Categories: `Mode 1`, `Mode 2`, ...
- Series:
  - line: `Sum UX`
  - line: `Sum UY`
- Smoothing: `true` for both
- Purpose: visual cumulative participation trend (threshold line is **not** rendered here)

## 5.2 Base reactions pie (`chart_build/base_shear.rs`)

- Chart type: Pie
- Value source: `abs(fz_kip)` from `BaseReactionsOutput.rows`
- Grouping:
  - if `config.base_reaction_groups.first()` exists:
    - whitelist by that first group's `load_cases`
    - sum by `output_case`
  - else:
    - include all output cases, sum by `output_case`
- Sorted descending by slice value

Important behavior:

- only the **first** configured base-reaction group is used for charting.

## 5.3 Story forces (`chart_build/story_forces.rs`)

Builds 4 charts:

- VX (bar)
- VY (bar)
- MY (bar)
- MX (bar)

Common behavior:

- chart type: horizontal bar (`swap_axes = true`)
- categories: story labels
- values: absolute values of selected metric
- source rows are reversed (`output.rows.iter().rev()`) to display bottom-up
- size:
  - width fixed at `620` (does not use `RenderConfig.width`)
  - height = `max(400, story_count * 20 + 100)`

## 5.4 Drift wind/seismic (`chart_build/drift.rs`)

Builds 4 charts total:

- Wind X, Wind Y
- Seismic X, Seismic Y

Per directional chart:

- type: horizontal line chart (`swap_axes = true`)
- demand series:
  - X chart uses `max(abs(max_drift_x_pos), abs(max_drift_x_neg))`
  - Y chart uses `max(abs(max_drift_y_pos), abs(max_drift_y_neg))`
- story aggregation:
  - `aggregate_story_max` keeps max demand per story across cases/groups
- limit series:
  - constant `drift.allowable_ratio` repeated for each story
- height scaling:
  - `max(config.height, story_count * 18 + 100)`

## 5.5 Displacement wind (`chart_build/displacement.rs`)

Builds 2 charts:

- Wind displacement X
- Wind displacement Y

Per directional chart:

- type: horizontal line chart (`swap_axes = true`)
- demand series:
  - X: `max(abs(max_disp_x_pos_ft), abs(max_disp_x_neg_ft))`
  - Y: `max(abs(max_disp_y_pos_ft), abs(max_disp_y_neg_ft))`
- story aggregation:
  - max per story via `aggregate_story_max`
- limit series:
  - constant `displacement.disp_limit.value` for each story
- labels are hard-coded as:
  - `"Demand (ft)"`
  - `"Limit (ft)"`
- height:
  - `max(config.height, story_count * 18 + 100)`

## 5.6 Pier shear stress wind/seismic (`chart_build/pier_shear.rs`)

Builds 2 charts:

- wind
- seismic

Per chart:

- type: horizontal mixed chart (`swap_axes = true`)
- categories: `"pier / story"`
- sorted by demand descending
- demand series:
  - bar: `row.stress_psi`
- limit series:
  - dashed line, constant value based on mean of `row.limit_individual`
- height:
  - `max(config.height, 30 * entry_count + 80)`

## 5.7 Pier axial (`chart_build/pier_axial.rs`)

Builds up to 3 charts:

- gravity
- wind
- seismic

A chart is emitted only if that category has rows.

Per category chart:

- type: horizontal multi-line (`swap_axes = true`)
- categories: stories (first-seen order in filtered rows)
- one line series per pier label
- y-value for each story:
  - governing signed `fa` (max absolute if duplicates exist for same pier/story)
- extra dashed `Zero` reference line
- height:
  - `max(config.height, story_count * 18 + 100)`

## 6) Chart rendering mechanics

## 6.1 HTML mode

`render_all_html(calc, config)`:

- loops through `build_report_charts`
- renders each `ChartSpec` with `HtmlRenderer`
- returns map: `logical_name -> html_string`

## 6.2 SVG mode

`render_all_svg(calc, config)` (`ssr` feature):

- loops through `build_report_charts`
- renders each with `ImageRenderer`
- returns vector of assets with:
  - `logical_name`
  - `caption`
  - `svg`

`write_svg_assets` writes to disk using `file_name(logical_name)` under output dir.

## 7) Table outputs paired with those charts (ext-report)

Current report page pattern (from `template.rs`):

- Modal: table + `images/modal.svg`
- Base reactions: table + `images/base_reactions.svg`
- Story forces:
  - X page: table from `story_forces.json` + VX/MY charts
  - Y page: table from same JSON + VY/MX charts
- Drift wind/seismic X,Y:
  - each page has table + corresponding drift chart image
- Displacement wind X,Y:
  - each page has table + corresponding displacement chart image
- Torsional:
  - table-only pages (no chart image)
- Pier shear wind/seismic:
  - table + chart image
- Pier axial:
  - three chart-only pages (gravity/wind/seismic) + assumptions text page

## 8) Decision notes (logic review points)

These are high-impact points for deciding final logic:

1. `base_shear` chart uses only `base_reaction_groups.first()`; additional configured groups are ignored.
2. `story_forces` chart dimensions ignore `RenderConfig.width` and use fixed `620`.
3. Drift/displacement story aggregation collapses all cases/groups to one max per story, so case/group traceability is intentionally lost in chart data.
4. Displacement series labels are fixed `"ft"` even though `CalcOutput` carries unit labels.
5. No torsional chart is emitted; torsional output is table-only.
6. `EXT_RENDER_DESIGN.md` contains architecture notes that do not fully match current implementation details (for example, current `ChartSpec` abstraction and concrete builders differ from parts of that design text).
7. Pier shear chart currently plots:
   - demand as `stress_psi`
   - limit from `limit_individual` (ratio-space value `8.0`)
   This is a unit-space mismatch in current implementation and should be resolved if chart is used for engineering decisions.

## 9) Quick chart type matrix

| Check/output | Asset(s) | Chart type | Series mix |
| --- | --- | --- | --- |
| Modal | `modal.svg` | Cartesian | line + line |
| Base reactions | `base_reactions.svg` | Pie | pie slices |
| Story forces | `story_force_vx/vy/my/mx.svg` | Cartesian (swapped axes) | bar |
| Drift wind/seismic | `drift_*_x/y.svg` | Cartesian (swapped axes) | line demand + dashed line limit |
| Displacement wind | `displacement_wind_x/y.svg` | Cartesian (swapped axes) | line demand + dashed line limit |
| Pier shear wind/seismic | `pier_shear_stress_*.svg` | Cartesian (swapped axes) | bar demand + dashed line limit |
| Pier axial categories | `pier_axial_gravity/wind/seismic.svg` | Cartesian (swapped axes) | multi-line per pier + dashed zero |

## 10) JSON Contract: `ext-calc` -> `ext-render`

`ext-render` consumes `ext_calc::output::CalcOutput` serialized in camelCase JSON (`calc_output.json`).

Contract behavior:

- Each top-level check block is `Option<T>` in Rust and nullable/optional in JSON.
- `ext-render` emits a chart only when that block exists.
- Missing block = no chart for that check (no error in normal flow).

## 10.1 Top-level shape used by ext-render

```json
{
  "meta": { "versionId": "...", "branch": "...", "code": "...", "generatedAt": "...", "units": { "force": "...", "length": "...", "stress": "...", "moment": "..." } },
  "summary": { "overallStatus": "...", "checkCount": 0, "passCount": 0, "failCount": 0, "lines": [] },
  "modal": { "...": "ModalOutput" },
  "baseReactions": { "...": "BaseReactionsOutput" },
  "storyForces": { "...": "StoryForcesOutput" },
  "driftWind": { "x": { "...": "DriftOutput" }, "y": { "...": "DriftOutput" } },
  "driftSeismic": { "x": { "...": "DriftOutput" }, "y": { "...": "DriftOutput" } },
  "displacementWind": { "x": { "...": "DisplacementOutput" }, "y": { "...": "DisplacementOutput" } },
  "torsional": { "...": "TorsionalOutput (not charted by ext-render)" },
  "pierShearStressWind": { "...": "PierShearStressOutput" },
  "pierShearStressSeismic": { "...": "PierShearStressOutput" },
  "pierAxialStress": { "...": "PierAxialStressOutput" }
}
```

## 10.2 JSON-path mapping by chart asset

| Chart asset | Required JSON paths from `calc_output.json` |
| --- | --- |
| `images/modal.svg` | `$.modal.rows[*].mode`, `$.modal.rows[*].sumUx`, `$.modal.rows[*].sumUy` |
| `images/base_reactions.svg` | `$.baseReactions.rows[*].outputCase`, `$.baseReactions.rows[*].fzKip` |
| `images/story_force_vx.svg` | `$.storyForces.rows[*].story`, `$.storyForces.rows[*].maxVxKip` |
| `images/story_force_vy.svg` | `$.storyForces.rows[*].story`, `$.storyForces.rows[*].maxVyKip` |
| `images/story_force_my.svg` | `$.storyForces.rows[*].story`, `$.storyForces.rows[*].maxMyKipFt` |
| `images/story_force_mx.svg` | `$.storyForces.rows[*].story`, `$.storyForces.rows[*].maxMxKipFt` |
| `images/drift_wind_x.svg` | `$.driftWind.x.rows[*].story`, `$.driftWind.x.rows[*].maxDriftXPos`, `$.driftWind.x.rows[*].maxDriftXNeg`, `$.driftWind.x.allowableRatio` |
| `images/drift_wind_y.svg` | `$.driftWind.y.rows[*].story`, `$.driftWind.y.rows[*].maxDriftYPos`, `$.driftWind.y.rows[*].maxDriftYNeg`, `$.driftWind.y.allowableRatio` |
| `images/drift_seismic_x.svg` | `$.driftSeismic.x.rows[*].story`, `$.driftSeismic.x.rows[*].maxDriftXPos`, `$.driftSeismic.x.rows[*].maxDriftXNeg`, `$.driftSeismic.x.allowableRatio` |
| `images/drift_seismic_y.svg` | `$.driftSeismic.y.rows[*].story`, `$.driftSeismic.y.rows[*].maxDriftYPos`, `$.driftSeismic.y.rows[*].maxDriftYNeg`, `$.driftSeismic.y.allowableRatio` |
| `images/displacement_wind_x.svg` | `$.displacementWind.x.rows[*].story`, `$.displacementWind.x.rows[*].maxDispXPosFt`, `$.displacementWind.x.rows[*].maxDispXNegFt`, `$.displacementWind.x.dispLimit.value` |
| `images/displacement_wind_y.svg` | `$.displacementWind.y.rows[*].story`, `$.displacementWind.y.rows[*].maxDispYPosFt`, `$.displacementWind.y.rows[*].maxDispYNegFt`, `$.displacementWind.y.dispLimit.value` |
| `images/pier_shear_stress_wind.svg` | `$.pierShearStressWind.perPier[*].pier`, `$.pierShearStressWind.perPier[*].story`, `$.pierShearStressWind.perPier[*].stressPsi`, `$.pierShearStressWind.perPier[*].limitIndividual` |
| `images/pier_shear_stress_seismic.svg` | `$.pierShearStressSeismic.perPier[*].pier`, `$.pierShearStressSeismic.perPier[*].story`, `$.pierShearStressSeismic.perPier[*].stressPsi`, `$.pierShearStressSeismic.perPier[*].limitIndividual` |
| `images/pier_axial_gravity.svg` | `$.pierAxialStress.piers[*]` filtered by `category == "gravity"`: use `story`, `pierLabel`, `fa.value` |
| `images/pier_axial_wind.svg` | `$.pierAxialStress.piers[*]` filtered by `category == "wind"`: use `story`, `pierLabel`, `fa.value` |
| `images/pier_axial_seismic.svg` | `$.pierAxialStress.piers[*]` filtered by `category == "seismic"`: use `story`, `pierLabel`, `fa.value` |

## 10.3 Contract fields used outside `calc_output.json`

Base reaction pie grouping uses `RenderConfig.base_reaction_groups` (from config), not calc output JSON:

- Source path in config: `[[calc.base-reactions.pie-groups]]`
- Flow: `ext-api::build_render_config` -> `ext-render::RenderConfig`

This means pie grouping is a two-input contract:

1. `calc_output.json` base reaction rows
2. runtime `RenderConfig.base_reaction_groups`

## 11) Mapping with your newest `config.toml`

Using the config you posted, this is the effective chart contract mapping:

| Config key | Drives ext-calc output field(s) | Consumed by ext-render chart(s) |
| --- | --- | --- |
| `[calc].modal-case`, `[calc.modal].*` | `modal.rows`, `modal.threshold` | `images/modal.svg` |
| `[calc.base-reactions].elf/rsa/rsa-scale-min` | `baseReactions.rows`, `directionX/Y` | `images/base_reactions.svg` (rows only) |
| `[[calc.base-reactions.pie-groups]]` | not stored in `calc_output.json`; goes via `RenderConfig.base_reaction_groups` | `images/base_reactions.svg` slice filtering/grouping |
| `[calc.story-forces].story-force-x-cases/y-cases` | `storyForces.rows[*].maxVx/Vy/My/Mx*` | `images/story_force_vx/vy/my/mx.svg` |
| `[calc.drift-wind].drift-*-cases`, `drift-limit` | `driftWind.x/y.rows`, `driftWind.x/y.allowableRatio` | `images/drift_wind_x/y.svg` |
| `[calc.drift-seismic].drift-*-cases`, `drift-limit` | `driftSeismic.x/y.rows`, `driftSeismic.x/y.allowableRatio` | `images/drift_seismic_x/y.svg` |
| `[calc.displacement-wind].disp-*-cases`, `disp-limit-h` | `displacementWind.x/y.rows`, `displacementWind.x/y.dispLimit.value` | `images/displacement_wind_x/y.svg` |
| `[calc.torsional].*` | `torsional.x/y` data | not charted in ext-render (table-only in ext-report) |
| `[calc.pier-shear-stress-wind].*` | `pierShearStressWind.perPier`, limits | `images/pier_shear_stress_wind.svg` |
| `[calc.pier-shear-stress-seismic].*` | `pierShearStressSeismic.perPier`, limits | `images/pier_shear_stress_seismic.svg` |
| `[calc.pier-axial-stress].*` | `pierAxialStress.piers[*].category/fa` | `images/pier_axial_gravity/wind/seismic.svg` |

## 11.1 Unit note for your posted config

You set:

- `[extract].units = "US_Kip_Ft"`

Current `ext-calc` unit context is resolved from `project.units` (`Config.project.units_or_default()`), not `extract.units`.

Practical implication:

- if `[project].units` is not set in local config, calc defaults to `"kip-ft-F"` regardless of `[extract].units`
- this matters for labels/quantity conversions in outputs

## 12) Minimal concrete contract example (with your config shape)

```json
{
  "modal": {
    "threshold": 0.9,
    "rows": [{ "mode": 1, "sumUx": 0.10, "sumUy": 0.08 }],
    "pass": true
  },
  "baseReactions": {
    "rows": [{ "outputCase": "Dead", "fzKip": -1234.0 }],
    "directionX": { "ratio": 1.03, "pass": true },
    "directionY": { "ratio": 1.01, "pass": true }
  },
  "driftWind": {
    "x": {
      "allowableRatio": 0.0025,
      "rows": [{ "story": "L10", "maxDriftXPos": 0.0019, "maxDriftXNeg": -0.0014 }]
    }
  },
  "displacementWind": {
    "x": {
      "dispLimit": { "value": 0.35, "unit": "ft" },
      "rows": [{ "story": "L10", "maxDispXPosFt": 0.21, "maxDispXNegFt": -0.18 }]
    }
  },
  "pierShearStressWind": {
    "perPier": [{ "story": "L10", "pier": "P1", "stressPsi": 420.0, "limitIndividual": 8.0 }]
  },
  "pierAxialStress": {
    "piers": [{ "category": "gravity", "story": "L10", "pierLabel": "P1", "fa": { "value": 0.82, "unit": "ksi" } }]
  }
}
```

This example shows field names and hierarchy expected by `ext-render`; numeric values are illustrative.
