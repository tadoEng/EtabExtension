# ETABS Extension — Architecture

System architecture, crate responsibilities, data flows, and design decisions.

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interfaces                        │
│                                                             │
│  ┌─────────────┐   ┌───────────────┐   ┌────────────────┐  │
│  │   ext CLI   │   │  ext-tauri    │   │   ext-web      │  │
│  │   (Rust)    │   │  (Desktop)    │   │   (Future)     │  │
│  └──────┬──────┘   └───────┬───────┘   └───────┬────────┘  │
└─────────┼──────────────────┼────────────────────┼───────────┘
          └──────────────────▼────────────────────┘
                             │
                ┌────────────▼────────────┐
                │         ext-api         │  ← Single source of truth
                │   Plain async Rust fns  │    No framework types
                └────────────┬────────────┘
                             │
             ┌───────────────┼───────────────┐
             │               │               │
       ┌─────▼──────┐  ┌─────▼─────┐  ┌─────▼──────┐
       │  ext-core  │  │  ext-db   │  │  ext-error  │
       │   domain   │  │  storage  │  │    types    │
       └─────┬──────┘  └───────────┘  └────────────┘
             │
   ┌─────────▼──────────────────────┐
   │         External Systems        │
   │  ┌────────────┐  ┌───────────┐  │
   │  │ etab-cli   │  │ git + gix │  │
   │  │ (C# .NET10)│  │  (vcs)    │  │
   │  └────────────┘  └───────────┘  │
   └─────────────────────────────────┘
```

---

## Why `ext-api` is the Center

All business operations are plain async Rust functions with no framework types. CLI, Tauri, and future web are thin adapters.

```rust
// ext-api — pure, framework-free
pub async fn commit_version(
    ctx: &AppContext,
    message: &str,
    options: CommitOptions,      // analyze: bool, no_e2k: bool
) -> Result<VersionCommitted, EtabsError>

pub async fn push(
    ctx: &AppContext,
    options: PushOptions,        // include_working: bool, branch: Option<String>
) -> Result<PushResult, EtabsError>

pub async fn clone_project(
    onedrive_path: &Path,
    local_path: &Path,
    identity: Identity,          // author, email, paths
) -> Result<(), EtabsError>
```

```rust
// CLI adapter — clap args → ext-api call
async fn handle_commit(args: &CommitArgs, ctx: &AppContext) -> Result<()> {
    let result = ext_api::commit_version(ctx, &args.message, args.into()).await?;
    out.write_result(result)
}

// Tauri adapter — IPC wrapper
#[tauri::command]
async fn commit_version(
    state: tauri::State<'_, AppContext>,
    message: String, analyze: bool,
) -> Result<VersionCommitted, String> {
    ext_api::commit_version(&state, &message, CommitOptions { analyze, ..Default::default() })
        .await.map_err(|e| e.to_string())
}
```

---

## Crate Responsibilities

### `ext-error`

Shared error types. Depends on nothing.

```rust
pub enum EtabsError {
    SidecarNotFound(PathBuf),
    SidecarFailed { command: String, message: String },
    EtabsNotRunning,
    EtabsFileOpen(PathBuf),
    ModelLocked,
    ModelMissing(PathBuf),
    SnapshotMissing { version: String },
    VersionNotFound(String),
    BranchNotFound(String),
    BranchAlreadyExists(String),
    InsufficientDiskSpace { required: u64, available: u64 },
    GitError(String),
    ParquetError(String),
    OneDriveConflict { version: String, remote_author: String, remote_message: String },
    OneDriveNotConfigured,
    IoError(#[from] std::io::Error),
}
```

---

### `ext-core`

Pure domain logic. No I/O frameworks, no clap, no Tauri.

```
ext-core/src/
  project/
    mod.rs          Project struct, init(), open()
    onedrive.rs     OneDrive path detection, project.json read/write
  branch/
    mod.rs          Branch struct, create(), list()
    copy.rs         Atomic .edb copy: disk check → temp write → rename
  version/
    mod.rs          Version struct, commit(), restore(), list(), show()
    manifest.rs     manifest.json + summary.json read/write
  state/
    mod.rs          WorkingFileState enum (9 states)
    machine.rs      Transitions, mtime detection, status resolution
  stash/
    mod.rs          Stash: save, pop, drop, list
  switch/
    mod.rs          ext switch logic + decision tree
  checkout/
    mod.rs          ext checkout logic + decision tree (cross-branch aware)
  sidecar/
    mod.rs          SidecarClient: spawn etab-cli, parse stdout JSON
    commands.rs     Typed request/response structs for all sidecar ops
    locate.rs       Find etab-cli.exe: config → env var → PATH
  vcs/
    git_ops.rs      Write: git subprocess (commit, branch, checkout, bundle)
    gix_ops.rs      Read: gix (log, diff, blob content at commit)
  remote/
    mod.rs          push(), pull(), clone_project(), remote_status()
    bundle.rs       git bundle create/unbundle wrappers
    transfer.rs     .edb file copy to/from OneDrive with progress
    conflict.rs     Conflict detection and resolution prompts
    project_json.rs project.json schema: read/write/merge
  diff/
    mod.rs          Phase 1: raw git diff passthrough on E2K files
  analyze/
    mod.rs          ext analyze: open snapshot, run, extract, close
  reports/
    world.rs        TypstWorld impl (font loading, Liberation Sans bundled)
    calculations/
      modal.rs      Polars: periods, mass participation
      drifts.rs     Polars: story drift ratios
      reactions.rs  Polars: base shear summary
      forces.rs     Polars: story forces, overturning moment
      dcr.rs        Polars: demand-capacity ratios
      materials.rs  Polars: takeoff aggregation, cost estimate
    generators/
      analysis.rs   generate_typst_from_data() → analysis PDF
      bom.rs        generate_typst_from_data() → BOM PDF
      comparison.rs generate_typst_from_data() → comparison PDF
```

---

### `ext-db`

Persistent storage.

```
ext-db/src/
  state.rs       Read/write state.json  (serde_json)
  config.rs      Read/write config.toml + config.local.toml  (toml crate)
                 → Handles config resolution order: local → shared → defaults
  registry.rs    SQLite list of known projects  (rusqlite)
```

**Config resolution in `ext-db`:**

```rust
pub struct Config {
    shared: SharedConfig,    // from config.toml  (git-tracked)
    local: LocalConfig,      // from config.local.toml  (git-ignored)
}

impl Config {
    pub fn author(&self) -> &str {
        self.local.git.author.as_deref()
            .or(self.shared.git.author.as_deref())
            .unwrap_or("Unknown")
    }

    pub fn reports_dir(&self) -> Option<&Path> {
        self.local.paths.reports_dir.as_deref()
            .or(self.shared.paths.reports_dir.as_deref())
    }

    pub fn onedrive_dir(&self) -> Option<&Path> {
        self.local.paths.onedrive_dir.as_deref()
    }
}
```

---

### `ext-api`

Orchestration layer. Thin functions over `ext-core` and `ext-db`.

```
ext-api/src/
  context.rs     AppContext: project path, resolved Config, sidecar path
  project.rs     init(), status()
  branch.rs      create(), list(), show(), delete()
  switch.rs      switch(), switch_and_create()
  checkout.rs    checkout()
  stash.rs       stash_save(), stash_pop(), stash_drop(), stash_list()
  version.rs     commit(), log(), show()
  analyze.rs     analyze()
  etabs.rs       open(), close(), status(), validate(), unlock(), recover()
  diff.rs        diff()
  report.rs      analysis(), bom(), comparison()
  remote.rs      push(), pull(), clone_project(), remote_status()
  config.rs      get(), set(), list()
```

---

### `ext` (CLI binary)

Thin clap layer over `ext-api`. Zero business logic.

```
ext/src/
  main.rs          Entry point, AppContext construction
  commands/
    project.rs     init, status, log, show
    branch.rs      branch (list/create/delete)
    switch.rs      switch, switch -c
    checkout.rs    checkout
    stash.rs       stash list/pop/drop
    version.rs     commit, analyze
    diff.rs        diff
    etabs.rs       etabs open/close/status/validate/unlock/recover
    report.rs      report analysis/bom/comparison
    remote.rs      push, pull, clone, remote status
    config.rs      config get/set/list/edit
  output.rs        Format: human / --json / --shell
```

---

## Data Flow: `ext commit "message"`

```
ext commit "message"
  │
  ├─ AppContext::load()           read config.toml + config.local.toml + state.json
  ├─ StateManager::resolve()      mtime check → state
  ├─ Guard: etabs_not_running()   sidecar get-status
  │
  ├─ vN = next_version_id()
  ├─ atomic_copy(working → vN/model.edb)
  │
  ├─ SidecarClient::save_snapshot(vN/model.edb)
  │     opens hidden, exports e2k, extracts materials, closes
  │
  ├─ write vN/manifest.json  { isAnalyzed: false }
  ├─ git add + git commit "message"
  ├─ state: CLEAN, basedOn=vN
  │
  │   with --analyze:
  ├─ SidecarClient::run_analysis(vN/model.edb)
  │     opens hidden, runs analysis, extracts results, closes
  ├─ write vN/summary.json
  ├─ update vN/manifest.json  { isAnalyzed: true }
  └─ git commit "ext: analysis results vN"  (internal)
```

---

## Data Flow: `ext push`

```
ext push
  │
  ├─ Resolve oneDriveDir from config.local.toml
  │     missing? → error: "Run: ext config set paths.oneDriveDir <path>"
  │
  ├─ Read OneDrive/project.json  (if exists)
  ├─ Conflict check: any local vN matches remote vN with different content?
  │     YES → prompt: rename to vN+1 / view diff / cancel
  │
  ├─ git bundle create OneDrive/git-bundle  (full history, all branches)
  │
  ├─ For each branch/version not yet on OneDrive:
  │     atomic_copy(vN/model.edb → OneDrive/edb/<branch>-vN.edb)
  │     show progress bar per file
  │
  ├─ If --include-working:
  │     copy working/model.edb → OneDrive/edb/<branch>-working.edb
  │
  └─ Write OneDrive/project.json  { pushedBy, pushedAt, branches, versions }
```

---

## Data Flow: `ext clone <onedrive-path> --to <local-path>`

```
ext clone OneDrive/HighRise --to C:\ETABSProjects\HighRise
  │
  ├─ Read OneDrive/project.json
  ├─ Create local .etabs-ext/ structure
  │
  ├─ git clone --local OneDrive/git-bundle .etabs-ext/.git/
  │     restores all text files: e2k, manifests, summaries, config.toml
  │
  ├─ For each branch/version in project.json:
  │     copy OneDrive/edb/<branch>-vN.edb → vN/model.edb
  │
  ├─ Interactive wizard:
  │     prompt author, email, oneDriveDir, reportsDir
  │     write config.local.toml
  │
  ├─ Set working file to latest version of main
  └─ Write state.json  { status: CLEAN, basedOn: latest }
```

---

## Data Flow: `ext report analysis --version v3`

```
ext report analysis --version v3
  │
  ├─ Verify v3/manifest.json { isAnalyzed: true }
  ├─ Load Parquet: modal, base_reactions, story_forces, story_drifts,
  │               wall_pier_forces, shell_stresses, materials/takeoff
  │
  ├─ Polars calculations:
  │     modal::dominant_periods()
  │     drifts::story_drift_ratios()
  │     reactions::base_shear_summary()
  │     forces::overturning_moment()
  │     dcr::shear_wall_utilization()
  │
  ├─ Build ReportData struct
  ├─ generators::analysis::generate_typst_from_data(&data)  → Typst String
  ├─ TypstWorld::compile()  → PDF bytes
  │
  ├─ Resolve output path:
  │     --out flag → use that path
  │     else → config.reports_dir() / auto_name(branch, version, type)
  │
  └─ fs::write(output_path, pdf_bytes)
```

---

## Sidecar IPC Protocol

```
stdin:   nothing
stdout:  Result<T> JSON (always, even on failure)
stderr:  human-readable progress (forwarded to terminal)
exit:    0 = success, 1 = failure
```

```rust
pub struct SidecarClient { path: PathBuf }

impl SidecarClient {
    pub async fn run<T: DeserializeOwned>(
        &self, args: &[&str],
    ) -> Result<T, EtabsError> {
        let output = Command::new(&self.path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|_| EtabsError::SidecarNotFound(self.path.clone()))?;

        let result: SidecarResult<T> = serde_json::from_slice(&output.stdout)?;
        match result.success {
            true  => Ok(result.data.unwrap()),
            false => Err(EtabsError::SidecarFailed {
                command: args.join(" "),
                message: result.error.unwrap_or_default(),
            })
        }
    }
}
```

**All sidecar commands:**

| Command | Key flags | Description |
|---|---|---|
| `get-status` | | Running, PID, open file, lock state |
| `validate` | `--file` | File validity, analysis status |
| `open-model` | `--file [--hidden]` | Open ETABS visible or hidden |
| `close-model` | `[--save\|--no-save]` | Close ETABS or model |
| `unlock-model` | `--file` | `SetModelIsLocked(false)` |
| `generate-e2k` | `--file --output` | Export E2K text file |
| `run-analysis` | `--file` | `RunCompleteAnalysis()`, blocks |
| `extract-results` | `--file --output-dir` | All result Parquet files |
| `extract-materials` | `--file --output` | Materials Parquet |
| `save-snapshot` | `--file --output-dir [--with-results]` | Composite: E2K + materials (+ results) in one session |

---

## OneDrive Layout

```
OneDrive/Structural/HighRise/        ← configured in config.local.toml: paths.oneDriveDir
  project.json                       ← manifest (who pushed what when)
  git-bundle                         ← full git history as single portable file
  edb/
    main-v1.edb                      ← version snapshots
    main-v2.edb
    main-v3.edb
    main-working.edb                 ← only if pushed with --include-working
    steel-columns-v1.edb
    jane/foundation-v1.edb
  reports/                           ← configured in config.local.toml: paths.reportsDir
    main-v3-analysis.pdf
    main-v3-bom.pdf
    steel-columns-v1-vs-main-v3-comparison.pdf
```

**`project.json`:**

```json
{
  "projectName": "HighRise Tower",
  "pushedAt": "2024-02-05T14:30:00Z",
  "pushedBy": "John Doe",
  "branches": {
    "main": {
      "latestVersion": "v3",
      "workingIncluded": false,
      "versions": ["v1", "v2", "v3"]
    },
    "steel-columns": {
      "latestVersion": "v1",
      "versions": ["v1"]
    }
  }
}
```

---

## Storage Layout (Local)

```
.etabs-ext/
  config.toml                    ← tracked (shared settings)
  config.local.toml              ← ignored (author, paths — machine-specific)
  state.json                     ← ignored
  .gitignore
  .git/
  stash/
    <branch>.edb                 ← ignored
    <branch>-meta.json           ← ignored

<branch>/
  working/
    model.edb                    ← ignored

  vN/
    model.edb                    ← ignored
    model.e2k                    ← TRACKED
    manifest.json                ← TRACKED
    summary.json                 ← TRACKED (if analyzed)
    results/                     ← ignored (parquet)
    materials/                   ← ignored (parquet)
```

---

## Technology Stack

| Layer | Technology | Reason |
|---|---|---|
| CLI | Rust + clap | Single binary, fast startup |
| Desktop app | Tauri + React/TS | Native shell, web frontend |
| Business logic | Rust (ext-api, ext-core) | Shared across all surfaces |
| ETABS COM | C# .NET 10 (etab-cli) | Only .NET can call COM |
| VCS writes | git subprocess | Simple, correct, reliable |
| VCS reads | gix (pure Rust) | Fast, no C dep |
| Remote transport | git bundle + file copy | No server needed, OneDrive-native |
| Analysis data | Apache Parquet | Columnar, compressed, Polars-native |
| Calculations | Polars (Rust) | DataFrame ops on Parquet |
| Reports | Typst (embedded crate) | Fast, programmatic PDF |
| Parquet write (C#) | Parquet.Net | Pure C#, no native deps |
| State/config | JSON + TOML | Human-readable, git-trackable |
| Project registry | SQLite (rusqlite) | Lightweight, no server |

---

## Key Design Decisions

**`ext-api` as single center.** All operations as plain Rust functions. CLI, Tauri, web are adapters. Tests target `ext-api` directly.

**`.edb` beside git, not in git.** Binary files would make `.git/` grow 50MB+ per version. Snapshots are stored in numbered folders and git-ignored.

**`.e2k` exists only for diff.** E2K round-trips are lossy. `.edb` is always canonical; `.e2k` is generated per commit for `ext diff` only.

**Analysis runs on committed snapshot.** ETABS locks the model post-analysis. Running on the snapshot keeps the working file clean and permanently attaches results to the version.

**`ext switch` and `ext checkout` are separate.** Switch = branch navigation (never touches file content). Checkout = version restoration. Mirrors the `git switch` / `git restore` split from 2019.

**OneDrive as transport layer, not storage.** All heavy files (`.edb`, Parquet, analysis) stay local. OneDrive stores the git bundle + `.edb` snapshots for machine-to-machine transfer, and receives PDF reports as deliverables.

**`config.local.toml` for machine-specific settings.** Author name, email, OneDrive paths, and report paths differ per machine. Git-ignored. Created interactively on `ext init` and `ext clone`. Never overwritten by `ext pull`.

**`git bundle` for history transport.** Self-contained, no server, works perfectly over OneDrive file sync. One file contains full history of all branches.

**Conflict prevention over conflict resolution.** Phase 1 detects version ID conflicts on `ext push` and offers safe rename (v4 → v5) rather than attempting to merge binary `.edb` files.

**One stash slot per branch.** Covers the primary use case without complexity.

**mtime for change detection.** SHA-256 of a 50MB `.edb` on every `ext status` is too slow. mtime is instant and sufficient for single-user local workflow.

---

## Phase 1 Build Order

```
Week 1–2: Foundation
  ├── ext-error crate (include OneDriveConflict, OneDriveNotConfigured)
  ├── ext-db: config.toml + config.local.toml resolution
  ├── Sidecar: fix validate --file, add get-status, open-model, close-model, unlock-model
  ├── Rust: SidecarClient (spawn + JSON parse)
  └── Rust: ext init (with OneDrive detection) + ext status

Week 3–4: Version Control Core
  ├── Rust: ext commit (e2k only)
  ├── Rust: ext log + ext show
  ├── Rust: ext branch + ext switch + ext switch -c
  ├── Rust: ext checkout (single-branch + cross-branch)
  ├── Rust: ext stash (save/pop/drop/list)
  └── git init, gitignore, subprocess ops, gix reads

Week 5–6: State Machine + ETABS Commands
  ├── Rust: full 9-state machine with mtime detection
  ├── Rust: ext etabs open/close/status/validate/unlock/recover
  ├── Rust: ext diff (raw git diff passthrough)
  └── Rust: ORPHANED + MISSING recovery paths

Week 7–8: Analysis Pipeline
  ├── C#: Parquet.Net, extract-results (all 7 schemas), extract-materials
  ├── C#: save-snapshot composite command
  ├── Rust: ext commit --analyze
  ├── Rust: ext analyze <version>
  └── Rust: Polars reads + all 6 calculation modules

Week 9–10: Reports + Remote
  ├── Rust: TypstWorld + Windows font loading + Liberation Sans
  ├── [VALIDATE EARLY Day 1]: hello-world PDF on Windows
  ├── Rust: analysis / BOM / comparison report generators
  ├── Rust: ext report commands (auto-path to reportsDir / OneDrive)
  ├── Rust: ext push (git bundle + edb copy + conflict detection)
  ├── Rust: ext pull (git fetch bundle + edb copy)
  ├── Rust: ext clone (wizard + full restore)
  └── Rust: ext remote status
```