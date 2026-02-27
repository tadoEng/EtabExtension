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
- **Analysis results** are captured at commit time with `--analyze` and stored as Parquet files. They are separate from the working file.

**Key rule:** The working file is never modified by `ext` commands except
`ext checkout` and `ext stash pop`. All other commands read from it or
write snapshots of it.

---

## Always Start Here

```bash
ext status --json
```

Read the output before doing anything. It tells you:
- Current branch and latest version
- Working file state (one of 9 states — see below)
- Whether ETABS is running and which file is open
- Whether any stash exists
- Whether local versions are pushed to OneDrive
- AI provider configured (if `--verbose`)

**Never skip this step.** The working file state determines which commands
are allowed. Running a blocked command returns a clear error — but it is
better to check state first and plan accordingly.

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
| `LOCKED` | Model locked post-analysis, cannot edit | Run `ext etabs unlock` to enable editing |
| `MISSING` | Working file deleted | Run `ext checkout vN` to restore |
| `ORPHANED` | ETABS crashed, state unknown | Run `ext etabs recover` |

**Blocked commands:** `ext commit`, `ext switch`, `ext checkout`, and `ext stash`
will fail when ETABS is open (`OPEN_*` states). Always confirm ETABS is closed
before attempting these operations.

---

## Standard Agent Workflow

For any task that modifies an ETABS model:

```
1. ext status --json                           → read current state
2. ext switch -c <task-branch> --from main/vN  → create isolated work branch
3. ext etabs open                              → open working file in ETABS
4. [engineer makes changes in ETABS, Ctrl+S, closes ETABS]
5. ext commit "engineering intent" [--analyze] → save version
6. ext diff main/vN <task-branch>/v1           → verify what changed
7. ext push                                    → sync to OneDrive (if configured)
```

**Commit early and often.** Versions are cheap. Small commits with clear
messages are better than one large commit. The agent should suggest committing
whenever the working file has been MODIFIED for a long time without a commit.

---

## Command Reference (Agent-Focused)

### State and Navigation

```bash
ext status --json                    # ALWAYS start here
ext log --json                       # list committed versions on current branch
ext log --branch <name> --json       # list versions on a specific branch
ext show v3 --json                   # details of a specific version
ext show main/v3 --json              # fully-qualified version reference
ext branch --json                    # list all branches
ext remote status --json             # OneDrive sync state
ext config list --json               # all resolved config values (keys masked)
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
ext branch -d <name> --force         # skip safety check
```

**Default `--from`:** If omitted, copies from the latest committed version
of the current branch. Never copies a dirty working file unless `--from working`
is explicit.

### Committing Versions

```bash
# Save working file as new version (e2k + materials only)
ext commit "Updated beam B45 to W21x93"

# Save AND run analysis on the snapshot (recommended when analysis matters)
ext commit "Updated beam B45 to W21x93" --analyze

# Skip E2K generation (fast save, no diff for this version)
ext commit "Quick save" --no-e2k
```

**Critical:** `--analyze` runs ETABS analysis on the committed **snapshot**
(`vN/model.edb`), not the working file. The working file is untouched.
This is intentional — it keeps the working file clean and permanently
attaches results to the version.

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
The agent should always prefer `[c]` or `[s]` over `[d]` — confirm
with the user before discarding changes.

### Stash (Temporary Save)

```bash
ext stash                    # save working file changes temporarily
ext stash list               # see all stashes across branches
ext stash pop                # restore stash to working file
ext stash drop               # discard stash (requires confirmation)
```

Use stash when the user needs to look at an old version but has uncommitted
changes they do not want to commit yet. One stash slot per branch —
check `ext status --json` before stashing to confirm no stash already exists.

### Post-Commit Analysis

```bash
# Run analysis on an already-committed version (no new version created)
ext analyze v3
ext analyze main/v3
```

Use when committed without `--analyze` and results are needed for that version.
Note: `ext analyze` is a Phase 2 agent tool. In Phase 1, suggest the user
run `ext commit --analyze` instead, or run `ext analyze` manually.

### Diff and Comparison

```bash
# Raw E2K diff between two versions
ext diff v2 v3
ext diff main/v2 steel-columns/v1    # across branches
```

Phase 1 diff is a raw unified text diff on E2K files. It shows exact
structural definition changes (section sizes, geometry, load cases, etc.).
The agent can read and summarize diff output for the user.

### ETABS Control

```bash
ext etabs open                       # open working file in ETABS (visible)
ext etabs open v3                    # open a snapshot (warn: read-only recommended)
ext etabs close                      # close ETABS
ext etabs close --save               # save then close
ext etabs close --no-save            # discard and close
ext etabs status --json              # ETABS running? which file? locked?
ext etabs validate --file model.edb  # check file validity
ext etabs unlock                     # clear analysis lock (Phase 2 agent tool)
ext etabs recover                    # recover from ETABS crash (ORPHANED state)
```

