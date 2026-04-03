# ext-calc Remaining Checks Plan (`ACI 318-14`)

## Summary

Implement the three remaining wall checks in `ext-calc` using strict TOML-driven inputs and `ACI 318-14` formulas:

- `pier_shear_wind`
- `pier_shear_seismic`
- `pier_axial`

The architecture is already ready for this work. The main job is to add strict pier config/runtime params, implement the engineering logic, tighten the output contract for signed axial behavior, and add deterministic fixture tests.

Locked engineering decisions from your earlier guidance:
- shear demand uses the absolute maximum `|V2|` across both `Max` and `Min`
- governing location is the max across `Top` and `Bottom`
- axial must report whether the wall is in compression or tension
- for axial tension, use **report-only** behavior, not automatic fail

## Key Changes

### 1. Add strict pier config/runtime params
Extend the current strict TOML-driven path so the remaining checks never infer combo names or design factors.

Add runtime params in `ext-calc::code_params` for:
- `PierShearParams`
  - `load_combos`
  - `phi_v`
  - `alpha_c`
  - `fy_ksi`
  - `rho_t`
  - `fc_default_ksi`
- `PierAxialParams`
  - `load_combos`
  - `phi_axial`

Validation rules:
- each pier combo list must be non-empty
- `phi_v`, `phi_axial`, `alpha_c`, `fy_ksi`, `fc_default_ksi` must be `> 0`
- `rho_t` must be `> 0`
- missing combo names or required numbers fail fast with the same current error style

Recommended config contract:
- `[calc.pier-shear-wind]`
- `[calc.pier-shear-seismic]`
- `[calc.pier-axial]`

### 2. Implement `pier_shear_wind` and `pier_shear_seismic`
Add `checks/pier_shear_wind.rs` and `checks/pier_shear_seismic.rs`.

Data flow per check:
- filter `pier_forces` to configured `load_combos`
- group rows by `(story, pier, output_case)`
- within each group, select the governing force row by:
  - maximum absolute `V2`
  - across both `Max` and `Min`
  - across both `Top` and `Bottom`
- join the governing force row to the matching `PierSectionRow` by `(story, pier)`
- resolve `f'c` from `material_props` using section material
- if material join misses, use `fc_default_ksi`

Wind shear formula:
- `Vu = max |V2|`
- `Acv = width_bot_ft * thick_bot_ft * 144`
- `f'c_psi = fc_ksi * 1000`
- `Vn = Acv * (alpha_c * sqrt(f'c_psi) + rho_t * fy_ksi * 1000) / 1000`
- `phiVn = phi_v * Vn`
- `DCR = Vu / phiVn`

Seismic shear uses the same demand and nominal strength path, but with seismic `phi_v`.

Result behavior:
- one detailed result row per `(story, pier, combo)`
- governing row is the max `DCR`
- overall pass is `true` only if all rows pass

Recommended output adjustments:
- keep `PierShearOutput`
- add `step_type` to `PierShearResult` so review can tell whether the governing row came from `Max` or `Min`
- keep `location` because governing `Top` vs `Bottom` matters for review

### 3. Implement `pier_axial` with signed compression/tension behavior
Add `checks/pier_axial.rs`.

Selection logic:
- filter `pier_forces` to configured axial combos
- group rows by `(story, pier, output_case)`
- choose the governing axial row by maximum absolute `P`
  - across both `Max` and `Min`
  - across both `Top` and `Bottom`
- preserve the sign of `P`

State classification:
- `P < 0` => `compression`
- `P > 0` => `tension`
- `P == 0` => treat as neutral and report as non-compression

Computation:
- `Ag = width_bot_ft * thick_bot_ft * 144`
- `fa_signed = P / Ag` in ksi
- resolve `f'c` from material join or axial default path if needed through the same section/material map

Compression-only strength check:
- `Pu = abs(P)`
- `Po = 0.85 * fc_ksi * Ag`
- `phiPo = phi_axial * Po`
- `fa_ratio = abs(fa_signed) / (0.85 * fc_ksi)`
- `DCR = Pu / phiPo`

Tension behavior:
- report signed force/stress and axial state
- do **not** compute compression strength DCR for tension rows
- tension rows are `report-only`

Recommended output change:
- revise `PierAxialResult` to support signed/report-only behavior:
  - add `axial_state`
  - add signed axial force field
  - add signed stress field
  - make `phi_po`, `fa_ratio`, `dcr`, and `pass` optional for tension rows
  - add `step_type` and `location` for review traceability
- revise `PierAxialOutput` to include:
  - `rows`
  - `governing_compression: Option<PierAxialResult>`
  - `governing_tension: Option<PierAxialResult>`
  - `pass`
- overall axial `pass` should be based only on compression rows
- if there are no compression rows, overall axial check remains pass with a summary note that only tension/report-only rows were found

### 4. Wire into `CalcRunner` and summary
Update `CalcRunner::run_all()` to stop ignoring loaded pier inputs and run the new checks when enabled.

Summary lines:
- `pierShearWind`
  - `max DCR = ... {pier}/{story}/{combo}/{step}/{location}`
- `pierShearSeismic`
  - same summary shape
- `pierAxial`
  - if compression rows exist:
    - `max compression DCR = ... {pier}/{story}/{combo}/{step}/{location}`
  - if only tension governs:
    - `governing tension stress = ... {pier}/{story}/{combo}/{step}/{location} (report-only)`

Keep `torsional` as a stub for now, but improve the placeholder wording only after the three wall checks are in.

## Public API / Interface Changes

- Add `pier_shear_wind`, `pier_shear_seismic`, and `pier_axial` params to `CodeParams`
- Add `step_type` to `PierShearResult`
- Revise `PierAxialResult` for signed/report-only tension handling
- Revise `PierAxialOutput` to expose separate governing compression vs tension results
- Keep top-level `CalcOutput` shape unchanged:
  - `pier_shear_wind`
  - `pier_shear_seismic`
  - `pier_axial`
  remain the attachment points

## Test Plan

### Loader/runtime validation
- config validation fails for missing pier combo lists
- config validation fails for missing required pier numeric parameters
- combo names must come only from TOML

### Shear checks
- one happy-path test for wind shear with exact fixture-derived governing row and `DCR`
- one happy-path test for seismic shear with exact fixture-derived governing row and `DCR`
- one negative test for missing configured combo
- one negative test for section/material join miss falling back to configured `fc_default_ksi`

### Axial check
- one compression happy-path test with exact governing `DCR`
- one tension happy-path test showing `axial_state = tension` and report-only fields
- one test proving governing row selection uses absolute max `P` across `Max/Min` and `Top/Bottom`
- one negative test for missing configured combo

### Integration
- update `CalcRunner` integration test so all implemented checks are `Some(...)`
- review runner:
  - `cargo run -p ext-calc -- crates/ext-calc/tests/fixtures/results_realistic`
- required green baseline:
  - `cargo test -p ext-db -p ext-calc --lib --target-dir .codex-target`

### Fixture expectation
Before locking exact-value assertions, ensure the realistic fixture contains all required pier combo families:
- wind combo(s)
- seismic combo(s)
- gravity combo(s)

If the current realistic fixture does not contain all three, refresh that fixture rather than weakening the tests.

## Assumptions

- Use `ACI 318-14` as the sole basis for these three checks.
- Shear demand is always based on `V2`, not `V3`.
- Governing wall shear location is the max of `Top` and `Bottom`.
- Governing axial row is selected by maximum absolute `P`, but axial reporting must preserve sign.
- Tension is report-only for the axial check and does not create a compression-strength failure by itself.
- `torsional` remains deferred until the wall checks are complete.
