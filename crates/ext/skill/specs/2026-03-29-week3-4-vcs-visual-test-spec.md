# Week 3–4 VCS Visual Test Spec
# ETABS Extension — Current Manual Validation With Real Sidecar

**Date:** 2026-04-08  
**Phase:** Phase 1, Weeks 3–4 plus ETABS contract follow-up  
**Status:** Active  
**Author:** EtabExtension Team  

---

## Overview

This is the current manual validation sheet for the real ETABS flow. It replaces
the older deferred note and reflects the latest behavior:

- `ext etabs open` now launches ETABS in a new instance by default
- open now fails closed if the PID cannot be confirmed
- `extract-results` now merges user table filters over the full default table
  set, so partial `[extract.tables.*]` config no longer disables the other
  required parquet outputs
- materials extraction writes `{table_slug}.parquet`, not `takeoff.parquet`
- `ext calc` still requires explicit engineering `[calc]` config; `ext init`
  does not generate defaults for those values

Primary implementation reference: `2026-03-29-week3-4-vcs-spec.md`.

---

## Recommended Environment

Use local repo paths that match the current workspace:

- Rust CLI repo: `D:\Work\EtabExtension`
- Sidecar repo: `D:\Work\EtabExtension.CLI`
- Preferred sidecar path:
  `D:\Work\EtabExtension.CLI\dist\etab-cli-x86_64-pc-windows-msvc.exe`
- Acceptable debug sidecar path:
  `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\bin\Debug\net10.0\win-x64\etab-cli.exe`
- Workspace root for the manual pass:
  `D:\Work\EtabExtension\proofs\week3-4-visual-pass`
- Real ETABS model:
  use any `.edb` that opens successfully in the installed ETABS 22 instance

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

Delete or empty:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass`

### 2. Initialize the project

Run from `D:\Work\EtabExtension`:

```powershell
.\target\debug\ext.exe init "Project Test" --edb "D:\Path\To\RealModel.edb" --path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" --author "tester" --email "tester@example.com"
```

Expect:

- `.etabs-ext\` is created
- `.etabs-ext\main\working\model.edb` is created
- the hint suggests `ext commit "Initial model"`

### 3. Configure the real sidecar

Edit:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\config.local.toml`

Set:

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

Then edit:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\config.toml`

Set:

```toml
[extract]
units = "US_Kip_Ft"
```

Leave `[extract.tables.*]` empty for the default manual pass. That path now
extracts the full parquet set `ext-calc` needs with no extra filters.

### 4. Add minimum calc config before running `ext calc`

Append a project-specific `[calc]` section to the same `config.toml` before the
calc/render/report steps:

```toml
[calc]
code = "ACI318-14"
occupancy-category = "II"
modal-case = "Modal-Rizt"
drift-tracking-groups = ["Joint48"]

[calc.modal]
min-mass-participation = 0.90
display-mode-limit = 20

[calc.base-shear]
elf-case-x = "ELF_X"
elf-case-y = "ELF_Y"
rsa-case-x = "RSA_X"
rsa-case-y = "RSA_Y"
rsa-scale-min = 1.0

[calc.drift-wind]
load-cases = ["Wind_10yr_Diagonal"]
drift-limit = 0.0025

[calc.drift-seismic]
load-cases = ["RSA_Y_Drift"]
drift-limit = 0.020

[calc.displacement-wind]
load-cases = ["Wind_10yr_Diagonal"]
disp-limit-h = 400

[calc.pier-shear-wind]
load-combos = ["EVN_LRFD_WIND"]
phi-v = 0.75
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

[calc.pier-shear-seismic]
load-combos = ["EVN_LRFD_EQ"]
phi-v = 0.60
alpha-c = 2.0
fy-ksi = 60.0
rho-t = 0.0025
fc-default-ksi = 8.0

[calc.pier-axial]
load-combos = ["EVN_LRFD_EQ"]
phi-axial = 0.65
```

Replace the case/combo names with the actual names from your model. `ext calc`
will fail fast if any required `[calc]` entry is missing.

### 5. Optional focused filter pass-through check

If you specifically want to verify that the Rust side preserves table filters,
add them only after the default pass is healthy:

```toml
[extract.tables.groupAssignments]
groups = ["Core"]

