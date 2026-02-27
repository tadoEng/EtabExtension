---
name: ext
version: 1.0.0
description: >
   Git-like version control for ETABS structural engineering models.
   Use for: managing ETABS projects, creating design alternatives (branches),
   committing model versions, running and capturing analysis results,
   comparing structural behavior between versions, generating PDF reports,
   and sharing projects across machines via OneDrive.
author: ETABS Extension Team
---

# ETABS Extension CLI — AI Agent Skill

This skill teaches an AI agent how to operate the `ext` CLI correctly.
Read this entire file before issuing any `ext` commands.

---

## Mental Model

```
Project
  └── Branch (design alternative)
        └── Working file  ← what ETABS edits right now
        └── v1, v2, v3…  ← committed snapshots
```

- A **project** is one ETABS structural model with full version history.
- A **branch** is an independent design alternative (e.g. `steel-columns`, `mat-foundation`).
- A **version** (`v1`, `v2`…) is a committed snapshot. Each has a `.edb` (binary) and a `.e2k` (diffable text).
- The **working file** is the live `.edb` the engineer edits in ETABS. It is the source for the next commit.
- **Analysis results** are captured at commit time with `--analyze` and stored as Parquet. They are separate from the working file.

**Key rule:** The working file is never modified by `ext` commands except `ext checkout` and `ext stash pop`. All other commands read from it or write snapshots of it.

---

## Always Start Here

```bash
ext status --json
```

Read the output before doing anything. It tells you:
- Current branch and latest version
- Working file state (`UNTRACKED`, `CLEAN`, `MODIFIED`, `OPEN_CLEAN`, `OPEN_MODIFIED`, `ANALYZED`, `LOCKED`, `MISSING`, `ORPHANED`)
- Whether ETABS is running and which file is open
- Whether any stash exists
- Whether local versions are pushed to OneDrive

**Never skip this step.** The working file state determines which commands are allowed.

---

## Working File States and What They Mean

| State | Meaning | What to do |
|---|---|---|
| `UNTRACKED` | Fresh project, no versions yet | Run `ext commit "Initial model"` |
| `CLEAN` | Working file matches latest version | Safe to open ETABS, branch, checkout |
| `MODIFIED` | Working file has unsaved changes | Run `ext commit "message"` or `ext stash` |
| `OPEN_CLEAN` | ETABS has file open, no edits yet | Work in ETABS or close with `ext etabs close` |
| `OPEN_MODIFIED` | ETABS open, changes made | Close ETABS, then `ext commit` |
| `ANALYZED` | ETABS closed, analysis results in working file | Run `ext commit --analyze` to capture results |
| `LOCKED` | Model locked post-analysis, can't edit | Run `ext etabs unlock` to enable editing |
| `MISSING` | Working file deleted | Run `ext checkout vN` to restore |
| `ORPHANED` | ETABS crashed, state unknown | Run `ext etabs recover` |

**Blocked commands:** `ext commit` and `ext switch` will fail when ETABS is open. Always close ETABS first.

---

## Standard Agent Workflow

For any task that modifies an ETABS model:

```
1. ext status --json                          → read current state
2. ext switch -c <task-branch> --from main/vN  → create isolated work branch
3. ext etabs open                              → open working file in ETABS
4. [make changes in ETABS, Ctrl+S, close]
5. ext commit "engineering intent" [--analyze] → save version
6. ext diff main/vN <task-branch>/v1           → verify what changed
7. ext push                                    → sync to OneDrive (if configured)
```

**Commit early and often.** Versions are cheap. Small commits with clear messages are better than one large commit.

---

## Command Reference (Agent-Focused)

### State and Navigation

```bash
ext status --json                   # ALWAYS start here
ext log --json                      # list committed versions on current branch
ext log --branch <name> --json      # list versions on a specific branch
ext show v3 --json                  # details of a specific version
ext show main/v3 --json             # fully-qualified version reference
ext branch --json                   # list all branches
```

### Branching

