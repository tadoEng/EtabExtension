# ext-calc Unified Specification
# Complete End-to-End Calculation & Validation Guide

**Status:** FINAL — Complete pipeline from parquet to output  
**Date:** 2026-04-12  
**Code Version:** ACI 318-14, ASCE 7-10  
**Supersedes:** EXT_CAL_SPEC_V3.md, EXT_CAL_SPEC_V4.md, EXT_CALC_SPEC.md

---

## Part 1: Overview & Architecture

### Executive Summary

This spec defines the complete **ext-calc** pipeline:
1. **Data Layer:** Load 9 parquet tables from ETABS API output
2. **Validation:** Validate config, check table existence, handle edge cases
3. **Calculation:** 7 structural checks with strict ACI 318-14 compliance
4. **Output:** Generate structured results and charts

### V3 Open Questions — All Resolved ✅

| Q# | Question | V4 Decision | User Confirmation | Impact |
|----|----------|------------|-------------------|--------|
| Q1 | Story forces chart pairing | 4 charts: VX/MY, VY/MX | ✅ Confirmed | Standard ASCE 7 output |
| Q2 | Torsional: multiple joint pairs | Per pair + governing | ✅ Confirmed | Complete torsion evaluation |
| Q3 | Torsional: multiple cases | Per case + governing | ✅ Confirmed | Worst-case ratio capture |
| Q4 | Pier shear: stress vs capacity | **Rectangular only** | ✅ **REFINED** | ACI §18.10.4.4 scope limit |
| Q5 | Wind drift: step selection | Envelope all 12 steps | ✅ Confirmed | Multi-step load envelope |

### ACI 318-14 & ASCE 7 Compliance ✅

| Check | Code Ref | Formula | Scope | Status |
|-------|----------|---------|-------|--------|
| **Story forces** | ASCE 7 §12.7.3 | VX, VY, MX, MY per story | All cases | ✅ |
| **Drift (wind)** | ASCE 7 §6.5 | Δ/h ≤ 0.0025 | X/Y independent | ✅ |
| **Drift (seismic)** | ASCE 7 §12.12 | Δ/h ≤ 0.02 | X/Y independent | ✅ |
| **Displacement (wind)** | ASCE 7 §6.5.10 | Δ ≤ H/400 | X/Y independent | ✅ |
| **Torsional** | ASCE 7 §12.3-1 | Type A >1.2, B >1.4 | **ETABS ecc included** | ✅ |
| **Pier shear stress** | ACI §18.10.4.4 | Ve/(ϕ·Acw)/√f'c ≤ 8/10 | **Rectangular only** | ✅ |
| **Pier axial** | ACI §9.3.2.2 | ϕ = 0.65, DCR = Pu/(ϕ·Po) | All sections | ✅ |

### Authoritative Configuration Template

Complete TOML configuration file (`project.ext.toml`):

```toml
[project]
name = "Sample 12-Story Building"
occupancy_category = "I/II"  # "II" or "III" (affects wind)

[extract]
# ETABS results directory containing parquet files
results_dir = "./sample_output"

[calc]
code = "ACI 318-14"  # Authoritative code version
modal_case = "Modal"  # Name of modal analysis case for participating mass
occupancy_category = "I/II"  # Matched with project level
joint_tracking_groups = ["Drift Monitoring Group"]  # Optional group names

  [calc.base-reactions]
  reactions_case = "ENV: WIND"
  
  [calc.modal]
  case_name = "Modal"
  cumulative_mass_ratio = 0.90  # Minimum 90% for seismic
  
  [calc.story-forces]
  story_force_x_cases = ["ELF_X", "W_10YRS"]
  story_force_y_cases = ["ELF_Y", "W_10YRS"]
  
  [calc.drift-wind]
  drift_x_cases = ["W_10YRS"]
  drift_y_cases = ["W_10YRS"]
  drift_limit = 0.0025  # H/400 equivalent
  
  [calc.drift-seismic]
  drift_x_cases = ["DBE_X*Cd/R", "ELF_X_Drift*Cd/Ie"]
  drift_y_cases = ["DBE_Y*Cd/R", "ELF_Y_Drift*Cd/Ie"]
  drift_limit = 0.02  # 2%
  
  [calc.displacement-wind]
  disp_x_cases = ["W_10YRS"]
  disp_y_cases = ["W_10YRS"]
  disp_limit_h = 400  # H/400
  
  [calc.torsional]
  torsional_x_case = "ELF_X"
  torsional_y_case = "ELF_Y"
  x_joints = ["J101", "J102"]  # Roof corner or center joints per direction
  y_joints = ["J101", "J103"]
  ecc_ratio = 0.05  # NOT APPLIED — ETABS steps 1/2/3 already include ±5%
  
  [calc.pier-shear-stress-wind]
  stress_cases = ["ENV: WIND"]
  phi_v = 0.75  # ACI strength reduction factor
  fc_default_ksi = 3.0  # Fallback if material not in parquet
  
  [calc.pier-shear-stress-seismic]
  stress_cases = ["ENV: DBE"]
  phi_v = 0.75
  fc_default_ksi = 3.0
  
  [calc.pier-axial-stress]
  gravity_combos = ["LC1: 1.4D"]
  wind_combos = ["LC2: 1.2D+1.6W", "LC3: 1.2D-1.6W"]
  seismic_combos = ["LC4: 1.2D+1.0E", "LC5: 0.9D+1.0E"]
  phi_axial = 0.65  # ACI §9.3.2.2
```

**Key Defaults:**
- Drift limits match ASCE 7 (wind: 0.0025, seismic: 0.02)
- Pier shear ϕ = 0.75 per ACI §18.10.4.4
- Pier axial ϕ = 0.65 per ACI §9.3.2.2
- All stress checks set `fc_default_ksi = 3.0 ksi` for missing materials
- Torsional ecc_ratio is ignored (ETABS already includes ±5%)

---

## Part 2: Data Layer

### 2.1 Parquet Tables (9 Total)

Loaded from `{results_dir}/` directory. All tables are **required** unless marked optional.

#### Table 1: story_definitions.parquet

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Story` | string | Story label / name | — | Unique identifier; used for sorting (elevation) |
| `Height` | float | Elevation above ground | ft | Used for drift limit calculation (H/400) |

**Usage:** Story ordering, elevation-based sorting, height lookup
**Validation:** At least 2 stories (roof + one below); heights should be monotonic

---

#### Table 2: joint_drifts.parquet

**Purpose:** Displacements and drifts per joint per case and step

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Story` | string | Story label | — | Join with story_definitions |
| `Label` | int | Joint label number | — | ETABS display label |
| `UniqueName` | string | Unique joint ID | — | Use as primary key for matching |
| `OutputCase` | string | Load case name | — | e.g., "ELF_X", "W_10YRS", "DBE_X" |
| `CaseType` | string | Case category | — | "LinStatic", "LinRespSpec", "Nonlinear" |
| `StepType` | string | Step classification | — | "Step By Step" (multi-step wind), "Max" (envelope), empty (static) |
| `StepNumber` | float | Step index | — | 1.0, 2.0, 3.0 (ELF with ecc), or integer steps (wind) |
| `DispX` | float | Building displacement (X) | ft | Building-frame displacement; can be negative |
| `DispY` | float | Building displacement (Y) | ft | Building-frame displacement; can be negative |
| `DriftX` | float | Story drift (X) | — | Δ/h (ratio); already computed by ETABS |
| `DriftY` | float | Story drift (Y) | — | Δ/h (ratio); already computed by ETABS |

**Usage:** Drift checks, displacement checks, torsional checks
**Key Notes:**
- Steps 1/2/3 in ELF cases already include ±5% accidental eccentricity from ETABS generation
- Wind cases have 12 steps (multi-directional envelope)
- RSA cases (DBE_X, etc.) have `StepType = "Max"` with empty `StepNumber`

---

#### Table 3: story_forces.parquet

