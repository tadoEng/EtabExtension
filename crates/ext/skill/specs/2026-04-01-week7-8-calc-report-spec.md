# Week 7-8 Implementation Spec
# ETABS Extension - Calc / Render / Report Pipeline

**Date:** 2026-04-01  
**Phase:** Phase 2, Weeks 7-8  
**Status:** Proposed for implementation  
**Author:** EtabExtension Team

---

## 1. Purpose

This spec merges two Week 7-8 proposals into one implementation baseline:

1. the confirmed Parquet/table design for structural calculations
2. the config ownership and unit-conversion design

Merged decisions:

- The Parquet schemas below are the ground truth and take precedence over any older placeholder docs.
- `[calc]` belongs entirely in shared `config.toml` because it represents engineering intent.
- `[project] units` stays in `config.local.toml` because it is a machine-local ETABS extraction preset.
- `ext-calc` ships first and becomes the stable data contract.
- `ext-render` and `ext-report` build on top of the stable `CalcOutput` contract instead of inventing their own data flow.
- Check 5 (torsional irregularity) remains an explicit stub until the engineer supplies the exact method.

This spec is written to favor small, testable delivery slices rather than one large Week 7 or Week 8 batch.

---

## 2. Scope

Week 7-8 introduces three new work areas:

- `ext-calc`: load extracted ETABS Parquet tables, convert units, run code checks, and emit `calc_output.json`
- `ext-render`: generate charts and figure assets from `CalcOutput`
- `ext-report`: compile a tabloid landscape report from project metadata, calc outputs, and rendered assets

Primary design targets:

- code standard: ACI 318-14 + ASCE 7
- default unit preset in current project fleet: `kip-ft-F`
- report paper size: tabloid landscape (`17 x 11 in`)
- load case / combo selection comes only from config, never hard-coded in checks

Out of scope for this phase:

- final torsional irregularity formula
- new ETABS sidecar table contracts beyond the extra table selections listed here
- speculative "validate-file" style sidecar work

---

## 3. Current Repo Impact

The merged design affects these areas:

- `crates/ext-db`
  - add shared `CalcConfig`
  - add 4 new extract table selections
- new crate `crates/ext-calc`
  - loaders
  - unit conversion
  - code params
  - checks
  - JSON output contract
- new crate `crates/ext-render`
  - render charts/figures from `CalcOutput`
- new crate `crates/ext-report`
  - produce final report artifacts, likely via Typst
- `crates/ext-api`
  - add orchestration entry points for calc/render/report
- `crates/ext`
  - CLI commands for `ext calc`, `ext render`, `ext report`

Important repo-fit note:

- `ext-db::Config` currently only resolves `project`, `extract`, `llm`, `git`, `paths`, and `onedrive`
- `TableSelections` currently has 7 tables, not the full Week 7-8 set
- there is no existing `ext-calc`, `ext-render`, or `ext-report` crate yet

---

## 4. Config Ownership Decision

### 4.1 Shared vs local

The existing two-tier config model stays in place:

- `.etabs-ext/config.toml`: shared, committed, synced
- `.etabs-ext/config.local.toml`: local, git-ignored, machine-specific

### 4.2 Final ownership rule

| Section | File | Reason |
|---|---|---|
| `[project] name` | `config.toml` | project identity is shared |
| `[project] sidecar-path` | `config.local.toml` | machine path is local |
| `[project] units` | `config.local.toml` | ETABS extraction preset is local |
| `[extract]` | `config.toml` | extraction contract is shared |
| `[calc]` | `config.toml` | engineering intent is shared |
| `[git]` | `config.local.toml` | user identity is local |
| `[llm]` | `config.local.toml` | secrets are local |
| `[onedrive]` | `config.local.toml` | machine sync state is local |

### 4.3 Implementation rule in `ext-db`

`CalcConfig` is added to the shared config file only. It is not merged from local config.

```rust
Ok(Self {
    project: base.project.merge(local.project),
    extract: base.extract,
    calc:    base.calc,
    llm:     local.llm,
    git:     local.git,
    paths:   local.paths,
    onedrive: local.onedrive,
})
```

