# ext-calc Calculation Spec v4
# Agent Implementation Guide — All Questions Resolved

**Status:** Final — ready for implementation
**Target crates:** `ext-calc`, `ext-db`, `ext-report`, `ext-render`
**Supersedes:** EXT_CALC_SPEC_V3.md

All open questions from v3 are resolved:
- Q1 ✅ Story forces: 4 charts — VX, MY (X-direction), VY, MX (Y-direction)
- Q2 ✅ Torsional: report per (story, case, joint-pair), govern on max ratio
- Q3 ✅ Torsional multiple cases: same as Q2 — per case, govern on worst
- Q4 ✅ Pier shear stress REPLACES pier shear capacity — remove [calc.pier-shear-wind/seismic]
- Q5 ✅ Wind drift: envelope all 12 steps, govern on configured direction column

---

## Authoritative Config Template

```toml
[project]
name = "Project Test"

[extract]
units = "US_Kip_Ft"

[calc]
code = "ACI318-14"
occupancy-category = "II"
modal-case = "Modal (Rizt)"
joint-tracking-groups = ["Joint47", "Joint49", "Joint50", "Joint51"]

[calc.modal]
min-mass-participation = 0.9
display-mode-limit = 20

[calc.base-reactions]
elf-case-x = "ELF_X"
elf-case-y = "ELF_Y"
rsa-case-x = "DBE_X"
rsa-case-y = "DBE_Y"
rsa-scale-min = 1.0

[[calc.base-reactions.pie-groups]]
label = "Gravity"
load-cases = ["Dead", "SDL", "Live (red)", "Live (non-red)", "Live (roof)"]

[calc.story-forces]
story-force-x-cases = ["ELF_X", "DBE_X", "MCER_X", "W_700YRS"]
story-force-y-cases = ["ELF_Y", "DBE_Y", "MCER_Y", "W_700YRS"]

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

[calc.torsional]
torsional-x-case = ["ELF_X", "DBE_X"]
torsional-y-case = ["ELF_Y", "DBE_Y"]
x-joints = [["Joint47", "Joint50"]]
y-joints = [["Joint49", "Joint51"]]
ecc-ratio = 0.05

# Pier shear stress check — REPLACES the old pier-shear-wind/seismic capacity check.
# Formula: Ve/(phi*Acw) / sqrt(f'c). Limits: 8 individual, 10 average.
[calc.pier-shear-stress-wind]
stress-combos = ["ENV: WIND"]
phi-v = 0.75
fc-default-ksi = 8.0

[calc.pier-shear-stress-seismic]
stress-combos = ["ENV: DBE"]
phi-v = 0.75
fc-default-ksi = 8.0

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

**Removed from config (Q4):** `[calc.pier-shear-wind]` and `[calc.pier-shear-seismic]`
capacity check sections. Delete their structs, params, output types, and check files entirely.
The stress check is the only pier shear check going forward.

---

## ext-db: Config Struct Changes

**File:** `crates/ext-db/src/config/calc.rs`

### Complete CalcConfig replacement

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalcConfig {
    pub code: Option<String>,
    pub occupancy_category: Option<String>,
    pub modal_case: Option<String>,

    // Renamed from drift-tracking-groups
    #[serde(default)]
    pub joint_tracking_groups: Vec<String>,

    #[serde(default)]
    pub modal: ModalCalcConfig,

    // Renamed from base-shear
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

    // REPLACES pier-shear-wind/seismic — stress check only
    #[serde(rename = "pier-shear-stress-wind", default)]
    pub pier_shear_stress_wind: PierShearStressCalcConfig,

    #[serde(rename = "pier-shear-stress-seismic", default)]
    pub pier_shear_stress_seismic: PierShearStressCalcConfig,

    #[serde(rename = "pier-axial-stress", default)]
    pub pier_axial_stress: PierAxialStressCalcConfig,
}
```

### New/updated config types

