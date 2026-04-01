# Week 7-8 Handoff

## Summary

Week 7-8 has been started and the foundation layer is now in place.

What is genuinely done:

- `ext-db` now has a shared `[calc]` config model
- the config split is implemented as strict local-only on writes for `project.sidecar-path` and `project.units`
- the unit default now matches the Week 7-8 spec baseline: `kip-ft-F`
- `ext-calc` now exists as a real library surface with shared output types, unit conversion, code params, loader scaffolding, and batch-1 loaders
- `ext-render` and `ext-report` now have library surfaces instead of being only empty binary placeholders
- the full workspace compiles with `cargo check --target-dir .codex-target`

What is not done yet:

- batch-2 loaders
- all engineering checks
- real `CalcRunner` outputs beyond loader summary
- render/report production pipeline
- `ext-api` and CLI integration
- test execution for the new Week 7-8 units/loaders

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

- loads batch-1 tables
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

Loader/unit tests were added in these modules, but have not been executed yet in this pass.

### 4. `ext-render` first library surface

Added:

- `crates/ext-render/src/lib.rs`

Current behavior:

- consumes `ext-calc::output::CalcOutput`
- can emit simple SVG drift charts from:
  - `calc.drift_wind`
  - `calc.drift_seismic`
- this is a lightweight first pass, not yet the full proof-of-concept chart system

### 5. `ext-report` first library surface

Added:

- `crates/ext-report/src/lib.rs`

Current behavior:

- consumes `CalcOutput`
- provides:
  - `ReportProjectMeta`
  - `ReportInput`
  - `build_typst_document(...)`
- `compile_pdf(...)` is still an intentional stub that returns an error

### 6. Workspace dependency fix

Updated:

- `Cargo.toml`

Reason:

- `polars 0.53.0` would not resolve with workspace `chrono 0.4.44`
- workspace `chrono` was pinned down to `0.4.41` to match the Polars dependency range and restore compile success

---

## Remaining Work

### 1. `ext-calc` loader batch 2

Still to implement:

- `joint_drifts`
- `pier_forces`
- `modal`
- `base_reactions`
- `story_forces`
- `material_by_story`

Recommended next step:

- add the six missing loader modules
- keep the same fixture-first test style used in batch 1

### 2. Engineering checks

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

### 3. Real `CalcRunner`

`CalcRunner::run_all()` must be upgraded from “load and summarize” to:

- loading all required tables
- building lookup maps
- running enabled checks
- assembling real `CalcOutput`
- writing stable JSON snapshots in tests later

### 4. Render/report continuation

`ext-render`

- keep drift-first
- refine the SVG output toward the proof-of-concept layout
- convert from actual `DriftOutput` results once the drift checks exist

`ext-report`

- replace the current Typst string skeleton with the real proof-of-concept layout structure
- wire image references from rendered SVG files
- add real PDF compilation using Typst once the report contract is stable

### 5. Integration not started yet

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

Not run yet in this pass:

- `cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target`

The test command was requested but not executed to completion because the session was interrupted before approval-based execution continued.

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

1. run `cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target`
2. fix any test/compile issues from the new batch-1 foundation
3. implement loader batch 2
4. implement checks 1-4
5. upgrade `CalcRunner` from summary-only to real `CalcOutput`

Only after that should the next pass expand `ext-render`, `ext-report`, and then `ext-api`/CLI integration.