### 4.4 Base shear default clarification

The two draft specs disagreed on the RSA/ELF minimum ratio (`0.85` vs `1.0`).

Merged decision:

- project config should set `rsa-scale-min` explicitly
- code fallback should be conservative at `1.0`
- if the engineer intentionally wants `0.85`, that must be written in `config.toml`

This avoids hidden assumptions and keeps engineering decisions visible in version control.

---

## 5. Unit System Design

### 5.1 Source of truth

`config.local.toml [project] units` is the ETABS preset string used for extraction.
All Parquet values reflect that preset.

Supported presets in Week 7-8:

- `kip-ft-F`
- `kN-m-C`

### 5.2 Internal calc units

All ACI formulas run in:

- force: kip
- length: inch
- stress: ksi or psi as needed by the formula

Display values may stay in project-friendly units, but internal check math should normalize early.

### 5.3 `UnitContext`

`UnitContext` is built once from `config.project.units` and passed through `ext-calc`.

Responsibilities:

- parse the ETABS preset string
- convert extracted values to calc units
- produce output labels/quantities for JSON and report rendering

Required conversions:

- force to kip
- length to inch
- length to ft
- stress to ksi
- display quantities for force, area, displacement, moment, and stress

### 5.4 Worked `f'c` conversion

For `kip-ft-F`:

```text
Fc = 1152 kip/ft^2
fc_ksi = 1152 / 144 = 8.0 ksi
fc_psi = 8000 psi
```

For `kN-m-C`:

```text
Fc = 55160 kN/m^2
fc_ksi = 55160 * 0.000145038 ~= 8.0 ksi
```

---

## 6. Confirmed Parquet Schemas

All column names below are confirmed from real ETABS sidecar output and should be treated as ground truth.

### 6.1 `joint_drifts.parquet`

| Column | Type | Notes |
|---|---|---|
| `Story` | Utf8 | story label |
| `Label` | Int64 | joint group label |
| `UniqueName` | Int64 | joint unique ID |
| `OutputCase` | Utf8 | load case / combo |
| `CaseType` | Utf8 | `LinStatic`, `LinRespSpec`, `Combination` |
| `StepType` | Utf8 | `Step By Step`, `Max`, `Min`, `Envelope` |
| `StepNumber` | Float64 | null for envelopes |
| `DispX` | Float64 | absolute displacement X in ft |
| `DispY` | Float64 | absolute displacement Y in ft |
| `DriftX` | Float64 | already dimensionless drift ratio |
| `DriftY` | Float64 | already dimensionless drift ratio |

Critical rule:

- `DriftX` and `DriftY` are already `delta / hsx`
- never divide them by story height again

### 6.2 `pier_forces.parquet`

| Column | Type | Notes |
|---|---|---|
| `Story` | Utf8 | story label |
| `Pier` | Utf8 | pier label |
| `OutputCase` | Utf8 | load case / combo |
| `CaseType` | Utf8 | ETABS case type |
| `StepType` | Utf8 | `Max`, `Min`, `Step By Step` |
| `Location` | Utf8 | `Top` or `Bottom` |
| `P` | Float64 | axial force in kip, compression negative |
| `V2` | Float64 | in-plane shear in kip |
| `V3` | Float64 | out-of-plane shear in kip |
| `T` | Float64 | torsion in kip-ft |
| `M2` | Float64 | moment in kip-ft |
| `M3` | Float64 | moment in kip-ft |

Critical rule:

- use governing `max(abs(V2))` across `Top` and `Bottom` at each story/combo for wall shear checks

### 6.3 `pier_section_properties.parquet`