```rust
// RENAMED from BaseShearCalcConfig — fields unchanged
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BaseReactionsCalcConfig {
    pub elf_case_x: Option<String>,
    pub elf_case_y: Option<String>,
    pub rsa_case_x: Option<String>,
    pub rsa_case_y: Option<String>,
    pub rsa_scale_min: Option<f64>,
    #[serde(default)]
    pub pie_groups: Vec<BaseReactionPieGroupConfig>,
}

// NEW
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoryForcesCalcConfig {
    #[serde(default)]
    pub story_force_x_cases: Vec<String>,
    #[serde(default)]
    pub story_force_y_cases: Vec<String>,
}

// REPLACES DriftCalcConfig
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DriftDirectionalCalcConfig {
    #[serde(default)]
    pub drift_x_cases: Vec<String>,
    #[serde(default)]
    pub drift_y_cases: Vec<String>,
    pub drift_limit: Option<f64>,
}

// REPLACES DisplacementCalcConfig
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DisplacementDirectionalCalcConfig {
    #[serde(default)]
    pub disp_x_cases: Vec<String>,
    #[serde(default)]
    pub disp_y_cases: Vec<String>,
    pub disp_limit_h: Option<u32>,
}

// NEW — supports multiple cases and multiple joint pairs per direction
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TorsionalCalcConfig {
    /// Cases for X-direction torsion (e.g. ELF_X with 3 eccentric steps each)
    #[serde(default)]
    pub torsional_x_case: Vec<String>,
    /// Cases for Y-direction torsion
    #[serde(default)]
    pub torsional_y_case: Vec<String>,
    /// List of joint pairs for X-direction monitoring.
    /// Each inner Vec must have exactly 2 UniqueName strings.
    #[serde(default)]
    pub x_joints: Vec<Vec<String>>,
    /// List of joint pairs for Y-direction monitoring.
    #[serde(default)]
    pub y_joints: Vec<Vec<String>>,
    pub ecc_ratio: Option<f64>,
}

impl TorsionalCalcConfig {
    pub fn ecc_ratio(&self) -> f64 { self.ecc_ratio.unwrap_or(0.05) }
    pub fn is_configured(&self) -> bool {
        !self.torsional_x_case.is_empty()
            && !self.x_joints.is_empty()
            && !self.y_joints.is_empty()
    }
}

// NEW — replaces PierShearCalcConfig (no alpha_c / fy_ksi / rho_t)
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
    pub fn is_configured(&self) -> bool { !self.stress_combos.is_empty() }
}

// NEW — replaces PierAxialCalcConfig (3 combo groups)
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
    pub fn all_combos(&self) -> Vec<&str> {
        self.stress_gravity_combos.iter()
            .chain(&self.stress_wind_combos)
            .chain(&self.stress_seismic_combos)
            .map(String::as_str)
            .collect()
    }
    pub fn is_configured(&self) -> bool { !self.all_combos().is_empty() }
}
```

**Delete entirely:** `PierShearCalcConfig`, `DriftCalcConfig`, `DisplacementCalcConfig`,
`PierAxialCalcConfig`, `BaseShearCalcConfig`.

---

## ext-calc: Output Types

**File:** `crates/ext-calc/src/output.rs`

### Rename and remove

```rust
// RENAME: BaseShearOutput → BaseReactionsOutput
// RENAME: BaseShearDir → BaseReactionDir
// Fields inside both structs are unchanged.
pub struct BaseReactionsOutput { ... }
pub struct BaseReactionDir { ... }

// REMOVE entirely:
// PierShearResult, PierShearOutput (replaced by PierShearStressOutput below)
```

### Update CalcOutput

```rust
pub struct CalcOutput {
    pub meta: CalcMeta,
    pub summary: CalcSummary,
    pub modal: Option<ModalOutput>,
    pub base_reactions: Option<BaseReactionsOutput>,   // was: base_shear
    pub story_forces: Option<StoryForcesOutput>,       // NEW
    pub drift_wind: Option<DriftWindOutput>,           // was: Option<DriftOutput>
    pub drift_seismic: Option<DriftSeismicOutput>,     // was: Option<DriftOutput>
    pub displacement_wind: Option<DisplacementWindOutput>, // was: Option<DisplacementOutput>
    pub torsional: Option<TorsionalOutput>,            // was: placeholder stub
    pub pier_shear_stress_wind: Option<PierShearStressOutput>,   // NEW (replaces pier_shear_wind)
    pub pier_shear_stress_seismic: Option<PierShearStressOutput>, // NEW (replaces pier_shear_seismic)
    pub pier_axial_stress: Option<PierAxialStressOutput>,  // NEW (replaces pier_axial)
}
```

### New output types

