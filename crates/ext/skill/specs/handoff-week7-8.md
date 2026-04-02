# Week 7-8 Handoff

## Summary

Week 7-8 has been started and the foundation layer is now in place.

What is genuinely done:

- `ext-db` now has a shared `[calc]` config model
- the config split is implemented as strict local-only on writes for `project.sidecar-path` and `project.units`
- the unit default now matches the Week 7-8 spec baseline: `kip-ft-F`
- `ext-calc` now exists as a real library surface with shared output types, unit conversion, code params, loader scaffolding, batch-1 loaders, and batch-2 loaders
- `ext-render` and `ext-report` now have library surfaces instead of being only empty binary placeholders
- the full workspace compiles with `cargo check --target-dir .codex-target`

What is not done yet:

- all engineering checks
- real `CalcRunner` outputs beyond “all required inputs loaded” summary data
- render/report production pipeline
- `ext-api` and CLI integration
- broader integration beyond the currently passing `ext-db` / `ext-calc` library test scope

---

## Implemented

### 1. `ext-db` config foundation

Added:

- `crates/ext-db/src/config/calc.rs`

Updated:

- `crates/ext-db/src/config/mod.rs`
- `crates/ext-db/src/config/project.rs`
- `crates/ext-db/src/config/extract.rs`

Behavior now:

- `Config` includes `calc: CalcConfig`
- shared config loads `[calc]` from `config.toml`
- local config still owns machine-specific project fields
- `Config::write_shared()` strips `project.sidecar-path` and `project.units`
- `Config::write_local()` writes only local-only project fields
- legacy shared reads still work because load still merges shared project values with local overrides
- `ProjectConfig::units_or_default()` now defaults to `"kip-ft-F"`

Also added config tests in `config/mod.rs` covering:

- shared writes omitting local-only project fields
- local writes persisting only local project fields
- loading legacy shared project fields with local override
- this crate needed `tempfile` added under `dev-dependencies` so its tests compile and run

### 2. `ext-calc` library-first scaffolding

Added:

- `crates/ext-calc/src/lib.rs`
- `crates/ext-calc/src/output.rs`
- `crates/ext-calc/src/unit_convert.rs`
- `crates/ext-calc/src/code_params.rs`
- `crates/ext-calc/src/checks/mod.rs`
- `crates/ext-calc/src/tables/mod.rs`

Current public shape:

- `CalcRunner::run_all(...) -> Result<CalcOutput>`
- shared DTOs live in `ext-calc::output`
- `CodeParams::from_config(&Config)` exists
- `UnitContext` supports:
  - `kip-ft-F`
  - `kN-m-C`

Current `CalcRunner` behavior:

- loads batch-1 and batch-2 tables
- returns a summary describing loaded inputs
- does not run engineering checks yet
- sets all detailed result sections to `None`

### 3. `ext-calc` batch-1 loaders

Implemented loaders:

- `crates/ext-calc/src/tables/story_def.rs`
- `crates/ext-calc/src/tables/material_props.rs`
- `crates/ext-calc/src/tables/pier_section.rs`
- `crates/ext-calc/src/tables/group_assignments.rs`

What each one does:

- `story_def`
  - reads `story_definitions.parquet`
  - computes `elevation_ft` from reversed cumulative story height
- `material_props`
  - reads `material_properties_concrete_data.parquet`
  - derives `fc_ksi` and `fc_psi`
- `pier_section`
  - reads `pier_section_properties.parquet`
  - derives `acv_in2` and `ag_in2`
- `group_assignments`
  - reads `group_assignments.parquet`
  - returns `HashMap<String, Vec<String>>`

Loader/unit tests in these modules now run successfully against the realistic fixture set.

### 4. `ext-calc` batch-2 loaders

Implemented loaders:

- `crates/ext-calc/src/tables/joint_drift.rs`
- `crates/ext-calc/src/tables/pier_forces.rs`
- `crates/ext-calc/src/tables/modal.rs`
- `crates/ext-calc/src/tables/base_reactions.rs`
- `crates/ext-calc/src/tables/story_forces.rs`
- `crates/ext-calc/src/tables/material_by_story.rs`

Updated:

- `crates/ext-calc/src/tables/mod.rs`
- `crates/ext-calc/src/lib.rs`

What each one does:

- `joint_drift`
  - reads `joint_drifts.parquet`
  - preserves ETABS `DriftX` / `DriftY` as direct drift ratios
- `pier_forces`
  - reads `pier_forces.parquet`
  - derives `shear_v2_abs_kip` for later governing shear selection
- `modal`
  - reads `modal_participating_mass_ratios.parquet`
  - preserves cumulative `SumUX` / `SumUY` / `SumRX` / `SumRY` / `SumRZ`
- `base_reactions`
  - reads `base_reactions.parquet`
  - maps directional force and moment components directly