| Column | Type | Notes |
|---|---|---|
| `Story` | Utf8 | story label |
| `Pier` | Utf8 | pier label |
| `AxisAngle` | Float64 | wall orientation angle |
| `NumAreaObj` | Int64 | number of shell elements |
| `NumLineObj` | Int64 | number of frame elements |
| `WidthBot` | Float64 | wall length at bottom in ft |
| `ThickBot` | Float64 | wall thickness at bottom in ft |
| `WidthTop` | Float64 | wall length at top in ft |
| `ThickTop` | Float64 | wall thickness at top in ft |
| `Material` | Utf8 | ETABS material name |
| `CGBotX/Y/Z` | Float64 | centroid coords at bottom |
| `CGTopX/Y/Z` | Float64 | centroid coords at top |

Critical rule:

- `Acv = WidthBot * ThickBot`
- for Week 7-8 assume ETABS has already aggregated multi-element piers into this section row

### 6.4 `material_properties_concrete_data.parquet`

| Column | Type | Notes |
|---|---|---|
| `Material` | Utf8 | material name |
| `Fc` | Float64 | concrete strength in force/area preset units |
| `LtWtConc` | Utf8 | `Yes` or `No` |
| `IsUserFr` | Utf8 | user-defined rupture flag |
| `SSCurveOpt` | Utf8 | stress-strain option |
| `SSHysType` | Utf8 | hysteresis option |
| `SFc` | Float64 | strain at peak stress |
| `SCap` | Float64 | strain capacity |

Critical rule:

- for `kip-ft-F`, `Fc` is `kip/ft^2`
- convert to ksi by dividing by `144`

### 6.5 `material_list_by_story.parquet`

| Column | Type | Notes |
|---|---|---|
| `Story` | Utf8 | story label |
| `ObjectType` | Utf8 | `Column`, `Beam`, `Wall`, `Floor` |
| `Material` | Utf8 | material name |
| `Weight` | Float64 | total weight in kip |
| `FloorArea` | Float64 | floor area in ft^2 |
| `UnitWeight` | Float64 | unit weight |
| `NumPieces` | Float64 | count |
| `NumStuds` | Float64 | studs for beams |

Primary use:

- support building weight and story weight summaries

### 6.6 `story_definitions.parquet`

| Column | Type | Notes |
|---|---|---|
| `Tower` | Utf8 | tower identifier |
| `Story` | Utf8 | story label |
| `Height` | Float64 | inter-story height in ft |
| `IsMaster` | Utf8 | master story flag |
| `SimilarTo` | Utf8 | similar story reference |
| `IsSpliced` | Utf8 | splice flag |
| `Color` | Utf8 | display color |
| `GUID` | Utf8 | unique id |

Critical rule:

- `Height` is inter-story height, not elevation
- elevation must be derived as cumulative sum from base upward

### 6.7 `modal_participating_mass_ratios.parquet`

| Column | Type | Notes |
|---|---|---|
| `Case` | Utf8 | modal case |
| `Mode` | Int64 | mode number |
| `Period` | Float64 | seconds |
| `UX` | Float64 | individual participation X |
| `UY` | Float64 | individual participation Y |
| `UZ` | Float64 | individual participation Z |
| `SumUX` | Float64 | cumulative X, already computed |
| `SumUY` | Float64 | cumulative Y, already computed |
| `RX/RY/RZ` | Float64 | rotational participation |
| `SumRX/RY/RZ` | Float64 | cumulative rotational |

Critical rule:

- `SumUX` and `SumUY` are already cumulative

### 6.8 `story_forces.parquet`

| Column | Type | Notes |
|---|---|---|
| `Story` | Utf8 | story label |
| `OutputCase` | Utf8 | case / combo |
| `CaseType` | Utf8 | ETABS case type |
| `StepType` | Utf8 | `Max`, `Min`, `Step By Step`, `Mode` |
| `StepNumber` | Float64 | step or mode number |
| `Location` | Utf8 | usually `Bottom` |
| `P` | Float64 | axial in kip |
| `VX` | Float64 | story shear X in kip |
| `VY` | Float64 | story shear Y in kip |
| `T` | Float64 | torsion in kip-ft |
| `MX` | Float64 | moment X |
| `MY` | Float64 | moment Y |

Primary use:

- torsional irregularity inputs
- story-level shear distribution support