```rust
// ── Story forces ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForceEnvelopeRow {
    pub story: String,
    /// Max |VX| across all X-direction cases and all steps [kip]
    pub max_vx_kip: f64,
    /// Max |MY| overturning at same story, X-direction cases [kip·ft]
    pub max_my_kip_ft: f64,
    /// Max |VY| across all Y-direction cases and all steps [kip]
    pub max_vy_kip: f64,
    /// Max |MX| overturning at same story, Y-direction cases [kip·ft]
    pub max_mx_kip_ft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryForcesOutput {
    pub rows: Vec<StoryForceEnvelopeRow>,  // sorted top-down (highest story first)
}

// ── Drift / displacement XY wrappers ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftWindOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftSeismicOutput {
    pub x: DriftOutput,
    pub y: DriftOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementWindOutput {
    pub x: DisplacementOutput,
    pub y: DisplacementOutput,
}

// ── Torsional ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalRow {
    pub story: String,
    pub case: String,
    pub joint_a: String,                 // near-edge joint UniqueName
    pub joint_b: String,                 // far-edge joint UniqueName
    /// Story drift of joint_a at each step [in].
    /// Index 0 = step 1 (nominal), 1 = step 2 (+ecc), 2 = step 3 (-ecc)
    pub drift_a_steps: Vec<f64>,
    /// Story drift of joint_b at each step [in]
    pub drift_b_steps: Vec<f64>,
    /// MAX(drift_a, drift_b) at each step [in]
    pub delta_max_steps: Vec<f64>,
    /// AVG(drift_a, drift_b) at each step [in]
    pub delta_avg_steps: Vec<f64>,
    /// Governing ratio = MAX(delta_max[i] / delta_avg[i]) across all steps
    pub ratio: f64,
    /// Ax = MAX(MIN(MAX((delta_max[i] / (1.2 * delta_avg[i]))^2 ...), 3.0), 1.0)
    pub ax: f64,
    /// Eccentricity = ecc_ratio × building dimension [ft]
    /// Building dimension is derived from the distance between joint_a and joint_b
    /// in the relevant plan direction.
    pub ecc_ft: f64,
    /// Redundancy factor: 1.3 if ratio > 1.4, else 1.0
    pub rho: f64,
    pub is_type_a: bool,                 // ratio > 1.2
    pub is_type_b: bool,                 // ratio > 1.4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalDirectionOutput {
    pub rows: Vec<TorsionalRow>,         // per (story, case, joint_pair)
    pub governing_story: String,
    pub governing_case: String,
    pub governing_joints: Vec<String>,
    pub max_ratio: f64,
    pub has_type_a: bool,
    pub has_type_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorsionalOutput {
    pub x: TorsionalDirectionOutput,
    pub y: TorsionalDirectionOutput,
    /// pass if neither direction has Type B.
    /// Type A is a warning only (requires dynamic analysis per ASCE 7).
    pub pass: bool,
}

// ── Pier shear stress ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressRow {
    pub story: String,
    pub pier: String,
    pub combo: String,
    /// "X" for piers with AxisAngle ≈ 0°/180°, "Y" for ≈ 90°/270°
    pub wall_direction: String,
    pub acw_in2: f64,
    pub fc_psi: f64,
    pub sqrt_fc: f64,
    pub ve_kip: f64,
    /// Ve [kip] × 1000 / (phi_v × Acw [in²])   [psi]
    pub stress_psi: f64,
    /// stress_psi / sqrt_fc   [n × √f'c, dimensionless]
    pub stress_ratio: f64,
    pub limit_individual: f64,           // 8.0
    pub pass: bool,                      // ratio <= 8.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressAverageRow {
    pub story: String,
    pub wall_direction: String,
    pub sum_ve_kip: f64,
    pub sum_acw_in2: f64,
    pub sqrt_fc: f64,
    /// sum_ve × 1000 / (phi_v × sum_acw) / sqrt_fc
    pub avg_stress_ratio: f64,
    pub limit_average: f64,              // 10.0
    pub pass: bool,                      // avg_ratio <= 10.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierShearStressOutput {
    pub phi_v: f64,
    pub limit_individual: f64,           // 8.0
    pub limit_average: f64,              // 10.0
    /// Per-pier rows, sorted by (story_elevation desc, pier asc)
    pub per_pier: Vec<PierShearStressRow>,
    /// Average per story for X-direction walls, sorted top-down
    pub x_average: Vec<PierShearStressAverageRow>,
    /// Average per story for Y-direction walls, sorted top-down
    pub y_average: Vec<PierShearStressAverageRow>,
    pub max_individual_ratio: f64,
    pub max_average_ratio: f64,
    pub pass: bool,
}

// ── Pier axial stress (3-category) ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PierAxialStressOutput {
    pub phi_axial: f64,
    /// All individual pier results across all 3 combo categories
    pub piers: Vec<PierAxialResult>,    // reuse existing PierAxialResult
    pub governing_gravity: Option<PierAxialResult>,
    pub governing_wind: Option<PierAxialResult>,
    pub governing_seismic: Option<PierAxialResult>,
    pub governing: PierAxialResult,
    pub pass: bool,
}
```

---

## ext-calc: Code Params

**File:** `crates/ext-calc/src/code_params.rs`

```rust
pub struct CodeParams {
    pub code: String,
    pub occupancy_category: String,
    pub modal_case: String,
    pub modal_threshold: f64,
    pub modal_display_limit: usize,
    pub joint_tracking_groups: Vec<String>,        // renamed
    pub base_reactions: BaseReactionsParams,       // renamed
    pub story_forces: Option<StoryForcesParams>,   // NEW, optional
    pub drift_wind: DriftDirectionalParams,        // updated
    pub drift_seismic: DriftDirectionalParams,     // updated
    pub displacement_wind: DisplacementDirectionalParams, // updated
    pub torsional: Option<TorsionalParams>,        // NEW, optional
    pub pier_shear_stress_wind: Option<PierShearStressParams>,   // NEW, replaces pier_shear_wind
    pub pier_shear_stress_seismic: Option<PierShearStressParams>, // NEW, replaces pier_shear_seismic
    pub pier_axial_stress: Option<PierAxialStressParams>,  // NEW, replaces pier_axial
    pub check_selection: CheckSelection,
    pub unit_context: UnitContext,
}

// Param structs:

pub struct BaseReactionsParams {           // renamed from BaseShearParams
    pub elf_case_x: String,
    pub elf_case_y: String,
    pub rsa_case_x: String,
    pub rsa_case_y: String,
    pub rsa_scale_min: f64,
}

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
    pub joint_a: String,                 // near-edge UniqueName
    pub joint_b: String,                 // far-edge UniqueName
}

pub struct TorsionalParams {
    pub x_cases: Vec<String>,
    pub y_cases: Vec<String>,
    pub x_pairs: Vec<TorsionalJointPair>,
    pub y_pairs: Vec<TorsionalJointPair>,
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
            .chain(&self.wind_combos)
            .chain(&self.seismic_combos)
            .map(String::as_str)
    }
}
```

**Validation in `from_config()`:**

```rust
// Torsional: each joint pair must have exactly 2 names
for pair in &config.calc.torsional.x_joints {
    if pair.len() != 2 {
        bail!("[calc.torsional].x-joints: each pair must have exactly 2 joint names, found {}", pair.len());
    }
}
// Story forces: optional — None if both case lists empty
// Torsional: optional — None if not configured
// Pier shear stress: optional — None if combos empty
// Pier axial stress: optional — None if all combos empty
```