**The agent can open and close ETABS but cannot operate ETABS.**
After `ext etabs open`, the agent must wait for the user to complete
their work in ETABS, save (Ctrl+S), and close ETABS before proceeding.
The agent cannot see the ETABS screen, click buttons, or enter values.

**Never open ETABS manually outside of `ext etabs open`.** The CLI tracks
which file ETABS has open via PID. Opening ETABS outside the CLI puts
`state.json` out of sync and can cause ORPHANED state on next command.

### Reports (PDF)

```bash
# Analysis report (requires --analyze to have been run on that version)
ext report analysis --version v3

# Bill of materials (always available — no analysis needed)
ext report bom --version v3

# Compare two versions
ext report comparison --from main/v3 --to steel-columns/v1

# Override output path
ext report analysis --version v3 --out "D:\Reports\analysis.pdf"
```

Note: `ext report` commands are Phase 2 agent tools. In Phase 1, the agent
should inform the user of the correct command to run manually rather than
attempting to call the tool directly.

### Sharing via OneDrive

```bash
ext push                             # push git history + .edb files
ext pull                             # pull new versions from OneDrive
ext clone <onedrive-path> --to <local-path>   # first-time setup
ext remote status --json             # see local vs OneDrive diff
```

`ext push` requires `paths.oneDriveDir` to be set in `config.local.toml`.
If not set, inform the user and provide the config command.

### Configuration

```bash
# Machine-specific (always written to config.local.toml)
ext config set git.author "Jane Smith"
ext config set git.email "jane@firm.com"
ext config set paths.oneDriveDir "C:\Users\Jane\OneDrive\Structural\HighRise"
ext config set paths.reportsDir "C:\Users\Jane\OneDrive\Structural\HighRise\reports"

# AI provider (always written to config.local.toml — API keys are private)
ext config set ai.provider ollama
ext config set ai.model "qwen2.5-coder:14b"
ext config set ai.baseUrl "http://localhost:11434/v1"

# Shared project settings (written to config.toml)
ext config set behavior.confirmDestructive true

ext config list --json               # see all resolved config (keys masked)
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
# [engineer runs analysis: Analyze → Run All, closes ETABS]
ext commit "Initial seismic analysis" --analyze
# Phase 2: ext report analysis --version v1
```

### Scenario: Go back to review an old version

```bash
ext log --json                       # find the version
ext checkout v2                      # working file modified → prompted
# agent chooses [s] to stash (never [d] without user confirmation)
ext etabs open                       # review v2
ext etabs close
ext stash pop                        # return to where you were
```

### Scenario: Compare two design alternatives

```bash
# Both branches should be analyzed for a meaningful comparison
ext log --branch steel-alternative --json    # confirm analysis status
ext diff main/v3 steel-alternative/v1        # E2K diff always available
# Phase 2: ext report comparison --from main/v3 --to steel-alternative/v1
```

### Scenario: Share project with colleague (first time)

```bash
# Engineer A pushes everything
ext push --include-working

# Engineer B clones on their machine
ext clone "C:\Users\B\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"
```

### Scenario: Pull colleague's branch and review

```bash
ext remote status --json             # see what's on OneDrive
ext pull --branch jane/foundation    # pull specific branch
ext diff main/v4 jane/foundation/v1  # compare immediately
```

### Scenario: ETABS crashed

```bash
ext status --json                    # state will show ORPHANED
ext etabs recover
# agent presents options — never auto-choose [r] restore without asking user
# [k] Keep changes = mark MODIFIED (usually safer)
# [r] Restore from last version = discard post-crash changes
```

### Scenario: Need to edit after analysis (model is LOCKED)

```bash
ext status --json                    # state shows LOCKED or ANALYZED
# Phase 2: ext etabs unlock          # clears the model lock
# Phase 1: inform user to run: ext etabs unlock
# [make edits, Ctrl+S, close]
ext commit "Revised post-analysis" --analyze
```

### Scenario: Set up local AI (Ollama, private)

```bash
# 1. Install Ollama: https://ollama.com
# 2. Pull a model (run in terminal, outside ext)
#    ollama pull qwen2.5-coder:14b
# 3. Configure ext to use it
ext config set ai.provider ollama
ext config set ai.model "qwen2.5-coder:14b"
ext config set ai.baseUrl "http://localhost:11434/v1"
# 4. Verify
ext chat --provider ollama
```

No API key needed. No data leaves the machine. The model runs entirely locally.

### Scenario: Switch to cloud AI temporarily