### 6.9 `base_reactions.parquet`

| Column | Type | Notes |
|---|---|---|
| `OutputCase` | Utf8 | case name |
| `CaseType` | Utf8 | ETABS case type |
| `StepType` | Utf8 | `Max`, `Step By Step`, `Mode` |
| `StepNumber` | Float64 | step number |
| `FX` | Float64 | base shear X in kip |
| `FY` | Float64 | base shear Y in kip |
| `FZ` | Float64 | vertical reaction |
| `MX/MY/MZ` | Float64 | overturning moments |
| `X/Y/Z` | Float64 | reference point |

### 6.10 `group_assignments.parquet`

| Column | Type | Notes |
|---|---|---|
| `GroupName` | Utf8 | ETABS group name |
| `ObjectType` | Utf8 | usually `Point` |
| `UniqueName` | Utf8 | member ID |

Primary use:

- map tracking groups to joints for drift filtering

---

## 7. New Config Structures

### 7.1 `ext-db/src/config/calc.rs`

Create a new shared config module with:

- `CalcConfig`
- `ModalCalcConfig`
- `BaseShearCalcConfig`
- `DriftCalcConfig`
- `PierShearCalcConfig`
- `PierAxialCalcConfig`

Key rule:

- code defaults may exist for ergonomics
- project templates should still write explicit engineering values into `config.toml`

### 7.2 `ext-db/src/config/mod.rs`

Required changes:

- add `pub mod calc;`
- export `CalcConfig`
- add `calc: CalcConfig` to `Config`
- add `calc: CalcConfig` to `SharedConfigFile`
- update `Config::load()` and `write_shared()`

### 7.3 `ext-db/src/config/extract.rs`

`TableSelections` must grow by 4 tables:

- `group_assignments`
- `material_properties_concrete_data`
- `material_list_by_story`
- `story_forces` is already present in current code and remains part of the Week 7-8 table set

Net Week 7-8 extract table set:

- `story_definitions`
- `pier_section_properties`
- `base_reactions`
- `story_forces`
- `joint_drifts`
- `pier_forces`
- `modal_participating_mass_ratios`
- `group_assignments`
- `material_properties_concrete_data`
- `material_list_by_story`

`merge()` and `is_empty()` must be updated accordingly.

---

## 8. `ext-calc` Architecture

### 8.1 Crate tree

```text
ext-calc/src/
  lib.rs
  output.rs
  unit_convert.rs
  code_params.rs
  tables/
    mod.rs
    story_def.rs
    joint_drift.rs
    pier_section.rs
    pier_forces.rs
    modal.rs
    base_reactions.rs
    story_forces.rs
    material_props.rs
    material_by_story.rs
    group_assignments.rs
  checks/
    mod.rs
    modal.rs
    base_reaction.rs
    drift_wind.rs
    drift_seismic.rs
    torsional.rs
    pier_shear_wind.rs
    pier_shear_seismic.rs
    pier_axial.rs
```

### 8.2 Loader rules

Loader modules should:

- map real column names exactly
- filter only at the loader boundary when the filter is table-specific
- compute cheap deterministic derived values during load
- avoid business-rule decisions that belong in check modules

Examples:

- `story_def.rs`: compute `elevation_ft`
- `pier_section.rs`: compute `acv_in2` and `ag_in2`
- `material_props.rs`: compute `fc_ksi` and `fc_psi`
- `group_assignments.rs`: aggregate `HashMap<String, Vec<String>>`

### 8.3 Shared lookup maps

Build these once in `CalcRunner`:

- `(pier, story) -> fc_ksi`
- `(group_name) -> joint ids`
- optional per-story elevation / height maps

---

## 9. Check Definitions

### 9.1 Check 1 - Modal mass participation

Standard:

- ASCE 7 modal participation threshold

Rules:

- filter to configured modal case
- use `SumUX` and `SumUY` directly
- determine first mode reaching threshold
- determine dominant periods from highest individual `UX` and `UY`

### 9.2 Check 2 - Base shear verification

