# ext-calc Calculation Breakdown (Deep Dive)

## 1) Scope and intent
This document breaks down how `crates/ext-calc` computes each check in the current implementation, including:

- input tables and preprocessing
- filtering and enveloping logic
- equations used
- governing value selection
- pass/fail criteria
- notable implementation behavior for review

Primary source modules:

- `crates/ext-calc/src/lib.rs`
- `crates/ext-calc/src/code_params.rs`
- `crates/ext-calc/src/unit_convert.rs`
- `crates/ext-calc/src/checks/*.rs`
- `crates/ext-calc/src/tables/*.rs`

## 2) End-to-end execution flow

`CalcRunner::run_all` performs this sequence:

1. Load all required result tables from parquet.
2. Build a `(pier, story) -> f'c (ksi)` map from pier section material + concrete properties.
3. Run checks conditionally based on `check_selection` and optional check params.
4. Assemble `CalcOutput` payload.
5. Build top-level summary (`overall_status`, `check_count`, `pass_count`, `fail_count`, summary lines).

### 2.1 Check enable defaults

`CheckSelection::default()`:

- enabled: modal, base reactions, story forces, drift wind, drift seismic, displacement wind, pier shear wind, pier shear seismic, pier axial
- disabled by default: torsional

## 3) Inputs and normalization

## 3.1 Parquet files consumed

| File | Loader | Used by |
| --- | --- | --- |
| `story_definitions.parquet` | `tables/story_def.rs` | story forces, drift, displacement, torsional ordering/height |
| `joint_drifts.parquet` | `tables/joint_drift.rs` | drift wind/seismic, displacement wind, torsional |
| `material_properties_concrete_data.parquet` | `tables/material_props.rs` | pier fc map |
| `material_list_by_story.parquet` | `tables/material_by_story.rs` | loaded for availability/summary context |
| `modal_participating_mass_ratios.parquet` | `tables/modal.rs` | modal check |
| `base_reactions.parquet` | `tables/base_reactions.rs` | base reaction check |
| `story_forces.parquet` | `tables/story_forces.rs` | story force envelopes |
| `pier_forces.parquet` | `tables/pier_forces.rs` | pier shear, pier axial |
| `pier_section_properties.parquet` | `tables/pier_section.rs` | pier shear, pier axial, fc map |
| `group_assignments.parquet` | `tables/group_assignments.rs` | drift/displacement group filtering |

## 3.2 Loader-level computed fields

- Story definition:
  - `elevation_ft` is computed as reverse cumulative sum of `Height`.
- Pier force:
  - `shear_v2_abs_kip = abs(V2)`.
- Pier section:
  - `acv_in2 = width_bot_ft * thick_bot_ft * 144`.
  - `ag_in2 = acv_in2` (same value in current implementation).
- Material properties:
  - `fc_ksi = Fc(kip/ft^2) / 144`.
  - `fc_psi = fc_ksi * 1000`.

## 3.3 Unit context

`UnitContext` supports presets:

- `kip-ft-F`
- `kip-in-F`
- `kN-m-C`

Conversions used in outputs:

- forces normalized internally around kip-style formulas
- area/force quantities converted for display in some outputs (`qty_force`, `qty_area_in2`)
- stress checks are reported in ksi/psi-based formulas

## 4) Per-check deep dive

## 4.1 Modal participation check (`checks/modal.rs`)

### Inputs

- modal table rows
- config:
  - `modal_case`
  - `modal_threshold`
  - `modal_display_limit`

### Steps

1. Filter rows where `Case == modal_case`.
2. Sort by `Mode` ascending.
3. Find first mode where `SumUX >= modal_threshold` -> `mode_reaching_ux`.
4. Find first mode where `SumUY >= modal_threshold` -> `mode_reaching_uy`.
5. Determine number of rows to display:
   - `required_rows = max(mode_reaching_ux, mode_reaching_uy)` (if any)
   - `display_limit = max(required_rows, modal_display_limit)`
6. Emit first `display_limit` modes.

### Pass/fail

- pass if both `mode_reaching_ux` and `mode_reaching_uy` exist
- fail if either direction never reaches threshold
- hard error if configured modal case is not found

## 4.2 Base reaction RSA-vs-ELF scaling check (`checks/base_reaction.rs`)

### Inputs

- base reaction rows
- config:
  - ELF X/Y cases
  - RSA X/Y cases
  - `rsa_scale_min`

### Directional algorithm

For each direction:

1. Collect rows matching RSA case and ELF case.
2. Use directional component:
   - X direction -> `FX`
   - Y direction -> `FY`
3. Compute:
   - `V_rsa = max(abs(component))` over RSA case rows
   - `V_elf = max(abs(component))` over ELF case rows
4. Compute ratio:
   - `ratio = V_rsa / V_elf`

### Pass/fail

- direction pass: `ratio >= rsa_scale_min`
- check pass only if both X and Y pass
- hard errors:
  - missing configured case
  - `V_elf` effectively zero

## 4.3 Story force envelope extraction (`checks/story_forces.rs`)

### Inputs

- story force rows
- story definitions
- config:
  - X case list
  - Y case list

