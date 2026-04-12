# ext-calc Calculation Spec v3
# Agent Implementation Guide — Restructured Config + All Checks

**Status:** Implementation-ready
**Target crates:** `ext-calc`, `ext-db`, `ext-report`, `ext-render`
**Supersedes:** EXT_CALC_SPEC_V2.md

---

## Open Questions (resolve before implementation)

Before starting, confirm these with the engineer:

**Q1 — Story forces chart pairing:**
Does the `story-forces` chart pair `VX` with `MY` (X-direction shear + overturning moment)
and `VY` with `MX` (Y-direction)? Assumed: YES — 4 line charts total.

**Q2 — Torsional multiple joint pairs:**
When `x-joints = [["Joint47","Joint50"], ["Joint48","Joint51"]]` (multiple pairs),
report each pair as a separate row in output, then take the governing ratio for summary?
Assumed: YES — report per pair, govern on max ratio.

**Q3 — Torsional multiple cases:**
When `torsional-x-case = ["ELF_X","DBE_X"]`, report per case and take governing?
Assumed: YES — report per case, govern on worst ratio across all cases.

**Q4 — Pier shear stress vs pier shear capacity:**
Is `[calc.pier-shear-stress-wind/seismic]` a SEPARATE check from the existing
`[calc.pier-shear-wind/seismic]`, or does it replace it?
Current assumption: SEPARATE. The existing capacity check (Vu ≤ ϕVn) stays.
The new stress check adds the ACI §18.10.4.4 normalized stress ratio.
Consequence: `alpha-c`, `fy-ksi`, `rho-t` are NOT needed in the stress config
(they belong only in the capacity config). Remove them from `pier-shear-stress-*`.

**Q5 — Wind step selection for drift/displacement:**
For multi-step wind cases (12 steps), the code currently takes max drift across
all steps for the configured direction. Confirmed correct?
Assumed: YES — envelope across all steps, then govern on the configured direction column.

---

## Config Structure Reference

Complete `config.toml` with all sections. This is the authoritative template.

```toml
[project]
name = "Project Test"

[extract]
units = "US_Kip_Ft"

# Leave [extract.tables] empty for the full default parquet set.
# Add per-table filters only when narrowing output.

[calc]
code = "ACI318-14"
occupancy-category = "II"
modal-case = "Modal (Rizt)"
# Renamed from drift-tracking-groups. Same joints used for drift, displacement,
# and torsional monitoring — "joint" is more accurate than "drift".
joint-tracking-groups = ["Joint47", "Joint49", "Joint50", "Joint51"]

[calc.modal]
min-mass-participation = 0.9
display-mode-limit = 20

# Renamed from [calc.base-shear]. Check logic unchanged.
[calc.base-reactions]
elf-case-x = "ELF_X"
elf-case-y = "ELF_Y"
rsa-case-x = "DBE_X"
rsa-case-y = "DBE_Y"
rsa-scale-min = 1.0

[[calc.base-reactions.pie-groups]]
label = "Gravity"
load-cases = ["Dead", "SDL", "Live (red)", "Live (non-red)", "Live (roof)"]

# NEW — story shear and overturning moment profiles per direction.
[calc.story-forces]
story-force-x-cases = ["ELF_X", "DBE_X", "MCER_X", "W_700YRS"]
story-force-y-cases = ["ELF_Y", "DBE_Y", "MCER_Y", "W_700YRS"]

# X and Y drift governed independently on their respective displacement columns.
# Both x and y can reference the same case (W_10YRS) — the step envelope is
# computed separately per direction column.
[calc.drift-wind]
drift-x-cases = ["W_10YRS"]
drift-y-cases = ["W_10YRS"]
drift-limit = 0.0025

[calc.drift-seismic]
drift-x-cases = ["DBE_X*Cd/R", "ELF_X_Drift*Cd/Ie"]
drift-y-cases = ["DBE_Y*Cd/R", "ELF_Y_Drift*Cd/Ie"]
drift-limit = 0.02

[calc.displacement-wind]
disp-x-cases = ["W_10YRS"]
disp-y-cases = ["W_10YRS"]
disp-limit-h = 400

# Supports multiple cases and multiple joint pairs.
# x-joints = list of pairs — each pair defines [near-edge joint, far-edge joint]
# for one X-direction monitoring section.
# torsional-x-case = list of cases — ratio computed per case, governing reported.
[calc.torsional]
torsional-x-case = ["ELF_X", "DBE_X"]
torsional-y-case = ["ELF_Y", "DBE_Y"]
x-joints = [["Joint47", "Joint50"]]
y-joints = [["Joint49", "Joint51"]]
ecc-ratio = 0.05

# Pier shear CAPACITY check (Vu <= phi*Vn) — existing checks, unchanged.
[calc.pier-shear-wind]
load-combos = ["ENV: WIND"]
phi-v = 0.75
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

[calc.pier-shear-seismic]
load-combos = ["ENV: DBE"]
phi-v = 0.75
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

# Pier shear STRESS check (Ve/(phi*Acw)/sqrt(f'c) vs 8 individual, 10 average).
# NEW — separate from capacity check above.
# alpha-c / fy-ksi / rho-t are NOT needed here (stress formula only uses Acw and f'c).
[calc.pier-shear-stress-wind]
stress-combos = ["ENV: WIND"]
phi-v = 0.75
fc-default-ksi = 8.0

[calc.pier-shear-stress-seismic]
stress-combos = ["ENV: DBE"]
phi-v = 0.75
fc-default-ksi = 8.0

# Pier axial stress check split by load category — produces 3 separate charts.
[calc.pier-axial-stress]
stress-gravity-combos = [
  "LC1: 1.4D",
  "LC2: 1.2D+1.6L",
]
stress-wind-combos = [
  "LC3.1: 1.2D+0.5W",
  "LC3.2: 1.2D-0.5W",
  "LC4.1: 1.2D+1.0W+1.0L",
  "LC4.2: 1.2D+1.0W-1.0L",
  "LC6.1: 0.9D+1.0W",
  "LC6.2: 0.9D-1.0W",
]
stress-seismic-combos = [
  "DBE1: (1.2+0.2Sds)D+0.5L+100X+30Y",
  "DBE2: (1.2+0.2Sds)D+0.5L+100Y+30X",
  "DBE3: (0.9-0.2Sds)D+100X+30Y",
  "DBE4: (0.9-0.2Sds)D+100Y+30X",
]
phi-axial = 0.65
```