**Delete from `from_config()`:** All parsing for `pier_shear_wind`, `pier_shear_seismic`,
`pier_axial` (old single-list), `drift_tracking_groups`.

---

## Check Algorithms

### Check: Story forces (NEW)

**File:** `crates/ext-calc/src/checks/story_forces.rs`

```
inputs:
  rows: &[StoryForceRow]
  stories: &[StoryDefRow]
  params: &StoryForcesParams

For X-direction (VX chart + MY chart):
  filter: output_case in params.x_cases AND location == "Bottom"
  reason: "Bottom" location in ETABS story forces = cumulative shear above that story
  per story: max_vx   = max(|vx_kip|) across all matching rows and all steps
             max_my   = max(|my_kip_ft|) across all matching rows and all steps
             (MY is the overturning moment about Y-axis, paired with X shear)

For Y-direction (VY chart + MX chart):
  filter: output_case in params.y_cases AND location == "Bottom"
  per story: max_vy   = max(|vy_kip|) across all matching rows and all steps
             max_mx   = max(|mx_kip_ft|) across all matching rows and all steps

Sort rows by story elevation descending (top story first — matches chart orientation).
Return StoryForcesOutput { rows }

No pass/fail. This is a review chart, not a code check.
```

### Check: Wind drift and displacement (UPDATED)

**File:** `crates/ext-calc/src/checks/drift_wind.rs`

Add `DriftDirection` enum and direction-aware governing:

```rust
pub enum DriftDirection { X, Y }
```

In `build_drift_output`, change the governing selection `candidates` to use only
the relevant direction's columns:

```rust
// For DriftDirection::X:
let candidates = [
    ("X", "positive", row.max_drift_x_pos.abs()),
    ("X", "negative", row.max_drift_x_neg.abs()),
];

// For DriftDirection::Y:
let candidates = [
    ("Y", "positive", row.max_drift_y_pos.abs()),
    ("Y", "negative", row.max_drift_y_neg.abs()),
];
```

All other logic (grouping by story/group/case, enveloping all steps) is unchanged.
The `DriftEnvelopeRow` still stores all 4 drift columns — direction only affects governing.

`run()` calls this twice:

```rust
pub fn run(rows: &[JointDriftRow], stories: &[StoryDefRow],
           group_map: &HashMap<String, Vec<String>>,
           params: &CodeParams) -> Result<DriftWindOutput> {
    Ok(DriftWindOutput {
        x: build_drift_output_directional(rows, stories, group_map,
               &params.joint_tracking_groups,
               &params.drift_wind.x_cases,
               params.drift_wind.drift_limit,
               DriftDirection::X)?,
        y: build_drift_output_directional(rows, stories, group_map,
               &params.joint_tracking_groups,
               &params.drift_wind.y_cases,
               params.drift_wind.drift_limit,
               DriftDirection::Y)?,
    })
}
```

`drift_seismic.rs` follows the same pattern with `params.drift_seismic.*`.
`displacement_wind.rs` follows the same pattern with `params.displacement_wind.*`,
governing on `max_disp_x_*` or `max_disp_y_*` per direction.

### Check: Torsional (NEW — verified from Excel formulas)

**File:** `crates/ext-calc/src/checks/torsional.rs`

**Source of truth:** Excel formulas from the spreadsheet, decoded:

```
For X-direction torsion, joints 47 (joint_a) and 50 (joint_b):

Columns in Excel:
  E, F, G  = building displacement of joint_a at steps 1, 2, 3  [in]
  K, L, M  = building displacement of joint_b at steps 1, 2, 3  [in]

  Story drift of joint_a at each step:
    H = E_row - E_row+1   (step 1)
    I = F_row - F_row+1   (step 2)
    J = G_row - G_row+1   (step 3)

  Story drift of joint_b at each step:
    N = K_row - K_row+1   (step 1)
    O = L_row - L_row+1   (step 2)
    P = M_row - M_row+1   (step 3)

Per step [s ∈ {1, 2, 3}]:
  delta_max[s] = MAX(drift_a[s], drift_b[s])   ← max between the two joints
  delta_avg[s] = AVG(drift_a[s], drift_b[s])   ← average between the two joints

RATIO  = MAX(delta_max[1]/delta_avg[1],
             delta_max[2]/delta_avg[2],
             delta_max[3]/delta_avg[3])

AX     = MAX(MIN(MAX((delta_max[1]/(1.2×delta_avg[1]))²,
                     (delta_max[2]/(1.2×delta_avg[2]))²,
                     (delta_max[3]/(1.2×delta_avg[3]))²),
                 3.0),
             1.0)

ECC    = ecc_ratio × building_dimension_ft
         (building dimension = |cg_x of joint_a - cg_x of joint_b| for X-direction
          — or use the plan dimension from story_definitions if available)

RHO    = IF(RATIO > 1.4, 1.3, 1.0)
```

**Key differences from my earlier spec — corrected here:**

1. The ratio uses **story drift** (difference between adjacent stories), NOT building displacement.
2. **All 3 steps are evaluated simultaneously** — not nominal vs eccentric separately. The ratio picks the worst step.
3. `AX` uses the max of all squared terms across steps, then clamps to [1.0, 3.0].
4. The `RHO` redundancy factor is binary: 1.3 if Type B (>1.4), else 1.0.

