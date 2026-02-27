# ETABS Extension CLI — Command Reference

Complete reference for all `ext` commands.

## Contents

- [Global Options](#global-options)
- [Project](#project) — `init`, `status`, `log`, `show`
- [Branch](#branch) — `branch`, `switch`, `checkout`
- [Stash](#stash) — `stash`
- [Version](#version) — `commit`, `analyze`
- [Diff](#diff) — `diff`
- [ETABS](#etabs) — `etabs open/close/status/validate/unlock/recover`
- [Report](#report) — `report analysis/bom/comparison`
- [Remote](#remote) — `push`, `pull`, `clone`, `remote status`
- [Config](#config) — `config get/set/list/edit`
- [AI Chat](#ai-chat) — `chat`
- [Aliases](#aliases)
- [Sidecar Reference](#sidecar-reference)
- [Quick Reference](#quick-reference)

---

## Global Options

| Flag | Description |
|---|---|
| `--json` | Structured JSON output |
| `--shell` | Minimal text output for scripting |
| `--verbose` | Show detailed information |
| `--quiet` | Suppress non-essential output |
| `--project-path <PATH>` | Override project directory |
| `--no-color` | Disable colored output |

**Environment variables:**

| Variable | Purpose |
|---|---|
| `ETABS_EXT_PROJECT` | Default project path |
| `ETABS_SIDECAR_PATH` | Path to `etab-cli.exe` |
| `ETABS_AI_API_KEY` | AI provider API key (overrides config.local.toml) |
| `NO_COLOR` | Disable color |

---

## Project

### `ext init <name> --edb <path>`

Initialize a new ETABS project with version control.

```bash
ext init "HighRise Tower" --edb "C:\Models\highrise.edb"
ext init "HighRise Tower" --edb "C:\Models\highrise.edb" --path "D:\Projects\HighRise"
```

| Flag | Description |
|---|---|
| `--edb <path>` | Path to existing `.edb` file (required) |
| `--path <path>` | Project directory (default: current directory) |
| `--author <name>` | Author name — saved to `config.local.toml` |
| `--email <email>` | Author email — saved to `config.local.toml` |
| `--onedrive <path>` | OneDrive project folder — saved to `config.local.toml` |
| `--reports <path>` | Reports output folder — saved to `config.local.toml` |

**Output:**

```
✓ Created project: HighRise Tower
✓ Initialized git repository
✓ Created main branch
✓ Copied model → main/working/model.edb (45.2 MB)
→ Next: ext commit "Initial model"
```

**OneDrive warning** (if `--edb` or `--path` is inside a OneDrive folder):

```
⚠ OneDrive path detected
  ETABS model files cannot be version-controlled inside OneDrive.
  Recommended: use a local path outside OneDrive.
  Reports will save to OneDrive via: ext config set paths.reportsDir <path>

  [c] Continue anyway  [x] Cancel
```

---

### `ext status`

Show full project state.

```bash
ext status
ext status --verbose
ext status --json
```

**Output:**

```
Project: HighRise Tower
Branch:  main (3 versions)

Working File: Modified
  Based on:  v3
  Changed:   14 minutes ago
  ETABS:     Not running

Stash:  main (uncommitted changes since v3, 2h ago)

OneDrive: main/v3 not pushed yet

Recent Versions:
  v3  Updated column sections    2d ago  ✓ analyzed
  v2  Added seismic loads        5d ago  ✓ analyzed
  v1  Initial structural layout  2w ago
```

**`--verbose` additions:** shows full file paths, ETABS PID if running,
OneDrive sync details, AI provider if configured.

---

### `ext log`

List committed versions on a branch.

```bash
ext log
ext log --branch steel-columns
ext log --all
ext log --json
```

**Output:**

```
Branch: main

  v3  Updated column sections per analysis    2d ago   ✓ analyzed
  v2  Added seismic loads                     5d ago   ✓ analyzed
  v1  Initial structural layout               2w ago
```

Internal `ext:` commits are always hidden from this view.

---

### `ext show <target>`

Show details of a version, branch, or project.

```bash
ext show v3
ext show main/v3
ext show branch main
ext show project
ext show v3 --json
```

---

## Branch

### `ext branch [name] [--from <ref>]`

**List branches** (no arguments):

```bash
ext branch
ext branch --json
```

```
Branches:
  main             3 versions  (active)
  steel-columns    1 version   from main/v3
  deep-foundation  2 versions  from main/v2
```

**Create a branch:**

```bash
ext branch steel-columns
ext branch steel-columns --from main/v3
ext branch steel-columns --from main
ext branch steel-columns --from working
```

| Flag | Description |
|---|---|
| `--from <ref>` | Source (default: latest committed version of current branch) |
| `--description <text>` | Branch description |

Creates but does NOT switch. Use `ext switch -c` to create + switch in one step.

**Delete a branch:**

```bash
ext branch -d steel-columns
ext branch -d steel-columns --force
```

Cannot delete `main` or the currently active branch.

---

### `ext switch <branch>`

Switch active branch. Each branch's `working/model.edb` is always preserved independently.

```bash
ext switch steel-columns
ext switch main
```

- Hard stop if ETABS is open
- Warns if current branch has uncommitted changes (proceeds — changes preserved)
- Reports target branch working file state on arrival

**Departure warning:**

```
⚠ Leaving main with uncommitted changes since v3
  Changes are preserved in main/working/model.edb
```

**Arrival report:**

```
✓ Switched to: steel-columns
⚠ Working file has uncommitted changes since v1
  Run: ext commit "msg"   to save them
       ext checkout v1    to discard them
```

---

### `ext switch -c <branch> [--from <ref>]`

Create and switch in one command. Mirrors `git switch -c`.

```bash
ext switch -c steel-columns
ext switch -c steel-columns --from main/v3
```

---

### `ext checkout <version>`

Restore the current branch's working file to a specific committed version.

```bash
ext checkout v1
ext checkout main/v1         # switch to main first, then restore to v1
```

- Hard stop if ETABS is open
- If working file MODIFIED, prompts:

```
⚠ Working file has changes since v3.
  Checking out v1 will overwrite them.

  [c] Commit current changes first, then checkout v1
  [s] Stash current changes  (restore later: ext stash pop)
  [d] Discard changes and checkout v1
  [x] Cancel
```

- Cross-branch: applies `ext switch` rules first
- If snapshot `vN/model.edb` missing: lists available versions
- `--force` implies `[d]` discard — for CI/automation

---

## Stash

One stash slot per branch.

### `ext stash`

```bash
ext stash
ext stash --message "WIP: trying larger columns"
```

### `ext stash list`

```bash
ext stash list
ext stash list --json
```

```
Stashes:
  main:          uncommitted changes since v3  (2h ago)
                 "auto-stash before checkout v1"
  steel-columns: uncommitted changes since v1  (3d ago)
```

### `ext stash pop`

```bash
ext stash pop
```

### `ext stash drop`

```bash
ext stash drop
ext stash drop --force
```

---

## Version

### `ext commit <message>`

Save working file as a new committed version.

```bash
ext commit "Updated beam sizes"
ext commit "Updated beam sizes" --analyze
ext commit "Quick save" --no-e2k
```

| Flag | Description |
|---|---|
| `--analyze` | Run analysis on snapshot and extract results to Parquet |
| `--no-e2k` | Skip E2K generation (faster, no diff for this version) |

**Sequence without `--analyze`:**

```
1. Verify ETABS not running
2. Copy working/model.edb → vN/model.edb
3. sidecar save-snapshot → vN/model.e2k + vN/materials/takeoff.parquet
4. Write vN/manifest.json  { isAnalyzed: false }
5. git commit: vN/model.e2k + vN/manifest.json
6. state: CLEAN, basedOn=vN
```

**With `--analyze`** (runs on snapshot — NOT working file):

```
7–12. sidecar: open hidden → run analysis → extract results → close
13.   Write vN/summary.json, update manifest { isAnalyzed: true }
14.   git commit (internal, hidden from ext log)

Working file: untouched throughout steps 7–14
```

---

### `ext analyze <version>`

Run analysis on an already-committed version. Working file untouched.

```bash
ext analyze v2
ext analyze main/v2
```

---

## Diff

### `ext diff [v1] [v2]`

```bash
ext diff v2 v3
ext diff main/v2 steel/v1
```

Phase 1: raw unified git diff on E2K files.

---

## ETABS

### `ext etabs open [version]`

```bash
ext etabs open              # open working file (visible ETABS)
ext etabs open v3           # open snapshot (warns: read-only recommended)
```

### `ext etabs close`

```bash
ext etabs close
ext etabs close --save
ext etabs close --no-save
```

### `ext etabs status`

```bash
ext etabs status
ext etabs status --json
```

```
ETABS Status:
  Running:    Yes
  Version:    22.0.0
  Open File:  D:\Projects\HighRise\main\working\model.edb
  PID:        12345
  Model Lock: No
  Analyzed:   Yes
```

### `ext etabs validate --file <path>`

```bash
ext etabs validate --file model.edb
```

### `ext etabs unlock`

Clear ETABS analysis lock on the working file.

```bash
ext etabs unlock
```

Calls `SapModel.SetModelIsLocked(false)`. Extracted Parquet files are preserved.

### `ext etabs recover`

Recover from ETABS crash (ORPHANED state). Always prompts — never automated.

```bash
ext etabs recover
```

```
⚠ ETABS closed unexpectedly (PID: 12345)
  File modified: Yes (14 minutes before crash)

  [k] Keep changes  (mark as MODIFIED)
  [r] Restore from v3  (discard changes)
```

---

## Report

Reports default to `paths.reportsDir` from `config.local.toml`. Override with `--out`.

### `ext report analysis --version <id>`

```bash
ext report analysis --version v3
ext report analysis --version v3 --out "D:\Reports\analysis.pdf"
```

Requires `isAnalyzed: true`. Contents: modal results, base reactions, story
forces, story drifts, code compliance checks.

### `ext report bom --version <id>`

```bash
ext report bom --version v3
```

Does not require analysis. Contents: material quantities by type and story,
weight and cost summary.

### `ext report comparison --from <id> --to <id>`

```bash
ext report comparison --from main/v3 --to steel-columns/v1
```

Contents: E2K change summary, result deltas (if both analyzed), material delta.

---

## Remote

### `ext push`

```bash
ext push
ext push --include-working
ext push --branch steel-columns
ext push --versions v1,v3
```

Requires `paths.oneDriveDir` in `config.local.toml`.

**Conflict handling:**

```
✗ Conflict: main/v4 already exists on OneDrive (pushed by Jane Smith, 2h ago)
  Your v4: "Increased column size"
  Remote:  "Updated foundation"

  [r] Rename yours to v5 and push  (recommended)
  [v] View diff between versions
  [x] Cancel
```

### `ext pull`

```bash
ext pull
ext pull --branch steel-columns
```

### `ext clone <onedrive-path> --to <local-path>`

```bash
ext clone "C:\Users\Jane\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"
```

Interactive wizard prompts for author, email, OneDrive folder, reports folder.
Writes `config.local.toml`. Never overwrites an existing `config.local.toml`.

### `ext remote status`

```bash
ext remote status
ext remote status --json
```

```
Remote: C:\Users\John\OneDrive\Structural\HighRise
Last pushed by: John Doe  (2h ago)

  main:              local v5  =  remote v5  ✓ in sync
  steel-columns:     local v1  =  remote v1  ✓ in sync
  jane/foundation:   ─────────   remote v1  ↓ not pulled
```

---

## Config

### `ext config get <key>`

```bash
ext config get git.author
ext config get paths.reportsDir
ext config get ai.provider
```

### `ext config set <key> <value>`

```bash
# Machine-specific — auto-routed to config.local.toml
ext config set git.author "John Doe"
ext config set git.email "john@firm.com"
ext config set paths.reportsDir "C:\Users\John\OneDrive\Structural\HighRise\reports"
ext config set paths.oneDriveDir "C:\Users\John\OneDrive\Structural\HighRise"

# AI provider — always config.local.toml (never config.toml — API keys are private)
ext config set ai.provider ollama
ext config set ai.model "qwen2.5-coder:14b"
ext config set ai.baseUrl "http://localhost:11434/v1"
ext config set ai.provider claude
ext config set ai.apiKey "sk-ant-..."
ext config set ai.autoConfirm false

# Shared project settings — routed to config.toml
ext config set behavior.confirmDestructive true
```

**Routing rules** (enforced automatically — user does not need to know):

| Key prefix | File |
|---|---|
| `git.*` | `config.local.toml` |
| `paths.*` | `config.local.toml` |
| `ai.*` | `config.local.toml` — **always**, even if user specifies `--global` |
| `onedrive.*` | `config.local.toml` |
| `project.*` | `config.toml` |
| `etabs.*` | `config.toml` |
| `behavior.*` | `config.toml` |

### `ext config list`

```bash
ext config list
ext config list --local
ext config list --json
```

`--local` shows only `config.local.toml` values. Useful to verify AI keys
are set correctly without seeing shared config noise. API key values are
always masked: `sk-ant-...●●●●●●●●`.

### `ext config edit`

```bash
ext config edit           # opens config.toml in $EDITOR
ext config edit --local   # opens config.local.toml in $EDITOR
```

---

## AI Chat

### `ext chat`

Start an interactive AI-assisted session for the current project.
The agent reads current project state before every turn and can execute
any `ext` operation on your behalf after confirmation.

```bash
ext chat
ext chat --provider ollama
ext chat --provider claude
ext chat --provider openai
ext chat --model "qwen2.5-coder:14b"
ext chat --no-confirm              # skip [y/n] for non-destructive writes
ext chat --non-interactive         # Phase 2: read from stdin, write to stdout
ext chat --resume                  # Phase 2: continue last saved session
ext chat --clear-history           # Phase 2: wipe saved session for this branch
```

| Flag | Description |
|---|---|
| `--provider <name>` | Override `ai.provider` for this session |
| `--model <name>` | Override `ai.model` for this session |
| `--no-confirm` | Skip `[y/n]` prompts for non-destructive write tools |
| `--non-interactive` | Phase 2: stdin → stdout, no REPL (for scripting) |
| `--resume` | Phase 2: load last saved conversation for this project/branch |
| `--clear-history` | Phase 2: delete saved conversation history |

**Session header:**

```
ETABS Agent — HighRise Tower
Provider: ollama / qwen2.5-coder:14b  ●  local — no data leaves your machine
Branch: main  ·  v3  ·  Modified  ·  ETABS not running
Type your question or instruction. Ctrl+C to exit.

You>
```

**Example session:**

```
You> what changed between v2 and v3?

Agent> Comparing v2 → v3 on main branch:
       • 6 column sections increased: C1–C6 from 30×30 to 36×36 cm
       • No load case changes
       • No geometry changes
       Committed 2 days ago by John Doe: "Updated column sections per analysis"

You> commit my current changes

Agent> I'll run: ext commit
       Message? [enter to describe your changes] Increased beam B45 to W21x93

Agent> I'll run: ext commit "Increased beam B45 to W21x93"
       Saves current working file as v4 on branch main. Confirm? [y/n] y

Agent> ✓ Version v4 saved. Working file is now CLEAN.
```

**What the agent can do (Phase 1):**

| Category | Operations |
|---|---|
| Read | status, log, show, branch list, diff, etabs status, remote status, config list |
| Write (with confirmation) | commit, create branch, switch branch, checkout, stash save/pop, etabs open/close/recover, push, pull |
| Deferred to Phase 2 | analyze, report generation, etabs unlock (require streaming UI) |

**What the agent cannot do:**

- Operate ETABS interactively (click buttons, enter values in the UI)
- See what is displayed on screen in ETABS
- Open multiple operations in parallel
- Bypass the state machine (same guards as the CLI)
- Access `.edb` binary content or raw Parquet data

**Privacy:** The agent sends your text messages and project state summary
to the configured provider. It never sends `.edb` binary data, raw Parquet,
full E2K files, or your `config.local.toml`. With `ai.provider = "ollama"`,
nothing leaves your machine.

See `references/ai.md` for full provider setup, privacy details, and
configuration options.

---

## Aliases

| Alias | Expands to |
|---|---|
| `ext ci` | `ext commit` |
| `ext co` | `ext checkout` |
| `ext sw` | `ext switch` |

---

## Sidecar Reference

`etab-cli.exe` is called automatically by the Rust CLI. Direct use is for debugging only.

```bash
etab-cli get-status
etab-cli validate --file <path>
etab-cli open-model --file <path> [--hidden]
etab-cli close-model [--save|--no-save]
etab-cli unlock-model --file <path>
etab-cli generate-e2k --file <path> --output <path> [--overwrite]
etab-cli run-analysis --file <path>
etab-cli extract-results --file <path> --output-dir <path>
etab-cli extract-materials --file <path> --output <path>
etab-cli save-snapshot --file <path> --output-dir <path> [--with-results] [--overwrite]
```

---

## Quick Reference

```bash
# Start
ext init "Project" --edb "C:\model.edb"
ext commit "Initial model"

# Daily work
ext etabs open
# work in ETABS, Ctrl+S, close ETABS
ext commit "What changed"

# Design alternative
ext switch -c steel --from main/v3
ext etabs open
# modify, close
ext commit "Steel columns" --analyze
ext diff main/v3 steel/v1

# Capture analysis
ext commit "Analyzed" --analyze
# or add to existing version:
ext analyze v3

# Navigate
ext log
ext checkout v1
ext checkout main/v3

# Stash
ext stash
ext checkout v1
ext stash pop

# Share / sync
ext push
ext pull
ext clone <onedrive-path> --to <local-path>

# Reports
ext report analysis --version v3
ext report comparison --from main/v3 --to steel/v1

# Recover
ext etabs recover
ext etabs unlock

# AI assistant
ext chat                                      # local Ollama (default)
ext chat --provider claude                    # cloud (requires api key)
ext config set ai.provider ollama
ext config set ai.model "qwen2.5-coder:14b"
```