---

## ext-db: Config Struct Changes

**File:** `crates/ext-db/src/config/calc.rs`

### Rename `drift_tracking_groups` → `joint_tracking_groups`

```rust
// In CalcConfig:
// Before:
#[serde(default)]
pub drift_tracking_groups: Vec<String>,
// After:
#[serde(rename = "joint-tracking-groups", default)]
pub joint_tracking_groups: Vec<String>,
```

Update all downstream references in `code_params.rs` and check files.

### Rename `base_shear` → `base_reactions`

```rust
// In CalcConfig:
// Before:
#[serde(default)]
pub base_shear: BaseShearCalcConfig,
// After:
#[serde(rename = "base-reactions", default)]
pub base_reactions: BaseReactionsCalcConfig,
```

Rename `BaseShearCalcConfig` → `BaseReactionsCalcConfig`.
Rename `BaseReactionPieGroupConfig` stays the same (already correct name).

### Replace `drift_wind` / `drift_seismic` / `displacement_wind` with directional versions

```rust
// Before:
pub drift_wind: DriftCalcConfig,          // load_cases: Vec<String>
pub drift_seismic: DriftCalcConfig,
pub displacement_wind: DisplacementCalcConfig,  // load_cases: Vec<String>

// After:
pub drift_wind: DriftDirectionalCalcConfig,
pub drift_seismic: DriftDirectionalCalcConfig,
pub displacement_wind: DisplacementDirectionalCalcConfig,
```

New config types:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DriftDirectionalCalcConfig {
    #[serde(default)]
    pub drift_x_cases: Vec<String>,
    #[serde(default)]
    pub drift_y_cases: Vec<String>,
    pub drift_limit: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DisplacementDirectionalCalcConfig {
    #[serde(default)]
    pub disp_x_cases: Vec<String>,
    #[serde(default)]
    pub disp_y_cases: Vec<String>,
    pub disp_limit_h: Option<u32>,
}
```

### Add new config structs

```rust
// Story forces — new
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoryForcesCalcConfig {
    #[serde(default)]
    pub story_force_x_cases: Vec<String>,
    #[serde(default)]
    pub story_force_y_cases: Vec<String>,
}

// Torsional — updated to support multiple joint pairs and multiple cases
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TorsionalCalcConfig {
    #[serde(default)]
    pub torsional_x_case: Vec<String>,
    #[serde(default)]
    pub torsional_y_case: Vec<String>,
    /// Each inner Vec must have exactly 2 joint names: [near-edge, far-edge].
    #[serde(default)]
    pub x_joints: Vec<Vec<String>>,
    #[serde(default)]
    pub y_joints: Vec<Vec<String>>,
    pub ecc_ratio: Option<f64>,
}

impl TorsionalCalcConfig {
    pub fn ecc_ratio(&self) -> f64 {
        self.ecc_ratio.unwrap_or(0.05)
    }
}

// Pier shear stress — NEW, separate from pier_shear_wind/seismic
// No alpha_c / fy_ksi / rho_t — stress formula only needs phi_v and Acw.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierShearStressCalcConfig {
    #[serde(default)]
    pub stress_combos: Vec<String>,
    pub phi_v: Option<f64>,
    pub fc_default_ksi: Option<f64>,
}

