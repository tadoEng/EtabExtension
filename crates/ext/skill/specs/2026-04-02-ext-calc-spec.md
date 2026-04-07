# `ext-calc` Specification
## Week 7-8 Calculation Contract, Testing Rules, and Change Workflow

**Status:** Active working spec  
**Scope:** `crates/ext-calc` only  
**Audience:** engineers and agents implementing or reviewing calculation logic  
**Current baseline date:** April 3, 2026

---

## 1. Purpose

`ext-calc` is the calculation engine for extracted ETABS result tables.

Its job is to:

- load extracted Parquet tables from a version/results folder
- validate that the project calculation config is complete
- run engineering checks in a deterministic order
- return a stable `CalcOutput` contract for:
  - CLI summaries
  - future `ext-render`
  - future `ext-report`
  - later `ext-api` orchestration
- provide a minimal review runner that writes `calc_output.json` for manual review

`ext-calc` does **not**:

- talk to ETABS directly
- decide load cases or limits by itself
- infer engineering intent from the extracted data
- own report layout or UI behavior

---

## 2. Source of Truth

### 2.1 Calculation-driving inputs

All calculation-driving inputs must come from:

- `.etabs-ext/config.toml`

This includes:

- modal case names
- base shear ELF/RSA case names
- drift wind load cases
- drift seismic load cases
- displacement wind load cases
- drift tracking groups
- code thresholds and limits
- later: pier combo names and pier design factors

`ext-calc` must **fail fast** when a required calculation input is missing.

### 2.2 Local-only inputs

Machine-specific inputs come from:

- `.etabs-ext/config.local.toml`

For `ext-calc`, the important local input is:

- `[project].units`

This is used to build the `UnitContext`.

### 2.3 No hidden defaults for calc behavior

For Week 7-8, there must be no runtime fallback defaults for calc-driving fields.

Allowed:

- metadata defaults such as display-only `code` or `occupancy-category` if needed

Not allowed:

- assuming `"Modal-Rizt"` when `modal-case` is missing
- assuming `"ELF_X"` when `elf-case-x` is missing
- assuming `0.90` when `min-mass-participation` is missing
- assuming `500` when `disp-limit-h` is missing
- assuming any `[calc.displacement-wind]` values when they are missing

If a field is required for a check, its absence must produce a clear config error.

Example error shape:

```text
missing required config: [calc.base-shear].elf-case-x
```

---

## 3. Current Crate Shape

Current public modules:

```text
ext-calc/src/
  lib.rs
  output.rs
  unit_convert.rs
  code_params.rs
  checks/
  tables/
```

### 3.1 Main entrypoint

Current top-level entrypoint:

```rust
CalcRunner::run_all(
    version_dir,
    results_dir,
    params,
    version_id,
    branch,
) -> Result<CalcOutput>
```

For manual review, `src/main.rs` now provides a minimal binary runner:

```powershell
cargo run -p ext-calc -- <path>
```

Behavior:

- loads `.etabs-ext/config.toml` and `config.local.toml`
- resolves a results directory from the input path
- runs `CalcRunner`
- writes `calc_output.json` into the results directory by default
- prints a short summary to stdout

### 3.2 Current implemented checks

Implemented now:

- Check 1: modal participation
- Check 2: base shear
- Check 3: wind drift + roof displacement
- Check 4: seismic drift
- Check 5: wind joint displacement

Not implemented yet:

- Check 6: torsional irregularity
- Check 7: pier shear wind
- Check 8: pier shear seismic
- Check 9: pier axial

### 3.3 Current output contract

`output.rs` is the canonical DTO boundary.

Important current types:

- `CalcOutput`
- `CalcSummary`
- `ModalOutput`
- `BaseShearOutput`
- `DriftOutput`
- `DisplacementOutput`
- `ModalModeRow`
- `BaseReactionCheckRow`
- `DriftEnvelopeRow`
- `DisplacementEnvelopeRow`

Other crates should consume `ext-calc::output::*` instead of inventing duplicate DTOs.

---

## 4. Current Check Rules

