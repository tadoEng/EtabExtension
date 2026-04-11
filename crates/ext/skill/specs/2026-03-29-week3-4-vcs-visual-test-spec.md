# Week 3–4 VCS Visual Test Spec
# ETABS Extension — Current Manual Validation With Real Sidecar

**Date:** 2026-04-11
**Phase:** Phase 1, Weeks 3–4 plus ETABS lifecycle fix
**Status:** Active
**Author:** EtabExtension Team

---

## Overview

Manual validation sheet for the real ETABS flow after the lifecycle fix applied
in this sprint. Key behavioral changes since the previous spec version:

- `ext etabs open` now **attaches to an already-running ETABS** (Mode A) by default
- `ext etabs open --new-instance` launches a new visible ETABS window (Mode B)
- Mode B no longer kills ETABS when the sidecar exits — the COM proxy is left
  for GC; ETABS stays running independently
- `ext etabs open` (without `--new-instance`) fails fast with a clear error if
  no ETABS instance is running, instructing the user to start one or add `--new-instance`
- The preflight guard now distinguishes ext-managed ETABS from out-of-band ETABS
  with separate error messages
- `extract-results` merges user table filters over the full default table set;
  partial `[extract.tables.*]` config no longer disables other required parquet outputs
- Materials extraction writes `{table_slug}.parquet`, not `takeoff.parquet`
- `ext calc` still requires explicit `[calc]` config; `ext init` does not
  generate defaults for those values

Primary implementation reference: `2026-03-29-week3-4-vcs-spec.md`.

---

## Recommended Environment

- Rust CLI repo:     `D:\Work\EtabExtension`
- Sidecar repo:      `D:\Work\EtabExtension.CLI`
- Preferred sidecar: `D:\Work\EtabExtension.CLI\dist\etab-cli-x86_64-pc-windows-msvc.exe`
- Debug sidecar:     `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\bin\Debug\net10.0\win-x64\etab-cli.exe`
- Workspace root:    `D:\Work\EtabExtension\proofs\week3-4-visual-pass`
- Real ETABS model:  use any `.edb` that opens successfully in the installed ETABS instance

Before the pass:

```powershell
cd D:\Work\EtabExtension
cargo build -p ext

cd D:\Work\EtabExtension.CLI
dotnet build .\src\EtabExtension.CLI\EtabExtension.CLI.csproj
```

---

## Config Setup

### 1. Clean the workspace

Delete or empty `D:\Work\EtabExtension\proofs\week3-4-visual-pass`.

### 2. Initialize the project

```powershell
.\target\debug\ext.exe init "Project Test" `
  --edb "D:\Path\To\RealModel.edb" `
  --path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" `
  --author "tester" `
  --email "tester@example.com"
```

Expect:
- `.etabs-ext\` created
- `.etabs-ext\main\working\model.edb` created
- hint suggests `ext commit "Initial model"`

### 3. Configure the real sidecar

Edit `.etabs-ext\config.local.toml`:

```toml
[project]
sidecar-path = "D:\\Work\\EtabExtension.CLI\\dist\\etab-cli-x86_64-pc-windows-msvc.exe"
units = "kip-ft-F"

[git]
author = "tester"
email = "tester@example.com"

[onedrive]
acknowledgedSync = true
```

### 4. Set the full project config

Replace the contents of `.etabs-ext\config.toml` with the following. All load
case and combo names are confirmed present in the test model.

```toml
[project]
name = "Project Test"

[extract]
units = "US_Kip_Ft"

# Leave [extract.tables] empty for the default pass — this extracts the full
# parquet set ext-calc consumes. Add per-table filters only after the
# baseline pass is healthy.
#
#[extract.tables.baseReactions]
#loadCases = ["DEAD", "LIVE"]
#loadCombos = ["COMB-ULS"]
#
#[extract.tables.groupAssignments]
#groups = ["Core"]
#
#[extract.tables.materialPropertiesConcreteData]
#fieldKeys = ["Fc", "Ec"]

[calc]
code = "ACI318-14"
occupancy-category = "II"
modal-case = "Modal (Rizt)"
drift-tracking-groups = ["Joint47", "Joint49", "Joint50", "Joint51"]

[calc.modal]
min-mass-participation = 0.9
display-mode-limit = 20