impl PierShearStressCalcConfig {
    pub fn phi_v(&self) -> f64 { self.phi_v.unwrap_or(0.75) }
    pub fn fc_default_ksi(&self) -> f64 { self.fc_default_ksi.unwrap_or(8.0) }
}

// Pier axial stress — replaces PierAxialCalcConfig with 3 combo groups
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PierAxialStressCalcConfig {
    #[serde(default)]
    pub stress_gravity_combos: Vec<String>,
    #[serde(default)]
    pub stress_wind_combos: Vec<String>,
    #[serde(default)]
    pub stress_seismic_combos: Vec<String>,
    pub phi_axial: Option<f64>,
}

impl PierAxialStressCalcConfig {
    pub fn phi_axial(&self) -> f64 { self.phi_axial.unwrap_or(0.65) }
    /// Flattened list of all combos for validation — checks that all combos
    /// exist in pier_forces before running.
    pub fn all_combos(&self) -> Vec<&str> {
        self.stress_gravity_combos.iter()
            .chain(self.stress_wind_combos.iter())
            .chain(self.stress_seismic_combos.iter())
            .map(String::as_str)
            .collect()
    }
}
```

Update `CalcConfig` to include all new fields:

```rust
pub struct CalcConfig {
    // ... (existing unchanged fields)
    #[serde(rename = "joint-tracking-groups", default)]
    pub joint_tracking_groups: Vec<String>,

    #[serde(rename = "story-forces", default)]
    pub story_forces: StoryForcesCalcConfig,

    #[serde(rename = "torsional", default)]
    pub torsional: TorsionalCalcConfig,

    #[serde(rename = "pier-shear-stress-wind", default)]
    pub pier_shear_stress_wind: PierShearStressCalcConfig,

    #[serde(rename = "pier-shear-stress-seismic", default)]
    pub pier_shear_stress_seismic: PierShearStressCalcConfig,

    #[serde(rename = "pier-axial-stress", default)]
    pub pier_axial_stress: PierAxialStressCalcConfig,
}
```

Keep the old `PierAxialCalcConfig` / `pier_axial` field for backward compatibility
with existing fixture configs. Both are parsed; `pier_axial_stress` takes precedence
if both are present.

---

## ext-calc: Output Types

**File:** `crates/ext-calc/src/output.rs`

### Rename `BaseShearOutput` → `BaseReactionsOutput`

```rust
// Rename type only. Internal fields unchanged.
pub struct BaseReactionsOutput {
    pub rows: Vec<BaseReactionCheckRow>,
    pub direction_x: BaseReactionDir,
    pub direction_y: BaseReactionDir,
}
// Also rename BaseShearDir → BaseReactionDir (fields unchanged).
```

Update `CalcOutput`:
```rust
pub base_reactions: Option<BaseReactionsOutput>,   // was: base_shear
```

### New: Story forces output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForceEnvelopeRow {
    pub story: String,
    /// Max |VX| across all configured X-direction cases and all steps.
    pub max_vx_kip: f64,
    /// Max |MY| (overturning about Y-axis) paired with X-direction shear.
    pub max_my_kip_ft: f64,
    /// Max |VY| across all configured Y-direction cases and all steps.
    pub max_vy_kip: f64,
    /// Max |MX| (overturning about X-axis) paired with Y-direction shear.
    pub max_mx_kip_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForcesOutput {
    /// Rows sorted top-down (highest story first) for chart plotting.
    pub rows: Vec<StoryForceEnvelopeRow>,
}
```

### Updated: Drift outputs with X/Y split

```rust
/// Wind drift split by principal direction.
/// x governs on DriftX column only; y governs on DriftY column only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftWindOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

/// Seismic drift split by principal direction.
/// x uses drift-x-cases; y uses drift-y-cases.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftSeismicOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

/// Wind displacement split by principal direction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementWindOutput {
    pub x: DisplacementOutput,
    pub y: DisplacementOutput,
}
```

Update `CalcOutput`:
```rust
pub drift_wind: Option<DriftWindOutput>,
pub drift_seismic: Option<DriftSeismicOutput>,
pub displacement_wind: Option<DisplacementWindOutput>,
```