Rules:

- read RSA and ELF cases from config
- compare ETABS base shear to configured minimum ratio
- direction X and Y are independent result objects

### 9.3 Check 3 - Wind drift and displacement

Rules:

- use `DriftX` and `DriftY` directly as dimensionless ratios
- use `DispX` and `DispY` for roof absolute displacement
- support multi-step wind cases by taking maxima across steps

### 9.4 Check 4 - Seismic drift

Rules:

- use `DriftX` and `DriftY` directly
- compare against configured drift limit
- response spectrum cases should typically use `StepType = Max`

### 9.5 Check 5 - Torsional irregularity

Week 7-8 status:

- explicit stub only
- produce note-level output
- never pretend formula is final

### 9.6 Check 6 - Pier shear stress, wind

Rules:

- use wind strength combos from config
- `Vu = max(abs(V2))`
- `Acv = WidthBot * ThickBot * 144`
- use material join to resolve `fc_ksi`
- use configured `phi_v`, `alpha_c`, `fy_ksi`, `rho_t`

### 9.7 Check 7 - Pier shear stress, seismic

Same data path as Check 6, but with:

- seismic combo filter
- different `phi_v`

### 9.8 Check 8 - Pier axial stress

Rules:

- use `P` from `pier_forces`
- compression is negative in ETABS, so use `abs(P)` for demand
- `Ag = WidthBot * ThickBot * 144`
- compute `fa`, `fa_ratio`, `phi_po`, `dcr`

---

## 10. Output Contract

`ext-calc` must emit a stable JSON contract before `ext-render` or `ext-report` are considered complete.

Top-level object:

```rust
pub struct CalcOutput {
    pub meta: CalcMeta,
    pub summary: CalcSummary,
    pub modal: Option<ModalOutput>,
    pub base_shear: Option<BaseShearOutput>,
    pub drift_wind: Option<DriftOutput>,
    pub drift_seismic: Option<DriftOutput>,
    pub torsional: Option<TorsionalOutput>,
    pub pier_shear_wind: Option<PierShearOutput>,
    pub pier_shear_seismic: Option<PierShearOutput>,
    pub pier_axial: Option<PierAxialOutput>,
}
```

Design rule:

- `CalcSummary` is the fast CLI/dashboard contract
- detailed outputs serve render/report generation

### 10.1 Quantity display rule

Use a `Quantity { value, unit }` pattern for user-facing numeric outputs where units matter.

Internal formulas should not depend on display units.

### 10.2 Torsional output

During Week 7-8:

```rust
pub struct TorsionalOutput {
    pub note: String,
}
```

---

## 11. `CalcRunner`

`CalcRunner::run_all()` is the top-level orchestration entry point.

Responsibilities:

- build `UnitContext`
- load all required tables
- build shared lookup maps
- run enabled checks
- assemble summary and final output
- write stable JSON output for downstream use

Design rule:

- `CalcRunner` orchestrates
- loaders load
- checks compute
- output builders format

---

## 12. `ext-render` and `ext-report`

### 12.1 `ext-render`

`ext-render` consumes `CalcOutput` and produces deterministic figure assets.

Initial output scope:

- modal mass chart
- base shear comparison chart
- drift envelopes
- pier DCR bar charts

Design rule:

- rendering must not rerun engineering formulas
- render from `CalcOutput`, not directly from Parquet

### 12.2 `ext-report`

`ext-report` consumes:

- project metadata
- `CalcOutput`
- rendered assets

and produces a tabloid landscape report artifact.

Preferred direction in this repo:

- Typst-based report generation

Design rule:

- report layout is downstream presentation
- it should not contain engineering logic that bypasses `ext-calc`

---

## 13. CLI / API Surface

Planned commands:

- `ext calc`
- `ext render`
- `ext report`

Recommended API sequencing:

1. `ext-api` gets `run_calc()`
2. calc JSON becomes stable
3. `ext-api` gets render/report orchestration
4. CLI commands land after API contracts stabilize

