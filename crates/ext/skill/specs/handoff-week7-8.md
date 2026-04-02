# Week 7-8 Handoff

## Summary

Week 7-8 is now past the foundation stage.

What is genuinely done:

- `ext-db` has the shared `[calc]` config model and strict local-only write behavior for `project.sidecar-path` and `project.units`
- `ext-calc` is a real library with:
  - output DTOs
  - unit conversion
  - strict config validation
  - batch-1 and batch-2 loaders
  - checks 1-5 implemented
  - a working `CalcRunner::run_all(...)`
  - a minimal review runner via `cargo run -p ext-calc -- <path>`
- `ext-calc` tests now use a real fixture `.etabs-ext/config.toml` and `config.local.toml`
- `ext-render` and `ext-report` have first library surfaces
- focused verification is green

What is not done yet:

- checks 5-8
- stable `CalcOutput` snapshots
- render/report production pipeline
- `ext-api` and CLI integration

Detailed `ext-calc` behavior now lives in:

- [2026-04-02-ext-calc-spec.md](/d:/Work/EtabExtension/crates/ext/skill/specs/2026-04-02-ext-calc-spec.md)

That file should be treated as the primary implementation/review spec for calc work from this point forward.

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

### 2. `ext-calc` core

Implemented:

- output contract in `output.rs`
- `UnitContext` and unit conversion
- `CodeParams::from_config(&Config)` with strict validation for calc-driving fields
- all current Week 7-8 loaders
- checks 1-5:
  - modal
  - base shear
  - drift wind
  - drift seismic
  - displacement wind
- `CalcRunner::run_all(...)` wiring for checks 1-5

Current `CalcRunner` behavior:

- loads all planned Week 7-8 input tables
- runs checks 1-5
- leaves checks 6-9 as `None`
- emits real summary lines for implemented checks
- writes `calc_output.json` through the minimal review runner in `src/main.rs`

### 3. Fixture config contract

The realistic fixture set now includes:

- `crates/ext-calc/tests/fixtures/results_realistic/.etabs-ext/config.toml`
- `crates/ext-calc/tests/fixtures/results_realistic/.etabs-ext/config.local.toml`

This is important:

- happy-path `ext-calc` tests now load config from the fixture directory with `Config::load(...)`
- calc behavior is therefore tested through the real config-loading path instead of hand-built in-memory config for the green path
- the fixture config now includes `[calc.displacement-wind]`

Current fixture limitation:

- the realistic `joint_drifts` fixture only contains `Wind_10yr_Diagonal`
- so the seismic drift happy-path test currently uses that same extracted case until a true seismic drift fixture is added

### 4. `ext-render` and `ext-report`

Implemented:

- `crates/ext-render/src/lib.rs`
- `crates/ext-report/src/lib.rs`

Current status:

- `ext-render` can consume `CalcOutput` and emit simple drift SVGs
- `ext-report` can build Typst document content, but PDF compilation is still intentionally stubbed

### 5. Workspace dependency fix

Updated:

- `Cargo.toml`

Reason:

- `polars 0.53.0` would not resolve with workspace `chrono 0.4.44`
- workspace `chrono` was pinned to `0.4.41`

---

## Remaining Work

### 1. Remaining engineering checks

Still to implement:

- Check 6: torsional irregularity
- Check 7: pier shear wind
- Check 8: pier shear seismic
- Check 9: pier axial

### 2. Snapshot and contract hardening

Still recommended:

- add stable JSON snapshot coverage for `CalcOutput`
- keep additive DTO changes where possible so `ext-render` and `ext-report` stay stable

### 3. Render/report continuation

`ext-render`

- keep drift-first
- refine the SVG output toward the proof-of-concept layout
- consume the existing grouped drift envelope output shape directly

`ext-report`

- replace the current Typst string skeleton with the proof-of-concept report layout
- wire rendered SVG assets into report sections
- add real PDF compilation once the report contract stabilizes

### 4. Integration not started yet

Still deferred:

- `ext-api` orchestration for calc/render/report
- CLI commands:
  - `ext calc`
  - `ext render`
  - `ext report`

---

## Verification Status

Passed:

- `cargo check --target-dir .codex-target`
- `cargo fmt --all`
- `cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target`
- `cargo run -p ext-calc -- crates/ext-calc/tests/fixtures/results_realistic`

Suggested on constrained machines:

- `cargo test -j 2 -p ext-db -p ext-calc --lib --target-dir .codex-target`

---

## Important Notes

### Primary calc spec

Use this file for detailed calc rules, testing rules, and change workflow:

- [2026-04-02-ext-calc-spec.md](/d:/Work/EtabExtension/crates/ext/skill/specs/2026-04-02-ext-calc-spec.md)

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

They are not the primary implementation target anymore.

### Review-runner note

The quickest way to inspect current calc output is:

```powershell
cargo run -p ext-calc -- crates/ext-calc/tests/fixtures/results_realistic
```

This writes:

- `crates/ext-calc/tests/fixtures/results_realistic/calc_output.json`

Current review-specific behavior:

- base shear review rows exclude modal cases and ETABS helper cases beginning with `~`
- displacement wind is now a separate output section driven by `[calc.displacement-wind]`

---

## Recommended Next Continuation

The next agent should start here:

1. keep using the focused verification command:
   - `cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target`
2. add `CalcOutput` snapshot coverage
3. implement checks 6-9 inside the same strict TOML-driven contract
4. expand `ext-render`
5. expand `ext-report`
6. only then wire `ext-api` and CLI integration