[extract.tables.materialPropertiesConcreteData]
fieldKeys = ["Fc", "Ec"]

[extract.tables.materialListByStory]
fieldKeys = ["Story"]
```

This is no longer expected to suppress the other seven default tables, but the
baseline pass should still be run without filters first.

---

## Quick Manual Tests

If you only want the highest-signal checks, do these first:

1. Baseline `commit` creates `v1`, `model.e2k`, and `materials\material_list_by_story.parquet`
2. `ext etabs open` opens the working file in a new ETABS instance and prints a real PID
3. `ext etabs open v1` shows the snapshot warning
4. `commit "Steel option" --analyze` creates `v2/results\` containing:
   - `group_assignments.parquet`
   - `material_properties_concrete_data.parquet`
   - `material_list_by_story.parquet`
5. `ext analyze steel-columns/v2` also works if `v2` already exists from an
   earlier non-analyzed commit
6. `ext log` hides internal `ext:` commits while raw `git log` still shows them

---

## Full Manual Flow

### 6. Baseline commit with real sidecar export

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" commit "Initial model"
```

Expect:

- success output for `v1`
- `.etabs-ext\main\v1\manifest.json`
- `.etabs-ext\main\v1\model.e2k`
- `.etabs-ext\main\v1\materials\material_list_by_story.parquet`

### 7. `show` and `log` spot-checks

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" show main/v1
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" show main/v1
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" log --branch main
```

Expect:

- `id = v1`
- `branch = main`
- `isAnalyzed = false`
- `gitCommitHash` populated
- JSON shape remains stable

### 8. `etabs open` working-file flow

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs open
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs status
```

Expect:

- ETABS launches visibly
- human output prints `✓ ETABS opened (PID: ...)`
- the opened file is `.etabs-ext\main\working\model.edb`
- JSON status reports:
  - `sidecarAvailable = true`
  - `isRunning = true`
  - `pid` populated
  - `workingFileStatus = "openClean"` or equivalent open state

### 9. `etabs open` snapshot flow

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs close --no-save
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs open v1
```

Expect:

- open succeeds
- JSON contains:
  - `isSnapshot = true`
  - `warning = "Opening a snapshot — changes will be discarded"`
  - `pid` populated

### 10. Branch create + switch

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs close --no-save
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" branch steel-columns --from main/v1
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" switch steel-columns
```

Expect:

- branch creation succeeds
- working path becomes `.etabs-ext\steel-columns\working\model.edb`
- active branch becomes `steel-columns`

### 11. Real ETABS edit on branch working model

Open this file directly in ETABS:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\steel-columns\working\model.edb`

Make one deterministic change, save, and keep ETABS open.

Then run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" status
```

Expect:

- status reflects an open model state
- after save, a later close with `--save` should land in `Modified`

### 12. Close prompt and modified-state landing

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs close
```

If prompted, choose save.

Expect:

- interactive prompt appears for unsaved changes when appropriate
- close succeeds
- human output includes `Working Status: Modified` after a saved edit

### 13. `commit --analyze`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" commit "Steel option" --analyze
```

Expect:

- success output for `v2`
- `Analysis: captured` in human mode, or a warning if analysis succeeded but some extraction failed
- `.etabs-ext\steel-columns\v2\summary.json`
- `.etabs-ext\steel-columns\v2\results\`
- `.etabs-ext\steel-columns\v2\model.e2k`
- `.etabs-ext\steel-columns\v2\materials\material_list_by_story.parquet`

### 14. Extract-results contract spot-check

Inspect:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\steel-columns\v2\results`

Expect the normal analysis outputs plus at least these current downstream
inputs:

- `story_definitions.parquet`
- `joint_drifts.parquet`
- `modal_participating_mass_ratios.parquet`
- `base_reactions.parquet`
- `story_forces.parquet`
- `pier_forces.parquet`
- `pier_section_properties.parquet`
- `group_assignments.parquet`
- `material_properties_concrete_data.parquet`
- `material_list_by_story.parquet`

For the contract-fix spot-check, these three are the most important:

- `group_assignments.parquet`
- `material_properties_concrete_data.parquet`
- `material_list_by_story.parquet`

If those three files are missing while analysis otherwise succeeded, the
request contract likely regressed again.

