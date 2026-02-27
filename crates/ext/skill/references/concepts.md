# ETABS Extension — Core Concepts

Deep dive into the conceptual model for the ETABS Extension project.

---

## 1. Projects

A **project** is the root container for an ETABS structural model with version control.

**Initialization:**

```bash
ext init "HighRise Tower" --edb "C:\Models\highrise.edb"
```

What happens:
- Creates `.etabs-ext/` inside the project folder
- Copies the `.edb` into `main/working/model.edb`
- Runs `git init` inside `.etabs-ext/`
- Writes `config.toml`, `config.local.toml`, `state.json`, `.gitignore`
- No version created yet — user must run `ext commit` explicitly

**Folder structure:**

```
HighRise/                                  ← user's project folder (untouched)
  .etabs-ext/                              ← all ext data lives here
    config.toml                            ← git-tracked (shared project settings)
    config.local.toml                      ← git-ignored (machine-specific: author, paths)
    state.json                             ← git-ignored (mutable runtime)
    .gitignore
    .git/                                  ← internal git repo (text files only)
    stash/                                 ← git-ignored
      main.edb
      main-meta.json
    │
    main/                                  ← one folder per branch
      working/
        model.edb                          ← git-ignored, active working file
      v1/
        model.edb                          ← git-ignored, committed snapshot
        model.e2k                          ← GIT TRACKED ✓
        manifest.json                      ← GIT TRACKED ✓
        summary.json                       ← GIT TRACKED ✓ (only if analyzed)
        results/                           ← git-ignored
          modal.parquet
          base_reactions.parquet
          story_forces.parquet
          story_drifts.parquet
          joint_displacements.parquet
          wall_pier_forces.parquet
          shell_stresses.parquet
        materials/
          takeoff.parquet
      v2/
        ...
    │
    steel-columns/                         ← created by ext branch
      working/
        model.edb
      v1/
        ...
```

**Why `.edb` is NOT in git:**

Git stores every version of every file as a full binary object with no delta compression. A 50MB `.edb` × 20 versions × 3 branches = 3GB+ in `.git/`. The `.edb` snapshots are stored beside git, managed by ext, and git-ignored.

**What git tracks (text files only):**

```
✓ vN/model.e2k        text export, enables diff between versions
✓ vN/manifest.json    version metadata
✓ vN/summary.json     key result scalars (when analyzed)
✓ config.toml         shared project settings
```

**`.gitignore`:**

```gitignore
*.edb
*.parquet
*/working/
state.json
config.local.toml
stash/
*.edb.lock
*.$et
*.mdb
*.OUT
```

**`config.toml`** (shared — tracked in git, synced to OneDrive):

```toml
[project]
name = "HighRise Tower"
created = "2024-02-05T14:30:00Z"

[etabs]
sidecarPath = "etab-cli.exe"
autoGenerateE2k = true

[behavior]
confirmDestructive = true
pushWorking = false          # whether ext push includes working file by default

[paths]
oneDriveDir = ""             # empty: each machine sets in config.local.toml
reportsDir  = ""             # empty: each machine sets in config.local.toml
```

**`config.local.toml`** (machine-specific — always git-ignored):

```toml
# config.local.toml — machine-specific, always git-ignored
[git]
author = "John Doe"
email  = "john@firm.com"

[paths]
oneDriveDir = "C:\Users\John\OneDrive\Structural\HighRise"
reportsDir  = "C:\Users\John\OneDrive\Structural\HighRise\reports"

[onedrive]
acknowledgedSync = false   # set to true to silence the OneDrive path warning

[ai]
provider    = "ollama"              # "ollama" | "claude" | "openai"
model       = "qwen2.5-coder:14b"  # model name for chosen provider
apiKey      = ""                    # required for claude/openai; empty for ollama
baseUrl     = "http://localhost:11434/v1"  # ollama default; change per provider
autoConfirm = false                 # skip [y/n] in ext chat (use carefully)
```

**Config resolution order:**