**Purpose:** Story shear and overturning moments per level and load case

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Story` | string | Story label | — | Join with story_definitions |
| `OutputCase` | string | Load case name | — | e.g., "ELF_X", "Dead", "W_700YRS" |
| `CaseType` | string | Case category | — | "LinStatic", "LinRespSpec" |
| `StepType` | string | Step classification | — | "Step By Step", empty (static) |
| `StepNumber` | float | Step index | — | 1–12 for wind, empty for static/envelope |
| `Location` | string | Height within story | — | "Top" or "Bottom" of story |
| `P` | float | Axial force | kip | Floor/roof load (not used in ext-calc) |
| `VX` | float | Shear in X direction | kip | Base shear contributor |
| `VY` | float | Shear in Y direction | kip | Base shear contributor |
| `T` | float | Torsion (about Z) | kip·ft | Not used directly |
| `MX` | float | Overturning moment about X-axis | kip·ft | Paired with VY shear |
| `MY` | float | Overturning moment about Y-axis | kip·ft | Paired with VX shear |

**Usage:** Story forces check only
**Key Notes:**
- "Bottom" location = cumulative shear above that story (standard ETABS convention)
- Steps 1–12 for wind (12 directional envelopes)
- Static cases (gravity) have no step information

---

#### Table 4: pier_forces.parquet

**Purpose:** Section forces (shear, moment, axial) per element and case

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Story` | string | Story label | — | Join with story_definitions |
| `Pier` | string | Pier section name | — | e.g., "PX1", "PY2", "C1" |
| `OutputCase` | string | Load case or combination | — | e.g., "LC1: 1.4D", "ENV: WIND" |
| `CaseType` | string | Case type | — | "Combination", "LinStatic", "LinRespSpec" |
| `StepType` | string | Envelope type | — | "Max", "Min", "Step By Step", empty |
| `StepNumber` | float | Multi-step index | — | Integer or empty |
| `Location` | string | Section location | — | "Top" or "Bottom" of pier |
| `P` | float | Axial force | kip | Demand for axial checks |
| `V2` | float | Shear (in-plane) | kip | Demand for shear stress check; **use V2 for wall strength** |
| `V3` | float | Shear (out-of-plane) | kip | Secondary; not used in stress check |
| `T` | float | Torsion | kip·ft | Not used in stress check |
| `M2` | float | Moment (in-plane) | kip·ft | Not used in stress check |
| `M3` | float | Moment (out-of-plane) | kip·ft | Not used in stress check |

**Usage:** Pier shear stress check, pier axial check
**Key Notes:**
- V2 = in-plane shear = `shear_v2_abs_kip` in checks
- "Bottom" location typical for strength evaluation
- ENV: WIND, ENV: DBE are envelope combinations

---

#### Table 5: pier_section_properties.parquet

**Purpose:** Geometric and material props per pier per story

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Story` | string | Story label | — | Join with story_definitions |
| `Pier` | string | Pier section name | — | e.g., "PX1", "PY2" |
| `AxisAngle` | float | Section orientation | degrees | 0°/180° = X-wall, 90°/270° = Y-wall |
| `NumAreaObj` | int | Count of area objects | — | Not used in ext-calc |
| `NumLineObj` | int | Count of line objects | — | Not used in ext-calc |
| `WidthBot` | float | Width at bottom | ft | For rectangular: plan dimension perpendicular to wall |
| `ThickBot` | float | Thickness at bottom | ft | For rectangular: wall thickness; Acw = WidthBot × ThickBot |
| `WidthTop` | float | Width at top | ft | For rectangular; typically equals WidthBot |
| `ThickTop` | float | Thickness at top | ft | For rectangular; typically equals ThickBot |
| `Material` | string | Material label | — | e.g., "6000Psi (Exp)", "3000Psi" |
| `CGBotX` | float | Center of gravity X (bottom) | ft | Unused in ext-calc; for reference |
| `CGBotY` | float | Center of gravity Y (bottom) | ft | Unused; for reference |
| `CGBotZ` | float | Elevation Z (bottom) | ft | Unused; story elevation from story_definitions |
| `CGTopX` | float | Center of gravity X (top) | ft | Unused |
| `CGTopY` | float | Center of gravity Y (top) | ft | Unused |
| `CGTopZ` | float | Elevation Z (top) | ft | Unused |

**Usage:** Pier shear stress check (Acw computation), pier axial check
**Key Notes:**
- **Rectangular identification:** `WidthBot > 0 && ThickBot > 0` (always true in ETABS pier tables)
- Non-rectangular piers (if present) should have `Width = diameter, Thickness = 0` or different representation

---

#### Table 6: material_properties_concrete_data.parquet

**Purpose:** Concrete strength per material name

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Material` | string | Material label | — | Join with pier_section_properties.Material |
| `fc` | float | Compressive strength | ksi | e.g., 6.0, 3.0 |

**Usage:** Strength lookup for pier shear stress, axial capacity
**Validation:** Material referenced in pier_section_properties must exist here

---

#### Table 7: base_reactions.parquet

**Purpose:** Base shear and reactions envelope

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Case` | string | Load case | — | e.g., "ELF_X", "ELF_Y", "DBE_X" |
| `VX` | float | Base shear (X direction) | kip | Used for base shear check |
| `VY` | float | Base shear (Y direction) | kip | Used for base shear check |
| `MX` | float | Overturning moment (about X) | kip·ft | Unused |
| `MY` | float | Overturning moment (about Y) | kip·ft | Unused |

**Usage:** Base reactions / base shear check
**Validation:** Cases specified in config must exist

---

#### Table 8: modal_participating_mass_ratios.parquet

**Purpose:** Modal analysis summary

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `Mode` | int | Mode number | — | Sequential 1, 2, 3, ... |
| `Case` | string | Modal case name | — | e.g., "Modal (Rizt)" |
| `Period` | float | Natural period | sec | Not used in ext-calc |
| `SumX` | float | Cumulative participation X | % | Used for modal check |
| `SumY` | float | Cumulative participation Y | % | Used for modal check |

**Usage:** Modal sufficiency check (min mass participation)
**Validation:** Configured modal case must exist; SumX, SumY ≥ min-mass-participation

---

#### Table 9: group_assignments.parquet (Optional)

**Purpose:** Joint grouping for drift monitoring (if used)

| Column | Type | Description | Units | Notes |
|--------|------|-------------|-------|-------|
| `GroupName` | string | Group label | — | e.g., "Zone A", "Central Core" |
| `JointUniqueName` | string | Joint ID | — | Foreign key to joint_drifts.UniqueName |

**Usage:** Optional grouping for drift checks (if configured)

---

### 2.2 Data Loading Pattern

```rust
pub fn load_all_tables(results_dir: &Path) -> Result<AllTables> {
    Ok(AllTables {
        story_definitions: load_parquet::<StoryDefRow>(results_dir.join("story_definitions.parquet"))?,
        joint_drifts: load_parquet::<JointDriftRow>(results_dir.join("joint_drifts.parquet"))?,
        story_forces: load_parquet::<StoryForceRow>(results_dir.join("story_forces.parquet"))?,
        pier_forces: load_parquet::<PierForceRow>(results_dir.join("pier_forces.parquet"))?,
        pier_sections: load_parquet::<PierSectionRow>(results_dir.join("pier_section_properties.parquet"))?,
        material_props: load_parquet::<MaterialPropRow>(results_dir.join("material_properties_concrete_data.parquet"))?,
        base_reactions: load_parquet::<BaseReactionRow>(results_dir.join("base_reactions.parquet"))?,
        modal_mass: load_parquet::<ModalMassRow>(results_dir.join("modal_participating_mass_ratios.parquet"))?,
        group_assignments: load_parquet_optional::<GroupAssignmentRow>(results_dir.join("group_assignments.parquet"))?,
    })
}

fn load_parquet<T: serde::de::DeserializeOwned>(path: PathBuf) -> Result<Vec<T>> {
    if !path.exists() {
        bail!("required parquet file not found: {:?}", path);
    }
    // Use polars/parquet crate to deserialize
    Ok(...)
}

fn load_parquet_optional<T: serde::de::DeserializeOwned>(path: PathBuf) -> Result<Option<Vec<T>>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(...))
}
```

---

### 2.3 Validation Rules

All validation is **synchronous** and returns structured errors. No check silently swallows a config error.

#### 2.3.1 Config Validation

```
Rule: Required config fields must not be empty

For each check:
  If [calc.{check}] section with non-empty config found:
    Validate all required fields present and > 0 if numeric
    Example: [calc.drift-wind].drift-limit > 0
  Else:
    Check is disabled (optional = None in CodeParams)

Rule: Referenced cases must exist in parquet

For each case in config (e.g., drift-x-cases = ["W_10YRS"]):
  Search joint_drifts, story_forces, or pier_forces for OutputCase == "W_10YRS"
  If not found:
    Error: "Configured case 'W_10YRS' not found in [table]; configure existing cases from ETABS"

