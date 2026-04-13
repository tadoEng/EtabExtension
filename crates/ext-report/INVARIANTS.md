# Critical Invariants for ext-report

This document codifies invariants that must be preserved when modifying report generation, PDF templates, Excel sheets, or check output formatting. Violations of these constraints can cause crashes, silent data loss, or incorrect results.

---

## 1. Typst Character Escaping (CRITICAL — Blocks PDF Reports)

**File:** `src/pdf/template.rs::escape_text()`

**Current Status:** ⚠️ **INCOMPLETE** — Missing `*` and `_` escaping.

**Issue:** ETABS load case names like `DBE_X*Cd/R` contain special characters that Typst interprets as markup:
- `_` triggers subscript mode (e.g., `DBE_X` becomes subscript if unescaped)
- `*` triggers bold mode or emphasis

Typst will **crash compilation** if these characters appear unescaped in content text.

### Fix Required

**Before expanding check output or report sections, `escape_text()` MUST escape `*` and `_`:**

```rust
pub(crate) fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('"', "\\\"")
        .replace('@', "\\@")
        .replace('*', "\\*")        // NEW
        .replace('_', "\\_")        // NEW
}
```

### Characters Currently Escaped
- `\` → `\\` (backslash)
- `[`, `]` → `\[`, `\]` (brackets)
- `#` → `\#` (hash)
- `"` → `\"` (quote)
- `@` → `\@` (at sign)

### Characters to Add
- `*` → `\*` (asterisk — Typst emphasis/bold)
- `_` → `\_` (underscore — Typst subscript)

### Test Case
```rust
assert_eq!(
    escape_text("DBE_X*Cd/R"),
    "DBE\\_X\\*Cd/R"
);
```

---

## 2. All New Checks Are Opt-In

**Scope:** Any check output added to `CalcOutput` in `ext-calc`.

**Invariant:** If a check is not enabled in config, its output struct field MUST be `None` in `CalcOutput`, not an empty struct or error status.

**Why:** Report sections (PDF and Excel) treat `None` as "skip this sheet entirely". An empty or error struct would produce incomplete or confusing output.

**Implementation Pattern:**

```rust
// In ext-calc/src/lib.rs or check module
if config.checks.drift_enabled {
    drift_output = Some(drift::compute(...)?);
} else {
    drift_output = None;  // ← Opt-in: disabled = None
}
```

**In ext-report PDF/Excel sheets:**
```rust
if let Some(drift) = &calc.drift_output {
    render_drift_sheet(drift)?;
    // else: no sheet created
}
```

---

## 3. Story Ordering Has a Single Source of Truth

**Source:** `crates/ext-calc/src/checks/drift_wind.rs::sort_rows_by_story()`

**Invariant:** ALL report sections that display story-indexed data (drift, displacement, pier checks) MUST use the exact same sort order.

**Why:** Users need consistent story ordering across all sheets for visual scanning and cross-referencing. Reordering differently in pier sheets vs. drift sheets causes confusion and manual auditing overhead.

**When Adding New Checks:**
1. Extract story list from the check output
2. Call `drift_wind::sort_rows_by_story(&mut rows)` to sort in-place
3. Never reimplement sorting inline (different sort criteria = bugs)

**Current Implementation:**
```rust
// In ext-report/src/excel/drift_wind.rs
let mut rows = drift_output.data.clone();
ext_calc::checks::drift_wind::sort_rows_by_story(&mut rows);
// Now write rows to Excel in their sorted order
```

---

## 4. Pier fc_map Is Built Once, Shared Across Four Checks

**Source:** `crates/ext-calc/src/checks/pier_shear.rs::build_pier_fc_map()`

**Invariant:** The `fc_map` (mapping pier/story/combo → concrete strength) MUST be built exactly once and shared via the same reference across:
1. Pier Shear Wind
2. Pier Shear Seismic
3. Pier Shear Stress Wind
4. Pier Shear Stress Seismic
5. Pier Axial

**Why:** `fc_map` is computed from ETABS model geometry and material properties—it's expensive to rebuild. More importantly, all four checks must use **identical `fc'` values** for each pier/story. Rebuilding separately can introduce inconsistencies (e.g., if rebar lookup logic differs).

**Current Implementation:**

```rust
// In ext-calc/src/lib.rs
let fc_map = pier_shear::build_pier_fc_map(&model)?;