**Implementation:**

```
inputs:
  joint_drifts: &[JointDriftRow]
  stories: &[StoryDefRow]
  params: &TorsionalParams

story_order = story_order_lookup(stories)   ← reuse from drift_wind.rs

for each direction in [X, Y]:
  cases  = params.x_cases (or y_cases)
  pairs  = params.x_pairs (or y_pairs)
  disp_fn = |row: &JointDriftRow| row.disp_x_ft  (X-direction)
             |row: &JointDriftRow| row.disp_y_ft  (Y-direction)

  for each case in cases:
    filter rows: output_case == case

    for each pair (joint_a, joint_b) in pairs:

      // Collect building displacement per (story, step_number) for each joint
      // StepNumber values: 1.0, 2.0, 3.0  (already f64 in JointDriftRow)

      let a_disp: HashMap<(story, step), f64>
        = rows filtered by unique_name == joint_a
          group by (story, step_number) → disp value

      let b_disp: HashMap<(story, step), f64>
        = rows filtered by unique_name == joint_b
          group by (story, step_number) → disp value

      // Compute story drift = this_story_disp - next_story_disp (per step)
      // Sort stories bottom-up by elevation.
      let sorted_stories = sort by elevation ascending

      for each adjacent story pair (story_n, story_n+1):

        for step in [1.0, 2.0, 3.0]:
          drift_a[step] = |a_disp[(story_n, step)] - a_disp[(story_n+1, step)]|
          drift_b[step] = |b_disp[(story_n, step)] - b_disp[(story_n+1, step)]|

          delta_max[step] = f64::max(drift_a[step], drift_b[step])
          delta_avg[step] = (drift_a[step] + drift_b[step]) / 2.0

        // RATIO: max ratio across all steps
        ratio = [1, 2, 3].iter()
          .map(|s| if delta_avg[s] < 1e-9 { 1.0 } else { delta_max[s] / delta_avg[s] })
          .fold(0.0_f64, f64::max)

        // AX: max squared term across all steps, clamped to [1.0, 3.0]
        let max_sq = [1, 2, 3].iter()
          .map(|s| {
            if delta_avg[s] < 1e-9 { 0.0 }
            else { (delta_max[s] / (1.2 * delta_avg[s])).powi(2) }
          })
          .fold(0.0_f64, f64::max);
        ax = max_sq.min(3.0).max(1.0)

        ecc_ft = params.ecc_ratio × building_dimension_ft
        rho    = if ratio > 1.4 { 1.3 } else { 1.0 }

        push TorsionalRow {
          story: story_n,
          case, joint_a, joint_b,
          drift_a_steps: [drift_a[1], drift_a[2], drift_a[3]],
          drift_b_steps: [drift_b[1], drift_b[2], drift_b[3]],
          delta_max_steps: [delta_max[1], delta_max[2], delta_max[3]],
          delta_avg_steps: [delta_avg[1], delta_avg[2], delta_avg[3]],
          ratio, ax, ecc_ft, rho,
          is_type_a: ratio > 1.2,
          is_type_b: ratio > 1.4,
        }

  sort all rows by story elevation descending
  governing = row with max ratio
  has_type_a = any row.is_type_a
  has_type_b = any row.is_type_b

pass = !x.has_type_b && !y.has_type_b

Building dimension for ecc_ft:
  Use the absolute distance between joint_a and joint_b in the relevant plan axis.
  For X-direction torsion: building dimension in X = |cg_x_joint_a - cg_x_joint_b|
  Joint plan coordinates are not currently in joint_drifts. Two options:
  Option A (preferred): add building_dimension_x/y to TorsionalCalcConfig as explicit values.
  Option B: derive from joint coordinates in a separate joints geometry table (future).
  For now, use Option A — add optional config fields:
    [calc.torsional]
    building-dim-x = 96.0   # ft — used for ECC column only
    building-dim-y = 56.0   # ft
  If not configured, ECC = 0 (omit the column from output rather than error).
```

**Note on step numbers:** The ETABS joint drifts parquet has `StepNumber` as `Option<f64>`.
For ELF cases with accidental eccentricity, steps are 1.0, 2.0, 3.0. Filter by `step_number == Some(s)`.
If a case has only one step (static, no eccentricity), treat it as step 1 only, skip steps 2 and 3.

### Check: Pier shear stress (NEW — replaces capacity check)

**File:** `crates/ext-calc/src/checks/pier_shear_stress.rs`

**Verified formulas from Excel:**