```bash
ext chat --provider claude --model claude-opus-4-5
# or set permanently:
ext config set ai.provider claude
ext config set ai.apiKey "sk-ant-..."
```

---

## What NOT to Do

```bash
# ❌ Do not copy .edb files manually — use ext branch + ext commit
cp model.edb model_v2.edb

# ❌ Do not open ETABS directly — always use ext etabs open
# Opening ETABS outside ext breaks PID tracking → ORPHANED on next command

# ❌ Do not run ext commit while ETABS is open
# ext commit will error: "Close ETABS before committing"

# ❌ Do not put the project inside a OneDrive folder
# ext init warns — use a local path like C:\ETABSProjects\...
# Reports can still auto-save to OneDrive via paths.reportsDir

# ❌ Do not use ext checkout to switch branches
# ext checkout restores a VERSION within a branch
# ext switch changes the active BRANCH
# Correct: ext switch main          → change to main branch
# Correct: ext checkout v1          → restore to v1 on current branch
# Correct: ext checkout main/v1     → switch to main AND restore to v1

# ❌ Do not expect --analyze to touch the working file
# Analysis runs on the committed snapshot vN/model.edb, not working/model.edb

# ❌ Do not auto-choose [d] Discard in checkout prompt without asking the user
# Always prefer [c] Commit or [s] Stash — discard loses work permanently

# ❌ Do not attempt to call analyze or report tools in Phase 1
# These are Phase 2 agent tools — inform the user and give the manual command

# ❌ Do not operate ETABS after ext etabs open
# The agent opens ETABS for the user — it cannot click, type, or see the screen
# Wait for the user to complete their work and confirm ETABS is closed

# ❌ Do not set ai.apiKey in config.toml
# config.toml is git-tracked and pushed to OneDrive
# AI keys always go to config.local.toml — ext config set routes this automatically

# ❌ Do not include raw .edb bytes or full Parquet data in responses
# The agent works with text summaries only — binary model data stays local
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
ext stash list --json
```

JSON output is stable — fields are only ever added, never renamed or removed.

---

## Phase 1 vs Phase 2 Tool Availability

The agent must know which tools are available in Phase 1 and which are deferred.
Attempting to call a Phase 2 tool will return a clear error, but it is better
to inform the user proactively and give the manual command.

**Phase 1 — available now:**

| Tool | Operation |
|---|---|
| `project_status` | `ext status` |
| `list_versions` | `ext log` |
| `show_version` | `ext show` |
| `list_branches` | `ext branch` |
| `diff_versions` | `ext diff` |
| `etabs_status` | `ext etabs status` |
| `remote_status` | `ext remote status` |
| `config_list` | `ext config list` |
| `commit_version` | `ext commit` |
| `create_branch` | `ext branch <name>` |
| `switch_branch` | `ext switch` |
| `checkout_version` | `ext checkout` |
| `stash_save` | `ext stash` |
| `stash_pop` | `ext stash pop` |
| `etabs_open` | `ext etabs open` |
| `etabs_close` | `ext etabs close` |
| `etabs_recover` | `ext etabs recover` |
| `push` | `ext push` |
| `pull` | `ext pull` |

**Phase 2 — deferred (require streaming UI):**

| Tool | Operation | Why deferred |
|---|---|---|
| `analyze_version` | `ext analyze` | 2–5 min runtime needs live progress |
| `generate_report` | `ext report` | PDF compilation needs streaming status |
| `etabs_unlock` | `ext etabs unlock` | Sensitive — needs careful UX review |

For Phase 2 tools, respond with:
```
I can't run this directly yet, but you can run it manually:
  ext analyze v3
```

---

## Key Constraints for Agents

- **ETABS must be closed** before: `ext commit`, `ext switch`, `ext checkout`,
  `ext stash`, `ext pull`
- **`--analyze` is expensive** — ETABS opens hidden, runs full analysis,
  extracts all Parquet results. Typical duration: 2–5 minutes. Only use
  when analysis results are explicitly needed.
- **One stash per branch** — if a stash already exists, pop or drop it
  before stashing again. Check `ext status --json` first.
- **`ext push` requires OneDrive config** — `paths.oneDriveDir` must be set
  in `config.local.toml`. If not set, provide the config command.
- **Reports require analysis** for `analysis` and `comparison` types —
  `bom` report does not require analysis.
- **Version numbering is per-branch** — `main/v3` and `steel-columns/v3`
  are completely independent versions.
- **The agent cannot operate ETABS** — it can open and close ETABS but
  cannot interact with the ETABS user interface in any way.
- **Phase 2 tools are not available in Phase 1** — `analyze`, `report`,
  `etabs_unlock`. Inform the user and give the manual command.
- **Local provider is recommended for sensitive projects** — use Ollama
  (`ai.provider = "ollama"`) so no project data leaves the machine.