### 4.1 Check 1: Modal Participation

Input:

- `modal_participating_mass_ratios.parquet`
- `config.toml [calc]`
- `config.toml [calc.modal]`

Rules:

- filter to the configured `modal-case`
- sort by mode ascending
- keep the first 20 rows in the detailed output table
- detailed row columns are exactly:
  - `case`
  - `mode`
  - `period`
  - `ux`
  - `uy`
  - `sum_ux`
  - `sum_uy`
  - `rz`
  - `sum_rz`
- pass when:
  - `sum_ux >= min-mass-participation`
  - and `sum_uy >= min-mass-participation`

### 4.2 Check 2: Base Shear

Input:

- `base_reactions.parquet`
- `config.toml [calc.base-shear]`

Rules:

- preserve review rows except raw coordinate columns `X`, `Y`, `Z`
- exclude rows that are not useful for review:
  - modal rows such as `Modal-Rizt` and `Modal-Eigen`
  - ETABS-generated helper cases beginning with `~`
  - `LinMod*` modal-style rows
- compute direction X summary from configured:
  - `rsa-case-x`
  - `elf-case-x`
- compute direction Y summary from configured:
  - `rsa-case-y`
  - `elf-case-y`
- ratio is:
  - `RSA / ELF`
- pass when ratio is `>= rsa-scale-min`

### 4.3 Check 3: Wind Drift

Input:

- `joint_drifts.parquet`
- `group_assignments.parquet`
- `story_definitions.parquet`
- `config.toml [calc]`
- `config.toml [calc.drift-wind]`

Rules:

- drift groups are configured by name
- each group represents a vertical stack of points
- group membership comes from `group_assignments`
- for each `Story + GroupName + OutputCase`, compute:
  - positive/negative displacement envelope in X
  - positive/negative displacement envelope in Y
  - positive/negative drift envelope in X
  - positive/negative drift envelope in Y
- governing result comes from the maximum absolute drift value
- roof displacement uses roof-story envelope rows
- displacement limit is total building height divided by configured `disp-limit-h`

### 4.4 Check 4: Seismic Drift

Input:

- same table family as wind drift
- `config.toml [calc.drift-seismic]`

Rules:

- same grouped-envelope logic as wind drift
- no roof displacement output
- pass/fail depends only on drift ratio vs configured seismic limit

### 4.5 Check 5: Wind Joint Displacement

Input:

- `joint_drifts.parquet`
- `group_assignments.parquet`
- `story_definitions.parquet`
- `config.toml [calc]`
- `config.toml [calc.displacement-wind]`

Rules:

- grouped by `Story + GroupName + OutputCase`
- compute positive and negative displacement envelope in both X and Y
- governing result comes from the maximum absolute displacement value
- displacement limit is total building height divided by configured `disp-limit-h`
- output is separate from drift so review and render can inspect displacement directly

---

## 5. Testing Rules

### 5.1 Fixture layout

Current fixture root:

- `crates/ext-calc/tests/fixtures/results_realistic/`

This fixture set now includes:

- Parquet tables
- `.etabs-ext/config.toml`
- `.etabs-ext/config.local.toml`

The fixture config is intentional and must be treated as part of the test contract.

### 5.2 Happy-path tests

Happy-path `ext-calc` tests should prefer:

- `Config::load(fixture_dir)`
- `CodeParams::from_config(&config)`

This is required so tests prove:

- config parsing works
- strict TOML rules are enforced
- calc logic follows config rather than hardcoded values

Avoid rebuilding the normal config path manually in tests unless the test is specifically about invalid config combinations.

### 5.3 Negative tests

Each calc-driving requirement should have a failure test where practical.

Current required negative coverage includes:

- missing modal case config
- missing base shear case config
- missing drift group config
- missing drift load-case config
- invalid wind displacement divisor
- missing displacement-wind config
- configured case/group not found in extracted data

### 5.4 Test command baseline

Current focused verification command:

```powershell
cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target
```

This is the required first verification step for any `ext-calc` change.

Manual review command:

```powershell
cargo run -p ext-calc -- crates/ext-calc/tests/fixtures/results_realistic
```

---

## 6. How To Add a New Calculation Check

Use this workflow for checks 5-8 or any future new check.

### Step 1: Define the config contract first

Before writing calculation code:

- identify all required TOML fields
- add them to `ext-db::config::calc`
- decide which fields are required
- do not add runtime fallback defaults for calc-driving behavior

If a new check needs:

- load combos
- case names
- design limits
- reduction factors

they must be explicitly represented in config.

### Step 2: Add or confirm table loaders

Before implementing a check:

- confirm the needed Parquet table already has a loader
- if not, add a typed loader in `tables/`
- add loader tests against the fixture set
- do derived field computation inside the loader when it is data-shaping, not design logic

Examples:

- converting `Fc` to `fc_ksi`
- computing `Acv`
- computing story elevation

### Step 3: Add output types before the algorithm

Add the result DTOs to `output.rs` first.

Rule:

- output shape should be useful for both report tables and future render
- the renderer should not need to reopen Parquet files to reconstruct a chart-friendly shape

For a new check, define:

- detailed rows
- governing result
- pass/fail
- any quantity/unit fields needed downstream

### Step 4: Implement the check in `checks/`

Each check should live in its own module and be pure relative to loaded rows + params.

Preferred shape:

```rust
pub fn run(rows..., params: &CodeParams) -> Result<CheckOutput>
```

Rules:

- validate configured names against the loaded rows
- error clearly when the configured case/combo/group does not exist
- do not silently skip missing configured inputs
- do not read config files directly inside check code
- use `CodeParams`, not raw TOML access

### Step 5: Wire into `CalcRunner`

After the check works in isolation:

- load needed tables in `CalcRunner`
- call the new check when its `CheckSelection` flag is enabled
- attach the result into `CalcOutput`
- add a concise summary line

### Step 6: Add tests

Required test layers:

- loader test if a loader changed
- check module unit test
- negative config/data mismatch test
- `CalcRunner` integration coverage if the check is wired in

---

## 7. How To Edit an Existing Calculation

When editing an existing check, keep this order:

### 7.1 Start from the config contract

If the formula or case selection changes:

- update the spec first
- then update fixture config if needed
- only then update implementation

Do not bury engineering behavior changes in code-only edits.

### 7.2 Preserve output compatibility unless a deliberate contract change is needed

Because `ext-render` and `ext-report` will consume `CalcOutput`:

- prefer additive output changes
- if a field must be renamed or removed, update the spec and downstream consumers in the same change set

### 7.3 Keep units explicit

If a calculation depends on converted units:

- use `UnitContext`
- keep internal formula units explicit in code comments when helpful
- keep output units aligned with `Quantity`

### 7.4 Update the fixture when behavior changes intentionally

If the expected calculation path changes:

- update fixture `.etabs-ext/config.toml`
- only update fixture data when the extracted source itself has intentionally changed

---

## 8. Review Checklist

When reviewing `ext-calc` work, check:

- Are all calc-driving inputs coming from TOML?
- Does missing TOML produce a clear fail-fast error?
- Does the code avoid inferring case/combo names from the extracted rows?
- Are loader responsibilities separated from engineering check responsibilities?
- Is the output shape useful for downstream render/report?
- Do tests load the fixture config for the normal path?
- Are negative config/data mismatch tests present?

---

## 9. Current Known Limits

- drift seismic currently uses the same fixture drift case as wind because the realistic fixture does not yet contain dedicated seismic drift rows
- displacement wind uses the same grouped joint-stack source table as drift
- original checks 6-9 are still pending
- `ext-render` and `ext-report` are not yet the final consumers, so output review should still be conservative and additive

---

## 10. Immediate Next Work

Recommended next `ext-calc` sequence:

1. add snapshot coverage for `CalcOutput`
2. implement torsional stub output cleanly in the same strict TOML style if needed
3. implement pier shear wind
4. implement pier shear seismic
5. implement pier axial
6. then expand `ext-render` and `ext-report`