```
config.local.toml  ← highest priority (machine-specific overrides)
config.toml        ← project defaults (shared)
ext defaults       ← built-in fallbacks

Keys that belong in config.local.toml: git.author, git.email,
paths.oneDriveDir, paths.reportsDir, onedrive.acknowledgedSync,
ai.provider, ai.model, ai.apiKey, ai.baseUrl, ai.autoConfirm.
AI keys always go in config.local.toml. The ext config set command
routes all ai.* keys there automatically — even if the user does not
specify which file. API keys must never appear in config.toml because
it is git-tracked and pushed to OneDrive.
```

**`state.json`:**

```json
{
  "version": 1,
  "currentBranch": "main",
  "workingFile": {
    "branch": "main",
    "basedOnVersion": "v3",
    "lastKnownMtime": "2024-02-05T14:30:00Z",
    "status": "modified"
  },
  "etabs": {
    "pid": null,
    "openFile": null
  },
  "stash": {
    "main": {
      "basedOn": "v3",
      "stashedAt": "2024-02-05T12:00:00Z",
      "description": "auto-stash before checkout v1"
    }
  }
}
```

---

## 2. Branches

Each branch has its **own independent `working/` directory**. Switching branches never copies or overwrites files — each branch always preserves its own working file exactly as left.

**Creating a branch:**

```bash
ext branch steel-columns --from main/v3
```

Sequence:
1. Check available disk space (requires model size + 10% buffer)
2. Create `steel-columns/working/`
3. Atomic copy: `main/v3/model.edb` → `steel-columns/working/model.edb`
4. Record `createdFrom: main/v3` in branch metadata
5. Does NOT auto-switch — use `ext switch -c` to create and switch in one step

**Default `--from` behavior:**

| Invocation | Source |
|---|---|
| `ext branch alt` | Latest committed version of current branch |
| `ext branch alt --from main` | Latest committed version of main |
| `ext branch alt --from main/v3` | Specific snapshot |
| `ext branch alt --from working` | Current dirty working file (explicit opt-in) |

Default never copies a dirty working file — always a clean committed snapshot.

---

## 3. Versions

A **version** is a committed snapshot of the model at a specific point in time.

**Version anatomy:**

```
vN/
  model.edb       ← binary snapshot, NOT in git, opened directly by ETABS
  model.e2k       ← text export, IN git, enables diff
  manifest.json   ← metadata, IN git
  summary.json    ← key result scalars, IN git (only if analyzed)
  results/        ← parquet files, NOT in git
  materials/      ← parquet files, NOT in git
```

**Why `.edb` must be stored (not reconstructed from `.e2k`):**

E2K round-trips are lossy. Re-importing loses analysis results, display settings, some load case parameters, section designer details, auto-mesh settings. The `.edb` is always opened directly — `.e2k` exists only for diffs.

**`manifest.json`:**

```json
{
  "id": "v3",
  "branch": "main",
  "message": "Updated column sections per analysis",
  "author": "John Doe",
  "timestamp": "2024-02-05T14:30:00Z",
  "parent": "v2",
  "edbSizeBytes": 47382528,
  "e2kSizeBytes": 2415620,
  "isAnalyzed": true,
  "gitCommitHash": "a3f8c2d..."
}
```

**`summary.json`** (only present if analyzed):

```json
{
  "analyzedAt": "2024-02-05T14:30:00Z",
  "loadCases": ["DEAD", "LIVE", "EQX", "EQY"],
  "modal": { "numModes": 12, "dominantPeriodX": 2.34, "massParticipationX": 0.95 },
  "baseReaction": { "maxBaseShearX": 4520.5, "maxBaseShearY": 4105.2 },
  "drift": { "maxDriftX": 0.0089, "maxDriftStory": "Story8" }
}
```

**Version identification:**

```bash
v3        # short form — current branch implied
main/v3   # fully qualified
```

---

## 4. Working File & State Machine

The working file is the active `.edb` in a branch's `working/` directory. Its state drives what commands are allowed.

**States:**

