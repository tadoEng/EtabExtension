# Week 7–8 Execution Handoff

## Summary

Proceed with Week 7–8 as a **library-first** implementation across:

- `crates/ext-calc`
- `crates/ext-render`
- `crates/ext-report`

Do **not** introduce an `ext-types` crate yet. Put the shared data contract in `ext-calc::output`, and have render/report consume it. The current realistic fixture set is already good enough for the first pass.

New repo facts discovered during planning:
- the new crates already exist and are already in the workspace
- they are currently `main.rs` placeholders and need to become library crates
- `ext-api::report` is still a clean stub, so there is no conflicting implementation to unwind
- the realistic fixture set is small enough to use early
- current `ProjectConfig::units_or_default()` still defaults to `"kip-in-F"`, which conflicts with the Week 7–8 spec baseline of `"kip-ft-F"`

Locked preference:
- config split should be **strict local-only**
    - shared reads may tolerate legacy `sidecar-path` / `units`
    - shared writes must stop persisting them

## Key Changes

### 1. Harden the config split first
- Add `ext-db/src/config/calc.rs` with the full shared `[calc]` model.
- Update `ext-db/src/config/mod.rs` so `calc` is loaded from shared config only.
- Keep `project.sidecar-path` and `project.units` effectively local-only going forward.
- Implement strict local-only behavior:
    - `Config::load()` may still read legacy values from shared config for backward compatibility
    - `Config::write_shared()` must not write `sidecar-path` or `units`
    - `Config::write_local()` is the only writer for those fields
- Fix the unit default mismatch:
    - Week 7–8 logic should default to `"kip-ft-F"`, not `"kip-in-F"`
- Extend `TableSelections` with:
    - `group_assignments`
    - `material_properties_concrete_data`
    - `material_list_by_story`

Important config note:
- the pasted `[calc.pier-axial].load-combos` list is invalid TOML unless a comma is added after `"EVN_LRFD_GRA"`

### 2. Convert the new crates to reusable libraries
- Replace the `main.rs` placeholder shape with `lib.rs` as the primary crate surface in:
    - `ext-calc`
    - `ext-render`
    - `ext-report`
- Optional binaries can come later, but the reusable lib surface is the real integration target.
- `ext-api` should orchestrate them later; the CLI should never bypass `ext-api`.

### 3. Build `ext-calc` as the system of record
Create these first:
- `output.rs`
- `unit_convert.rs`
- `code_params.rs`
- `tables/mod.rs`
- `checks/mod.rs`

Public contract:
- `CalcRunner::run_all(...) -> Result<CalcOutput>`
- `CalcOutput` and nested result structs live in `ext-calc::output`

Implementation order:
1. unit conversion + code params
2. loader batch 1
    - story definitions
    - material props
    - pier sections
    - group assignments
3. loader batch 2
    - joint drifts
    - pier forces
    - modal
    - base reactions
    - story forces
    - material-by-story
4. checks 1–4
    - modal
    - base shear
    - wind drift/displacement
    - seismic drift
5. checks 6–8 plus torsional stub
    - pier shear wind
    - pier shear seismic
    - pier axial
    - torsional note output
6. `CalcRunner` assembly and JSON snapshot output

### 4. Build `ext-render` on `CalcOutput`, not raw tables
- Use the existing drift SVG proof-of-concept as the first-pass visual baseline.
- First render target is drift only.
- Add a converter from `CalcOutput.drift_wind` / `CalcOutput.drift_seismic` into render series data.
- Keep SVG as the first output format.
- Delay extra chart types until the drift pipeline is stable.

### 5. Build `ext-report` on `CalcOutput` + rendered assets
- Use the existing Typst proof-of-concept as the first-pass report composition baseline.
- Remove all random/demo data generation.
- Report inputs should be:
    - project metadata
    - `CalcSummary`
    - detailed calc outputs
    - rendered SVG asset references
- First report scope:
    - cover
    - summary
    - drift figure pages
    - calc tables

### 6. Integrate through `ext-api` last
After `ext-calc`, `ext-render`, and `ext-report` stabilize:
- add `ext-api` wrappers for calc/render/report
- then add CLI commands:
    - `ext calc`
    - `ext render`
    - `ext report`

## Test Plan

### Config and unit tests
- shared `[calc]` loads correctly
- shared writes no longer persist local-only project fields
- local reads/writes preserve `sidecar-path` and `units`
- legacy shared `units` / `sidecar-path` can still be read
- default preset path uses `"kip-ft-F"`
- invalid TOML combo lists fail clearly

### Loader tests
Use:
- `crates/ext-calc/tests/fixtures/results_realistic`

Assert:
- schema mapping is exact
- `story_definitions -> elevation_ft` is correct
- `WidthBot * ThickBot -> Acv/Ag` is correct
- `Fc -> fc_ksi` is correct
- group aggregation is correct
- case/group filtering behaves correctly

### Check tests
First pass:
- deterministic smoke/invariant tests
- `CalcOutput` JSON snapshots

Second pass after engineering references are supplied:
- modal threshold mode exact value
- base shear ratio exact value
- governing wind drift exact value
- pier shear exact value
- axial exact value

### Render/report tests
- drift SVG render smoke test from fixed `CalcOutput`
- Typst PDF compile smoke test from fixed calc/report inputs
- one fixture pipeline test:
    - Parquet fixture -> `CalcOutput` -> drift SVG -> report PDF

## Assumptions

- No `ext-types` crate in the initial implementation.
- `ext-calc` is the canonical shared DTO boundary.
- `results_realistic` is the only required fixture set for the first pass.
- The drift renderer proof-of-concept is the approved first-pass style.
- The Typst proof-of-concept is the approved first-pass report style.
- Torsional irregularity remains a stub until engineering guidance arrives.
- `ext-api::report` is intentionally the later integration point, not the starting implementation surface.
