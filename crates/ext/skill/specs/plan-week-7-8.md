# Week 7–8 Initial Implementation Plan

## Summary

Implement Week 7–8 in a **library-first** shape, using the new realistic Parquet fixture set as the first test source. We will **not** create an `ext-types` crate yet; the shared DTO/output contract will live in `ext-calc` and be consumed by `ext-render`, `ext-report`, and later `ext-api`/CLI.

Locked decisions:
- `ext-calc`, `ext-render`, and `ext-report` become reusable library crates first
- shared engineering config lives in `[calc]` inside `config.toml`
- `[project].units` stays machine-local in `config.local.toml`
- the current proof-of-concept render/report style is the temporary visual baseline
- realistic fixture-first testing is the default; `results_minimal` is deferred until needed
- first pass focuses on **stable calc/output contracts**, then drift rendering/report composition on top of them

Important correction to carry into implementation:
- the pasted `[calc.pier-axial].load-combos` list needs a comma after `"EVN_LRFD_GRA"` or the TOML is invalid

## Key Changes

### 1. Crate shape and boundaries
- Convert `ext-calc`, `ext-render`, and `ext-report` from `main.rs` placeholders into library crates with `src/lib.rs`.
- Do not add `ext-types`; put the shared DTOs in `ext-calc::output`.
- `ext-render` consumes `ext-calc::output::*`, not raw Parquet.
- `ext-report` consumes `CalcOutput` plus rendered asset metadata, not raw Parquet and not ad hoc demo structs.
- `ext-api` becomes the orchestration layer later; no direct CLI-to-calc/render/report logic.

### 2. `ext-db` config foundation
- Add `ext-db::config::calc` with `CalcConfig`, `ModalCalcConfig`, `BaseShearCalcConfig`, `DriftCalcConfig`, `PierShearCalcConfig`, and `PierAxialCalcConfig`.
- Update config loading so `calc` comes only from shared `config.toml`.
- Keep local-only ownership of `project.sidecar-path` and `project.units`.
- Extend extract table selections to cover:
    - `group_assignments`
    - `material_properties_concrete_data`
    - `material_list_by_story`
    - keep existing `story_forces` in the Week 7–8 extraction set
- Add config tests for:
    - shared/local ownership
    - defaulting behavior
    - new extract table serialization
    - invalid TOML rejection for malformed combo lists

### 3. `ext-calc` phase 1 and 2
- Build `ext-calc` around four stable modules first:
    - `output.rs`
    - `unit_convert.rs`
    - `code_params.rs`
    - `tables/*`
- Public API for first calc pass:
    - `CalcRunner::run_all(...) -> Result<CalcOutput>`
    - public output structs in `ext-calc::output`
- `UnitContext` supports `kip-ft-F` and `kN-m-C`, converting formulas into kip/inch internally.
- Loader order:
    1. `story_def`
    2. `material_props`
    3. `pier_section`
    4. `group_assignments`
    5. `joint_drift`
    6. `pier_forces`
    7. `modal`
    8. `base_reactions`
    9. `story_forces`
    10. `material_by_story`
- Check order:
    1. modal participation
    2. base shear
    3. wind drift + roof displacement
    4. seismic drift
    5. torsional stub
    6. pier shear wind
    7. pier shear seismic
    8. pier axial
- Because hand-check reference values are not finalized yet, first-pass assertions should be:
    - exact schema/column/derived-field assertions
    - exact unit-conversion assertions
    - deterministic structural smoke assertions from fixture data
    - stable JSON snapshots for `CalcOutput`
- Once you provide hand-check values later, we tighten the check tests from “smoke + invariant” to “exact engineering result”.

### 4. `ext-render` first pass
- Use your drift-chart proof-of-concept as the baseline look and API direction.
- First render target is **drift only**, not all chart types at once.
- Adapt the input model away from ad hoc `StoryDriftSeries` into a converter from `CalcOutput.drift_wind` / `CalcOutput.drift_seismic`.
- Output format for first pass:
    - SVG drift charts
    - one file per direction
- Keep the palette, legend strip, and chart composition style from the proof code.
- Delay PNG/PDF-specific render backends until after the SVG path is stable.

### 5. `ext-report` first pass
- Use the Typst proof-of-concept as the baseline report composition style.
- Strip out demo/random data generation and replace it with:
    - project metadata
    - `CalcSummary`
    - detailed calc sections
    - drift SVG references from `ext-render`
- First report scope:
    - title/cover
    - summary section
    - drift figure pages
    - calc result tables
- Do not try to fully style all eight checks in the first report pass; wire the layout so more sections can be added without redesigning the world/template layer.

### 6. Integration order
- Milestone A: `ext-db` config + `ext-calc` scaffolding + unit conversion
- Milestone B: loaders + checks 1–4 + `CalcOutput` snapshots
- Milestone C: checks 6–8 + torsional stub + `CalcRunner`
- Milestone D: `ext-render` drift SVGs from `CalcOutput`
- Milestone E: `ext-report` Typst report from calc output + drift assets
- Milestone F: `ext-api` orchestration (`run_calc`, then render/report wrappers)
- Milestone G: CLI commands `ext calc`, `ext render`, `ext report`

## Public APIs / Types

- New shared config type:
    - `ext_db::config::CalcConfig`
- New calc runtime types:
    - `EtabsPreset`
    - `UnitContext`
    - `CodeParams`
    - `CalcRunner`
    - `CalcOutput` and nested output/result structs
- Shared DTO location:
    - `ext-calc::output::*`
- First stable integration surface:
    - `CalcRunner::run_all(...)`
- Later orchestration surface:
    - `ext-api` wrappers for calc/render/report, with CLI calling only `ext-api`

## Test Plan

### Config and units
- load shared `[calc]` correctly from `config.toml`
- preserve local-only `[project].units`
- serialize new extract table selections correctly
- verify `kip-ft-F` and `kN-m-C` conversions to internal kip/inch units
- reject malformed TOML list syntax

### Loaders
- use `crates/ext-calc/tests/fixtures/results_realistic/` as the first fixture source
- assert exact derived values where deterministic:
    - story elevations
    - `Acv`
    - `Ag`
    - `fc_ksi`
    - group aggregation
- assert exact column mapping, filtering, and optional-step handling

### Checks
- first pass: deterministic smoke/invariant tests plus JSON snapshots
- later pass, after you provide hand-check references:
    - modal threshold mode exact value
    - base shear ratio exact value
    - one governing wind drift exact result
    - one pier shear exact result
    - one axial exact result

### Render/report
- render smoke test produces drift SVGs from fixed `CalcOutput`
- report smoke test compiles Typst output to PDF from fixed `CalcOutput`
- integration test path:
    - Parquet fixture -> `CalcOutput` -> drift SVGs -> report PDF

## Assumptions and Defaults

- No `ext-types` crate in this phase.
- `ext-calc` owns the shared output contract.
- `results_realistic` is the only fixture set for the first implementation pass.
- `results_minimal` is deferred until tests become too slow or too hard to assert cleanly.
- The current proof-of-concept drift chart style is the approved first-pass render style.
- The current Typst proof-of-concept is the approved first-pass report style.
- Torsional irregularity remains a stub with explicit note output.
- The current shared config values you pasted are the baseline defaults once the axial combo TOML comma is fixed.