### Steps

1. X-direction envelope uses rows where:
   - `Location == "Bottom"`
   - `OutputCase` in configured X cases
2. For each story, take maxima:
   - `max_vx = max(abs(VX))`
   - `max_my = max(abs(MY))`
3. Y-direction envelope uses rows where:
   - `Location == "Bottom"`
   - `OutputCase` in configured Y cases
4. For each story, take maxima:
   - `max_vy = max(abs(VY))`
   - `max_mx = max(abs(MX))`
5. Output all stories (including zero rows) sorted top-down by elevation.

### Pass/fail

- no explicit pass/fail in this check
- summary marks this as `loaded` only

## 4.4 Drift checks (wind + seismic) (`checks/drift_wind.rs`, `checks/drift_seismic.rs`)

The seismic check reuses the same directional engine as wind, with different case lists and drift limit.

### Inputs

- joint drift rows
- story definitions
- group assignments map
- config:
  - tracking groups
  - directional case lists
  - `drift_limit`

### Group and case filtering

1. Resolve configured tracking groups to member joints.
2. Filter rows by selected output cases.
3. Assign each row into buckets keyed by:
   - `(story, group_name, output_case)`
   - only if row joint is in that group's members

### Envelope per bucket

For each bucket:

- `max_disp_x_pos_ft = max positive DispX`
- `max_disp_x_neg_ft = most negative DispX`
- `max_disp_y_pos_ft = max positive DispY`
- `max_disp_y_neg_ft = most negative DispY`
- `max_drift_x_pos = max positive DriftX`
- `max_drift_x_neg = most negative DriftX`
- `max_drift_y_pos = max positive DriftY`
- `max_drift_y_neg = most negative DriftY`

Rows are sorted by story order (bottom-up internal order map from elevation).

### Governing selection

Per output row:

- choose candidate magnitude in target direction:
  - X check: `max(abs(max_drift_x_pos), abs(max_drift_x_neg))`
  - Y check: `max(abs(max_drift_y_pos), abs(max_drift_y_neg))`

Global governing row = row with maximum candidate magnitude.

### Equation and pass/fail

- `drift_ratio = governing_magnitude`
- `dcr = drift_ratio / drift_limit`
- pass if `dcr <= 1.0`

### Special behavior

- if no cases configured for that direction, returns empty rows and pass=true
- hard error if no envelope rows were generated after filtering
- hard error if configured group missing/empty

## 4.5 Wind displacement check (`checks/displacement_wind.rs`)

### Inputs

- same row/group filtering pattern as drift
- config:
  - directional wind displacement cases
  - `disp_limit_h` (H divisor)

### Envelope per bucket

For each `(story, group_name, output_case)` bucket:

- `max_disp_x_pos_ft`, `max_disp_x_neg_ft`, `max_disp_y_pos_ft`, `max_disp_y_neg_ft`

### Governing displacement

Directional governing magnitude:

- X: `max(abs(max_disp_x_pos_ft), abs(max_disp_x_neg_ft))`
- Y: `max(abs(max_disp_y_pos_ft), abs(max_disp_y_neg_ft))`

### Limit and DCR

The limit uses total building height from story definitions:

- `H = max(story.elevation_ft)`
- `disp_limit = H / disp_limit_h`
- `dcr = governing_disp / disp_limit`
- pass if `dcr <= 1.0`

### Special behavior

- if no cases configured in a direction, returns empty/pass
- hard error if filtering produced zero rows

## 4.6 Torsional irregularity check (`checks/torsional.rs`)

### Inputs

- joint drift rows
- story definitions
- config:
  - directional case lists
  - directional joint pairs
  - `ecc_ratio`
  - building dimensions

### Preprocessing

1. Sort stories bottom-up by elevation.
2. For selected cases, build map:
   - key: `(joint, story, case, step)`
   - value: displacement in target direction (`DispX` or `DispY`)
3. Step number default fallback: `step=1` when missing.

### Story-by-story, pair-by-pair, case-by-case loop

For each adjacent story pair (bottom/top), each case, each configured joint pair:

1. For each step:
   - `drift_a = abs(disp_top_a - disp_bot_a) * 12`
   - `drift_b = abs(disp_top_b - disp_bot_b) * 12`
   - `delta_max = max(drift_a, drift_b)`
   - `delta_avg = (drift_a + drift_b) / 2`
2. Across steps, compute governing ratio:
   - `ratio_step = delta_max / delta_avg` (if avg > tiny)
   - `ratio = max(ratio_step)`
3. Compute:
   - `ax_base_step = (delta_max / (1.2 * delta_avg))^2`
   - `ax = clamp(max(ax_base_step), 1.0, 3.0)`
4. Compute eccentricity and rho:
   - `ecc_ft = ecc_ratio * perpendicular_building_dimension`
   - `is_type_a = ratio > 1.2`
   - `is_type_b = ratio > 1.4`
   - `rho = 1.3 if type_b else 1.0`

Rows without complete data for all required joints/stories/steps are skipped.

### Direction-level governing and pass/fail

- governing row = maximum `ratio`
- directional flags:
  - `has_type_a = any row type A`
  - `has_type_b = any row type B`