Rule: Joint tracking groups must exist

For each group in config (e.g., joint-tracking-groups = ["Joint47"]):
  Search joint_drifts for UniqueName == "Joint47"
  If not found:
    Warning: "Configured joint 'Joint47' not found; check spelling"

Rule: Material names must be in material_properties table

For each (pier, story):
  material_name = pier_section_properties[pier, story].Material
  If material_name NOT in material_properties.Material:
    Warning: "Material '{material_name}' not found for {pier} at {story}; using fc_default"
```

#### 2.3.2 Data Quality Checks

```
Rule: Story elevations must be monotonic
  Validation: story_definitions.Height should be strictly increasing or decreasing
  Violation: Warning (sort anyway)

Rule: Modal mass participation must reach threshold
  Validation: max(modal_mass.SumX, modal_mass.SumY) ≥ config.min-mass-participation
  Violation: Error (cannot evaluate seismic without sufficient modal mass)

Rule: Base shear must not be zero
  Validation: For each case in base_reactions: VX ≠ 0 or VY ≠ 0
  Violation: Error at check time

Rule: Pier sections must have positive area
  Validation: pier_section_properties.WidthBot > 0 && ThickBot > 0
  Violation: Warning, skip pier (already rectangular in ETABS)

Rule: No duplicate (story, pier, case) in pier_forces
  Validation: Group by (story, pier, case) should have 1 or 2 rows (Top + Bottom)
  Violation: Warning, use deterministic combination (e.g., max absolute force)

Rule: Step numbers in joint_drifts must be consistent per case type
  Validation: ELF cases should have steps [1, 2, 3]; Wind should have [1..12]
  Violation: Warning, process available steps
```

---

### 2.4 Edge Case Handling

#### Missing Load Case in ETABS

**Scenario:** Config specifies `drift-x-cases = ["W_10YRS"]` but ETABS did not run case "W_10YRS"

**Location:** Where to warn user?
- **Best:** During config validation in `CodeParams::from_config()` — catch early, inform before checks run
- **Secondary:** At check time, skip with detailed warning

**Implementation:**
```rust
// In from_config():
for case in &config.calc.drift_wind.drift_x_cases {
    if !joint_drifts_table.iter().any(|r| r.output_case == *case) {
        eprintln!("[ext-calc] warn: configured case '{}' not found in joint_drifts", case);
        eprintln!("[ext-calc] warn: run this case in ETABS or remove from config");
    }
}
```

#### Non-Rectangular Piers in Stress Check

**Scenario:** Circular column in pier_section_properties with `WidthBot = diameter, ThickBot = 0`

**Current Assumption:** "ETABS table always reports rectangular" — but safeguard needed

**Implementation:**
```rust
fn is_rectangular_pier(section: &PierSectionRow) -> bool {
    section.width_bottom_ft > 0.0 && section.thickness_bottom_ft > 0.0
}

// In stress check:
if !is_rectangular_pier(&section) {
    log::warn!("Skipping non-rectangular pier {} from ACI §18.10.4.4 stress check", pier);
    continue;  // Do not include in per_pier results
}
```

#### Missing Section Properties

**Scenario:** Pier "PX1" in pier_forces but not in pier_section_properties at story "L36"

**Implementation:**
```rust
for (story, pier, ve) in grouped_forces {
    let section = match section_map.get(&(pier.clone(), story.clone())) {
        Some(s) => s,
        None => {
            log::warn!("[ext-calc] warn: no section props for pier '{}' at story '{}'; skipped", pier, story);
            continue;
        }
    };
    // ... process
}
```

#### Zero Drift (Delta Average → Divide by Zero)

**Scenario:** Torsional check: `delta_avg[step] ≈ 0` → ratio = delta_max / 0

**Implementation:**
```rust
let ratio = if delta_avg[step] < 1e-9 {
    1.0  // Fallback: no irregularity if near-zero drift
} else {
    delta_max[step] / delta_avg[step]
};
```

---

## Part 3: Structural Checks (7 Total)

Each check follows this structure:
1. **Config section** — TOML configuration
2. **Data extraction** — Which tables, filtering, grouping
3. **Validation** — Pre-check validation rules
4. **Algorithm** — Step-by-step calculation
5. **Unit handling** — Via UnitContext (NO hardcoded multipliers)
6. **Output** — Rust struct serialized to JSON
7. **Report** — Chart and table generation

---

### Check 1: Story Forces

**Code Ref:** ASCE 7 §12.7.3 — Story shear and overturning moment

#### Config

```toml
[calc.story-forces]
story-force-x-cases = ["ELF_X", "DBE_X", "MCER_X", "W_700YRS"]
story-force-y-cases = ["ELF_Y", "DBE_Y", "MCER_Y", "W_700YRS"]
```

**Validation:**
- Both lists non-empty OR both list empty (symmetric)
- Each case must exist in story_forces.parquet

#### Data Extraction

```
Source: story_forces.parquet
Filter: 
  - OutputCase in story-force-x-cases OR story-force-y-cases
  - Location == "Bottom"  (cumulative shear above story level)
Group by: (Story, OutputCase)
Select columns: Story, OutputCase, VX, VY, MX, MY, StepNumber
```

#### Calculation

```
For X-direction:
  Per story:
    max_vx = MAX(|VX|) across all cases and all steps
    max_my = MAX(|MY|) across all cases and all steps
    (MY = overturning moment about Y-axis, paired with X-direction shear)

For Y-direction:
  Per story:
    max_vy = MAX(|VY|) across all cases and all steps
    max_mx = MAX(|MX|) across all cases and all steps
    (MX = overturning moment about X-axis, paired with Y-direction shear)

Sort by story elevation: highest first (for chart top-down orientation)
```

#### Output

```rust
#[derive(Serialize)]
pub struct StoryForceEnvelopeRow {
    pub story: String,
    pub max_vx_kip: f64,
    pub max_my_kip_ft: f64,
    pub max_vy_kip: f64,
    pub max_mx_kip_ft: f64,
}

#[derive(Serialize)]
pub struct StoryForcesOutput {
    pub rows: Vec<StoryForceEnvelopeRow>,
    pub pass: bool,  // Always true (no code limit)
}
```

#### Report

- **Chart 1:** Max VX per story (X-direction shear envelope)
- **Chart 2:** Max MY per story (overturning moment paired with X shear)
- **Chart 3:** Max VY per story (Y-direction shear envelope)
- **Chart 4:** Max MX per story (overturning moment paired with Y shear)

---

### Check 2: Drift Wind (X/Y Independent)

**Code Ref:** ASCE 7 §6.5.10 — Wind drift and deflection  
**Limit:** 0.0025 (H/400)

#### Config

```toml
[calc.drift-wind]
drift-x-cases = ["W_10YRS"]
drift-y-cases = ["W_10YRS"]
drift-limit = 0.0025
```

#### Data Extraction

```
Source: joint_drifts.parquet
Filter:
  - OutputCase in drift-x-cases (for X) or drift-y-cases (for Y)
  - UniqueName in joint-tracking-groups (from config)
Group by: (Story, Group, OutputCase, StepNumber)
Select: Story, UniqueName, DriftX, DriftY, DispX, DispY, StepNumber
```

#### Calculation

```
Per direction (X or Y):
  For each group:
    For each case:
      Envelope all steps 1..12:
        drift_x_max = MAX(|DriftX|) across all steps
        drift_x_min = MIN(DriftX) across all steps
        (similar for drift_y_pos, drift_y_neg, disp_x, disp_y)

Governing selection:
  For X-direction: Pick story+group+case with MAX(drift_x) from [drift_x_pos, drift_x_neg]
  For Y-direction: Pick story+group+case with MAX(drift_y) from [drift_y_pos, drift_y_neg]
  (Independent governing per direction)

Pass/fail:
  Pass if drift_x ≤ drift_limit AND drift_y ≤ drift_limit
```

#### Unit Handling

All drift values from parquet are **ratios** (Δ/h), dimensionless. No conversion needed.

#### Output

```rust
pub struct DriftWindOutput {
    pub x: DriftOutput,  // Governed on DriftX columns only
    pub y: DriftOutput,  // Governed on DriftY columns only
}