| State | Meaning |
|---|---|
| `UNTRACKED` | No version committed yet (fresh init) |
| `CLEAN` | Working file matches latest committed version |
| `MODIFIED` | Working file differs from latest committed version |
| `OPEN_CLEAN` | ETABS has file open, no changes since open |
| `OPEN_MODIFIED` | ETABS has file open, changes detected |
| `ANALYZED` | ETABS closed, analysis results embedded in working file |
| `LOCKED` | ETABS model lock set post-analysis, must unlock before editing |
| `MISSING` | `working/model.edb` does not exist on disk |
| `ORPHANED` | ETABS crashed or killed, lock state unknown |

**State transitions:**

```
UNTRACKED      ──[ext commit]────────────────────► CLEAN
CLEAN          ──[ext etabs open]────────────────► OPEN_CLEAN
CLEAN          ──[external file edit]────────────► MODIFIED
CLEAN          ──[file deleted externally]────────► MISSING
MODIFIED       ──[ext commit]────────────────────► CLEAN
MODIFIED       ──[ext etabs open]────────────────► OPEN_MODIFIED
MODIFIED       ──[ext checkout vN → discard]──────► CLEAN
MODIFIED       ──[ext checkout vN → stash]────────► CLEAN (stash saved)
OPEN_CLEAN     ──[user edits in ETABS]───────────► OPEN_MODIFIED
OPEN_CLEAN     ──[user runs analysis in ETABS]───► ANALYZED (still open)
OPEN_MODIFIED  ──[user runs analysis in ETABS]───► ANALYZED (still open)
OPEN_MODIFIED  ──[ext etabs close --no-save]─────► MODIFIED
ANALYZED       ──[ext etabs close]───────────────► ANALYZED (closed)
ANALYZED       ──[ext commit --analyze]──────────► CLEAN
ANALYZED       ──[user tries to edit in ETABS]───► LOCKED
LOCKED         ──[ext etabs unlock]──────────────► OPEN_CLEAN (results cleared)
OPEN_*         ──[ETABS crashes]─────────────────► ORPHANED
ORPHANED       ──[ext etabs recover]─────────────► MODIFIED or CLEAN
MISSING        ──[ext checkout vN]───────────────► CLEAN
```

**Change detection** uses mtime — fast, zero overhead:

`stat(working/model.edb).mtime > lastKnownMtime` → `MODIFIED`

**Two distinct lock types:**
- **File lock** (OS): ETABS has `.edb` open exclusively. Detected via sidecar `get-status`.
- **Analysis lock** (ETABS): `SapModel.ModelIsLocked = true`. Cleared by `ext etabs unlock`.

---

## 5. `ext switch` and `ext checkout`

Two commands with intentionally separate concerns — the split git introduced in 2019.

### `ext switch <branch>` — navigate between branches

Never changes working file content. Each branch preserves its own working file independently.

```
Decision tree:
  1. ETABS open? → hard stop: "Close ETABS first"
  2. Warn on departure if current branch MODIFIED (does not block)
  3. Load target branch working/model.edb as-is
  4. Report target state: CLEAN=silent, MODIFIED=warn, MISSING=warn, ORPHANED=warn
```

**`ext switch -c <branch>`** — create and switch in one command (mirrors `git switch -c`).

### `ext checkout <version>` — restore working file to a specific version

```
Decision tree:
  1. Cross-branch? → apply ext switch rules first
  2. ETABS open? → hard stop
  3. Working file MODIFIED?
       → prompt: [c] Commit first  [s] Stash  [d] Discard  [x] Cancel
  4. Snapshot vN/model.edb missing? → error with available versions listed
  5. Copy vN/model.edb → working/model.edb
  6. state: basedOn=vN, CLEAN
```

---

## 6. Stash

One stash slot per branch. Not a stack.

```bash
ext stash                    # stash current working file
ext stash list               # show all stashes across branches
ext stash pop                # restore stash to current working file
ext stash drop               # discard stash for current branch
```

Stored in `.etabs-ext/stash/<branch>.edb` + `<branch>-meta.json`. Triggered automatically during `ext checkout` when user chooses `[s]`.