```
Per pier per story:
  Acw [in²]  = Thickness_bottom [in] × Width_bottom [in]
               Note: pier_section_properties.parquet stores widths in FEET.
               Rust code: acw_in2 = width_bot_ft × thick_bot_ft × 144.0
               (Already computed as PierSectionRow.acw_in2 — no change needed)

  f'c [psi]  = from fc_map[(pier, story)] × 1000

  sqrt_fc    = f'c_psi.sqrt()
               (The Excel MIN(100×√f'c, √f'c) simplifies to √f'c for normal concrete;
                100×√6000 = 7746 > 77 = √6000. The MIN is a dead branch here.)

  Ve [kip]   = Vu from pier_forces — envelope across all combos and locations
               (The Excel uses a scale factor Input.E7 = 1.0, so Ve = Vu)

  stress_psi = Ve [kip] × 1000 / (phi_v × Acw [in²])

  stress_ratio = stress_psi / sqrt_fc   [n × √f'c, dimensionless]

  limit_individual = 8.0
  pass = stress_ratio <= 8.0

Wall direction from AxisAngle:
  PX piers (AxisAngle = 0°)  → wall_direction = "X"
  PY piers (AxisAngle = 90°) → wall_direction = "Y"
  More precisely:
    |axis_angle| < 15° or |axis_angle - 180°| < 15° → "X"
    |axis_angle - 90°| < 15° or |axis_angle - 270°| < 15° → "Y"
  Piers outside these bands → skip with eprintln! warning

Average per direction per story:
  For each direction in [X, Y]:
    per story:
      sum_ve  = sum of Ve_kip for all piers of this direction at this story
      sum_acw = sum of Acw_in2 for all piers of this direction at this story
      fc_psi  = fc_psi of any pier in this group at this story
                (all should be same concrete grade — use first encountered)
      sqrt_fc = fc_psi.sqrt()
      avg_stress = sum_ve × 1000 / (phi_v × sum_acw)
      avg_ratio  = avg_stress / sqrt_fc
      limit_average = 10.0
      pass = avg_ratio <= 10.0
```

**Algorithm:**

```
step 1 — envelope Ve per (story, pier):
  filter: output_case in params.combos
  per (story, pier): Ve = max(shear_v2_abs_kip) across all matching rows

step 2 — per-pier stress ratio:
  build section_map: (pier, story) → PierSectionRow
  for each (story, pier, Ve) in grouped:
    section   = section_map[(pier, story)]
    fc_ksi    = fc_map[(pier, story)] or fc_default_ksi
    fc_psi    = fc_ksi × 1000.0
    acw       = section.acw_in2
    sqrt_fc   = fc_psi.sqrt()
    direction = classify from section.axis_angle
    stress    = Ve × 1000.0 / (phi_v × acw)
    ratio     = stress / sqrt_fc
    pass      = ratio <= 8.0
    push PierShearStressRow

step 3 — average per direction per story:
  group per_pier rows by (wall_direction, story)
  per group:
    sum_ve, sum_acw = sum across piers
    fc_psi = from first pier in group
    avg_ratio = (sum_ve × 1000 / (phi_v × sum_acw)) / fc_psi.sqrt()
    pass = avg_ratio <= 10.0
  sort by story elevation desc

step 4 — overall pass:
  pass = all per_pier.pass AND all x_average.pass AND all y_average.pass
```

### Check: Pier axial stress (UPDATED — 3 categories)

**File:** `crates/ext-calc/src/checks/pier_axial.rs`

Add `run_stress()` alongside existing `run()` (keep `run()` for backward compat with tests
until old pier_axial config is removed):

```rust
pub fn run_stress(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    fc_map: &HashMap<(String, String), f64>,
    params: &CodeParams,
) -> Result<PierAxialStressOutput> {
    let axial_params = params.pier_axial_stress.as_ref()
        .ok_or_else(|| anyhow::anyhow!("pier-axial-stress not configured"))?;

    // Run the same demand-capacity logic three times, once per category.
    // Reuse the existing formula: Pu / (phi * 0.85 * fc * Ag)
    let gravity = run_for_combos(forces, sections, fc_map,
                                 &axial_params.gravity_combos, axial_params.phi_axial,
                                 params)?;
    let wind    = run_for_combos(forces, sections, fc_map,
                                 &axial_params.wind_combos, axial_params.phi_axial,
                                 params)?;
    let seismic = run_for_combos(forces, sections, fc_map,
                                 &axial_params.seismic_combos, axial_params.phi_axial,
                                 params)?;

    // Combine all results
    let mut all_piers = Vec::new();
    all_piers.extend(gravity.clone());
    all_piers.extend(wind.clone());
    all_piers.extend(seismic.clone());

    let governing = all_piers.iter()
        .max_by(|a, b| a.dcr.partial_cmp(&b.dcr).unwrap())
        .cloned()
        .expect("at least one result");

    Ok(PierAxialStressOutput {
        phi_axial: axial_params.phi_axial,
        piers: all_piers,
        governing_gravity: gravity.iter().max_by(|a,b| a.dcr.partial_cmp(&b.dcr).unwrap()).cloned(),
        governing_wind:    wind.iter().max_by(|a,b| a.dcr.partial_cmp(&b.dcr).unwrap()).cloned(),
        governing_seismic: seismic.iter().max_by(|a,b| a.dcr.partial_cmp(&b.dcr).unwrap()).cloned(),
        governing,
        pass: all_piers.iter().all(|r| r.pass),
    })
}

// Private helper — existing run() logic extracted into this, parameterized by combo list
fn run_for_combos(
    forces: &[PierForceRow],
    sections: &[PierSectionRow],
    fc_map: &HashMap<(String, String), f64>,
    combos: &[String],
    phi_axial: f64,
    params: &CodeParams,
) -> Result<Vec<PierAxialResult>> { ... }
```

---

## ext-calc: lib.rs Wiring

**File:** `crates/ext-calc/src/lib.rs`