```bash
# Create a branch (does NOT switch)
ext branch <name>
ext branch <name> --from main/v3

# Create AND switch in one step (preferred for agents)
ext switch -c <name> --from main/v3

# Switch to existing branch
ext switch <name>

# Delete a branch
ext branch -d <name>
ext branch -d <name> --force        # skip safety check
```

**Default `--from`:** If omitted, copies from the latest committed version of the current branch. Never copies a dirty working file unless `--from working` is explicit.

### Committing Versions

```bash
# Save working file as new version (e2k + materials only)
ext commit "Updated beam B45 to W21x93"

# Save AND run analysis on the snapshot (recommended when analysis matters)
ext commit "Updated beam B45 to W21x93" --analyze

# Skip E2K generation (fast save, no diff for this version)
ext commit "Quick save" --no-e2k
```

**Critical:** `--analyze` runs ETABS analysis on the committed **snapshot** (`vN/model.edb`), not the working file. The working file is untouched. This is intentional — it keeps the working file clean and permanently attaches results to the version.

### Restoring Versions

```bash
# Restore working file to a specific version (current branch)
ext checkout v2

# Switch to another branch AND restore to a specific version
ext checkout main/v2
```

If the working file is `MODIFIED`, `ext checkout` will prompt:
```
[c] Commit first   [s] Stash   [d] Discard   [x] Cancel
```
For automation, pass `--force` to discard without prompting.

### Stash (Temporary Save)

```bash
ext stash                    # save working file changes temporarily
ext stash list               # see all stashes across branches
ext stash pop                # restore stash to working file
ext stash drop               # discard stash
```

Use stash when you need to look at an old version but have uncommitted changes you don't want to lose or commit yet.

### Post-Commit Analysis

```bash
# Run analysis on an already-committed version (no new version created)
ext analyze v3
ext analyze main/v3
```

Use when you committed without `--analyze` and need results for that version later.

### Diff and Comparison

```bash
# Raw E2K diff between two versions (fast, no ETABS)
ext diff v2 v3
ext diff main/v2 steel-columns/v1    # across branches
```

Phase 1 diff is a raw unified text diff on E2K files. It shows exact structural definition changes (section sizes, geometry, load cases, etc.).

### ETABS Control

```bash
ext etabs open                       # open working file in ETABS (visible)
ext etabs open v3                    # open a snapshot (read-only recommended)
ext etabs close                      # close ETABS
ext etabs close --save               # save then close
ext etabs close --no-save            # discard and close
ext etabs status --json              # ETABS running? which file? locked?
ext etabs validate --file model.edb  # check file validity
ext etabs unlock                     # clear analysis lock (enables editing)
ext etabs recover                    # recover from ETABS crash (ORPHANED state)
```

**Never open ETABS manually outside of `ext etabs open`.** The CLI tracks which file ETABS has open via PID. Opening ETABS outside the CLI puts state.json out of sync.

### Reports (PDF, auto-saved to OneDrive)

```bash
# Analysis report (requires --analyze to have been run)
ext report analysis --version v3

# Bill of materials (always available — no analysis needed)
ext report bom --version v3

# Compare two versions (E2K diff + result deltas + material delta)
ext report comparison --from main/v3 --to steel-columns/v1

# Override output path (default is paths.reportsDir in config.local.toml)
ext report analysis --version v3 --out "D:\Reports\analysis.pdf"
```

### Sharing via OneDrive

```bash
# Push git history + .edb files to OneDrive
ext push

# Pull new versions from OneDrive
ext pull

# First-time setup on a new machine
ext clone "C:\Users\Jane\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"

# See what's on OneDrive vs local
ext remote status --json
```

### Configuration

```bash
# Machine-specific settings (written to config.local.toml, git-ignored)
ext config set git.author "Jane Smith"
ext config set git.email "jane@firm.com"
ext config set paths.oneDriveDir "C:\Users\Jane\OneDrive\Structural\HighRise"
ext config set paths.reportsDir "C:\Users\Jane\OneDrive\Structural\HighRise\reports"

# Project settings (written to config.toml, shared via OneDrive)
ext config set behavior.confirmDestructive true

ext config list --json              # see all resolved config
```

---

## Fully-Qualified Version References

When referencing versions across branches, use `<branch>/<version>`:

```bash
main/v3              # version 3 on main branch
steel-columns/v1     # version 1 on steel-columns branch
v3                   # short form — current branch implied
```

---

## Common Scenarios

### Scenario: Engineer wants to try a design alternative

```bash
ext status --json
ext switch -c steel-alternative --from main/v3
ext etabs open
# [engineer modifies columns in ETABS, Ctrl+S, closes ETABS]
ext commit "W14x120 steel columns"
ext diff main/v3 steel-alternative/v1
```

### Scenario: Capture analysis results

```bash
ext etabs open
# [engineer runs analysis in ETABS: Analyze → Run All, closes ETABS]
ext commit "Initial seismic analysis" --analyze
ext report analysis --version v1
# → PDF auto-saved to OneDrive reports folder
```

### Scenario: Go back to review an old version

```bash
ext log --json                       # find the version
ext checkout v2                      # working file has changes → prompted
# choose [s] to stash
ext etabs open                       # review v2
ext etabs close
ext stash pop                        # return to where you were
```

### Scenario: Compare two design alternatives

```bash
# Both branches must be analyzed
ext analyze steel-alternative/v1    # if not already analyzed
ext report comparison --from main/v3 --to steel-alternative/v1
```

### Scenario: Share project with colleague (first time)

```bash
# Engineer A pushes
ext push --include-working

# Engineer B clones on their machine
ext clone "C:\Users\B\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"
```

### Scenario: Pull colleague's branch and review

```bash
ext remote status --json             # see what's on OneDrive
ext pull --branch jane/foundation    # pull specific branch
ext report comparison --from main/v4 --to jane/foundation/v1
```

### Scenario: ETABS crashed

```bash
ext status --json                    # state will show ORPHANED
ext etabs recover
# choose [k] Keep changes or [r] Restore from last version
```

### Scenario: Need to edit after analysis (model is LOCKED)

```bash
ext status --json                    # state shows LOCKED or ANALYZED
ext etabs unlock                     # clears the model lock
# [ETABS opens automatically in OPEN_CLEAN state]
# [make edits, Ctrl+S, close]
ext commit "Revised post-analysis" --analyze
```

---

## What NOT to Do

```bash
# ❌ Do not copy .edb files manually — use ext branch + ext commit
cp model.edb model_v2.edb

# ❌ Do not open ETABS directly — use ext etabs open
# (Opening ETABS outside ext breaks state tracking)

# ❌ Do not run ext commit while ETABS is open
# (ext commit will error: "Close ETABS before committing")

# ❌ Do not put the project inside OneDrive
# (ext init will warn — use a local path like C:\ETABSProjects\...)

# ❌ Do not use ext checkout to switch branches
# (ext checkout restores VERSION within a branch)
# (ext switch changes the active BRANCH)
# Correct: ext switch main        → change to main branch
# Correct: ext checkout v1        → restore working file to v1 on current branch
# Correct: ext checkout main/v1   → switch to main AND restore to v1

# ❌ Do not expect --analyze to touch the working file
# (analysis runs on the committed snapshot vN/model.edb, not working/model.edb)
```

---

## Output Format for Agents

Always append `--json` when parsing output programmatically:

```bash
ext status --json
ext log --json
ext branch --json
ext etabs status --json
ext remote status --json
ext show v3 --json
ext config list --json
```

JSON output is stable — fields are only ever added, never renamed or removed.

---

## Key Constraints for Agents

- **ETABS must be closed** before: `ext commit`, `ext switch`, `ext checkout`, `ext stash`, `ext pull`
- **`--analyze` is expensive** — ETABS opens hidden, runs full analysis, extracts all Parquet results. Typical duration: 2–5 minutes. Only use when analysis results are needed.
- **One stash per branch** — if a stash already exists, pop or drop it before stashing again
- **`ext push` requires OneDrive config** — `paths.oneDriveDir` must be set in `config.local.toml`
- **Reports require analysis** for `analysis` and `comparison` types — `bom` report does not require analysis
- **Version numbering is per-branch** — `main/v3` and `steel-columns/v3` are completely independent versions