### New: Torsional output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalRow {
    pub story: String,
    pub elf_case: String,
    pub joint_pair: Vec<String>,     // ["Joint47", "Joint50"]
    pub disp_near: f64,              // displacement of near-edge joint [ft]
    pub disp_far: f64,               // displacement of far-edge joint [ft]
    pub delta_avg: f64,              // (|disp_near| + |disp_far|) / 2 from nominal step [ft]
    pub delta_max: f64,              // max displacement across eccentric steps [ft]
    pub ratio: f64,                  // delta_max / delta_avg
    pub ax: f64,                     // (ratio/1.2)^2 capped at 3.0, or 1.0 if ratio <= 1.2
    pub is_type_a: bool,             // ratio > 1.2
    pub is_type_b: bool,             // ratio > 1.4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalDirectionOutput {
    /// All rows: one per (story, case, joint_pair).
    pub rows: Vec<TorsionalRow>,
    /// Story with the highest ratio across all cases and joint pairs.
    pub governing_story: String,
    pub governing_case: String,
    pub governing_joint_pair: Vec<String>,
    pub max_ratio: f64,
    pub has_type_a: bool,
    pub has_type_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalOutput {
    pub x: TorsionalDirectionOutput,
    pub y: TorsionalDirectionOutput,
    /// pass if no Type B irregularity in either direction.
    /// Type A triggers a warning but is not a code-level failure by itself.
    pub pass: bool,
}
```

### New: Pier shear stress output

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressRow {
    pub story: String,
    pub pier: String,
    pub combo: String,
    pub wall_direction: String,      // "X" or "Y" — derived from AxisAngle
    pub acw_in2: f64,
    pub fc_psi: f64,
    pub sqrt_fc: f64,
    pub ve_kip: f64,
    /// ve_kip * 1000 / (phi_v * acw_in2)   [psi]
    pub stress_psi: f64,
    /// stress_psi / sqrt_fc   [n × √f'c, dimensionless]
    pub stress_ratio: f64,
    /// 8.0 for individual pier check (ACI §18.10.4.4)
    pub limit: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressAverageRow {
    pub story: String,
    pub sum_ve_kip: f64,
    pub sum_acw_in2: f64,
    /// (sum_ve * 1000) / (phi_v * sum_acw) / sqrt_fc
    pub avg_stress_ratio: f64,
    /// 10.0 for average stress check (ACI §18.10.4.4 commentary)
    pub limit: f64,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressOutput {
    pub phi_v: f64,
    /// Individual per-pier rows.
    pub per_pier: Vec<PierShearStressRow>,
    /// Average stress per story for X-direction walls (AxisAngle ≈ 0°).
    pub x_average: Vec<PierShearStressAverageRow>,
    /// Average stress per story for Y-direction walls (AxisAngle ≈ 90°).
    pub y_average: Vec<PierShearStressAverageRow>,
    /// Max individual stress ratio across all piers.
    pub max_individual_ratio: f64,
    /// Max average stress ratio across X and Y groups.
    pub max_average_ratio: f64,
    /// pass if all individual ratios <= 8.0 AND all average ratios <= 10.0
    pub pass: bool,
}
```

### Updated: Pier axial output split by load category

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierAxialStressOutput {
    pub phi_axial: f64,
    /// All piers across all combos from all three categories.
    pub piers: Vec<PierAxialResult>,   // reuse existing PierAxialResult
    /// Governing from gravity combos only.
    pub governing_gravity: Option<PierAxialResult>,
    /// Governing from wind combos only.
    pub governing_wind: Option<PierAxialResult>,
    /// Governing from seismic combos only.
    pub governing_seismic: Option<PierAxialResult>,
    /// Overall governing (highest DCR across all categories).
    pub governing: PierAxialResult,
    pub pass: bool,
}
```

Update `CalcOutput`:
```rust
pub torsional: Option<TorsionalOutput>,
pub pier_shear_stress_wind: Option<PierShearStressOutput>,
pub pier_shear_stress_seismic: Option<PierShearStressOutput>,
// Renamed from pier_axial — keep pier_axial as an alias for backward compat
pub pier_axial_stress: Option<PierAxialStressOutput>,
```

---

## ext-calc: Params Structs

**File:** `crates/ext-calc/src/code_params.rs`

### Rename fields to match new config names

```rust
pub struct CodeParams {
    // Before: drift_tracking_groups
    pub joint_tracking_groups: Vec<String>,

    // Before: base_shear: BaseShearParams
    pub base_reactions: BaseReactionsParams,   // same fields, renamed type

    // New
    pub story_forces: Option<StoryForcesParams>,

    // Updated: separate X/Y case lists
    pub drift_wind: DriftDirectionalParams,
    pub drift_seismic: DriftDirectionalParams,
    pub displacement_wind: DisplacementDirectionalParams,