[calc.base-shear]
elf-case-x = "ELF_X"
elf-case-y = "ELF_Y"
rsa-case-x = "DBE_X"
rsa-case-y = "DBE_Y"
rsa-scale-min = 1.0

[[calc.base-shear.pie-groups]]
label = "Gravity"
load-cases = ["Dead", "SDL", "Live (red)", "Live (non-red)", "Live (roof)"]

[calc.drift-wind]
load-cases = ["W_10YRS", "W_10YRS"]
drift-limit = 0.0025

[calc.drift-seismic]
load-cases = ["DBE_X*Cd/R", "DBE_Y*Cd/R", "ELF_X_Drift*Cd/Ie", "ELF_Y_Drift*Cd/Ie"]
drift-limit = 0.02

[calc.displacement-wind]
load-cases = ["W_10YRS", "W_10YRS"]
disp-limit-h = 400

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

[calc.pier-axial]
load-combos = [
  "LC1: 1.4D",
  "LC2: 1.2D+1.6L",
  "LC3.1: 1.2D+0.5W",
  "LC3.2: 1.2D-0.5W",
  "LC4.1: 1.2D+1.0W+1.0L",
  "LC4.2: 1.2D+1.0W-1.0L",
  "LC6.1: 0.9D+1.0W",
  "LC6.2: 0.9D-1.0W",
  "DBE1: (1.2+0.2Sds)D+0.5L+100X+30Y",
  "DBE2: (1.2+0.2Sds)D+0.5L+100Y+30X",
  "DBE3: (0.9-0.2Sds)D+100X+30Y",
  "DBE4: (0.9-0.2Sds)D+100Y+30X",
]
phi-axial = 0.65

# Note: drift-tracking-groups must match names extracted into
# results/group_assignments.parquet after analysis.
```

### 5. Optional focused filter pass-through check

Run this only after the default baseline pass is healthy. Add to `config.toml`:

```toml
[extract.tables.groupAssignments]
groups = ["Core"]

[extract.tables.materialPropertiesConcreteData]
fieldKeys = ["Fc", "Ec"]

[extract.tables.materialListByStory]
fieldKeys = ["Story"]
```

These filters should narrow output for those three tables but must not suppress
the other seven required parquet outputs. Verify all ten files still appear in
`results\`.

---

## Quick Manual Tests

Run these first for the highest-signal checks:

1. Baseline `commit` creates `v1`, `model.e2k`, and `materials\material_list_by_story.parquet`
2. `ext etabs open --new-instance` opens the working file in a **new** ETABS
   window and prints a real PID. ETABS must stay open after the command returns.
3. `ext etabs open` (no flag) attaches to a **running** ETABS and switches it
   to the working file. Fails fast with a clear error if ETABS is not running.
4. `ext etabs open v1` shows the snapshot warning (`isSnapshot = true`)
5. `commit "Steel option" --analyze` creates `v2/results\` containing all ten
   required parquet files (see step 14)
6. `ext analyze steel-columns/v2` also works if `v2` exists from a prior
   non-analyzed commit
7. `ext log` hides internal `ext:` commits; raw `git log` still shows them

---

## Full Manual Flow

### 6. Baseline commit with real sidecar export

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" commit "Initial model"
```

Expect:
- success output for `v1`
- `.etabs-ext\main\v1\manifest.json`
- `.etabs-ext\main\v1\model.e2k`
- `.etabs-ext\main\v1\materials\material_list_by_story.parquet`

### 7. `show` and `log` spot-checks

```powershell
.\target\debug\ext.exe --project-path "..." show main/v1
.\target\debug\ext.exe --json --project-path "..." show main/v1
.\target\debug\ext.exe --json --project-path "..." log --branch main
```

Expect:
- `id = v1`, `branch = main`, `isAnalyzed = false`, `gitCommitHash` populated
- JSON shape stable

### 8. `etabs open --new-instance` flow (Mode B)

Start with no ETABS running.

```powershell
.\target\debug\ext.exe --project-path "..." etabs open --new-instance
.\target\debug\ext.exe --json --project-path "..." etabs status
```

Expect:
- A new ETABS window appears and **stays open** after the command returns
- Human output: `✓ ETABS opened (PID: <N>)`
- Opened file is `.etabs-ext\main\working\model.edb`
- JSON status:
  - `sidecarAvailable = true`
  - `isRunning = true`
  - `pid` populated
  - `workingFileStatus = "openClean"`