---

## 7. `ext commit` — Full Sequence

```bash
ext commit "Updated column sections"
ext commit "Final design" --analyze
ext commit "Quick save" --no-e2k
```

**Without `--analyze`:**

```
1. Verify ETABS not running
2. Copy working/model.edb → vN/model.edb  (snapshot)
3. sidecar save-snapshot on vN/model.edb:
     export → vN/model.e2k
     extract → vN/materials/takeoff.parquet
4. Write vN/manifest.json  { isAnalyzed: false }
5. git commit: vN/model.e2k + vN/manifest.json
6. state: CLEAN, basedOn=vN
```

**With `--analyze`** (after steps 1–6, on the snapshot — NOT the working file):

```
7. sidecar: open vN/model.edb --hidden
8. sidecar: run-analysis (blocks)
9. sidecar: extract-results → vN/results/*.parquet
10. sidecar: close
11. Write vN/summary.json, update vN/manifest.json { isAnalyzed: true }
12. git commit: summary.json + manifest (internal — hidden from ext log)

Working file: untouched throughout steps 7–12
```

**Why analysis runs on snapshot:** ETABS locks the model post-analysis. Running on the working file would lock it and require unlocking (which clears results). Running on the snapshot keeps the working file clean and permanently attaches results to the version.

---

## 8. `ext analyze` — Post-Commit Analysis

```bash
ext analyze v2
ext analyze main/v2
```

Runs analysis on an already-committed version. Working file untouched.

---

## 9. E2K Files

E2K is ETABS's text-based model export — the diff layer. `.edb` is binary (no diff); `.e2k` is structured text (full diff). Always generated via sidecar, never via ETABS auto-save. Requires ETABS to be closed. Duration: 15–60s.

**`--no-e2k`:** Skip generation for rapid iteration.

**Phase 1 diff** — raw unified git diff:

```diff
--- a/main/v2/model.e2k
+++ b/main/v3/model.e2k
-  Frame=C1 Shape=Rectangular Depth=30 Width=30
+  Frame=C1 Shape=Rectangular Depth=36 Width=36
```

---

## 10. Analysis Results & Parquet Storage

**Why Parquet:** Joint displacements on a 30-story building = 360,000+ rows. Parquet is columnar, ~10x smaller than JSON, Polars-native.

**Schemas:** `modal`, `base_reactions`, `story_forces`, `story_drifts`, `joint_displacements`, `wall_pier_forces`, `shell_stresses`, `materials/takeoff` — see `architecture.md` for full column definitions.

---

## 11. ETABS Sidecar

The sidecar (`etab-cli.exe`) is a C# .NET 10 console app — the only component that can call ETABS COM APIs.

**IPC:** stdin=nothing, stdout=`Result<T>` JSON, stderr=progress, exit=0/1.

**`save-snapshot`** is the key composite: opens ETABS once → exports E2K → extracts materials → optionally runs analysis → closes. Minimizes startup overhead.

**All operations:** `get-status`, `validate`, `open-model [--hidden]`, `close-model`, `unlock-model`, `generate-e2k`, `run-analysis`, `extract-results`, `extract-materials`, `save-snapshot [--with-results]`

---

## 12. OneDrive Integration

All heavy work (`.edb`, Parquet, analysis) stays local. OneDrive is used only for:
- Transporting the project to other machines (`ext push` / `ext pull` / `ext clone`)
- Storing PDF reports (`paths.reportsDir`)

**OneDrive path detection:**

On `ext init`, if the project path or `--edb` is inside a OneDrive-synced folder:

```
⚠ OneDrive path detected

  ETABS model files cannot be version-controlled inside OneDrive.
  OneDrive sync interferes with ETABS file writes and would upload
  gigabytes of binary data continuously.

  Recommended: Initialize in a local folder.
  Example: C:\ETABSProjects\HighRise

  Reports will save to OneDrive automatically via paths.reportsDir.

  [c] Continue anyway (not recommended)
  [x] Cancel
```

If `ext status` detects the project is inside a synced OneDrive path, it warns persistently until acknowledged:

```toml
# config.local.toml
[onedrive]
acknowledgedSync = true   # silence the warning after user reads it
```

**OneDrive layout (set by `ext push`):**

```
OneDrive/Structural/HighRise/        ← shared project folder
  project.json                       ← manifest: branches, versions, who pushed when
  git-bundle                         ← full git history as single portable file
  edb/
    main-v1.edb
    main-v2.edb
    main-v3.edb
    steel-columns-v1.edb
  reports/                           ← PDF reports saved here by ext report
    main-v3-analysis.pdf
    main-v3-bom.pdf
```

---

## 13. `ext push` / `ext pull` / `ext clone`

These commands move the project between machines via OneDrive. No merge logic — just transport.

### `ext push`

```bash
ext push                          # push all new versions to OneDrive
ext push --include-working        # also push current working file
ext push --branch steel-columns   # push one branch only
```

Sequence:
1. Resolve `oneDriveDir` from `config.local.toml`
2. Read `OneDrive/project.json` — check what's already there
3. **Conflict check:** if remote has a version with same ID but different content → prompt (see below)
4. Bundle git history: `git bundle create → OneDrive/git-bundle`
5. Copy new `.edb` files: `vN/model.edb → OneDrive/edb/<branch>-vN.edb` (only new ones)
6. Copy working file if `--include-working`
7. Update `OneDrive/project.json`

**Conflict prompt:**

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
ext pull                          # pull all new versions from OneDrive
ext pull --branch steel-columns   # pull one branch only
```

Sequence:
1. Read `OneDrive/project.json` — find versions not in local
2. `git fetch` from `OneDrive/git-bundle` → restores text files
3. Copy new `.edb` files: `OneDrive/edb/<branch>-vN.edb → vN/model.edb`
4. Report what was pulled

### `ext clone`

First-time setup on a new machine. Interactive wizard.

```bash
ext clone "C:\Users\Jane\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"
```

Sequence:
1. Read `OneDrive/project.json`
2. Create local `.etabs-ext/` structure
3. `git clone --local OneDrive/git-bundle` → restores all text files
4. Copy all `.edb` files from `OneDrive/edb/`
5. Interactive prompts:
   ```
   Author name:          [Jane Smith        ]
   Author email:         [jane@firm.com     ]
   OneDrive reports dir: [C:\Users\Jane\OneDrive\Structural\HighRise\reports]
   ```
6. Write `config.local.toml` with machine-specific settings
7. Set working file to latest version of main
8. Write `state.json`

---

## 14. Reports

Reports generated in Rust using Typst as an embedded crate. No `.typ` template files on disk.

**Pipeline:** Polars calculations on Parquet → Rust structs → Typst markup string → `TypstWorld::compile()` → PDF bytes → write to `reportsDir`.

**Output path:** If `paths.reportsDir` is set in `config.local.toml`, all reports default to that location (typically OneDrive). `--out` flag overrides for one-off outputs.

**Auto-naming** (optional):

```toml
[paths]
reportNaming = "{branch}-{version}-{type}"  # → main-v3-analysis.pdf
```

**Phase 1 reports:**

```bash
ext report analysis --version v3               # → reportsDir/main-v3-analysis.pdf
ext report bom --version v3                    # → reportsDir/main-v3-bom.pdf
ext report comparison --from main/v3 --to steel/v1   # → reportsDir/...
```

---

## 15. Recommended Team Workflow (Phase 1)

Since Phase 1 has no merge — prevent conflicts by convention:

```
main           → shared milestone versions only (pushed after major decisions)
john/          → John's working branch
jane/          → Jane's working branch
```

Jane finishes a design iteration:
```bash
ext switch -c jane/foundation-v2 --from main/v3
# ... work, commit, analyze ...
ext push --branch jane/foundation-v2
```

John reviews and adopts:
```bash
ext pull --branch jane/foundation-v2
ext report comparison --from main/v3 --to jane/foundation-v2/v1
ext switch main
ext checkout jane/foundation-v2/v1   # copies her edb to main/working
ext commit "Adopt Jane's foundation redesign" --analyze
ext push
```

---

## 16. Version Control Internals

**Writes → git subprocess:** `git add`, `git commit`, `git checkout -b` — simple, correct.

**Reads → gix crate:** fast, pure Rust, no C dependency. Used for `ext log`, `ext diff`, blob reads.

**Internal vs user-visible commits:** Internal git commits (analysis results, init) are prefixed `ext:` and filtered from `ext log`.

---

## 17. Calculation Layer

Polars processes Parquet data into engineering outputs used by reports:

| Raw Parquet | Calculated output |
|---|---|
| `joint_displacements` | Story drift ratios, max displacement envelope |
| `story_forces` | Overturning moment, cumulative shear |
| `base_reactions` | Base shear vs code minimum check |
| `modal` | Effective mass check, torsional irregularity |
| `wall_pier_forces` | Shear stress ratio V/Ag, DCR |
| `shell_stresses` | Principal stress, utilization ratio |
| `takeoff` | Total weight by material, cost estimate |

---

## 18. Phase 1 Scope

**In scope:**
- Local single-user workflow
- `init`, `status`, `log`, `show`, `diff`
- `branch`, `switch`, `switch -c`, `checkout`, `stash`
- `commit`, `analyze`
- `etabs open/close/status/validate/unlock/recover`
- `push`, `pull`, `clone` (OneDrive transport)
- E2K generation, Parquet results extraction
- Polars calculations, Typst PDF reports
- OneDrive detection and warning
- `config.local.toml` per-machine settings

**Out of scope:**
- Branch merge / conflict resolution
- Structured E2K section parser (raw diff only)
- Geometry diff, 3D visualization, mode shape diagrams
- Web interface, AI-assisted analysis
- Multiple stashes per branch
- Real-time collaboration

---

## 19. AI Agent (`ext chat`)

The AI agent is an optional assistant that can answer questions about the
project and execute `ext` operations on behalf of the user after confirmation.

**Architecture:** The agent calls `ext-api` functions as tools — the same
functions used by the CLI. The state machine and permission matrix are
enforced identically. The agent cannot bypass any guard the CLI cannot bypass.

```
ext chat (REPL)
    ↓
