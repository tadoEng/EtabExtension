# `ext-calc` — Calculation Specification

**Crate:** `crates/ext-calc`  
**Purpose:** All structural engineering calculations for the EtabExtension workspace. Takes raw ETABS Parquet tables + user configuration, runs code checks, and produces a fully typed `CalcOutput` for downstream rendering and reporting.  
**Code standard:** ACI 318-14 (configurable)  
**Audience:** Structural engineers performing hand-verification, developers adding new checks.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Inputs](#2-inputs)
   - 2.1 [Parquet Tables](#21-parquet-tables)
   - 2.2 [Configuration Parameters (CodeParams)](#22-configuration-parameters-codeparams)
   - 2.3 [Unit System](#23-unit-system)
3. [Validation and Errors](#3-validation-and-errors)
4. [Checks](#4-checks)
   - 4.1 [Modal Mass Participation](#41-modal-mass-participation)
   - 4.2 [Base Shear — RSA vs ELF](#42-base-shear--rsa-vs-elf)
   - 4.3 [Story Drift — Wind](#43-story-drift--wind)
   - 4.4 [Story Drift — Seismic](#44-story-drift--seismic)
   - 4.5 [Lateral Displacement — Wind](#45-lateral-displacement--wind)
   - 4.6 [Pier Shear (Wind)](#46-pier-shear-wind)
   - 4.7 [Pier Shear (Seismic)](#47-pier-shear-seismic)
   - 4.8 [Pier Axial Stress](#48-pier-axial-stress)
   - 4.9 [Torsional Irregularity (Pending)](#49-torsional-irregularity-pending)
5. [Output Structure](#5-output-structure)
6. [Summary Roll-Up](#6-summary-roll-up)
7. [Unit Conversion Reference](#7-unit-conversion-reference)
8. [Hand-Verification Examples](#8-hand-verification-examples)

---

## 1. Architecture Overview

```
CalcRunner::run_all(results_dir, params)
    │
    ├── tables::*::load_*()              ← read Parquet files from results_dir
    │
    ├── pier_shear::build_pier_fc_map()  ← shared (pier, story) → fc_ksi map
    │
    ├── checks::modal::run()             ← §4.1
    ├── checks::base_reaction::run()     ← §4.2
    ├── checks::drift_wind::run()        ← §4.3
    ├── checks::drift_seismic::run()     ← §4.4
    ├── checks::displacement_wind::run() ← §4.5
    ├── checks::pier_shear_wind::run()   ← §4.6
    ├── checks::pier_shear_seismic::run() ← §4.7
    ├── checks::pier_axial::run()        ← §4.8
    │
    └── build_summary()                  ← §6
            │
            └── CalcOutput               ← §5
```

Each check is **independently gated** by `CheckSelection` (all 8 checks enabled by default; `torsional = false`). A check returning `None` does not affect the pass/fail of other checks.

The **pier fc map** (`HashMap<(pier_label, story), f64>`) is built once before the pier checks and shared across all three (wind shear, seismic shear, axial) to avoid redundant material lookups.

---

## 2. Inputs

### 2.1 Parquet Tables

All files are read from `results_dir`. Files are produced by the C# sidecar (ETABS COM API extraction).

| Parquet file | Loader | Key columns used in checks |
|---|---|---|
| `story_definitions.parquet` | `tables::story_def` | `story`, `elevation_ft` |
| `joint_drifts.parquet` | `tables::joint_drift` | `unique_name`, `story`, `output_case`, `disp_x_ft`, `disp_y_ft`, `drift_x`, `drift_y` |
| `material_props.parquet` | `tables::material_props` | `name`, `fc_ksi`, `fc_psi`, `is_lightweight` |
| `material_by_story.parquet` | `tables::material_by_story` | loaded; currently unused |
| `modal_participation.parquet` | `tables::modal` | `case_name`, `mode`, `period_sec`, `ux`, `uy`, `sum_ux`, `sum_uy`, `rz`, `sum_rz` |
| `base_reactions.parquet` | `tables::base_reactions` | `output_case`, `case_type`, `step_type`, `fx_kip`, `fy_kip` (and moments for review table) |
| `story_forces.parquet` | `tables::story_forces` | loaded; currently unused |
| `pier_forces.parquet` | `tables::pier_forces` | `output_case`, `story`, `pier`, `shear_v2_abs_kip`, `axial_p_kip` |
| `pier_section_properties.parquet` | `tables::pier_section` | `story`, `pier`, `width_bot_ft`, `thick_bot_ft`, `material`, `acv_in2`, `ag_in2` |
| `group_assignments.parquet` | `tables::group_assignments` | `group_name` → `Vec<unique_joint_name>` |

> **`acv_in2` and `ag_in2`** are pre-computed at Parquet load time as:
>   `acv_in2 = width_bot_ft × thick_bot_ft × 144`
>   `ag_in2  = width_bot_ft × thick_bot_ft × 144`  (solid wall; same as Acv)
>
> Check modules never re-derive these — they use the stored values directly.

### 2.2 Configuration Parameters (`CodeParams`)

All parameters are loaded from the project TOML config via `CodeParams::from_config()`.

| Parameter | Config key | Type | Required | Default / Constraint |
|---|---|---|---|---|
| `code` | `[calc].code` | String | No | `"ACI318-14"` |
| `occupancy_category` | `[calc].occupancy` | String | No | `"II"` |
| `modal_case` | `[calc].modal-case` | String | **Yes** | non-empty |
| `modal_threshold` | `[calc.modal].min-mass-participation` | f64 | **Yes** | > 0.0 |
| `modal_display_limit` | `[calc.modal].display-mode-limit` | u32 | **Yes** | > 0 |
| `drift_tracking_groups` | `[calc].drift-tracking-groups` | Vec\<String\> | **Yes** | non-empty |
| `base_shear.elf_case_x` | `[calc.base-shear].elf-case-x` | String | **Yes** | non-empty |
| `base_shear.elf_case_y` | `[calc.base-shear].elf-case-y` | String | **Yes** | non-empty |
| `base_shear.rsa_case_x` | `[calc.base-shear].rsa-case-x` | String | **Yes** | non-empty |
| `base_shear.rsa_case_y` | `[calc.base-shear].rsa-case-y` | String | **Yes** | non-empty |
| `base_shear.rsa_scale_min` | `[calc.base-shear].rsa-scale-min` | f64 | **Yes** | > 0.0 |
| `drift_wind.load_cases` | `[calc.drift-wind].load-cases` | Vec\<String\> | **Yes** | non-empty |
| `drift_wind.drift_limit` | `[calc.drift-wind].drift-limit` | f64 | **Yes** | > 0.0 |
| `drift_seismic.load_cases` | `[calc.drift-seismic].load-cases` | Vec\<String\> | **Yes** | non-empty |
| `drift_seismic.drift_limit` | `[calc.drift-seismic].drift-limit` | f64 | **Yes** | > 0.0 |
| `displacement_wind.load_cases` | `[calc.displacement-wind].load-cases` | Vec\<String\> | **Yes** | non-empty |
| `displacement_wind.disp_limit_h` | `[calc.displacement-wind].disp-limit-h` | u32 | **Yes** | > 0 |
| `pier_shear_wind.load_combos` | `[calc.pier-shear-wind].load-combos` | Vec\<String\> | **Yes** | non-empty |
| `pier_shear_wind.phi_v` | `[calc.pier-shear-wind].phi-v` | f64 | No | **0.75** |
| `pier_shear_wind.alpha_c` | `[calc.pier-shear-wind].alpha-c` | f64 | No | **2.0** |
| `pier_shear_wind.fy_ksi` | `[calc.pier-shear-wind].fy-ksi` | f64 | No | **60.0** |
| `pier_shear_wind.rho_t` | `[calc.pier-shear-wind].rho-t` | f64 | No | **0.0025** |
| `pier_shear_wind.fc_default_ksi` | `[calc.pier-shear-wind].fc-default-ksi` | f64 | No | **8.0** |
| `pier_shear_seismic.load_combos` | `[calc.pier-shear-seismic].load-combos` | Vec\<String\> | **Yes** | non-empty |
| `pier_shear_seismic.phi_v` | `[calc.pier-shear-seismic].phi-v` | f64 | No | **0.60** |
| `pier_shear_seismic.alpha_c / fy_ksi / rho_t / fc_default_ksi` | (same section) | f64 | No | same as wind |
| `pier_axial.load_combos` | `[calc.pier-axial].load-combos` | Vec\<String\> | **Yes** | non-empty |
| `pier_axial.phi_axial` | `[calc.pier-axial].phi-axial` | f64 | No | **0.65** |

### 2.3 Unit System

The `UnitContext` is set once from `[project].units` and propagates to all output quantities.  
All internal arithmetic uses **kip** and **inch**. Conversion to display units is applied only via `qty_*` helpers at the point of output construction.

| Preset string(s) | Force | Length | Moment | Stress |
|---|---|---|---|---|
| `kip-ft-F`, `US_Kip_Ft`, `kip_ft` | kip | ft | kip·ft | native ÷ 144 → ksi |
| `kip-in-F`, `US_Kip_In`, `kip_in` | kip | in | kip·in | native → ksi |
| `kN-m-C`, `SI_kN_m`, `kN_m` | kN | m | kN·m | native × 0.000145038 → ksi |

> **Stress is always reported in ksi** regardless of the project preset.

---

## 3. Validation and Errors

All errors are `anyhow::Error`, returned through `?` and propagated to the caller.  
No check silently swallows a configuration error.

### Fatal Errors (abort the check)

| Condition | Error message |
|---|---|
| Required config field absent | `"missing required config: [section].key"` |
| Required config field ≤ 0 | `"invalid required config: [section].key must be greater than zero"` |
| Configured modal case not in data | `"Configured modal case 'X' not found in modal participation results"` |
| Configured base shear case not in data | `"Configured base shear case 'X' not found"` |
| ELF base shear is zero | `"ELF base shear for case 'X' is zero"` |
| Configured drift load case not in data | `"Configured drift load case 'X' not found"` |
| Configured drift group not in group_map | `"Configured drift group 'X' not found"` |
| Drift group has no member joints | `"Configured drift group 'X' has no members"` |
| No data rows for a group+case pair | `"No drift rows found for group 'X' and case 'Y'"` |
| Configured displacement case not in data | `"Configured displacement load case 'X' not found"` |
| Configured pier shear combo not in forces | `"Configured pier shear combo 'X' not found in pier_forces table"` |
| Configured pier axial combo not in forces | `"Configured pier axial combo 'X' not found in pier_forces table"` |
| No pier results produced | `"No pier shear results produced — check section/force table alignment"` |

### Non-Fatal Warnings (print to stderr, execution continues)

| Condition | Warning message |
|---|---|
| Material name for a pier/story not in material_props | `"[ext-calc] warn: material 'M' not found for pier 'P' at story 'S'; using fc_default = X ksi"` |
| Section properties missing for pier/story in forces | `"[ext-calc] warn: no section properties for pier 'P' at story 'S'; row skipped"` |

---

## 4. Checks

### 4.1 Modal Mass Participation

**Source:** `checks::modal`  
**Code ref:** ASCE 7 §12.9.1 — cumulative modal mass participation ≥ threshold in each direction

#### Inputs

| Input | Type | Source |
|---|---|---|
| `rows` | `&[ModalParticipationRow]` | `modal_participation.parquet` |
| `params.modal_case` | String | Config |
| `params.modal_threshold` | f64 | Config (e.g. 0.90) |
| `params.modal_display_limit` | usize | Config (e.g. 20) |

#### Procedure

```
1. Filter rows where case_name == modal_case
   → error if empty

2. Sort by mode number ascending

3. Find mode_reaching_ux = first mode where sum_ux >= threshold
   Find mode_reaching_uy = first mode where sum_uy >= threshold

4. required_rows = max(mode_reaching_ux, mode_reaching_uy) [or 0]
   display_limit  = max(required_rows, modal_display_limit)

5. Output = first display_limit rows

6. pass = mode_reaching_ux.is_some() AND mode_reaching_uy.is_some()
```

#### Output: `ModalOutput`

| Field | Type | Description |
|---|---|---|
| `rows` | `Vec<ModalModeRow>` | Mode table (up to display_limit rows) |
| `threshold` | f64 | Configured threshold (e.g. 0.90) |
| `mode_reaching_ux` | `Option<i64>` | Mode number where ΣUX ≥ threshold |
| `mode_reaching_uy` | `Option<i64>` | Mode number where ΣUY ≥ threshold |
| `pass` | bool | Both UX and UY thresholds reached |

`ModalModeRow` fields: `case`, `mode`, `period`, `ux`, `uy`, `sum_ux`, `sum_uy`, `rz`, `sum_rz`.

#### Hand Check (fixture)

```
Fixture: Modal-Rizt case, threshold = 0.90
  ΣUX reaches 0.90 at mode 12  →  mode_reaching_ux = 12
  ΣUY reaches 0.90 at mode 23  →  mode_reaching_uy = 23
  display_limit = max(23, 20) = 23  →  23 rows in output
  pass = true
```

---

### 4.2 Base Shear — RSA vs ELF

**Source:** `checks::base_reaction`  
**Code ref:** ASCE 7 §12.9.4 — RSA base shear ≥ `rsa_scale_min × V_ELF`

#### Inputs

| Input | Type | Source |
|---|---|---|
| `rows` | `&[BaseReactionRow]` | `base_reactions.parquet` |
| `params.base_shear.elf_case_x / _y` | String | Config |
| `params.base_shear.rsa_case_x / _y` | String | Config |
| `params.base_shear.rsa_scale_min` | f64 | Config (e.g. 1.0) |

#### Procedure (per direction, X and Y independently)

```
For X direction:
  V_RSA_X = max |Fx| across all rows where output_case == rsa_case_x
  V_ELF_X = max |Fx| across all rows where output_case == elf_case_x
  → error if either case not found
  → error if V_ELF_X == 0

  ratio_x = V_RSA_X / V_ELF_X
  pass_x  = ratio_x >= rsa_scale_min

For Y direction: same logic using Fy
```

**Review table filtering** (for reporting, not for the ratio calculation):
Rows excluded from the review table:
- `output_case` starts with `~`
- `output_case` is `Modal-Rizt` or `Modal-Eigen`
- `case_type` starts with `LinMod`

#### Output: `BaseShearOutput`

| Field | Type | Description |
|---|---|---|
| `rows` | `Vec<BaseReactionCheckRow>` | Filtered review table |
| `direction_x` | `BaseShearDir` | X-direction RSA/ELF check |
| `direction_y` | `BaseShearDir` | Y-direction RSA/ELF check |

`BaseShearDir` fields: `rsa_case`, `elf_case`, `v_rsa` (Qty), `v_elf` (Qty), `ratio`, `pass`.

Overall check: `pass = direction_x.pass AND direction_y.pass`.

#### Hand Check (fixture)

```
Direction X:
  V_RSA_X ≈ 1421.5 kip,  V_ELF_X ≈ 1373.2 kip
  ratio = 1421.5 / 1373.2 ≈ 1.0347  ≥ 1.0  →  PASS

Direction Y:
  V_RSA_Y ≈ 1589.4 kip,  V_ELF_Y ≈ 1275.5 kip
  ratio = 1589.4 / 1275.5 ≈ 1.2454  ≥ 1.0  →  PASS
```

---

### 4.3 Story Drift — Wind

**Source:** `checks::drift_wind` → `build_drift_output()`  
**Code ref:** ASCE 7 / local code — interstory drift ratio ≤ `drift_limit`

#### Inputs

| Input | Type | Source |
|---|---|---|
| `rows` | `&[JointDriftRow]` | `joint_drifts.parquet` |
| `stories` | `&[StoryDefRow]` | `story_definitions.parquet` |
| `group_map` | `&HashMap<String, Vec<String>>` | `group_assignments.parquet` |
| `params.drift_tracking_groups` | `Vec<String>` | Config |
| `params.drift_wind.load_cases` | `Vec<String>` | Config |
| `params.drift_wind.drift_limit` | f64 | Config (e.g. 0.0025 = H/400) |

#### Procedure

```
1. Resolve groups:
   For each group in drift_tracking_groups:
     members = group_map[group]  → error if missing or empty

2. Validate load cases exist in data  → error if any case not found

3. Group rows by (story, group_name, output_case) for rows where:
   - output_case in configured load_cases
   - unique_name in current group's member set

4. Validate each (group, case) pair has ≥ 1 row  → error if missing

5. For each (story, group_name, output_case) bucket, compute envelope:
   max_disp_x_pos_ft  = max(disp_x_ft for disp_x_ft > 0), else 0.0
   max_disp_x_neg_ft  = min(disp_x_ft for disp_x_ft < 0), else 0.0
   max_disp_y_pos_ft  = max(disp_y_ft for disp_y_ft > 0), else 0.0
   max_disp_y_neg_ft  = min(disp_y_ft for disp_y_ft < 0), else 0.0
   max_drift_x_pos    = max(drift_x  for drift_x  > 0), else 0.0
   max_drift_x_neg    = min(drift_x  for drift_x  < 0), else 0.0
   max_drift_y_pos    = max(drift_y  for drift_y  > 0), else 0.0
   max_drift_y_neg    = min(drift_y  for drift_y  < 0), else 0.0

6. Sort rows by story elevation ascending (L01 → ROOF)

7. Find governing:
   For each row, pick best direction/sense = max of:
     |max_drift_x_pos|, |max_drift_x_neg|, |max_drift_y_pos|, |max_drift_y_neg|
   Governing row = the row+direction with the global maximum drift ratio

8. dcr  = governing_drift_ratio / drift_limit
   pass = dcr <= 1.0
```

> **Note:** `roof_disp_x`, `roof_disp_y`, `disp_limit`, and `disp_pass` are set to `None` for wind and seismic drift checks. They are only populated by the displacement check (§4.5).

#### Output: `DriftOutput`

| Field | Type | Description |
|---|---|---|
| `allowable_ratio` | f64 | drift_limit from config |
| `rows` | `Vec<DriftEnvelopeRow>` | Per-(story, group, case) envelope, elevation-sorted |
| `governing` | `StoryDriftResult` | Worst story / group / case / direction / sense |
| `pass` | bool | DCR ≤ 1.0 |
| `roof_disp_x / _y` | None | (reserved for §4.5) |
| `disp_limit / disp_pass` | None | (reserved for §4.5) |

`StoryDriftResult` fields: `story`, `group_name`, `output_case`, `direction` ("X" or "Y"), `sense` ("positive" or "negative"), `drift_ratio`, `dcr`, `pass`.

#### Hand Check (fixture)

```
Governing: story=L35, group=Joint48, case=Wind_10yr_Diagonal, direction=Y, sense=positive
drift_ratio (from model)  /  drift_limit = 0.0025  =  dcr ≈ 0.4028  →  PASS
```

---

### 4.4 Story Drift — Seismic

**Source:** `checks::drift_seismic`  
**Identical procedure** to §4.3, substituting:
- Load cases: `params.drift_seismic.load_cases`
- Limit: `params.drift_seismic.drift_limit` (typical: 0.020 per ASCE 7 Table 12.12-1, Risk Category II)

---

### 4.5 Lateral Displacement — Wind

**Source:** `checks::displacement_wind`  
**Code ref:** Serviceability — absolute roof displacement ≤ H / `disp_limit_h`

#### Inputs

Same joint drift data as §4.3, using:
- `params.displacement_wind.load_cases`
- `params.displacement_wind.disp_limit_h` (e.g. 500 → H/500)

#### Procedure

```
1. Same group resolution as §4.3, steps 1–4

2. Group rows by (story, group_name, output_case)
   For each bucket, compute displacement envelope:
     max_disp_x_pos_ft, max_disp_x_neg_ft
     max_disp_y_pos_ft, max_disp_y_neg_ft

3. Sort by story elevation ascending

4. total_height_ft = max elevation across all stories in story_definitions
   disp_limit_ft   = total_height_ft / disp_limit_h

5. Find governing:
   For each row, pick best direction/sense = max of:
     |max_disp_x_pos_ft|, |max_disp_x_neg_ft|,
     |max_disp_y_pos_ft|, |max_disp_y_neg_ft|
   Governing = global maximum

6. dcr  = governing_displacement_ft / disp_limit_ft
   pass = dcr <= 1.0
```

#### Unit Conversion for Display

```
qty_length_disp(ft):
  kip-ft / kip-in:  ft × 12  →  inches
  kN-m:             ft × 304.8  →  mm
```

#### Output: `DisplacementOutput`

| Field | Type | Description |
|---|---|---|
| `rows` | `Vec<DisplacementEnvelopeRow>` | Per-(story, group, case) displacement envelope |
| `governing` | `JointDisplacementResult` | Worst story / direction |
| `disp_limit` | `Quantity` | H / disp_limit_h in display units |
| `pass` | bool | DCR ≤ 1.0 |

`JointDisplacementResult` fields: `story`, `group_name`, `output_case`, `direction`, `sense`, `displacement` (Qty), `dcr`, `pass`.

#### Hand Check (fixture)

```
total_height → yields disp_limit_ft = H / 500
governing: story=ROOF, group=Joint48, case=Wind_10yr_Diagonal, direction=Y, sense=positive
displacement = 3.944 in
disp_limit   = 11.232 in   → disp_limit_ft = 11.232/12 = 0.936 ft → H = 0.936 × 500 = 468 ft
dcr = 3.944 / 11.232 ≈ 0.3512  →  PASS
```

---

### 4.6 Pier Shear (Wind)

**Source:** `checks::pier_shear_wind` → `checks::pier_shear::run()`  
**Code ref:** ACI 318-14 §11.5.4.3  
**ϕ = 0.75** (ACI 318-14 §9.3.2.3)

#### Formula (psi-based US customary)

```
Acv  [in²]  = lw [ft] × t [ft] × 144              (stored in pier_section)
fc'  [psi]  = fc_ksi × 1000
fy   [psi]  = fy_ksi × 1000                         default: 60,000 psi

Vn   [lb]   = Acv × (αc × √fc'  +  ρt × fy)        αc = 2.0 (psi form, hw/lw ≥ 2)
Vn   [kip]  = Vn [lb] / 1000
ϕVn  [kip]  = ϕ × Vn
DCR         = Vu / ϕVn
```

> **CRITICAL:** αc = 2.0 is the **psi-based** form. Do NOT use αc = 0.17 (MPa form).

#### Inputs

| Input | Type | Source |
|---|---|---|
| `forces` | `&[PierForceRow]` | `pier_forces.parquet` — `shear_v2_abs_kip` = |V2| pre-computed |
| `sections` | `&[PierSectionRow]` | `pier_section_properties.parquet` |
| `fc_map` | `&HashMap<(String,String), f64>` | pre-built in `CalcRunner::run_all` |
| `params.pier_shear_wind` | `PierShearParams` | Config |

`PierShearParams` defaults: `phi_v=0.75`, `alpha_c=2.0`, `fy_ksi=60.0`, `rho_t=0.0025`, `fc_default_ksi=8.0`.

#### Procedure

```
1. Validate all configured combos exist in forces table  → error if missing

2. Build section_map: (pier, story) → PierSectionRow

3. Group forces by (story, pier, combo):
   Vu = max(shear_v2_abs_kip) across all locations (Top/Bottom) and steps (Max/Min/StepByStep)
   → envelopes over all ETABS output sub-rows for that pier+story+combo

4. For each (story, pier, combo) group:
   a. Look up section  → skip with stderr warning if absent
   b. Look up fc_ksi from fc_map  → use fc_default_ksi if absent (with stderr warning)
   c. Compute Acv = section.acv_in2
   d. Apply ACI formula above
   e. dcr = Vu / phi_vn;  pass = dcr <= 1.0

5. governing = result with max dcr
   pass = all(results[i].pass)
```

#### Output: `PierShearOutput`

| Field | Type | Description |
|---|---|---|
| `phi_v` | f64 | ϕ used (0.75 for wind) |
| `piers` | `Vec<PierShearResult>` | One row per (pier, story, combo) |
| `governing` | `PierShearResult` | Row with max DCR |
| `pass` | bool | All piers pass |

`PierShearResult` fields: `pier_label`, `story`, `combo`, `location` ("envelope"), `vu` (Qty), `acv` (Qty), `fc_ksi`, `vn` (Qty), `phi_vn` (Qty), `dcr`, `pass`, `section_id`, `material`.

#### Hand Check — C1Y1 at L20

```
Input:
  lw = 22 ft,  t = 2 ft
  fc' = 8.0 ksi → 8,000 psi
  αc = 2.0,  fy = 60,000 psi,  ρt = 0.0025
  ϕ  = 0.75

Step 1:  Acv = 22 × 2 × 144 = 6,336 in²
Step 2:  Vn  = 6336 × (2.0 × √8000  +  0.0025 × 60000) / 1000
             = 6336 × (178.885 + 150.0) / 1000
             = 6336 × 328.885 / 1000
             ≈ 2083.9 kip
Step 3:  ϕVn = 0.75 × 2083.9 ≈ 1562.9 kip
Step 4:  Vu  ≈ 159.5 kip  (from Parquet, max |V2|)
Step 5:  DCR = 159.5 / 1562.9 ≈ 0.102  →  ✅ PASS
```

---

### 4.7 Pier Shear (Seismic)

**Source:** `checks::pier_shear_seismic` → `checks::pier_shear::run()`  
**Code ref:** ACI 318-14 §11.5.4.3 + §18.10.4  
**ϕ = 0.60** (ACI 318-14 §21.2.4.1)

**Identical formula to §4.6.** The only differences are:
- `phi_v = 0.60` (seismic ductility demand) instead of 0.75
- Different load combos (`pier_shear_seismic.load_combos`)

For the same `Vn`, seismic `ϕVn = 0.60 × Vn` is 20% lower than wind, producing proportionally higher DCR.

#### Hand Check — C1Y1 at L20 (seismic)

```
Vn  ≈ 2083.9 kip  (same section and material)
ϕVn = 0.60 × 2083.9 ≈ 1250.3 kip
Vu  = seismic combo demand (from Parquet)
DCR = Vu / 1250.3
```

---

### 4.8 Pier Axial Stress

**Source:** `checks::pier_axial`  
**Code ref:** ACI 318-14 §22.4 — simplified squash load (rebar omitted)  
**ϕ = 0.65** (ACI 318-14 §9.3.2.2, tied column)

#### Formula

```
Pu   [kip] = |P|                     (largest absolute axial across Top/Bottom/Max/Min)
Ag   [in²] = lw [ft] × t [ft] × 144 (gross section, same as Acv for solid wall)

Po   [kip] = 0.85 × fc' [ksi] × Ag  (nominal squash load, no rebar)
ϕPo  [kip] = ϕ × Po
DCR         = Pu / ϕPo

fa   [ksi]  = Pu / Ag                (computed axial stress, for reference)
fa_ratio    = fa / (0.85 × fc')      (utilisation vs squash stress)
```

> **ETABS sign convention:** Compression is **negative** in ETABS. The check takes `|P|` (absolute value) to obtain `Pu`. The value with the largest absolute force governs.

> **Simplification:** The rebar contribution `Ast × fy` is omitted. This is the conservative simplified form for preliminary wall checks. Phase 2 enhancement: add `Ast × fy` when rebar area data is available from ETABS.

#### Inputs

| Input | Type | Source |
|---|---|---|
| `forces` | `&[PierForceRow]` | `pier_forces.parquet` — `axial_p_kip` |
| `sections` | `&[PierSectionRow]` | `pier_section_properties.parquet` — `ag_in2` |
| `fc_map` | `&HashMap<(String,String), f64>` | shared pier fc map |
| `params.pier_axial` | `PierAxialParams` | Config |

#### Procedure

```
1. Validate combos exist in forces table  → error if missing

2. Group forces by (story, pier, combo):
   Governing P = value with largest |axial_p_kip|
   (for compression walls this is the most negative P)

3. For each (story, pier, combo) group:
   a. Look up section  → skip with stderr warning if absent
   b. Look up fc_ksi   → use fallback if absent
   c. Pu = |P|,  Ag = section.ag_in2
   d. Apply formula above

4. governing = result with max DCR
   pass = all(results[i].pass)
```

#### Output: `PierAxialOutput`

| Field | Type | Description |
|---|---|---|
| `piers` | `Vec<PierAxialResult>` | One row per (pier, story, combo) |
| `governing` | `PierAxialResult` | Row with max DCR |
| `pass` | bool | All piers pass |

`PierAxialResult` fields: `pier_label`, `story`, `combo`, `pu` (Qty), `ag` (Qty), `phi_po` (Qty), `fa` (Qty, always ksi), `fa_ratio`, `dcr`, `pass`, `fc_ksi`, `material`.

#### Hand Check — C1Y1 at L01

```
Input:
  lw = 42 ft,  t = 2 ft
  fc' = 8.0 ksi,  ϕ = 0.65

Step 1:  Ag   = 42 × 2 × 144    = 12,096 in²
Step 2:  Po   = 0.85 × 8.0 × 12096 = 82,252.8 kip
Step 3:  ϕPo  = 0.65 × 82,252.8    = 53,464.3 kip
Step 4:  Pu   = |P from seismic combo|  (read from model Parquet)
         DCR  = Pu / 53,464.3
```

---

### 4.9 Torsional Irregularity (Pending)

**Status:** Not implemented. `CalcOutput.torsional` is always `None`.  
**Summary line:** key=`torsional`, status=`"pending"`, message=`"torsional irregularity check not implemented yet"`.

**Planned formula:** ASCE 7 Table 12.3-1 Type 1a/1b:
- Type 1a: `δ_max > 1.2 × δ_avg`
- Type 1b: `δ_max > 1.4 × δ_avg`

where `δ_max` and `δ_avg` are computed from the two extreme edge points at each story under seismic load.

---

## 5. Output Structure

`CalcOutput` is the root serialized output (Serde → camelCase JSON).

```
CalcOutput
├── meta: CalcMeta
│   ├── version_id: String
│   ├── branch: String
│   ├── code: String
│   ├── generated_at: DateTime<Utc>
│   └── units: UnitLabels { force, length, stress, moment }
│
├── summary: CalcSummary
│
├── modal: Option<ModalOutput>
├── base_shear: Option<BaseShearOutput>
├── drift_wind: Option<DriftOutput>
├── drift_seismic: Option<DriftOutput>
├── displacement_wind: Option<DisplacementOutput>
├── torsional: Option<TorsionalOutput>      ← always None currently
├── pier_shear_wind: Option<PierShearOutput>
├── pier_shear_seismic: Option<PierShearOutput>
└── pier_axial: Option<PierAxialOutput>
```

`Some(...)` → check was enabled and ran.  
`None` → check was disabled via `CheckSelection` or not yet implemented.

---

## 6. Summary Roll-Up

`build_summary()` aggregates all check outputs into `CalcSummary`.

| Field | Type | Description |
|---|---|---|
| `overall_status` | String | `"pass"` / `"fail"` / `"pending"` |
| `check_count` | u32 | Number of checks that ran |
| `pass_count` | u32 | |
| `fail_count` | u32 | |
| `lines` | `Vec<SummaryLine>` | One line per check or informational item |

`overall_status` logic:
- `"fail"` if `fail_count > 0`
- `"pass"` if `fail_count == 0` and `pass_count > 0`
- `"pending"` if no checks ran

`SummaryLine` fields: `key` (camelCase), `status` ("pass"/"fail"/"pending"/"loaded"), `message` (human-readable).

Fixed non-check lines always appended: `materials` (loaded count), `driftGroups` (loaded count), `torsional` (pending).

---

## 7. Unit Conversion Reference

| `UnitContext` method | kip-ft-F | kip-in-F | kN-m-C |
|---|---|---|---|
| `force_to_kip(v)` | v | v | v × 0.224809 |
| `length_to_inch(ft)` | ft × 12 | ft × 1 | ft × 39.3701 |
| `length_to_ft(v)` | v | v ÷ 12 | v × 3.28084 |
| `stress_to_ksi(v)` | v ÷ 144 | v | v × 0.000145038 |
| `qty_force(kip)` | kip | kip | kip ÷ 0.224809 → kN |
| `qty_area_in2(in2)` | in2 ÷ 144 → ft² | in2 | in2 × 0.00064516 → m² |
| `qty_length_disp(ft)` | ft × 12 → in | ft × 12 → in | ft × 304.8 → mm |

> Stress is always in **ksi** regardless of preset.

---

## 8. Hand-Verification Examples

### Example A — Pier Shear Wind: C1Y1 at L20

| Step | Variable | Calculation | Result |
|---|---|---|---|
| Section | lw, t | given | 22 ft, 2 ft |
| Gross area | Acv | 22 × 2 × 144 | **6,336 in²** |
| Concrete | fc' | 8.0 ksi × 1000 | **8,000 psi** |
| Term 1 | αc × √fc' | 2.0 × 89.443 | 178.885 |
| Term 2 | ρt × fy | 0.0025 × 60,000 | 150.0 |
| Nominal Vn | Acv × (T1+T2) / 1000 | 6336 × 328.885 / 1000 | **2,083.9 kip** |
| ϕVn (wind) | 0.75 × Vn | 0.75 × 2083.9 | **1,562.9 kip** |
| Demand | Vu | from model Parquet | ~159.5 kip |
| DCR | Vu / ϕVn | 159.5 / 1562.9 | **≈ 0.102 ✅** |

### Example B — Pier Shear Seismic: C1Y1 at L20

Same Vn = 2,083.9 kip.

| Step | Variable | Calculation | Result |
|---|---|---|---|
| ϕVn (seismic) | 0.60 × Vn | 0.60 × 2083.9 | **1,250.3 kip** |
| DCR | Vu_seismic / ϕVn | (seismic demand) / 1250.3 | higher than wind |

### Example C — Pier Axial: C1Y1 at L01

| Step | Variable | Calculation | Result |
|---|---|---|---|
| Section | lw, t | given | 42 ft, 2 ft |
| Gross area | Ag | 42 × 2 × 144 | **12,096 in²** |
| Concrete | fc' | 8.0 ksi | |
| Squash load | Po | 0.85 × 8.0 × 12096 | **82,252.8 kip** |
| Design squash | ϕPo | 0.65 × 82,252.8 | **53,464.3 kip** |
| Demand | Pu | \|P from combo\| (Parquet) | read from model |
| DCR | Pu / ϕPo | | compute from Pu |

### Example D — Lateral Displacement: H/500

| Step | Variable | Formula | Result |
|---|---|---|---|
| Building height | H_ft | max story elevation | from model |
| Limit | disp_limit_ft | H_ft / 500 | |
| Display limit | disp_limit (in) | disp_limit_ft × 12 | |
| Governing disp | δ_ft | max \|disp_{X/Y}\| at roof | from Parquet |
| DCR | δ_ft / disp_limit_ft | | ≤ 1.0 → pass |

---

*Generated from source: `crates/ext-calc/src/`*  
*Last updated: 2026-04-05*  
*Next checks planned: torsional irregularity (ASCE 7 Table 12.3-1)*