### 9. `etabs open` Mode A flow (attach to running ETABS)

Close the Mode B ETABS from step 8 first, then open ETABS manually.

```powershell
# With ETABS already running:
.\target\debug\ext.exe --project-path "..." etabs open
.\target\debug\ext.exe --json --project-path "..." etabs status
```

Expect:
- ETABS switches to the working file without launching a new window
- Human output: `✓ ETABS opened (PID: <N>)`
- JSON status: `isRunning = true`, `workingFileStatus = "openClean"`

Negative test — run with no ETABS open:

```powershell
.\target\debug\ext.exe --project-path "..." etabs open
```

Expect:
- Error: `✗ ETABS is not running. Start ETABS first, or use --new-instance to launch one.`
- No crash, clean exit

### 10. Preflight guard — out-of-band ETABS

Open ETABS manually without going through `ext`. Then run:

```powershell
.\target\debug\ext.exe --project-path "..." etabs open --new-instance
```

Expect:
- Error: `✗ ETABS is already running (started outside ext)\n  Close ETABS manually and try again`
- **Not** `ext etabs close` (which would not work for out-of-band instances)

### 11. `etabs open` snapshot flow

```powershell
.\target\debug\ext.exe --project-path "..." etabs close --no-save
.\target\debug\ext.exe --json --project-path "..." etabs open v1
```

Expect:
- Open succeeds
- JSON: `isSnapshot = true`, `warning = "Opening a snapshot — changes will be discarded"`, `pid` populated

### 12. Branch create + switch

```powershell
.\target\debug\ext.exe --project-path "..." etabs close --no-save
.\target\debug\ext.exe --project-path "..." branch steel-columns --from main/v1
.\target\debug\ext.exe --project-path "..." switch steel-columns
```

Expect:
- Branch creation succeeds
- Working path becomes `.etabs-ext\steel-columns\working\model.edb`
- Active branch becomes `steel-columns`

### 13. Real ETABS edit on branch working model

