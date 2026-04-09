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
- `extract-results` now forwards the newer table selectors all the way through
  to the CLI sidecar
- materials extraction writes `{table_slug}.parquet`, not `takeoff.parquet`

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

### 3. Configure the real sidecar and extraction tables

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

Add a focused extract-results contract check:

```toml
[extract]
units = "US_Kip_Ft"

[extract.tables.groupAssignments]
groups = ["Core"]

[extract.tables.materialPropertiesConcreteData]
fieldKeys = ["Fc", "Ec"]

[extract.tables.materialListByStory]
fieldKeys = ["Story"]
```

These three selectors are the latest contract-fix target and should no longer be
dropped by Rust before reaching the sidecar.

---

## Quick Manual Tests

If you only want the highest-signal checks, do these first:

1. Baseline `commit` creates `v1`, `model.e2k`, and `materials\material_list_by_story.parquet`
2. `ext etabs open` opens the working file in a new ETABS instance and prints a real PID
3. `ext etabs open v1` shows the snapshot warning
4. `commit --analyze` or `analyze v2` creates a `results\` directory containing:
   - `group_assignments.parquet`
   - `material_properties_concrete_data.parquet`
   - `material_list_by_story.parquet`
5. `ext log` hides internal `ext:` commits while raw `git log` still shows them

---

## Full Manual Flow

### 4. Baseline commit with real sidecar export

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" commit "Initial model"
```

Expect:

- success output for `v1`
- `.etabs-ext\main\v1\manifest.json`
- `.etabs-ext\main\v1\model.e2k`
- `.etabs-ext\main\v1\materials\material_list_by_story.parquet`

### 5. `show` and `log` spot-checks

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

### 6. `etabs open` working-file flow

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

### 7. `etabs open` snapshot flow

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

### 8. Branch create + switch

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

### 9. Real ETABS edit on branch working model

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

### 10. Close prompt and modified-state landing

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" etabs close
```

If prompted, choose save.

Expect:

- interactive prompt appears for unsaved changes when appropriate
- close succeeds
- human output includes `Working Status: Modified` after a saved edit

### 11. `commit --analyze`

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

### 12. Extract-results contract spot-check

Inspect:

`D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext\steel-columns\v2\results`

Expect the normal analysis outputs plus the three contract-fix outputs:

- `group_assignments.parquet`
- `material_properties_concrete_data.parquet`
- `material_list_by_story.parquet`

If those three files are missing while analysis otherwise succeeded, the
request contract likely regressed again.

### 13. Analyzed-version verification

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" log --branch steel-columns
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" show steel-columns/v2
```

Expect:

- `ext log` shows only user-visible commits
- `show` reports `isAnalyzed = true`

### 14. Raw git log vs `ext log`

Run:

```powershell
git -C "D:\Work\EtabExtension\proofs\week3-4-visual-pass\.etabs-ext" log --oneline
```

Expect:

- raw git log includes internal commits such as `ext: analysis results v2`
- `ext log` continues to hide those internal commits

### 15. `diff`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" diff main/v1 steel-columns/v2
```

Expect:

- real diff text
- no `No E2K generated` warning if both versions exported E2K successfully

### 16. `checkout`

Run:

```powershell
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" checkout main/v1
.\target\debug\ext.exe --project-path "D:\Work\EtabExtension\proofs\week3-4-visual-pass" status
```

Expect:

- branch context returns to `main`
- working file is based on `v1`
- status is `Clean`

### 17. `stash save` / `stash pop`

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

### 18. JSON smoke checks

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
  - `group_assignments.parquet`
  - `material_properties_concrete_data.parquet`
  - `material_list_by_story.parquet`
- `ext log` hides internal `ext:` commits while raw git log shows them
- `ext diff` produces a real diff between `main/v1` and `steel-columns/v2`
- `checkout`, `stash save`, and `stash pop` behave correctly in human mode
- human-readable CLI output is clear and JSON output remains stable

---

## Notes

- Run the CLI as `.\target\debug\ext.exe` from `D:\Work\EtabExtension`.
- If the `dist` sidecar is stale, use the debug sidecar exe from the CLI repo instead.
- If ETABS opens but `ext etabs open` fails with a PID-confirmation error, treat that as a bug and capture the exact console output.