This avoids having the CLI shape churn while the core data contract is still moving.

---

## 14. Smaller Delivery Slices

The original proposals were correct on architecture but still a bit too large for safe implementation in one sweep. The merged plan below breaks Week 7-8 into small, testable slices.

### Slice A - Shared config + unit foundations

Files:

- `ext-db/src/config/calc.rs`
- `ext-db/src/config/mod.rs`
- `ext-db/src/config/extract.rs`
- `ext-calc/src/unit_convert.rs`

Acceptance:

- shared `[calc]` loads from `config.toml`
- local `[project] units` still loads from `config.local.toml`
- new extract tables serialize correctly
- unit conversion tests pass for both presets

Tests:

- TOML load/merge tests
- `UnitContext` parse/convert tests

### Slice B - `ext-calc` scaffolding + output contract

Files:

- `ext-calc/Cargo.toml`
- `ext-calc/src/lib.rs`
- `ext-calc/src/output.rs`
- `ext-calc/src/code_params.rs`
- `ext-calc/src/checks/mod.rs`
- `ext-calc/src/tables/mod.rs`

Acceptance:

- crate compiles
- `CalcOutput` serializes
- `CodeParams::from_config()` normalizes shared config into runtime params

Tests:

- output serialization tests
- params/defaulting tests

### Slice C - Loader batch 1

Files:

- `story_def.rs`
- `material_props.rs`
- `pier_section.rs`
- `group_assignments.rs`

Acceptance:

- all 4 loaders parse fixture parquet correctly
- derived fields are computed correctly

Tests:

- fixture-based loader tests
- exact numeric checks for `elevation_ft`, `acv_in2`, `fc_ksi`

### Slice D - Loader batch 2

Files:

- `joint_drift.rs`
- `pier_forces.rs`
- `modal.rs`
- `base_reactions.rs`
- `story_forces.rs`
- `material_by_story.rs`

Acceptance:

- case/group filtering works
- optional step numbers are handled safely
- no placeholder column names remain

Tests:

- fixture-based loader tests
- filter behavior tests

### Slice E - Checks 1 and 2

Files:

- `checks/modal.rs`
- `checks/base_reaction.rs`

Acceptance:

- modal check identifies threshold mode and dominant periods
- base shear check resolves configured RSA/ELF pairs and ratios

Tests:

- fixed numeric fixtures from real data
- threshold edge cases

### Slice F - Checks 3 and 4

Files:

- `checks/drift_wind.rs`
- `checks/drift_seismic.rs`

Acceptance:

- wind drift uses `DriftX/DriftY` directly
- roof displacement uses `DispX/DispY`
- seismic drift compares direct ratios to allowable

Tests:

- max-over-step tests
- roof displacement conversion tests
- pass/fail threshold tests

### Slice G - Checks 6, 7, 8 plus torsional stub

Files:

- `checks/pier_shear_wind.rs`
- `checks/pier_shear_seismic.rs`
- `checks/pier_axial.rs`
- `checks/torsional.rs`

Acceptance:

- wind/seismic pier shear differ only where expected
- axial check computes `fa`, `phi_po`, and `dcr`
- torsional output emits explicit stub note

Tests:

- hand-check numeric fixtures
- material fallback test
- top/bottom governing shear test

### Slice H - `CalcRunner` + API integration

Files:

- `ext-calc/src/lib.rs`
- `ext-api/src/report.rs` or new calc orchestration module

Acceptance:

- one call produces `calc_output.json`
- summary is stable and CLI-friendly

Tests:

- end-to-end calc integration test from fixture results dir
- golden JSON snapshot for `calc_output.json`

### Slice I - `ext-render`

Acceptance:

- deterministic charts from fixed `CalcOutput`
- assets are reproducible enough for snapshot/file-size sanity checks

Tests:

- render smoke tests
- artifact existence + metadata assertions

### Slice J - `ext-report` + CLI wiring

Acceptance:

- `ext report` produces a report from existing calc/render outputs
- report composes without recomputing engineering logic

Tests:

- report smoke test
- CLI integration tests for `ext calc`, `ext render`, `ext report`

---

## 15. Test Strategy

### 15.1 Preferred pyramid

1. config and unit tests
2. loader tests against fixed Parquet fixtures
3. formula tests against hand-check numbers
4. JSON snapshot tests for `CalcOutput`
5. render/report smoke tests
6. one full pipeline integration test

### 15.2 Fixture strategy

Use a stable checked-in results fixture directory rather than live ETABS output for most Week 7-8 tests.

Reason:

- deterministic
- fast
- avoids ETABS startup cost
- isolates loader/check correctness from sidecar variability

### 15.3 Numeric test philosophy

For engineering checks:

- always include at least one hand-check fixture with the exact expected value
- avoid "non-empty output" tests when a formula result can be asserted numerically

---

## 16. Build Order

Recommended sequence:

1. Slice A
2. Slice B
3. Slice C
4. Slice D
5. Slice E
6. Slice F
7. Slice G
8. Slice H
9. Slice I
10. Slice J

Important sequencing rule:

- do not start `ext-render` or `ext-report` until `CalcOutput` and `run_calc()` are stable

---

## 17. Open Items

Only three open items remain at the design level:

| # | Item | Status |
|---|---|---|
| 1 | exact wind/seismic combo names in the live project | engineer confirms against real `OutputCase` values |
| 2 | torsional irregularity formula | deferred pending engineer method |
| 3 | multi-element pier `Acv` interpretation | current assumption: use aggregated `WidthBot * ThickBot` from ETABS section props |

Everything else in the two proposals is now merged and implementation-ready.

---

## 18. Initial Shared Config Template

```toml
[project]
name = "Tower Project A"

[extract.tables]
story-definitions                 = { load-cases = [], load-combos = [] }
pier-section-properties           = {}
base-reactions                    = { load-cases = ["*"], load-combos = ["*"] }
story-forces                      = { load-cases = ["*"], load-combos = ["*"] }
joint-drifts                      = { load-cases = ["*"], load-combos = ["*"] }
pier-forces                       = { load-cases = [], load-combos = ["*"] }
modal-participating-mass-ratios   = {}
group-assignments                 = {}
material-properties-concrete-data = {}
material-list-by-story            = {}

[calc]
code = "ACI318-14"
occupancy-category = "II"
modal-case = "Modal-Rizt"
drift-tracking-groups = ["Tracking_Points"]

[calc.modal]
min-mass-participation = 0.90

[calc.base-shear]
elf-case-x = "ELF_X"
elf-case-y = "ELF_Y"
rsa-case-x = "RSA_X"
rsa-case-y = "RSA_Y"
rsa-scale-min = 1.0

[calc.drift-wind]
load-cases = ["Wind_ASCE_10yr", "Wind_10yr_Diagonal"]
drift-limit = 0.0025
disp-limit-h = 500

[calc.drift-seismic]
load-cases = ["RSA_X_Drift", "RSA_Y_Drift", "ELF_X_Drift", "ELF_Y_Drift"]
drift-limit = 0.020

[calc.pier-shear-wind]
load-combos = ["EVN_LRFD_Wind"]
phi-v = 0.75
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

[calc.pier-shear-seismic]
load-combos = ["EVN_LRFD_EQ"]
phi-v = 0.60
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

[calc.pier-axial]
load-combos = ["EVN_LRFD_GRA", "EVN_LRFD_EQ", "EVN_LRFD_Wind"]
phi-axial = 0.65
```

Local companion:

```toml
[project]
sidecar-path = "D:\\repo\\EtabExtension.CLI\\dist\\etab-cli-x86_64-pc-windows-msvc.exe"
units = "kip-ft-F"
```

---

## 19. Acceptance for the Merged Spec

This merged spec is considered ready when:

- schema references are explicit and no longer contradictory
- config ownership is unambiguous
- unit conversion rules are locked
- the implementation path is split into small slices with test gates
- the repo can implement `ext-calc` first without blocking on render/report polish