pub struct DriftOutput {
    pub rows: Vec<DriftEnvelopeRow>,
    pub governing: DriftEnvelopeRow,
    pub drift_limit: f64,
    pub pass: bool,
}
```

---

### Check 3: Drift Seismic (X/Y Independent)

**Code Ref:** ASCE 7 Table 12.12-1 — Seismic drift limit  
**Limit:** 0.02 (2%)

#### Config

```toml
[calc.drift-seismic]
drift-x-cases = ["DBE_X*Cd/R", "ELF_X_Drift*Cd/Ie"]
drift-y-cases = ["DBE_Y*Cd/R", "ELF_Y_Drift*Cd/Ie"]
drift-limit = 0.02
```

**Note:** Case names with `*Cd/R` are metadata hints — the actual factors are applied by ETABS during case generation.

#### Algorithm

Same as drift wind, but with seismic cases and higher limit.

---

### Check 4: Displacement Wind (X/Y Independent)

**Code Ref:** ASCE 7 §6.5.10 — Lateral deflection  
**Limit:** `disp_limit_h` (e.g., 400 ft → H/400)

#### Config

```toml
[calc.displacement-wind]
disp-x-cases = ["W_10YRS"]
disp-y-cases = ["W_10YRS"]
disp-limit-h = 400
```

#### Data Extraction

```
Source: joint_drifts.parquet
Select: DispX [ft], DispY [ft]
(absolute maximum displacement per joint per case)
```

#### Unit Handling

- **DispX, DispY:** Input in feet (from ETABS)
- **Limit:** disp_limit_h in feet (config)
- **Comparison:** |DispX| ≤ disp_limit_h, |DispY| ≤ disp_limit_h

---

### Check 5: Torsional Irregularity

**Code Ref:** ASCE 7 Table 12.3-1 via ACI Chapter 12

#### ✅ CRITICAL: ETABS Eccentricity Already Included

ETABS ELF generation includes ±5% accidental eccentricity in steps 2 and 3 **in the parquet output**.
- Step 1 = nominal (ecc = 0)
- Step 2 = +5% eccentricity already applied
- Step 3 = -5% eccentricity already applied

**No manual eccentricity application needed.** Read 3 steps as-is.

#### Config

```toml
[calc.torsional]
torsional-x-case = ["ELF_X", "DBE_X"]
torsional-y-case = ["ELF_Y", "DBE_Y"]
x-joints = [["Joint47", "Joint50"]]
y-joints = [["Joint49", "Joint51"]]
ecc-ratio = 0.05
building-dim-x = 96.0
building-dim-y = 56.0
```

#### Data Extraction

```
Source: joint_drifts.parquet
Filter:
  - OutputCase in torsional-x-case (or torsional-y-case)
  - UniqueName in [joint_a, joint_b] for each pair
Group by: (Story, OutputCase, StepNumber)
Select: Story, UniqueName, DispX, DispY, StepNumber

Steps:
  - ELF cases: steps 1.0, 2.0, 3.0
  - RSA (DBE) cases: may have step_number = null (envelope)
    → Treat as single Step 1.0
```

#### Calculation

```
Per direction (X or Y):
  For each case:
    For each joint pair (joint_a, joint_b):
      For each story:
        
        // Collect displacements per step
        For step in [1.0, 2.0, 3.0]:
          disp_a[step] = DispX (or DispY) of joint_a at step  [ft]
          disp_b[step] = DispX (or DispY) of joint_b at step  [ft]
        
        // Story drift = this story disp − next story disp
        For step in [1.0, 2.0, 3.0]:
          drift_a[step] = |disp_a[step] − disp_a_next_story[step]|  [ft]
          drift_b[step] = |disp_b[step] − disp_b_next_story[step]|  [ft]
          
          delta_max[step] = MAX(drift_a[step], drift_b[step])
          delta_avg[step] = AVG(drift_a[step], drift_b[step])
        
        // RATIO: Worst case across 3 steps
        ratio = MAX(
          delta_max[1]/delta_avg[1],
          delta_max[2]/delta_avg[2],
          delta_max[3]/delta_avg[3]
        )
        
        (Handle divide-by-zero: if delta_avg < 1e-9, set ratio = 1.0)
        
        // AX: Amplification factor for dynamic analysis
        max_sq = MAX(
          (delta_max[1]/(1.2·delta_avg[1]))²,
          (delta_max[2]/(1.2·delta_avg[2]))²,
          (delta_max[3]/(1.2·delta_avg[3]))²
        )
        ax = CLAMP(max_sq, 1.0, 3.0)
        
        // Type classification (ASCE 7 Table 12.3-1)
        is_type_a = ratio > 1.2
        is_type_b = ratio > 1.4
        
        // Redundancy (ASCE 7 §12.3.4)
        rho = (ratio > 1.4) ? 1.3 : 1.0
        
        // Eccentricity (optional output)
        ecc_ft = ecc_ratio × building_dim  (e.g., 0.05 × 96 = 4.8 ft)

Output row: {story, case, joint_a, joint_b, ratio, ax, rho, ecc_ft, is_type_a, is_type_b}

Overall pass = !x.has_type_b && !y.has_type_b
```

#### Output

```rust
pub struct TorsionalRow {
    pub story: String,
    pub case: String,
    pub joint_a: String,
    pub joint_b: String,
    pub ratio: f64,
    pub ax: f64,
    pub rho: f64,
    pub ecc_ft: f64,
    pub is_type_a: bool,
    pub is_type_b: bool,
}

pub struct TorsionalOutput {
    pub x: TorsionalDirectionOutput,
    pub y: TorsionalDirectionOutput,
    pub pass: bool,  // !x.has_type_b && !y.has_type_b
}
```

---

### Check 6: Pier Shear Stress (Rectangular Piers ONLY)

**Code Ref:** ACI §18.10.4.4 — Shear strength of structural walls

#### ✅ CRITICAL: Rectangular Pier Scope

ACI §18.10.4.4 applies to **rectangular wall sections ONLY**.
- **Non-rectangular** (circular, T-section, etc.) are **EXCLUDED** with warning
- **Identification:** rectangular = `WidthBot > 0 && ThickBot > 0`

#### Config

```toml
[calc.pier-shear-stress-wind]
stress-combos = ["ENV: WIND"]
phi-v = 0.75
fc-default-ksi = 8.0

[calc.pier-shear-stress-seismic]
stress-combos = ["ENV: DBE"]
phi-v = 0.75
fc-default-ksi = 8.0
```

#### Data Extraction

```
Source: pier_forces.parquet
Filter:
  - OutputCase in stress-combos
  - Location = "Bottom" (typical for strength evaluation)
Group by: (Story, Pier)
Select: V2 [kip]  (in-plane shear, aka shear_v2_abs_kip)

Source: pier_section_properties.parquet
Select: WidthBot [ft], ThickBot [ft], AxisAngle [degrees]

Source: material_properties_concrete_data.parquet / fc_map
Select: fc [ksi]
```

#### Calculation

```
Step 1: Envelope Ve per (story, pier)
  For each (story, pier):
    Ve = MAX(|V2|) across all combos  [kip]

Step 2: Build lookups
  section_map[(pier, story)] = WidthBot, ThickBot, AxisAngle
  fc_map[(pier, story)] = fc [ksi]
  (fallback to fc_default if not in materials table)

