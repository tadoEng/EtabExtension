# Week 3–4 VCS Visual Test Spec
# ETABS Extension — Manual Validation With Real Sidecar

**Date:** 2026-03-29  
**Phase:** Phase 1, Weeks 3–4  
**Status:** Deferred until ETABS/sidecar versions match  
**Author:** EtabExtension Team  

---

## Overview

This document preserves the deferred manual validation plan for the Week 3–4
VCS flow, including `ext commit --analyze`, so the test can be resumed later
once the installed ETABS version and the sidecar/EtabSharp build are compatible.

Primary implementation reference: `2026-03-29-week3-4-vcs-spec.md`.

---

## Environment Used

Use the exact environment captured during the deferred visual pass:

- Sidecar: `D:\repo\EtabExtension.CLI\dist\etab-cli-x86_64-pc-windows-msvc.exe`
- Model: `D:\repo\bookmarkr\Sample.edb`
- Workspace: `D:\repo\bookmarkr\sidecar_test_output`

---

## Current Blocker

The full analysis-backed visual pass was deferred because ETABS 22 is installed
locally, while the current sidecar/EtabSharp build targets a newer ETABS API
contract. During analysis, ETABS reports that the API client must be updated to
work with the installed ETABS version, so `commit --analyze` cannot currently
be validated end to end in this environment.

This blocker affects live ETABS/sidecar analysis validation only. The rest of
the Week 3–4 VCS surface can still be tested independently.

---

## Full Manual Flow

### 1. Clean workspace

Reset `D:\repo\bookmarkr\sidecar_test_output` so the folder is empty before
starting the pass.

### 2. Initialize the project

Run:

```powershell
.\ext.exe init "Project Test" --edb "D:\repo\bookmarkr\Sample.edb" --path "D:\repo\bookmarkr\sidecar_test_output" --author "tado" --email "tado@email"
```

Expect:

- `.etabs-ext\` is created
- `.etabs-ext\main\working\model.edb` is created
- the next-step hint suggests `ext commit "Initial model"`

### 3. Configure sidecar in `config.local.toml`

Edit `D:\repo\bookmarkr\sidecar_test_output\.etabs-ext\config.local.toml` and
set:

```toml
[project]
sidecar-path = "D:\\repo\\EtabExtension.CLI\\dist\\etab-cli-x86_64-pc-windows-msvc.exe"
units = "kip-ft-F"

[git]
author = "tado"
email = "tado@email"

[onedrive]
acknowledgedSync = true
```

Use `project.sidecar-path` and `project.units`, not `[paths]`.

### 4. Baseline commit with real sidecar export

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" commit "Initial model"
```

Expect on a compatible ETABS/sidecar setup:

- success output for `v1`
- `.etabs-ext\main\v1\manifest.json`
- `.etabs-ext\main\v1\model.e2k`
- `.etabs-ext\main\v1\materials\takeoff.parquet`

### 5. `show` and `log` spot-checks

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" show main/v1
.\ext.exe --json --project-path "D:\repo\bookmarkr\sidecar_test_output" show main/v1
.\ext.exe --json --project-path "D:\repo\bookmarkr\sidecar_test_output" log --branch main
```

Expect:

- `id = v1`
- `branch = main`
- `isAnalyzed = false`
- `gitCommitHash` populated
- stable JSON structure

### 6. Branch create + switch

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" branch steel-columns --from main/v1
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" switch steel-columns
```

Expect:

- branch creation succeeds
- working path points to `.etabs-ext\steel-columns\working\model.edb`
- active branch becomes `steel-columns`

### 7. Real ETABS edit on branch working model

Open this file directly in ETABS:

`D:\repo\bookmarkr\sidecar_test_output\.etabs-ext\steel-columns\working\model.edb`

Make one small deterministic change, save, and fully close ETABS.

Then run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" status
```

Expect:

- status becomes `Modified`

### 8. `commit --analyze`

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" commit "Steel option" --analyze
```

Expect on a compatible ETABS/sidecar setup:

- success output for `v2`
- analysis reported as captured
- `.etabs-ext\steel-columns\v2\summary.json`
- `.etabs-ext\steel-columns\v2\results\`
- `.etabs-ext\steel-columns\v2\model.e2k`
- `.etabs-ext\steel-columns\v2\materials\takeoff.parquet`

### 9. Analyzed-version verification

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" log --branch steel-columns
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" show steel-columns/v2
```

Expect:

- `ext log` shows only user-visible commits
- `show` reports `isAnalyzed = true`

### 10. Raw git log vs `ext log`

Run:

```powershell
git -C "D:\repo\bookmarkr\sidecar_test_output\.etabs-ext" log --oneline
```

Expect:

- raw git log includes internal commits such as `ext: finalize manifest v2`
- raw git log includes `ext: analysis results v2`
- `ext log` continues to hide those internal commits

### 11. `diff`

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" diff main/v1 steel-columns/v2
```

Expect:

- actual diff text
- no `No E2K generated` warning if both versions were exported correctly

### 12. `checkout`

Run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" checkout main/v1
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" status
```

Expect:

- branch context returns to `main`
- working file is based on `v1`
- status is `Clean`

### 13. `stash save` / `stash pop`

Modify `main\working\model.edb` so status becomes `Modified`, then run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" stash --message "main WIP"
```

Expect:

- stash saved for `main`

Modify the working file again, then run:

```powershell
.\ext.exe --project-path "D:\repo\bookmarkr\sidecar_test_output" stash pop
```

Choose overwrite when prompted.

Expect:

- overwrite prompt appears
- stash restores successfully
- `stash list` is empty afterward
- status becomes `Modified`

### 14. Final `--json` smoke check

After one more saved edit, run:

```powershell
.\ext.exe --json --project-path "D:\repo\bookmarkr\sidecar_test_output" commit "JSON smoke" --no-e2k
```

Expect JSON fields:

- `versionId`
- `branch`
- `gitHash`
- `e2kGenerated`
- `materialsExtracted`
- `analyzed`
- `elapsedMs`
- optional `warning`

---

## Acceptance

The deferred visual pass is considered successful when all of the following are
true on a compatible ETABS/sidecar environment:

- baseline `main/v1` contains manifest, E2K, and materials output
- analyzed branch version contains manifest, summary, results, E2K, and materials output
- `ext log` hides internal `ext:` commits while raw git log shows them
- `ext diff` produces a real diff between `main/v1` and `steel-columns/v2`
- `checkout`, `stash save`, and `stash pop` behave correctly in human mode
- human-readable CLI output is clear and JSON output remains stable

---

## Notes

- Run the CLI as `.\ext.exe` from `target\debug` in PowerShell.
- Use `Sample.edb` with lowercase `.edb` until the Windows case-sensitivity bug
  in `init` is fixed.