**Remove from `run_all()`:**
- `pier_shear_wind`, `pier_shear_seismic` calls and their `build_pier_fc_map` usage

**Add/update:**
```rust
// Build fc_map once — shared by stress check and axial check
let pier_fc_map = checks::pier_shear_stress::build_pier_fc_map(
    &pier_sections,
    &material_props,
    // use fc_default from whichever stress config is present
    params.pier_shear_stress_seismic
        .as_ref().map(|p| p.fc_default_ksi)
        .or(params.pier_shear_stress_wind.as_ref().map(|p| p.fc_default_ksi))
        .unwrap_or(8.0),
);

let story_forces_output = if params.story_forces.is_some() {
    Some(checks::story_forces::run(&story_forces, &story_defs, params)?)
} else { None };

let drift_wind_output = if params.check_selection.drift_wind {
    Some(checks::drift_wind::run(&joint_drifts, &story_defs, &group_map, params)?)
} else { None };

let drift_seismic_output = if params.check_selection.drift_seismic {
    Some(checks::drift_seismic::run(&joint_drifts, &story_defs, &group_map, params)?)
} else { None };

let displacement_wind_output = if params.check_selection.displacement_wind {
    Some(checks::displacement_wind::run(&joint_drifts, &story_defs, &group_map, params)?)
} else { None };

let torsional_output = if params.torsional.is_some() {
    Some(checks::torsional::run(&joint_drifts, &story_defs, params)?)
} else { None };

let pier_shear_stress_wind_output = if params.pier_shear_stress_wind.is_some() {
    Some(checks::pier_shear_stress::run(
        &pier_forces, &pier_sections, &pier_fc_map, params, "wind")?)
} else { None };

let pier_shear_stress_seismic_output = if params.pier_shear_stress_seismic.is_some() {
    Some(checks::pier_shear_stress::run(
        &pier_forces, &pier_sections, &pier_fc_map, params, "seismic")?)
} else { None };

let pier_axial_stress_output = if params.pier_axial_stress.is_some() {
    Some(checks::pier_axial::run_stress(
        &pier_forces, &pier_sections, &pier_fc_map, params)?)
} else { None };
```

**Update `build_summary()`:**

| Old key | New key | Change |
|---------|---------|--------|
| `baseShear` | `baseReaction` | rename |
| `driftWind` | `driftWindX`, `driftWindY` | split |
| `driftSeismic` | `driftSeismicX`, `driftSeismicY` | split |
| `displacementWind` | `displacementWindX`, `displacementWindY` | split |
| `pierShearWind` | `pierShearStressWind` | replace |
| `pierShearSeismic` | `pierShearStressSeismic` | replace |
| `pierAxial` | `pierAxialStress` | replace |
| — | `storyForces` | new (always `"loaded"`, no pass/fail) |
| — | `torsionalX`, `torsionalY` | new (`"warn"` if Type A, `"fail"` if Type B) |

---

## CheckSelection Update

**File:** `crates/ext-calc/src/checks/mod.rs`

```rust
pub struct CheckSelection {
    pub modal: bool,
    pub base_reactions: bool,          // renamed from base_shear
    pub story_forces: bool,            // NEW
    pub drift_wind: bool,
    pub drift_seismic: bool,
    pub displacement_wind: bool,
    pub torsional: bool,               // default false (requires explicit config)
    pub pier_shear_stress_wind: bool,  // replaces pier_shear_wind
    pub pier_shear_stress_seismic: bool, // replaces pier_shear_seismic
    pub pier_axial_stress: bool,       // replaces pier_axial
}

impl Default for CheckSelection {
    fn default() -> Self {
        Self {
            modal: true,
            base_reactions: true,
            story_forces: true,          // auto-enable if cases configured
            drift_wind: true,
            drift_seismic: true,
            displacement_wind: true,
            torsional: false,            // opt-in
            pier_shear_stress_wind: true,
            pier_shear_stress_seismic: true,
            pier_axial_stress: true,
        }
    }
}
```

**Delete from mod.rs:** `pub mod pier_shear_wind; pub mod pier_shear_seismic;`

---

## ext-render: Chart Constants

**File:** `crates/ext-render/src/lib.rs`

**Remove:**
```rust
// DELETE:
pub const DRIFT_WIND_IMAGE: ...
pub const DRIFT_SEISMIC_IMAGE: ...
pub const DISPLACEMENT_WIND_IMAGE: ...
pub const PIER_AXIAL_IMAGE: ...
// Also delete: PIER_SHEAR_WIND_IMAGE, PIER_SHEAR_SEISMIC_IMAGE
```