ext-agent         ← conversation loop, tool dispatch, confirmation gate
    ↓
ext-agent-llm     ← LlmClient trait (provider-agnostic)
    ↓
Claude / Ollama / OpenAI-compat
    ↓  (tool calls only)
ext-api           ← same functions as CLI
```

**Provider configuration** (all in `config.local.toml`):

```toml
[ai]
provider = "ollama"                        # local, private, no API key needed
model    = "qwen2.5-coder:14b"
baseUrl  = "http://localhost:11434/v1"
```

```toml
[ai]
provider = "claude"                        # cloud, requires API key
model    = "claude-opus-4-5"
apiKey   = "sk-ant-..."
```

**Privacy:** The agent sends message text and project state summary (from
`ext status`) to the configured provider. It never sends `.edb` binary data,
raw Parquet files, or `config.local.toml` contents. With `provider = "ollama"`,
nothing leaves the machine.

**Phase 1 tools** — available at launch:

| Category | Tools |
|---|---|
| Read | status, log, show, branch list, diff, etabs status, remote status, config list |
| Write (confirmed) | commit, create branch, switch, checkout, stash save/pop, etabs open/close/recover, push, pull |

**Phase 2 tools** — require streaming UI:

| Tool | Why deferred |
|---|---|
| `analyze_version` | 2–5 minute runtime needs live progress |
| `generate_report` | PDF compilation needs streaming status |
| `etabs_unlock` | Sensitive operation, needs careful UX |

**What the agent cannot do:**

- Operate the ETABS user interface (click, type, read screen)
- Access `.edb` binary content directly
- Bypass state machine guards
- Write to `config.toml` (agent config is always `config.local.toml`)
- Run Phase 2 tools before Phase 2 ships

See `references/ai.md` for full provider setup, privacy details, and
implementation guide.