let pier_shear_wind = pier_shear::compute_wind(&model, &fc_map)?;
let pier_shear_seismic = pier_shear::compute_seismic(&model, &fc_map)?;
let pier_shear_stress_wind = pier_shear_stress::compute_wind(&model, &fc_map)?;
let pier_shear_stress_seismic = pier_shear_stress::compute_seismic(&model, &fc_map)?;
let pier_axial = pier_axial::compute(&model, &fc_map)?;
```

**Red Flag:** If you see separate `build_fc_map()` calls for different checks, consolidate back to single shared instance.

---

## 5. Unit Conversion Uses UnitContext

**Scope:** All output values in `CalcOutput`, Excel sheets, and PDF text.

**Invariant:** Output values are pre-computed in display units by `ext-calc`. Report generation MUST NOT hardcode unit strings or scale factors. Instead, use `UnitContext` from the config.

**Why:** If a project switches from imperial (in, ksi) to metric (mm, MPa), the values in `CalcOutput` are already correct. The unit labels must also flip. Hardcoding "ksi" in template strings causes misalignment.

**Current Implementation:**

```rust
// In ext-report Excel sheet headers:
let units = &calc.config.units;  // UnitContext from config
let header = format!("DCR ({})", units.force);  // → "DCR (ksi)" or "DCR (MPa)"

// In PDF template:
let force_unit = escape_text(&calc.config.units.force);
format!("Shear Force ({})", force_unit)
```

**Red Flag:** Any `"ksi"`, `"in"`, `"pounds"` string literal in report code that isn't part of a comment = wrong approach.

---

## 6. Pre-Existing Bugs Blocking Report Expansion

### Bug A: Typst Character Escaping

**Status:** ⚠️ **UNFIXED** — Blocks any report section using ETABS identifiers with `*` or `_`

**Details:** See [Section 1: Typst Character Escaping](#1-typst-character-escaping-critical--blocks-pdf-reports)

**Impact:** Load case names, pier labels, group names, or any user-entered text with `_` or `*` will crash `typst compile`.

**Fix:** Add two `.replace()` calls to `escape_text()` (see above).

### Bug B: Sidecar Process Exit Timeout

**Status:** ⚠️ **UNFIXED** — Blocks reliable opening of ETABS files from reports

**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelService.cs`

**Details:** Mode B (new ETABS instance) returns from sidecar slowly (~3+ seconds) because:
- Sidecar client awaits `read_to_string()` on the ETABS process
- ETABS process exit is slow due to COM RCW finalizer attempting cross-process teardown
- Finalizer runs in sidecar's thread, blocking return to Rust CLI

**Fix:** Replace empty finally block with `Marshal.ReleaseComObject(app)` to bypass Dispose and force immediate RCW cleanup.

**Impact:** Users see 3-5 second delay when opening new ETABS instances; feels like app is frozen.

**Priority:** Fix Bug A (Typst escaping) before adding any new check output. Fix Bug B (process exit) before shipping to users.

---

## 7. Report Architecture Decisions

### PDF is Typst-compiled, not embedded

- `ext-report` generates `.typ` source and calls `typst compile` CLI
- User must have Typst installed
- Future: embed Typst library when public Rust API stabilizes
- SVG assets are written to `assets/` subdirectory adjacent to `.typ` file

### Excel uses pre-computed values, no formulas

- Values come directly from `CalcOutput`, computed by `ext-calc`
- No live Excel formulas — avoids divergence from Rust spec
- Each sheet header cites code section and config values for audit trail
- Engineers modify parameters in config and re-run Rust tool, not Excel formulas

### Page format: Tabloid landscape (17"×11")

- Baseline from Week 7-8 specification
- Older A4 examples should not be used for chart sizing or page breaks
- Desktop preview must target tabloid dimensions

---

## 8. Checklist for New Check Integration

Before adding a new check output to reports:

- [ ] Check output added to `ext-calc/src/lib.rs::CalcOutput` as `Option<T>`
- [ ] Check is disabled if config flag is false (produces `None`)
- [ ] All story-indexed rows sorted via `drift_wind::sort_rows_by_story()`
- [ ] Unit labels derived from `config.units`, not hardcoded
- [ ] If used in PDF, all user-facing strings passed through `escape_text()`
- [ ] If pier-related, uses shared `fc_map` from `pier_shear::build_pier_fc_map()`
- [ ] Excel sheet added with standardized header (title, code ref, parameters)
- [ ] PDF section defined and included only if `Some`
- [ ] Typst `escape_text()` updated to handle `*` and `_` (BLOCKING until fixed)

---

## References

- **Rust code:** `crates/ext-report/src/pdf/template.rs`
- **Escape test:** `crates/ext-report/src/pdf/template.rs::tests`
- **Unit context:** `crates/ext-calc/src/config.rs::UnitContext`
- **Story sort:** `crates/ext-calc/src/checks/drift_wind.rs::sort_rows_by_story()`
- **Pier fc_map:** `crates/ext-calc/src/checks/pier_shear.rs::build_pier_fc_map()`