### 15. Configure drift groups against extracted ETABS group names

Before running `ext calc`, inspect `group_assignments.parquet` and make sure
`[calc].drift-tracking-groups` matches real ETABS group names exactly. If the
config says `"Core"` but the parquet only contains `"Core_Walls"`, drift checks
will fail by design.

### 16. `calc`, `render`, and `report`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" calc steel-columns/v2
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" render steel-columns/v2
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" report steel-columns/v2
```

Expect:

- `calc` succeeds only after the `[calc]` section is configured with real model
  case/combo names
- `render` writes SVG/chart assets
- `report` writes a PDF successfully
- the pier axial report content includes a note that the current axial check is
  conservative and excludes rebar contribution

### 17. Analyzed-version verification

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" log --branch steel-columns
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" show steel-columns/v2
```

Expect:

- `ext log` shows only user-visible commits
- `show` reports `isAnalyzed = true`

### 18. Raw git log vs `ext log`

Run:

```powershell
git -C "D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext" log --oneline
```

Expect:

- raw git log includes internal commits such as `ext: analysis results v2`
- `ext log` continues to hide those internal commits

### 19. `diff`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" diff main/v1 steel-columns/v2
```

Expect:

- real diff text
- no `No E2K generated` warning if both versions exported E2K successfully

### 20. `checkout`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" checkout main/v1
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" status
```

Expect:

- branch context returns to `main`
- working file is based on `v1`
- status is `Clean`

### 21. `stash save` / `stash pop`

Modify `main\working\model.edb` so status becomes `Modified`, then run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" stash save --message "main WIP"
```

Modify the working file again, then run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" stash pop
```

Choose overwrite when prompted.

Expect:

- stash save succeeds
- overwrite prompt appears on pop when appropriate
- stash restores successfully
- `stash list` is empty afterward
- status becomes `Modified`

### 22. JSON smoke checks

Run:

```powershell
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" commit "JSON smoke" --no-e2k
.\target\debug\ext.exe --json --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" analyze steel-columns/v2
```

Expect commit JSON fields:

- `versionId`
- `branch`
- `gitHash`
- `message`
- `e2kGenerated`
- `e2kSizeBytes`
- `materialsExtracted`
- `analyzed`
- `elapsedMs`
- optional `warning`

Expect analyze JSON fields:

- `versionId`
- `branch`
- `resultsDir`
- `elapsedMs`
- `alreadyAnalyzed`
- optional `warning`

---

## Acceptance

The visual pass is successful when all of the following are true:

- baseline `main/v1` contains manifest, E2K, and `materials\material_list_by_story.parquet`
- `ext etabs open` launches a new ETABS instance and returns a real PID
- snapshot open returns the discard-warning flow
- analyzed branch version contains manifest, summary, results, E2K, and materials output
- `results\` contains:
  - `story_definitions.parquet`
  - `joint_drifts.parquet`
  - `modal_participating_mass_ratios.parquet`
  - `base_reactions.parquet`
  - `story_forces.parquet`
  - `pier_forces.parquet`
  - `pier_section_properties.parquet`
  - `group_assignments.parquet`
  - `material_properties_concrete_data.parquet`
  - `material_list_by_story.parquet`
- `ext calc`, `ext render`, and `ext report` succeed once real `[calc]`
  values are configured
- `ext log` hides internal `ext:` commits while raw git log shows them
- `ext diff` produces a real diff between `main/v1` and `steel-columns/v2`
- `checkout`, `stash save`, and `stash pop` behave correctly in human mode
- human-readable CLI output is clear and JSON output remains stable

---

## Notes

- Run the CLI as `.\target\debug\ext.exe` from `D:\Work\EtabExtension`.
- If the `dist` sidecar is stale, use the debug sidecar exe from the CLI repo instead.
- If ETABS opens but `ext etabs open` fails with a PID-confirmation error, treat that as a bug and capture the exact console output.
- `ext analyze steel-columns/v2` is a valid command only after `steel-columns/v2`
  already exists. `commit "Steel option" --analyze` is the one-step path that
  both creates and analyzes `v2`.
- Partial `[extract.tables.*]` config is now merged over defaults. It should no
  longer suppress the other seven required result tables, but the baseline
  manual pass should still start with no filters.