- overall torsion pass:
  - pass only if neither X nor Y has type B

### Special behavior

- if cases or pairs are empty for a direction, returns empty output for that direction

## 4.7 Pier shear stress checks (wind and seismic) (`checks/pier_shear_stress.rs`)

Both wind and seismic checks use the same engine with different combo lists and phi/default-fc params.

### Inputs

- pier forces
- pier sections
- fc map `(pier, story) -> fc_ksi`
- config:
  - combos
  - `phi_v`
  - `fc_default_ksi`

### Envelope creation

1. Keep rows whose `OutputCase` is in configured combos.
2. For each `(story, pier)`, keep single governing combo with max `abs(V2)`.

### Section and orientation data

- `Acw` from pier sections (`acv_in2`)
- wall direction inferred from `AxisAngle`:
  - X if `|cos(angle)| > |sin(angle)|`, else Y

### Per-pier stress equation

For each enveloped pier:

- `fc_psi = fc_ksi * 1000`
- `sqrt_fc = sqrt(fc_psi)`
- `stress_psi = (Ve_kip * 1000) / (phi_v * Acw_in2)`
- `stress_ratio = stress_psi / sqrt_fc`

Limits:

- individual limit = `8.0`
- pass if `stress_ratio <= 8.0`

### Story-level average check by direction

For each story and wall direction:

1. Sum enveloped shears and areas:
   - `sum_ve`, `sum_acw`
2. Use minimum encountered `fc_psi` in bucket (conservative for mixed strengths).
3. Compute:
   - `avg_stress_psi = (sum_ve * 1000) / (phi_v * sum_acw)`
   - `avg_stress_ratio = avg_stress_psi / sqrt(min_fc_psi)`

Average limit:

- `10.0`
- pass if `avg_stress_ratio <= 10.0`

### Overall pass/fail

- pass if:
  - `max_individual_ratio <= 8.0`
  - `max_average_ratio <= 10.0`

## 4.8 Pier axial stress check (`checks/pier_axial.rs`)

### Inputs

- pier forces
- pier sections
- fc map `(pier, story) -> fc_ksi`
- config:
  - gravity combos
  - wind combos
  - seismic combos
  - `phi_axial`
  - `fc_default_ksi`

### Envelope creation

1. Keep rows belonging to any configured combo category.
2. Group by `(story, pier, combo)`.
3. Store axial demand with max absolute value in each group.

### Strength and stress equations

For each grouped result with matching section:

- `Pu = abs(P)`
- `Ag = section.ag_in2`
- `Po = 0.85 * fc_ksi * Ag`
- `phiPo = phi_axial * Po`
- `dcr = Pu / phiPo`
- `fa = Pu / Ag`
- `fa_ratio = fa / (0.85 * fc_ksi)`

Pass:

- row pass if `dcr <= 1.0`

### Governing outputs

- overall governing = row with max `dcr`
- category governing tracked separately for gravity/wind/seismic
- overall pass = all rows pass

### Special behavior

- rows without matching section are skipped
- if no qualifying results, returns empty/check-passing dummy output instead of error

## 5) Summary aggregation behavior (`build_summary` in `lib.rs`)

Checks that increment `check_count`:

- modal
- base reactions
- drift wind
- drift seismic
- displacement wind
- torsional
- pier shear wind
- pier shear seismic
- pier axial

`story_forces` does not increment pass/fail counts (it is marked as loaded data extraction).

`overall_status`:

- `fail` if any failing check
- `pass` if no failing check and at least one passing check
- `pending` otherwise

Additional summary info lines always appended:

- concrete material count loaded
- group mapping count loaded

## 6) Review-focused implementation notes

These are implementation details worth explicitly reviewing against design/code intent:

- Displacement check labels use project length label, while source displacement fields are consumed directly from table columns without explicit unit conversion in this module.
- Story elevations are derived from reverse cumulative height order; correctness assumes parquet row order aligns with expected story sequence.
- Torsional drift conversion multiplies displacement differences by `12` inside the check, effectively treating source displacement as feet for the irregularity ratio trace.
- Pier fc map keying is `(pier, story)` and falls back to defaults when material lookup is missing.
- Pier axial check currently returns a pass/empty payload when no qualifying rows exist, rather than failing hard.

## 7) Quick formula index

- Modal threshold reach: `SumUX >= threshold`, `SumUY >= threshold`
- Base reaction ratio: `V_rsa / V_elf`
- Drift DCR: `drift_ratio / drift_limit`
- Displacement limit: `H / disp_limit_h`
- Displacement DCR: `disp_governing / (H / disp_limit_h)`
- Torsion ratio: `delta_max / delta_avg`
- Torsion `Ax`: `(delta_max / (1.2 * delta_avg))^2`, clamped `[1, 3]`
- Pier shear stress: `(Ve*1000)/(phi_v*Acw)`
- Pier shear ratio: `stress_psi / sqrt(fc_psi)`
- Pier axial nominal: `Po = 0.85 * fc * Ag`
- Pier axial strength: `phiPo = phi_axial * Po`
- Pier axial DCR: `Pu / phiPo`