Step 3: Rectangular filter & stress calculation
  For each (story, pier, Ve):
    
    section = section_map[(pier, story)]
    
    // RECTANGULAR FILTER: Skip non-rectangular
    if !(section.width_bot > 0 && section.thick_bot > 0) {
      log::warn!("Skipping non-rectangular pier {}", pier);
      continue;
    }
    
    // All conversions via UnitContext — NO *1000, *144
    fc_ksi = fc_map[(pier, story)]
    fc_psi = unit_context.convert(fc_ksi, "ksi", "psi")   // VIA UNITCONTEXT
    sqrt_fc = fc_psi.sqrt()
    
    acw_in2 = section.width_bot_ft × section.thick_bot_ft × 144.0  // NO HARDCODING
           -> use: acw_in2 = unit_context.area_ft_to_in2(width_ft, thick_ft)
    
    // Identify wall direction from AxisAngle
    wall_direction = if section.axis_angle < 15.0 || section.axis_angle > 165.0 {
        "X"
    } else if (section.axis_angle - 90).abs() < 15.0 || (section.axis_angle - 270).abs() < 15.0 {
        "Y"
    } else {
        log::warn!("Pier {} has ambiguous AxisAngle {}°", pier, section.axis_angle);
        continue;
    };
    
    // Stress ratio = Ve / (ϕ × Acw) / √f'c
    stress_psi = unit_context.convert_force_over_area(
        ve_kip,
        &acw_in2,
        phi_v,
    );
    
    stress_ratio = stress_psi / sqrt_fc  [n × √f'c, dimensionless]
    
    pass = stress_ratio <= 8.0
    
    Push PierShearStressRow

Step 4: Average per direction per story
  Group per_pier rows by (wall_direction, story)
  For each group:
    sum_ve = SUM(ve_kip) for all piers in group
    sum_acw = SUM(acw_in2) for all piers in group
    fc_psi = fc_psi of first pier
    sqrt_fc = fc_psi.sqrt()
    
    avg_stress = unit_context.convert_force_over_area(
        sum_ve,
        &sum_acw,
        phi_v,
    )
    avg_ratio = avg_stress / sqrt_fc
    pass = avg_ratio <= 10.0

Overall pass = ALL(per_pier.pass) && ALL(x_avg.pass) && ALL(y_avg.pass)
```

#### Unit Conversion (VIA UNITCONTEXT ONLY)

```rust
// ❌ FORBIDDEN:
stress = ve_kip * 1000.0 / (phi_v * acw_in2);

// ✅ REQUIRED:
stress = unit_context.convert_force_over_area(ve_kip, &acw_in2, phi_v);
```

#### Output

```rust
pub struct PierShearStressRow {
    pub story: String,
    pub pier: String,
    pub combo: String,
    pub wall_direction: String,
    pub acw_in2: f64,
    pub fc_psi: f64,
    pub sqrt_fc: f64,
    pub ve_kip: f64,
    pub stress_psi: f64,
    pub stress_ratio: f64,
    pub limit_individual: f64,  // 8.0
    pub pass: bool,
}

pub struct PierShearStressOutput {
    pub phi_v: f64,
    pub per_pier: Vec<PierShearStressRow>,       // RECTANGULAR ONLY
    pub x_average: Vec<PierShearStressAverageRow>,
    pub y_average: Vec<PierShearStressAverageRow>,
    pub max_individual_ratio: f64,
    pub max_average_ratio: f64,
    pub pass: bool,
}
```

#### Report Summary

- **Non-rectangular piers excluded:** Count and list of skipped piers
- **Rectangular piers only:** Per-pier stress profile + group average lines
- **Limits:** Individual =  8.0, Average = 10.0 (per √f'c)

---

### Check 7: Pier Axial Stress (3 Categories)

**Code Ref:** ACI §9.3.2.2 — Axial compression, ϕ = 0.65

#### Config

```toml
[calc.pier-axial-stress]
stress-gravity-combos = ["LC1: 1.4D", "LC2: 1.2D+1.6L"]
stress-wind-combos = ["LC3.1: 1.2D+0.5W", ...]
stress-seismic-combos = ["DBE1: ...", ...]
phi-axial = 0.65
```

#### Data Extraction

```
Source: pier_forces.parquet
Filter: OutputCase in any of the three combo lists
Group by: (Category, Story, Pier, OutputCase)
Select: P [kip]  (axial demand)

Source: pier_section_properties.parquet
Select: WidthBot, ThickBot (for area Ag)
```

#### Calculation

```
For each category (gravity, wind, seismic):
  
  For each (story, pier, case, P):
    ag = width_bot_ft × thick_bot_ft  [ft²]  (convert via UnitContext)
    
    po = phi_axial × 0.85 × fc_psi × ag  [kip]
    dcr = P / po  [dimensionless]
    
    Push PierAxialResult {story, pier, case, p, po, dcr, pass: dcr ≤ 1.0}

Select governing:
  gravity_governing = max DCR from gravity_combos
  wind_governing = max DCR from wind_combos
  seismic_governing = max DCR from seismic_combos
  overall_governing = max DCR across all three

Pass: all DCR ≤ 1.0
```

#### Output

```rust
pub struct PierAxialStressOutput {
    pub phi_axial: f64,
    pub piers: Vec<PierAxialResult>,
    pub governing_gravity: Option<PierAxialResult>,
    pub governing_wind: Option<PierAxialResult>,
    pub governing_seismic: Option<PierAxialResult>,
    pub governing: PierAxialResult,
    pub pass: bool,
}
```

---

## Part 4: Error Handling & Reporting

### 4.1 Error Categories

#### Fatal Errors (Check Aborted)

| Error | Message | Action |
|-------|---------|--------|
| Parquet file missing | "required parquet file not found: story_definitions.parquet" | Exit, inform user to check ETABS export |
| Config case not in data | "configured case 'W_10YRS' not found in joint_drifts" | Exit, list available cases |
| Modal mass insufficient | "modal participation X=45% < required 90%" | Exit, run more modes |
| Zero base shear | "ELF base shear for case 'ELF_X' is zero" | Exit, check load cases |

#### Non-Fatal Warnings (Check Continues)

| Warning | Message | Impact |
|---------|---------|--------|
| Material not found | "material 'XXX' not found for pier PX1; using fc_default=8.0 ksi" | Uses fallback |
| Section props missing | "no section properties for pier PX1 at L36; row skipped" | Row excluded |
| Non-rectangular pier | "skipping non-rectangular pier C5 from stress check" | Row excluded |
| Joint not found | "configured joint J100 not found; check spelling" | Group incomplete |

---

### 4.2 Summary Status Levels

```rust
pub enum CheckStatus {
    Pass,        // All criteria met
    Warn,        // Type A only (torsional)
    Fail,        // Code violation
    Unrated,     // No data / not configured
}

pub struct CalcSummary {
    pub story_forces: CheckStatus,      // Always Pass (no limits)
    pub drift_wind_x: CheckStatus,
    pub drift_wind_y: CheckStatus,
    pub drift_seismic_x: CheckStatus,
    pub drift_seismic_y: CheckStatus,
    pub displacement_wind_x: CheckStatus,
    pub displacement_wind_y: CheckStatus,
    pub torsional_x: CheckStatus,       // Warn if Type A, Fail if Type B
    pub torsional_y: CheckStatus,
    pub pier_shear_stress_wind: CheckStatus,
    pub pier_shear_stress_seismic: CheckStatus,
    pub pier_axial_stress: CheckStatus,
}
```

---

## Part 5: Implementation Details

### 5.1 ext-db Config Structs

Located in: `crates/ext-db/src/config/calc.rs`

#### Main Config

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalcConfig {
    pub code: Option<String>,  // e.g., "ACI 318-14"
    pub occupancy_category: Option<String>,  // "I/II" or "III"
    pub modal_case: Option<String>,  // Modal case name
    
    #[serde(default)]
    pub joint_tracking_groups: Vec<String>,  // Group names for drift
    
    #[serde(default)]
    pub modal: ModalCalcConfig,
    
    #[serde(rename = "base-reactions", default)]
    pub base_reactions: BaseReactionsCalcConfig,
    
    #[serde(rename = "story-forces", default)]
    pub story_forces: StoryForcesCalcConfig,
    
    #[serde(rename = "drift-wind", default)]
    pub drift_wind: DriftDirectionalCalcConfig,
    
    #[serde(rename = "drift-seismic", default)]
    pub drift_seismic: DriftDirectionalCalcConfig,
    
    #[serde(rename = "displacement-wind", default)]
    pub displacement_wind: DisplacementDirectionalCalcConfig,
    
    #[serde(rename = "torsional", default)]
    pub torsional: TorsionalCalcConfig,
    
    #[serde(rename = "pier-shear-stress-wind", default)]
    pub pier_shear_stress_wind: PierShearStressCalcConfig,
    
    #[serde(rename = "pier-shear-stress-seismic", default)]
    pub pier_shear_stress_seismic: PierShearStressCalcConfig,
    
    #[serde(rename = "pier-axial-stress", default)]
    pub pier_axial_stress: PierAxialStressCalcConfig,
}

impl CalcConfig {
    pub fn is_configured(&self) -> bool {
        self.base_reactions.is_configured() 
            || self.story_forces.is_configured()
            || self.drift_wind.is_configured()
            || self.drift_seismic.is_configured()
            || self.displacement_wind.is_configured()
            || self.torsional.is_configured()
            || self.pier_shear_stress_wind.is_configured()
            || self.pier_shear_stress_seismic.is_configured()
            || self.pier_axial_stress.is_configured()
    }
}

#### Individual Config Types

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModalCalcConfig {
    pub case_name: Option<String>,
    pub cumulative_mass_ratio: Option<f64>,  // e.g., 0.90
}

impl ModalCalcConfig {
    pub fn is_configured(&self) -> bool {
        self.case_name.is_some()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaseReactionsCalcConfig {
    pub reactions_case: Option<String>,  // "ENV: WIND"
}

impl BaseReactionsCalcConfig {
    pub fn is_configured(&self) -> bool {
        self.reactions_case.is_some()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoryForcesCalcConfig {
    pub story_force_x_cases: Vec<String>,
    pub story_force_y_cases: Vec<String>,
}

impl StoryForcesCalcConfig {
    pub fn is_configured(&self) -> bool {
        !self.story_force_x_cases.is_empty() || !self.story_force_y_cases.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DriftDirectionalCalcConfig {
    pub drift_x_cases: Vec<String>,
    pub drift_y_cases: Vec<String>,
    pub drift_limit: Option<f64>,  // as ratio, e.g., 0.0025
}

impl DriftDirectionalCalcConfig {
    pub fn is_configured(&self) -> bool {
        !self.drift_x_cases.is_empty() || !self.drift_y_cases.is_empty()
    }
    
    pub fn drift_limit_or_default(&self) -> f64 {
        self.drift_limit.unwrap_or(0.0025)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DisplacementDirectionalCalcConfig {
    pub disp_x_cases: Vec<String>,
    pub disp_y_cases: Vec<String>,
    pub disp_limit_h: Option<f64>,  // in feet, e.g., 400 for H/400
}

impl DisplacementDirectionalCalcConfig {
    pub fn is_configured(&self) -> bool {
        !self.disp_x_cases.is_empty() || !self.disp_y_cases.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TorsionalCalcConfig {
    pub torsional_x_case: Option<String>,
    pub torsional_y_case: Option<String>,
    pub x_joints: Vec<String>,  // Joint unique names for X-case
    pub y_joints: Vec<String>,  // Joint unique names for Y-case
    pub ecc_ratio: Option<f64>,  // Ignored; ETABS steps 1/2/3 already include ±5%
}

impl TorsionalCalcConfig {
    pub fn is_configured(&self) -> bool {
        self.torsional_x_case.is_some() || self.torsional_y_case.is_some()
    }
    
    pub fn ecc_ratio_ignored_note(&self) -> &'static str {
        "ETABS parquet steps 1/2/3 already include ±5% accidental ecc; ecc_ratio field is ignored"
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierShearStressCalcConfig {
    pub stress_cases: Vec<String>,  // Load combination names
    pub phi_v: Option<f64>,  // Strength reduction factor; default 0.75
    pub fc_default_ksi: Option<f64>,  // Fallback concrete strength in ksi
}

impl PierShearStressCalcConfig {
    pub fn is_configured(&self) -> bool {
        !self.stress_cases.is_empty()
    }
    
    pub fn phi_v_or_default(&self) -> f64 {
        self.phi_v.unwrap_or(0.75)
    }
    
    pub fn fc_default_or_fallback(&self) -> f64 {
        self.fc_default_ksi.unwrap_or(3.0)  // 3.0 ksi default
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierAxialStressCalcConfig {
    pub gravity_combos: Vec<String>,   // e.g., ["LC1: 1.4D"]
    pub wind_combos: Vec<String>,      // e.g., ["LC2: 1.2D+1.6W"]
    pub seismic_combos: Vec<String>,   // e.g., ["LC4: 1.2D+1.0E"]
    pub phi_axial: Option<f64>,  // Default 0.65 per ACI §9.3.2.2
}

impl PierAxialStressCalcConfig {
    pub fn is_configured(&self) -> bool {
        !self.gravity_combos.is_empty() || !self.wind_combos.is_empty() || !self.seismic_combos.is_empty()
    }
    
    pub fn phi_axial_or_default(&self) -> f64 {
        self.phi_axial.unwrap_or(0.65)
    }
    
    pub fn all_combos(&self) -> Vec<&String> {
        let mut combos = Vec::new();
        combos.extend(&self.gravity_combos);
        combos.extend(&self.wind_combos);
        combos.extend(&self.seismic_combos);
        combos
    }
}
```

### 5.2 ext-calc Output Types

Located in: `crates/ext-calc/src/output.rs`

#### Root Output Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcOutput {
    pub meta: CalcMeta,
    pub summary: CalcSummary,
    pub modal: Option<ModalOutput>,
    pub base_reactions: Option<BaseReactionsOutput>,
    pub story_forces: Option<StoryForcesOutput>,
    pub drift_wind: Option<DriftWindOutput>,
    pub drift_seismic: Option<DriftSeismicOutput>,
    pub displacement_wind: Option<DisplacementWindOutput>,
    pub torsional: Option<TorsionalOutput>,
    pub pier_shear_stress_wind: Option<PierShearStressOutput>,
    pub pier_shear_stress_seismic: Option<PierShearStressOutput>,
    pub pier_axial_stress: Option<PierAxialStressOutput>,
}