    // Updated: multi-pair, multi-case
    pub torsional: Option<TorsionalParams>,

    // Existing (unchanged)
    pub pier_shear_wind: PierShearParams,
    pub pier_shear_seismic: PierShearParams,

    // New
    pub pier_shear_stress_wind: Option<PierShearStressParams>,
    pub pier_shear_stress_seismic: Option<PierShearStressParams>,

    // Updated (split by category)
    pub pier_axial_stress: PierAxialStressParams,
}
```

New param structs:

```rust
pub struct StoryForcesParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
}

pub struct DriftDirectionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub drift_limit: f64,
}

pub struct DisplacementDirectionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub disp_limit_h: u32,
}

pub struct TorsionalJointPair {
    pub near: String,   // joint name — UniqueName in joint_drifts
    pub far: String,
}

pub struct TorsionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub x_joint_pairs: Vec<TorsionalJointPair>,
    pub y_joint_pairs: Vec<TorsionalJointPair>,
    pub ecc_ratio: f64,
}

pub struct PierShearStressParams {
    pub combos: Vec<String>,
    pub phi_v: f64,
    pub fc_default_ksi: f64,
}

pub struct PierAxialStressParams {
    pub gravity_combos: Vec<String>,
    pub wind_combos: Vec<String>,
    pub seismic_combos: Vec<String>,
    pub phi_axial: f64,
}

impl PierAxialStressParams {
    pub fn all_combos(&self) -> impl Iterator<Item = &str> {
        self.gravity_combos.iter()
            .chain(self.wind_combos.iter())
            .chain(self.seismic_combos.iter())
            .map(String::as_str)
    }
}
```

Validation in `from_config()`:
- `torsional`: require each joint pair has exactly 2 names; at least 1 pair per direction.
- `drift_wind` / `drift_seismic`: require both `x_cases` and `y_cases` non-empty.
- `story_forces`: optional — if both case lists empty, `story_forces` stays `None`.
- `torsional`: optional — if `x_joints` is empty, `torsional` stays `None`.
- `pier_shear_stress_*`: optional — if combos empty, stays `None`.

---

## Check Algorithms

### Check: Story forces (NEW)

**File:** `crates/ext-calc/src/checks/story_forces.rs`

```
inputs:
  rows: &[StoryForceRow]
  stories: &[StoryDefRow]
  params: &StoryForcesParams

for X-direction:
  filter rows where output_case in x_cases AND location == "Bottom"
  (Bottom gives the cumulative story shear above that level — matches ETABS convention)
  per story: max_vx  = max(|VX|) across all matching rows and all steps
             max_my  = max(|MY|) across all matching rows and all steps at same story

for Y-direction:
  filter rows where output_case in y_cases AND location == "Bottom"
  per story: max_vy  = max(|VY|) across all matching rows and all steps
             max_mx  = max(|MX|) across all matching rows and all steps

sort by story elevation descending (top-down for chart — highest story plotted first)
return StoryForcesOutput { rows: Vec<StoryForceEnvelopeRow> }
```

No pass/fail — this is a review table and chart, not a code check.

### Check: Wind drift and displacement (UPDATED logic)

**File:** `crates/ext-calc/src/checks/drift_wind.rs`

Add `DriftDirection` enum:

```rust
pub enum DriftDirection { X, Y }
```

Modify `build_drift_output` to accept direction. The only change from existing code:

In the governing-selection `candidates` array, restrict to one direction:
```rust
// DriftDirection::X:
let candidates = [
    ("X", "positive", row.max_drift_x_pos.abs()),
    ("X", "negative", row.max_drift_x_neg.abs()),
];

// DriftDirection::Y:
let candidates = [
    ("Y", "positive", row.max_drift_y_pos.abs()),
    ("Y", "negative", row.max_drift_y_neg.abs()),
];
```

The row-building (grouping, max/min per step) remains identical — all four columns
(`DriftX`, `DriftY`, `DispX`, `DispY`) are still stored in `DriftEnvelopeRow`.
The direction parameter only affects what governs the summary.

`run()` now returns `DriftWindOutput`:

```rust
pub fn run(..., params: &CodeParams) -> Result<DriftWindOutput> {
    // X pass: uses drift_wind.x_cases, DriftDirection::X
    let x = build_drift_output_directional(
        rows, stories, group_map,
        &params.joint_tracking_groups,
        &params.drift_wind.x_cases,
        params.drift_wind.drift_limit,
        DriftDirection::X,
    )?;
    // Y pass: uses drift_wind.y_cases, DriftDirection::Y
    let y = build_drift_output_directional(
        rows, stories, group_map,
        &params.joint_tracking_groups,
        &params.drift_wind.y_cases,
        params.drift_wind.drift_limit,
        DriftDirection::Y,
    )?;
    Ok(DriftWindOutput { x, y })
}
```

Same pattern for `drift_seismic.rs` (uses `drift_seismic.x_cases`/`y_cases`).
Same pattern for `displacement_wind.rs` (governs on `DispX` or `DispY`).

### Check: Torsional (NEW — updated from v2 spec)

**File:** `crates/ext-calc/src/checks/torsional.rs`

```
inputs:
  joint_drifts: &[JointDriftRow]
  stories: &[StoryDefRow]
  params: &TorsionalParams