**Add:**
```rust
// Story forces — 4 charts
pub const STORY_FORCE_VX_IMAGE: &str = "images/story_force_vx.svg";
pub const STORY_FORCE_MY_IMAGE: &str = "images/story_force_my.svg";
pub const STORY_FORCE_VY_IMAGE: &str = "images/story_force_vy.svg";
pub const STORY_FORCE_MX_IMAGE: &str = "images/story_force_mx.svg";

// Drift — 2 per check
pub const DRIFT_WIND_X_IMAGE: &str = "images/drift_wind_x.svg";
pub const DRIFT_WIND_Y_IMAGE: &str = "images/drift_wind_y.svg";
pub const DRIFT_SEISMIC_X_IMAGE: &str = "images/drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_IMAGE: &str = "images/drift_seismic_y.svg";

// Displacement — 2
pub const DISP_WIND_X_IMAGE: &str = "images/disp_wind_x.svg";
pub const DISP_WIND_Y_IMAGE: &str = "images/disp_wind_y.svg";

// Torsional — 2
pub const TORSIONAL_X_IMAGE: &str = "images/torsional_x.svg";
pub const TORSIONAL_Y_IMAGE: &str = "images/torsional_y.svg";

// Pier shear stress — 2
pub const PIER_SHEAR_STRESS_WIND_IMAGE: &str = "images/pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEISMIC_IMAGE: &str = "images/pier_shear_stress_seismic.svg";

// Pier axial — 3 charts (by load category)
pub const PIER_AXIAL_GRAVITY_IMAGE: &str = "images/pier_axial_gravity.svg";
pub const PIER_AXIAL_WIND_IMAGE: &str = "images/pier_axial_wind.svg";
pub const PIER_AXIAL_SEISMIC_IMAGE: &str = "images/pier_axial_seismic.svg";
```

**Chart orientations** (all use story on the vertical axis, value on horizontal — matches
engineering convention for profile plots):
- Story forces: horizontal line/bar chart, story (top→bottom) on Y axis
- Drift: horizontal line chart, drift ratio on X, story on Y, reference limit line
- Torsional: line chart, ratio on X, story on Y, reference lines at 1.2 and 1.4
- Pier shear stress: multi-line chart per pier + bold average line, 8.0 and 10.0 limits
- Pier axial: DCR per pier per story, story on Y, limit line at DCR = 1.0

---

## Report: New Sections

**File:** `crates/ext-report/src/report_document.rs`

Report page order (add after existing modal and base reactions pages):

| Page | Type | Content |
|------|------|---------|
| Story forces X | TwoChartsPage | VX shear profile + MY overturning profile |
| Story forces Y | TwoChartsPage | VY shear profile + MX overturning profile |
| Wind drift X | ChartAndTablePage | drift_wind.x chart + governing table |
| Wind drift Y | ChartAndTablePage | drift_wind.y chart + governing table |
| Seismic drift X | ChartAndTablePage | drift_seismic.x chart + table |
| Seismic drift Y | ChartAndTablePage | drift_seismic.y chart + table |
| Wind displacement X | ChartAndTablePage | disp_wind.x chart + table |
| Wind displacement Y | ChartAndTablePage | disp_wind.y chart + table |
| Torsional X | ChartAndTablePage | ratio chart + Ax/Type table |
| Torsional Y | ChartAndTablePage | ratio chart + Ax/Type table |
| Pier shear stress wind | SingleChartPage | per-pier + avg stress profile |
| Pier shear stress seismic | SingleChartPage | per-pier + avg stress profile |
| Pier axial gravity | ChartAndTablePage | DCR chart + governing table |
| Pier axial wind | ChartAndTablePage | DCR chart + governing table |
| Pier axial seismic | ChartAndTablePage | DCR chart + governing table |
| Pier axial assumptions | CalculationPage | conservative capacity note |

---

## Implementation Order

Work in this exact sequence to maintain a compiling project at each step:

1. **`ext-db/config/calc.rs`** — add/rename all structs. Delete old ones.
   Compile check: zero errors before moving on.

2. **`ext-calc/output.rs`** — add/rename all output types. Delete removed types.
   Compile check: fix all references to deleted types across the codebase.

3. **`ext-calc/code_params.rs`** — update `CodeParams`, all param structs, `from_config()`.
   Delete old validation code.

4. **`ext-calc/checks/drift_wind.rs`** — add `DriftDirection`, split `run()`.
   Update `drift_seismic.rs` and `displacement_wind.rs` with the same pattern.

5. **`ext-calc/checks/story_forces.rs`** — new file.

6. **`ext-calc/checks/torsional.rs`** — new file.

7. **`ext-calc/checks/pier_shear_stress.rs`** — new file.
   Move `build_pier_fc_map()` here from `pier_shear.rs` (it is now shared only with axial).

8. **`ext-calc/checks/pier_axial.rs`** — add `run_stress()`, keep old `run()` for tests.

9. **`ext-calc/checks/mod.rs`** — register new modules. Remove pier_shear_wind/seismic.
   Delete `pier_shear_wind.rs` and `pier_shear_seismic.rs` entirely.

10. **`ext-calc/lib.rs`** — wire all new checks. Update `build_summary()`.

11. **`ext-render/src/lib.rs`** — update chart constants. Add render functions for new charts.

12. **`ext-report/report_document.rs`** — add new report sections.

---

## Invariants

- `escape_text()` in `ext-report/src/pdf/template.rs` MUST escape `*` and `_`.
  Do this BEFORE any report work. Load case names like `DBE_X*Cd/R` crash typst.
- `build_pier_fc_map()` moves from `pier_shear.rs` to `pier_shear_stress.rs`.
  Axial check imports it from there.
- Story ordering always uses `sort_rows_by_story()` from `checks/drift_wind.rs`.
- All new checks return `None` when not configured — no panics, no bail on missing optional config.
- Unit conversion uses `UnitContext` — no hardcoded unit strings in output struct fields.
- The `--new-instance` hang fix (3s process exit timeout in `sidecar/client.rs`) is a
  pre-existing bug blocking the full test. Fix it before integration testing.