pub struct CalcMeta {
    pub timestamp: String,  // ISO 8601
    pub code: String,  // "ACI 318-14"
    pub version: String,  // semantic version of ext-calc
    pub results_dir: String,  // path to ETABS parquet output
}

pub struct CalcSummary {
    pub checks_run: usize,  // Number of checks executed
    pub overall_pass: bool,  // All checks passed
    pub pass_count: usize,
    pub fail_count: usize,
    pub warn_count: usize,
    pub unrated_count: usize,
    pub messages: Vec<String>,  // Non-fatal warnings
}
```

#### Story Forces Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryForcesOutput {
    pub rows: Vec<StoryForceEnvelopeRow>,  // One per story+case
    pub max_vx_kip: f64,  // Governing shear X
    pub max_vy_kip: f64,  // Governing shear Y
    pub max_mx_kip_ft: f64,  // Governing moment X
    pub max_my_kip_ft: f64,  // Governing moment Y
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryForceEnvelopeRow {
    pub story: String,
    pub output_case: String,
    pub location: String,  // "Top" or "Bottom"
    pub vx_kip: f64,
    pub vy_kip: f64,
    pub mx_kip_ft: f64,
    pub my_kip_ft: f64,
}
```

#### Drift Output (Wind & Seismic)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftWindOutput {
    pub x: DriftOutput,  // X-direction drift
    pub y: DriftOutput,  // Y-direction drift
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSeismicOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftOutput {
    pub rows: Vec<DriftEnvelopeRow>,
    pub governing: Option<DriftEnvelopeRow>,
    pub drift_limit: f64,  // As ratio, e.g., 0.0025
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEnvelopeRow {
    pub story: String,
    pub group_name: String,  // Joint group or "all"
    pub output_case: String,
    pub max_drift: f64,  // Δ/h (ratio)
    pub drift_ratio: f64,  // Δ/h (dimensionless)
}
```

#### Displacement Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplacementWindOutput {
    pub x: DisplacementOutput,
    pub y: DisplacementOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplacementOutput {
    pub rows: Vec<DisplacementRow>,
    pub governing: Option<DisplacementRow>,
    pub disp_limit_ft: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplacementRow {
    pub story: String,
    pub group_name: String,
    pub output_case: String,
    pub max_disp_ft: f64,
    pub h_limit_ft: f64,
}
```

#### Torsional Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorsionalOutput {
    pub x: TorsionalDirectionOutput,
    pub y: TorsionalDirectionOutput,
    pub pass: bool,  // Pass if no Type B found
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorsionalDirectionOutput {
    pub direction: String,  // "X" or "Y"
    pub case_name: String,  // e.g., "ELF_X"
    pub rows: Vec<TorsionalRow>,  // One per joint pair per story
    pub governing: Option<TorsionalRow>,
    pub has_type_a: bool,  // Any ratio > 1.2
    pub has_type_b: bool,  // Any ratio > 1.4 (fail)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorsionalRow {
    pub story: String,
    pub pair_name: String,  // e.g., "J101-J102"
    pub joint_a: String,
    pub joint_b: String,
    pub torsional_ratio: f64,
    pub amplification_ax: f64,
    pub classification: String,  // "Type A" (>1.2), "Type B" (>1.4), "OK" (<=1.2)
}
```

#### Pier Shear Stress Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PierShearStressOutput {
    pub rows: Vec<PierShearStressRow>,  // Rectangular piers only
    pub x_average: PierShearStressAverageRow,
    pub y_average: PierShearStressAverageRow,
    pub pass: bool,  // Pass if all individual <= 8.0 AND averages <= 10.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PierShearStressRow {
    pub pier_name: String,
    pub story: String,
    pub direction: String,  // "X" or "Y"
    pub shear_v2_kip: f64,
    pub section_width_ft: f64,
    pub section_thick_ft: f64,
    pub fc_ksi: f64,
    pub stress_psi: f64,
    pub sqrt_fc_psi: f64,
    pub stress_ratio: f64,  // stress_psi / √f'c
    pub pass: bool,  // stress_ratio <= 8.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PierShearStressAverageRow {
    pub direction: String,  // "X" or "Y"
    pub stress_psi: f64,
    pub sqrt_fc_psi: f64,
    pub average_ratio: f64,
    pub pass: bool,  // average_ratio <= 10.0
}
```

#### Pier Axial Stress Output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PierAxialStressOutput {
    pub piers: Vec<PierAxialStressRow>,
    pub governing_gravity: Option<PierAxialStressRow>,
    pub governing_wind: Option<PierAxialStressRow>,
    pub governing_seismic: Option<PierAxialStressRow>,
    pub governing_overall: Option<PierAxialStressRow>,
    pub pass: bool,  // Pass if governing_overall DCR < 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PierAxialStressRow {
    pub pier_name: String,
    pub story: String,
    pub combo_type: String,  // "Gravity", "Wind", "Seismic"
    pub combo_name: String,
    pub axial_demand_kip: f64,
    pub ag_in2: f64,
    pub fc_ksi: f64,
    pub phi_axial: f64,
    pub axial_capacity_kip: f64,
    pub dcr: f64,  // Demand / Capacity Ratio
    pub pass: bool,  // dcr < 1.0
}
```

### 5.3 CodeParams & Check Modules

#### CodeParams Structure (ext-calc/src/lib.rs)

```rust
/// Validated parameters derived from config + parquet tables
pub struct CodeParams {
    pub modal: Option<ModalParams>,
    pub base_reactions: Option<BaseReactionsParams>,
    pub story_forces: Option<StoryForcesParams>,
    pub drift_wind: Option<DriftDirectionalParams>,
    pub drift_seismic: Option<DriftDirectionalParams>,
    pub displacement_wind: Option<DisplacementDirectionalParams>,
    pub torsional: Option<TorsionalParams>,
    pub pier_shear_stress_wind: Option<PierShearStressParams>,
    pub pier_shear_stress_seismic: Option<PierShearStressParams>,
    pub pier_axial_stress: Option<PierAxialStressParams>,
    pub check_selection: CheckSelection,
}