for each direction in [X, Y]:
  cases     = params.x_cases  (or y_cases)
  pairs     = params.x_joint_pairs  (or y_joint_pairs)
  disp_col  = DispX for X, DispY for Y
  
  for each case in cases:
    filter rows: output_case == case
    
    for each pair (near_joint, far_joint) in pairs:
      for each story:
        
        NOMINAL step (StepNumber == 1.0 OR StepType == "Max"):
          disp_near_nom = disp_col value for near_joint, story, step=1
          disp_far_nom  = disp_col value for far_joint,  story, step=1
          delta_avg = (|disp_near_nom| + |disp_far_nom|) / 2.0
        
        ECCENTRIC steps (StepNumber in {2.0, 3.0}):
          disp_near_ecc = max |disp_col| for near_joint, story, steps 2 and 3
          disp_far_ecc  = max |disp_col| for far_joint,  story, steps 2 and 3
          delta_max = max(disp_near_ecc, disp_far_ecc)
        
        if delta_avg < 1e-9: ratio = 1.0 (base level or near-zero)
        else: ratio = delta_max / delta_avg
        
        ax = if ratio <= 1.2 { 1.0 }
             else { (ratio / 1.2_f64).powi(2).min(3.0) }
        
        push TorsionalRow {
          story, elf_case: case, joint_pair: [near, far],
          disp_near: disp_near_nom, disp_far: disp_far_nom,
          delta_avg, delta_max, ratio, ax,
          is_type_a: ratio > 1.2,
          is_type_b: ratio > 1.4,
        }
  
  sort all rows by story elevation descending
  governing = row with max ratio
  has_type_a = any row.is_type_a
  has_type_b = any row.is_type_b