- `story_forces`
  - reads `story_forces.parquet`
  - maps story-level shear/torsion inputs for later torsional work
- `material_by_story`
  - reads `material_list_by_story.parquet`
  - maps story material weight/takeoff rows for later summaries

`CalcRunner::run_all()` now loads all ten planned Week 7-8 input tables and reports them in the summary output, but still does not perform engineering checks.

Additional implementation detail from verification fixes:

- the realistic Parquet fixtures expose several engineering columns as strings instead of strict numeric columns
- table loaders were updated to parse numeric-looking strings at the value layer rather than assuming `Float64` / `Int64` typed columns
- nullable `NumPieces` and `NumStuds` in `material_list_by_story.parquet` are now treated as `0.0`

### 5. `ext-render` first library surface

Added:

- `crates/ext-render/src/lib.rs`

Current behavior:

- consumes `ext-calc::output::CalcOutput`
- can emit simple SVG drift charts from:
  - `calc.drift_wind`
  - `calc.drift_seismic`
- this is a lightweight first pass, not yet the full proof-of-concept chart system

### 6. `ext-report` first library surface

Added:

- `crates/ext-report/src/lib.rs`

Current behavior:

- consumes `CalcOutput`
- provides:
  - `ReportProjectMeta`
  - `ReportInput`
  - `build_typst_document(...)`
- `compile_pdf(...)` is still an intentional stub that returns an error

### 7. Workspace dependency fix

Updated:

- `Cargo.toml`

Reason:

- `polars 0.53.0` would not resolve with workspace `chrono 0.4.44`
- workspace `chrono` was pinned down to `0.4.41` to match the Polars dependency range and restore compile success

---

## Remaining Work

### 1. Engineering checks

Still to implement:

- modal participation
- base shear
- wind drift + roof displacement
- seismic drift
- torsional stub output
- pier shear wind
- pier shear seismic
- pier axial

Recommended sequencing:

1. modal
2. base shear
3. drift wind
4. drift seismic
5. pier shear wind
6. pier shear seismic
7. pier axial
8. torsional stub

### 2. Real `CalcRunner`

`CalcRunner::run_all()` must be upgraded from “load every required table and summarize” to:

- building lookup maps
- running enabled checks
- assembling real `CalcOutput`
- writing stable JSON snapshots in tests later

### 3. Render/report continuation

`ext-render`

- keep drift-first
- refine the SVG output toward the proof-of-concept layout
- convert from actual `DriftOutput` results once the drift checks exist

`ext-report`

- replace the current Typst string skeleton with the real proof-of-concept layout structure
- wire image references from rendered SVG files
- add real PDF compilation using Typst once the report contract is stable

### 4. Integration not started yet

Still deferred:

- `ext-api` orchestration for calc/render/report
- CLI commands:
  - `ext calc`
  - `ext render`
  - `ext report`

This is intentional. Core calc types and behavior should stabilize first.

---

## Verification Status

Passed:

- `cargo check --target-dir .codex-target`
- `cargo fmt --all`
- `cargo test -j 2 -p ext-calc --lib --target-dir .codex-target`
- `cargo test -j 2 -p ext-db -p ext-calc --lib --target-dir .codex-target`

Important verification note:

- use limited cargo parallelism such as `-j 2` for this workspace on constrained machines
- unrestricted cargo parallelism causes avoidable CPU/RAM pressure during the Polars-heavy compile/test path

---

## Important Notes

### Config note

The sample axial combo list still needs this syntax fix in project config examples:

```toml
[calc.pier-axial]
load-combos = [
    "EVN_LRFD_GRA",
    "EVN_LRFD_EQ",
    "EVN_LRFD_Wind",
]
```

Without that comma after `"EVN_LRFD_GRA"`, the TOML is invalid.

### Crate-shape note

The new crates now have `lib.rs`, but the original `main.rs` placeholder files still exist physically in:

- `crates/ext-calc/src/main.rs`
- `crates/ext-render/src/main.rs`
- `crates/ext-report/src/main.rs`

They are no longer the primary design target, but they have not been removed in this pass.

### Fixture note

Current testing fixture set:

- `crates/ext-calc/tests/fixtures/results_realistic/`

There is still no `results_minimal/` fixture set.

That is acceptable for now; it can be added later if formula tests become too awkward or slow against the realistic fixture set.

---

## Recommended Next Continuation

The next agent should start here:

1. keep using constrained cargo parallelism for local verification, for example:
   - `cargo test -j 2 -p ext-db -p ext-calc --lib --target-dir .codex-target`
2. implement checks 1-4:
   - modal
   - base shear
   - drift wind
   - drift seismic
3. upgrade `CalcRunner` from loader-summary-only to real `CalcOutput`
4. add stable calc snapshots once the first four checks are wired

Only after that should the next pass expand `ext-render`, `ext-report`, and then `ext-api`/CLI integration.