impl CodeParams {
    /// Validate config against parquet tables
    pub fn from_config(config: &CalcConfig, tables: &LoadedTables) -> Result<Self> {
        // 1. Verify all required cases exist in parquet
        // 2. Verify all joint groups exist in joint_drifts
        // 3. Build fc_map from material properties
        // 4. Convert each config sub-struct to params
        // Returns fatal error if case/group not found
        Ok(CodeParams { ... })
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckSelection {
    pub modal: bool,
    pub base_reactions: bool,
    pub story_forces: bool,
    pub drift_wind: bool,
    pub drift_seismic: bool,
    pub displacement_wind: bool,
    pub torsional: bool,
    pub pier_shear_stress_wind: bool,
    pub pier_shear_stress_seismic: bool,
    pub pier_axial_stress: bool,
}

impl CheckSelection {
    pub fn from_code_params(params: &CodeParams) -> Self {
        CheckSelection {
            modal: params.modal.is_some(),
            base_reactions: params.base_reactions.is_some(),
            story_forces: params.story_forces.is_some(),
            drift_wind: params.drift_wind.is_some(),
            drift_seismic: params.drift_seismic.is_some(),
            displacement_wind: params.displacement_wind.is_some(),
            torsional: params.torsional.is_some(),
            pier_shear_stress_wind: params.pier_shear_stress_wind.is_some(),
            pier_shear_stress_seismic: params.pier_shear_stress_seismic.is_some(),
            pier_axial_stress: params.pier_axial_stress.is_some(),
        }
    }
}

/// Parameters per check after validation

pub struct StoryForcesParams {
    pub cases_x: Vec<String>,
    pub cases_y: Vec<String>,
}

pub struct DriftDirectionalParams {
    pub cases_x: Vec<String>,
    pub cases_y: Vec<String>,
    pub drift_limit: f64,
    pub joint_groups: Vec<String>,
}

pub struct TorsionalParams {
    pub case_x: String,
    pub case_y: String,
    pub x_joints: Vec<String>,
    pub y_joints: Vec<String>,
}

pub struct PierShearStressParams {
    pub cases: Vec<String>,
    pub phi_v: f64,
    pub fc_map: HashMap<String, f64>,  // material_name -> fc_ksi
    pub fc_default_ksi: f64,
}

pub struct PierAxialStressParams {
    pub gravity_combos: Vec<String>,
    pub wind_combos: Vec<String>,
    pub seismic_combos: Vec<String>,
    pub phi_axial: f64,
    pub fc_map: HashMap<String, f64>,
}
```

#### Check Module Organization

```
crates/ext-calc/src/checks/
  ├── mod.rs              (exports, CheckSelection)
  ├── story_forces.rs     (NEW)
  ├── drift_wind.rs       (UPDATED — direction enum)
  ├── drift_seismic.rs    (UPDATED — direction enum)
  ├── displacement_wind.rs (UPDATED — direction enum)
  ├── torsional.rs        (NEW — with ETABS ecc handling)
  ├── pier_shear_stress.rs (NEW — rectangular filter, UnitContext)
  ├── pier_axial.rs       (UPDATED — run_stress() variant)
  └── common.rs           (shared utilities)
```

### 5.4 Data Loading in lib.rs

```rust
pub fn run_all(config_path: &Path, results_dir: &Path) -> Result<CalcOutput> {
    // 1. Load config
    let config = Config::load(config_path)?;
    
    // 2. Load all parquet tables
    let tables = load_all_tables(results_dir)?;
    
    // 3. Convert to CodeParams (validate config against tables)
    let params = CodeParams::from_config(&config, &tables)?;
    
    // 4. Run each check (only if configured)
    let story_forces = if params.story_forces.is_some() {
        Some(checks::story_forces::run(&tables.story_forces, &tables.story_definitions, &params)?)
    } else { None };
    
    let drift_wind = if params.check_selection.drift_wind {
        Some(checks::drift_wind::run(&tables.joint_drifts, &tables.story_definitions, &params)?)
    } else { None };
    
    // ... (similar for other checks)
    
    // 5. Build summary from results
    let summary = build_summary(&story_forces, &drift_wind, ...);
    
    Ok(CalcOutput {
        meta, summary,
        story_forces, drift_wind, ...,
    })
}
```

### 5.4 Chart Constants (ext-render)

Located in: `crates/ext-render/src/charts.rs`

```rust
/// Story forces charts
pub const STORY_FORCE_VX_IMAGE: &str = "story_force_vx.svg";
pub const STORY_FORCE_MY_IMAGE: &str = "story_force_my.svg";
pub const STORY_FORCE_VY_IMAGE: &str = "story_force_vy.svg";
pub const STORY_FORCE_MX_IMAGE: &str = "story_force_mx.svg";

/// Drift charts
pub const DRIFT_WIND_X_IMAGE: &str = "drift_wind_x.svg";
pub const DRIFT_WIND_Y_IMAGE: &str = "drift_wind_y.svg";
pub const DRIFT_SEISMIC_X_IMAGE: &str = "drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_IMAGE: &str = "drift_seismic_y.svg";

/// Displacement charts
pub const DISPLACEMENT_WIND_X_IMAGE: &str = "displacement_wind_x.svg";
pub const DISPLACEMENT_WIND_Y_IMAGE: &str = "displacement_wind_y.svg";

/// Torsional charts
pub const TORSIONAL_X_IMAGE: &str = "torsional_x.svg";
pub const TORSIONAL_Y_IMAGE: &str = "torsional_y.svg";

/// Pier shear stress charts
pub const PIER_SHEAR_STRESS_WIND_IMAGE: &str = "pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEISMIC_IMAGE: &str = "pier_shear_stress_seismic.svg";

/// Pier axial stress charts
pub const PIER_AXIAL_STRESS_IMAGE: &str = "pier_axial_stress.svg";

/// Chart data structure
#[derive(Debug, Clone)]
pub struct ChartData {
    pub title: String,
    pub x_axis: String,
    pub y_axis: String,
    pub data_points: Vec<(String, f64)>,  // (label, value)
    pub limit_line: Option<f64>,
}
```

### 5.5 UnitContext Usage (from ext-core)

```rust
// In pier_shear_stress check:
use ext_core::unit::UnitContext;

let unit_context = UnitContext::new(units_in: "imperial", units_out: "imperial")?;

// Convert concrete strength
let fc_psi = unit_context.convert_pressure(fc_ksi, "ksi", "psi")?;

// Convert cross-section area (ft² to in²)
let acw_in2 = unit_context.convert_area(
    width_ft * thick_ft,
    from_unit: "ft2",
    to_unit: "in2",
)?;

// Convert shear stress (kip / in² = psi, but normalized)
let stress_psi = unit_context.convert_force_over_area(
    ve_kip,
    acw_in2,
    to_unit: "psi",
)?;

// Calculate stress ratio (no unit conversion needed for ratio)
let sqrt_fc_psi = fc_psi.sqrt();
let stress_ratio = stress_psi / sqrt_fc_psi;  // dimensionless

// Check: ratio should not exceed 8.0 per ACI §18.10.4.4
assert!(stress_ratio <= 8.0, "Individual pier exceeds 8√f'c");
```

### 5.6 Config Validation: from_config() Implementation

Located in: `crates/ext-calc/src/lib.rs`

```rust
impl CodeParams {
    pub fn from_config(config: &CalcConfig, tables: &LoadedTables) -> Result<Self> {
        // Phase 1: Verify all configured cases exist in parquet
        Self::validate_cases(config, tables)?;
        
        // Phase 2: Verify all joint groups exist in joint_drifts
        Self::validate_joint_groups(config, tables)?;
        
        // Phase 3: Build material fc_map for pier stress checks
        let fc_map = Self::build_fc_map(&tables, config)?;
        
        // Phase 4: Build individual params structs
        let story_forces = config.story_forces.is_configured()
            .then(|| StoryForcesParams {
                cases_x: config.story_forces.story_force_x_cases.clone(),
                cases_y: config.story_forces.story_force_y_cases.clone(),
            });
        
        let drift_wind = config.drift_wind.is_configured()
            .then(|| DriftDirectionalParams {
                cases_x: config.drift_wind.drift_x_cases.clone(),
                cases_y: config.drift_wind.drift_y_cases.clone(),
                drift_limit: config.drift_wind.drift_limit_or_default(),
                joint_groups: config.joint_tracking_groups.clone(),
            });
        
        let pier_shear_stress_wind = config.pier_shear_stress_wind.is_configured()
            .then(|| PierShearStressParams {
                cases: config.pier_shear_stress_wind.stress_cases.clone(),
                phi_v: config.pier_shear_stress_wind.phi_v_or_default(),
                fc_map: fc_map.clone(),
                fc_default_ksi: config.pier_shear_stress_wind.fc_default_or_fallback(),
            });
        
        // ... similar for other checks
        
        let check_selection = CheckSelection::from_code_params(&params);
        
        Ok(CodeParams {
            story_forces,
            drift_wind,
            pier_shear_stress_wind,
            // ...
            check_selection,
        })
    }
    
    /// Verify all configured load cases exist in parquet tables
    fn validate_cases(config: &CalcConfig, tables: &LoadedTables) -> Result<()> {
        let mut missing_cases = Vec::new();
        
        for case in &config.story_forces.story_force_x_cases {
            if !tables.story_forces.iter().any(|row| &row.output_case == case) {
                missing_cases.push(case.clone());
            }
        }
        
        if !missing_cases.is_empty() {
            return Err(format!(
                "Story forces check: load case(s) not found in parquet: {:?}",
                missing_cases
            ));
        }
        
        Ok(())
    }
    
    /// Verify all joint groups exist and have at least one joint
    fn validate_joint_groups(config: &CalcConfig, tables: &LoadedTables) -> Result<()> {
        if config.joint_tracking_groups.is_empty() {
            return Ok(());  // No groups specified is OK
        }
        
        let available_joints: HashSet<_> = tables.joint_drifts
            .iter()
            .map(|row| &row.unique_name)
            .collect();
        
        for group in &config.joint_tracking_groups {
            // In real implementation, group is a name; would need mapping
            // For now, verify at least one joint is present
            if available_joints.is_empty() {
                return Err(format!("Joint tracking group '{}': no joints found", group));
            }
        }
        
        Ok(())
    }
    
    /// Build material-to-fc map from parquet material properties
    fn build_fc_map(tables: &LoadedTables, config: &CalcConfig) -> Result<HashMap<String, f64>> {
        let mut fc_map = HashMap::new();
        
        for material_row in &tables.material_properties_concrete_data {
            fc_map.insert(material_row.material.clone(), material_row.fc_ksi);
        }
        
        if fc_map.is_empty() {
            // Non-fatal: fc_default will be used
            println!("Warning: No materials found in parquet; will use fc_default");
        }
        
        Ok(fc_map)
    }
}
```

---

## Part 6: Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Test parquet loading
    #[test]
    fn test_load_story_definitions() { ... }
    
    // Test config validation
    #[test]
    fn test_missing_case_error() { ... }
    
    // Test drift calculation
    #[test]
    fn test_drift_wind_envelope() { ... }
    
    // Test torsional with 3 steps
    #[test]
    fn test_torsional_ratio_calculation() { ... }
    
    // Test pier shear stress rectangular filter
    #[test]
    fn test_rectangular_pier_filter() { ... }
    
    // Test unit conversion
    #[test]
    fn test_stress_formula_via_unitcontext() { ... }
}
```

### Integration Test

```rust
#[test]
fn test_full_pipeline_with_sample_project() {
    let results_dir = PathBuf::from("tests/fixtures/sample_etabs_output");
    let config_path = PathBuf::from("tests/fixtures/sample_config.toml");
    
    let output = run_all(&config_path, &results_dir).expect("pipeline failed");
    
    // Verify all checks ran
    assert!(output.story_forces.is_some());
    assert!(output.drift_wind.is_some());
    assert!(output.torsional.is_some());
    
    // Spot-check values
    let sf = output.story_forces.as_ref().unwrap();
    assert!(sf.rows.len() > 0);
    assert!(sf.rows[0].max_vx_kip > 0.0);
}
```

---

## Part 7: Checklist & Before-Going-Live

### Pre-Implementation Fixes

- [ ] `escape_text()` in `ext-report/pdf/template.rs` escapes `*` and `_` for load case names like `DBE_X*Cd/R`
- [ ] `build_pier_fc_map()` moved to `pier_shear_stress.rs` and shared with axial check
- [ ] All unit conversions use `UnitContext`, zero hardcoded multipliers
- [ ] Rectangular pier filter implemented in stress check

### Config Validation (Before Any Check Runs)

- [ ] All required sections have non-empty config OR disabled (None)
- [ ] Each case in config exists in parquet (warn if not)
- [ ] Each joint in tracking groups exists (warn if not)
- [ ] Modal mass reaches threshold (error if not)

### Data Quality Checks

- [ ] Story elevations are monotonic (warn if not, sort anyway)
- [ ] No null critical fields (story, pier, case, step)
- [ ] Pier shear velocity never infinite or NaN

### Error Message Standards

All errors include:
- **What went wrong** — clear + specific (not generic)
- **Why** — explains code requirement or data issue
- **Remedy** — actionable user steps

---

## Part 8: Validation Against Live Data

Use the sample ETABS project (steel-columns) to verify:

```bash
# 1. Load parquet outputs
head -20 story_definitions.parquet
head -20 pier_forces.parquet

# 2. Run config validation
ext-calc config.toml results_dir/

# 3. Verify outputs
cat calc_output.json | jq '.story_forces.rows | length'

# 4. Spot-check values
# Drift wind X should be < 0.0025
# Torsional ratio should be computed from steps 1/2/3
# Pier shear stress only on rectangular piers
```

---

## Summary

This unified spec provides:
1. **Complete parquet schema** with 9 tables, column definitions, units, and usage
2. **Data validation rules** (fatal, non-fatal, edge cases)
3. **7 structural checks** — ACI 318-14 compliant, end-to-end algorithms
4. **Unit handling** — UnitContext ONLY, no hardcoded multipliers
5. **Config & output types** — Rust structs, serialization
6. **Error handling** — Categorized, actionable messages
7. **Implementation guide** — Module structure, testing strategy, checklist

**Ready for implementation.** Each check has data source, validation, algorithm, and output defined. All exceptions documented.