pass = !x.has_type_b && !y.has_type_b
```

**Step number handling note:**
ETABS exports ELF with accidental eccentricity as:
- `StepNumber = 1.0` → nominal (no eccentricity)
- `StepNumber = 2.0` → +5% eccentricity
- `StepNumber = 3.0` → -5% eccentricity

All three steps have `StepType = "Step By Step"`. Filter by `step_number` value, not by `step_type`.

### Check: Pier shear stress (NEW — simplified from v2 spec)

**File:** `crates/ext-calc/src/checks/pier_shear_stress.rs`

**Formula (ACI 318-14 §18.10.4.4):**
```
stress_psi  = Ve [kip] × 1000 / (phi_v × Acw [in²])
stress_ratio = stress_psi / sqrt(f'c [psi])
limit_individual = 8.0   → per pier
limit_average    = 10.0  → average across all walls of same direction
```

**Wall direction classification** from `AxisAngle` in `pier_section_properties`:
```
AxisAngle ≈ 0°   (|angle| < 15° or |angle - 180°| < 15°) → X-wall (resists Y seismic? No...)
```

**Clarification on wall direction convention:**
In ETABS, `AxisAngle = 0°` means the pier's strong axis is along the global X direction
— the wall runs in the X direction and resists **Y-direction** lateral forces (V2 = in-plane
shear along the wall's strong axis). `AxisAngle = 90°` means strong axis along Y, wall
resists **X-direction** forces.

Check which convention matches your model by inspecting `pier_section_properties.parquet`
alongside the pier names. The spec uses the nomenclature from your Excel:
- `PX` piers → X-direction walls (config `[calc.pier-shear-stress-seismic].x-wall-piers`)
- `PY` piers → Y-direction walls

**Auto-detection fallback** (when `x_wall_piers`/`y_wall_piers` not configured):
```
AxisAngle ≈ 0° or 180°  → assign to the group that governs on VX
AxisAngle ≈ 90° or 270° → assign to the group that governs on VY
```
If auto-detection is ambiguous, require explicit config.

**Algorithm:**
```
step 1 — envelope Ve per (story, pier):
  filter pier_forces: output_case in stress_combos
  per (story, pier): Ve = max(shear_v2_abs_kip) across all rows

step 2 — per-pier stress ratio:
  for each (story, pier):
    section = pier_sections[(pier, story)]
    fc_ksi  = fc_map[(pier, story)]
    fc_psi  = fc_ksi * 1000
    acw     = section.acw_in2
    sqrt_fc = fc_psi.sqrt()
    stress  = Ve * 1000 / (phi_v * acw)
    ratio   = stress / sqrt_fc
    direction = "X" if pier in x_wall_piers, "Y" if in y_wall_piers, skip otherwise
    pass    = ratio <= 8.0

step 3 — average per direction per story:
  for direction in [X, Y]:
    group by story
    per story:
      sum_ve  = sum(Ve_kip) for all piers in this direction
      sum_acw = sum(acw_in2) for all piers in this direction
      fc_psi  = fc_psi of first pier (assume uniform concrete grade per direction per story)
      sqrt_fc = fc_psi.sqrt()
      avg_stress = sum_ve * 1000 / (phi_v * sum_acw)
      avg_ratio  = avg_stress / sqrt_fc
      pass = avg_ratio <= 10.0
  sort by story elevation
```

### Check: Pier axial stress (UPDATED — split by load category)

**File:** `crates/ext-calc/src/checks/pier_axial.rs` (modify existing)

The calculation formula is **unchanged** — only the input partitioning and output structure change.

```
run() takes PierAxialStressParams instead of PierAxialParams.

For each combo category [gravity, wind, seismic]:
  run the existing demand-capacity logic against that combo subset
  → Vec<PierAxialResult> per category

Combine into PierAxialStressOutput:
  governing_gravity  = max DCR from gravity results
  governing_wind     = max DCR from wind results
  governing_seismic  = max DCR from seismic results
  governing          = max DCR across all three
  pass               = all DCR <= 1.0
  piers              = all results concatenated (for the full detail table)
```

---

## ext-calc: lib.rs Wiring

**File:** `crates/ext-calc/src/lib.rs`

```rust
// Load inputs (unchanged — all tables already loaded):
let story_forces_rows = tables::story_forces::load_story_forces(results_dir)?;

// Run checks:
let story_forces_output = if params.story_forces.is_some() {
    Some(checks::story_forces::run(&story_forces_rows, &story_defs, params)?)
} else { None };

let drift_wind_output = if params.check_selection.drift_wind {
    Some(checks::drift_wind::run(&joint_drifts, &story_defs, &group_map, params)?)
} else { None };
// (drift_seismic and displacement_wind same pattern)

let torsional_output = if params.torsional.is_some() {
    Some(checks::torsional::run(&joint_drifts, &story_defs, params)?)
} else { None };

let pier_shear_stress_wind_output = if params.pier_shear_stress_wind.is_some() {
    Some(checks::pier_shear_stress::run(&pier_forces, &pier_sections, &pier_fc_map, params, "wind")?)
} else { None };

let pier_shear_stress_seismic_output = if params.pier_shear_stress_seismic.is_some() {
    Some(checks::pier_shear_stress::run(&pier_forces, &pier_sections, &pier_fc_map, params, "seismic")?)
} else { None };

let pier_axial_stress_output = if params.check_selection.pier_axial {
    Some(checks::pier_axial::run_stress(&pier_forces, &pier_sections, &pier_fc_map, params)?)
} else { None };
```

Update `build_summary()` with new summary keys:
- `storyForces` — always `"loaded"` (no pass/fail)
- `driftWindX`, `driftWindY`
- `driftSeismicX`, `driftSeismicY`
- `displacementWindX`, `displacementWindY`
- `torsionalX`, `torsionalY` — `"warn"` for Type A, `"fail"` for Type B, `"pass"` otherwise
- `pierShearStressWind`, `pierShearStressSeismic`
- `pierAxialStress` (replaces `pierAxial`)

---

## ext-render: Chart Constants

**File:** `crates/ext-render/src/lib.rs`

Add new logical image name constants:

```rust
pub const STORY_FORCES_VX_IMAGE: &str = "images/story_forces_vx.svg";
pub const STORY_FORCES_VY_IMAGE: &str = "images/story_forces_vy.svg";
pub const STORY_FORCES_MX_IMAGE: &str = "images/story_forces_mx.svg";
pub const STORY_FORCES_MY_IMAGE: &str = "images/story_forces_my.svg";

pub const DRIFT_WIND_X_IMAGE: &str = "images/drift_wind_x.svg";
pub const DRIFT_WIND_Y_IMAGE: &str = "images/drift_wind_y.svg";
pub const DISPLACEMENT_WIND_X_IMAGE: &str = "images/displacement_wind_x.svg";
pub const DISPLACEMENT_WIND_Y_IMAGE: &str = "images/displacement_wind_y.svg";

pub const DRIFT_SEISMIC_X_IMAGE: &str = "images/drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_IMAGE: &str = "images/drift_seismic_y.svg";

pub const TORSIONAL_X_IMAGE: &str = "images/torsional_x.svg";
pub const TORSIONAL_Y_IMAGE: &str = "images/torsional_y.svg";

pub const PIER_SHEAR_STRESS_WIND_IMAGE: &str = "images/pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEISMIC_IMAGE: &str = "images/pier_shear_stress_seismic.svg";

pub const PIER_AXIAL_GRAVITY_IMAGE: &str = "images/pier_axial_gravity.svg";
pub const PIER_AXIAL_WIND_IMAGE: &str = "images/pier_axial_wind.svg";
pub const PIER_AXIAL_SEISMIC_IMAGE: &str = "images/pier_axial_seismic.svg";
```

Remove (replaced):
```rust
// Remove: DRIFT_WIND_IMAGE, DRIFT_SEISMIC_IMAGE, DISPLACEMENT_WIND_IMAGE,
//         PIER_AXIAL_IMAGE
```

Update `render_all_svg()` to produce all new charts. Chart type for each:
- Story forces VX, VY, MX, MY → horizontal bar or horizontal line chart (story on Y-axis, value on X-axis)
- Drift charts → horizontal line chart (drift ratio on X, story on Y)
- Displacement charts → horizontal line chart
- Torsional → horizontal line chart (ratio on X, story on Y, reference line at 1.2 and 1.4)
- Pier shear stress → combined per-pier line + average group line, story on Y
- Pier axial (3 charts) → DCR per pier per story, story on Y

---

## Report: New Pages

**File:** `crates/ext-report/src/report_document.rs`

New report sections to add (in order):

1. `StoryForcesPage` — X shear + overturning, Y shear + overturning, 4 charts on two pages
2. `DriftWindXPage` — chart + table for X-direction wind drift
3. `DriftWindYPage` — chart + table for Y-direction wind drift
4. `DriftSeismicXPage` — chart + table
5. `DriftSeismicYPage` — chart + table
6. `DisplacementWindXPage` — chart + table
7. `DisplacementWindYPage` — chart + table
8. `TorsionalXPage` — ratio chart + torsion type summary table
9. `TorsionalYPage`
10. `PierShearStressWindPage` — per-pier lines + average group lines
11. `PierShearStressSeismicPage`
12. `PierAxialGravityPage` — gravity combos DCR chart
13. `PierAxialWindPage` — wind combos DCR chart
14. `PierAxialSeismicPage` — seismic combos DCR chart

---

## Implementation Order

1. `ext-db/config/calc.rs` — add/rename all config structs (compile target: zero errors)
2. `ext-calc/output.rs` — add/rename all output types (compile target: zero errors)
3. `ext-calc/code_params.rs` — update `CodeParams` + `from_config()` validation
4. `ext-calc/checks/drift_wind.rs` — add `DriftDirection`, update `run()` → `DriftWindOutput`
5. `ext-calc/checks/drift_seismic.rs` — update to use directional params
6. `ext-calc/checks/displacement_wind.rs` — same pattern as drift_wind
7. `ext-calc/checks/story_forces.rs` — new file
8. `ext-calc/checks/torsional.rs` — new file
9. `ext-calc/checks/pier_shear_stress.rs` — new file
10. `ext-calc/checks/pier_axial.rs` — add `run_stress()` variant
11. `ext-calc/checks/mod.rs` — register new modules; update `CheckSelection`
12. `ext-calc/lib.rs` — wire all checks; update `build_summary()`
13. `ext-render` — add chart constants and render functions
14. `ext-report/report_document.rs` — add new report pages

---

## Key Invariants to Maintain

- `escape_text()` in `ext-report/src/pdf/template.rs` MUST escape `*` and `_` before
  any of these new checks can produce a valid PDF. Load case names like `DBE_X*Cd/R`
  will crash typst without this fix. Do this FIRST, before any report-side work.
- All new checks are opt-in: missing config → `None` in `CalcOutput`, no error.
- Story ordering always uses `sort_rows_by_story()` from `checks/drift_wind.rs`. Never reimplement.
- `fc_map` from `pier_shear::build_pier_fc_map()` is built ONCE and shared across
  all four pier checks (shear wind, shear seismic, shear stress wind, shear stress seismic, axial).
- Unit conversion uses `UnitContext` — no hardcoded unit strings in output structs.
- The typst `escape_text` fix and the `--new-instance` hang fix (3s process exit timeout
  in `sidecar/client.rs`) are pre-existing bugs that block the report path.
  Fix both before starting report section work.