Open the branch working file directly in ETABS:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\steel-columns\working\model.edb`

Make one deterministic change, save, keep ETABS open. Then:

```powershell
.\target\debug\ext.exe --project-path "..." status
```

Expect:
- Status reflects an open model state (`openModified` after save)

### 14. Close prompt and modified-state landing

```powershell
.\target\debug\ext.exe --project-path "..." etabs close
```

If prompted (`OpenModified` + `Interactive` mode), choose save.

Expect:
- Interactive prompt appears when file has unsaved changes
- Close succeeds
- Human output includes `Working Status: Modified` after a saved edit

### 15. `commit --analyze`

```powershell
.\target\debug\ext.exe --project-path "..." commit "Steel option" --analyze
```

Expect:
- Success for `v2`
- `Analysis: captured` in human mode (or a warning if some extraction failed)
- `.etabs-ext\steel-columns\v2\summary.json`
- `.etabs-ext\steel-columns\v2\results\`
- `.etabs-ext\steel-columns\v2\model.e2k`
- `.etabs-ext\steel-columns\v2\materials\material_list_by_story.parquet`

### 16. Extract-results contract spot-check

Inspect `.etabs-ext\steel-columns\v2\results\`. All ten files must be present:

| File | Priority |
|------|----------|
| `story_definitions.parquet` | required |
| `joint_drifts.parquet` | required |
| `modal_participating_mass_ratios.parquet` | required |
| `base_reactions.parquet` | required |
| `story_forces.parquet` | required |
| `pier_forces.parquet` | required |
| `pier_section_properties.parquet` | required |
| `group_assignments.parquet` | **contract critical** |
| `material_properties_concrete_data.parquet` | **contract critical** |
| `material_list_by_story.parquet` | **contract critical** |

If the three contract-critical files are missing while analysis otherwise
succeeded, the request contract likely regressed again.

### 17. Drift group verification before `ext calc`

Inspect `group_assignments.parquet` and confirm the group names match
`drift-tracking-groups` in config exactly: `Joint47`, `Joint49`, `Joint50`,
`Joint51`. If names differ, update the config before proceeding — `ext calc`
will fail fast on mismatched group names.

### 18. `calc`, `render`, and `report`

```powershell
.\target\debug\ext.exe --project-path "..." calc steel-columns/v2
.\target\debug\ext.exe --project-path "..." render steel-columns/v2
.\target\debug\ext.exe --project-path "..." report steel-columns/v2
```

Expect:
- `calc` succeeds with the confirmed load case/combo names from config
- `render` writes SVG/chart assets
- `report` writes a PDF successfully
- Pier axial report content includes a note that the check is conservative and
  excludes rebar contribution

### 19. Analyzed-version verification

```powershell
.\target\debug\ext.exe --project-path "..." log --branch steel-columns
.\target\debug\ext.exe --project-path "..." show steel-columns/v2
```

Expect:
- `ext log` shows only user-visible commits
- `show` reports `isAnalyzed = true`

### 20. Raw git log vs `ext log`

```powershell
git -C "D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext" log --oneline
```

Expect:
- Raw git log includes internal commits such as `ext: analysis results v2`
- `ext log` hides those internal commits

### 21. `diff`

```powershell
.\target\debug\ext.exe --project-path "..." diff main/v1 steel-columns/v2
```

Expect:
- Real diff text
- No `No E2K generated` warning if both versions exported E2K successfully

### 22. `checkout`

```powershell
.\target\debug\ext.exe --project-path "..." checkout main/v1
.\target\debug\ext.exe --project-path "..." status
```

Expect:
- Branch context returns to `main`
- Working file based on `v1`
- Status is `Clean`

### 23. `stash save` / `stash pop`

Modify `main\working\model.edb` so status becomes `Modified`, then:

```powershell
.\target\debug\ext.exe --project-path "..." stash save --message "main WIP"
```

Modify the working file again, then:

```powershell
.\target\debug\ext.exe --project-path "..." stash pop
```

Choose overwrite when prompted.

Expect:
- Stash save succeeds
- Overwrite prompt appears on pop when appropriate
- Stash restores successfully
- `stash list` is empty afterward
- Status becomes `Modified`

### 24. JSON smoke checks

```powershell
.\target\debug\ext.exe --json --project-path "..." commit "JSON smoke" --no-e2k
.\target\debug\ext.exe --json --project-path "..." analyze steel-columns/v2
```

Commit JSON must include: `versionId`, `branch`, `gitHash`, `message`,
`e2kGenerated`, `e2kSizeBytes`, `materialsExtracted`, `analyzed`, `elapsedMs`,
and optionally `warning`.

Analyze JSON must include: `versionId`, `branch`, `resultsDir`, `elapsedMs`,
`alreadyAnalyzed`, and optionally `warning`.

---

## Acceptance Criteria

The visual pass is successful when all of the following are true:

- Baseline `main/v1` contains manifest, E2K, and `materials\material_list_by_story.parquet`
- `ext etabs open --new-instance` launches a new ETABS instance, returns a
  real PID, and ETABS **stays open** after the command returns
- `ext etabs open` (Mode A) attaches to a running ETABS and switches to the
  working file; fails fast with a clear error when no ETABS is running
- Preflight guard produces the correct error for out-of-band ETABS (manual
  close instruction, not `ext etabs close`)
- Snapshot open returns `isSnapshot = true` and the discard warning
- Analyzed branch version contains manifest, summary, results, E2K, and materials
- `results\` contains all ten required parquet files listed in step 16
- `ext calc`, `ext render`, and `ext report` succeed with the confirmed config
- `ext log` hides internal `ext:` commits; raw git log shows them
- `ext diff` produces a real diff between `main/v1` and `steel-columns/v2`
- `checkout`, `stash save`, and `stash pop` behave correctly in human mode
- Human-readable CLI output is clear and JSON output shape is stable

---

## Notes

- Run the CLI as `.\target\debug\ext.exe` from `D:\Work\EtabExtension`
- If the `dist` sidecar is stale, use the debug sidecar from the CLI repo instead
- If ETABS opens but `ext etabs open` fails with a PID-confirmation error,
  treat as a bug and capture the exact console output
- `ext analyze steel-columns/v2` is valid only after `steel-columns/v2` exists.
  `commit "Steel option" --analyze` is the one-step path that creates and
  analyzes `v2` together
- Partial `[extract.tables.*]` config is merged over defaults — it should not
  suppress the other seven required result tables; verify in step 16
- `drift-tracking-groups` entries are joint names (`Joint47` etc.) and must
  exactly match the group names in `group_assignments.parquet` after extraction