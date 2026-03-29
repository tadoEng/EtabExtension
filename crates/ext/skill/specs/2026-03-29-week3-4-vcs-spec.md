# Week 3–4 VCS Implementation Spec
# ETABS Extension — Version Control Core

**Date:** 2026-03-29  
**Phase:** Phase 1, Weeks 3–4  
**Status:** Approved for implementation  
**Author:** EtabExtension Team  

---

## Overview

This spec covers the full implementation of the version control core: the
`commit`, `log`, `show`, `branch`, `switch`, `checkout`, and `stash` commands,
together with all supporting domain logic in `ext-core` and orchestration in
`ext-api`.

Week 3–4 is the largest single increment in Phase 1. Everything above it
(analysis, reports, remote, AI) depends on a correct and well-tested VCS core.

**Prerequisite:** Week 1–2 foundation is complete and all tests pass.
(`ext init`, `ext status`, `AppContext`, `SidecarClient`, `StateFile`,
`Config` two-tier resolution.)

**Deliverable at end of Week 4:** Engineers can run the full cycle:

```bash
ext init "HighRise" --edb model.edb
ext commit "Initial model"
ext branch steel-columns --from main/v1
ext switch steel-columns
ext etabs open           # open in ETABS, edit, Ctrl+S, close
ext commit "Steel option"
ext checkout main/v1
ext stash
ext stash pop
ext log
ext show v1
```

All without analysis, reports, or remote sync.

---

## 1. Module Structure

### 1.1 `ext-core` layout (new modules this week)

```
ext-core/src/
  vcs/
    mod.rs          re-exports, module-level doc only
    subprocess.rs   all git write ops via std::process::Command
    read.rs         all git read ops via gix crate
  version/
    mod.rs          commit(), list(), show(), next_version_id()
    snapshot.rs     atomic copy + disk space check + .partial sentinel
    manifest.rs     manifest.json + summary.json read/write
  branch/
    mod.rs          create(), list(), delete(), branch metadata
    copy.rs         atomic edb copy for branch creation
  stash/
    mod.rs          save(), pop(), drop(), list()
```

`ext-core/src/state.rs` (exists, no changes needed) — already correct.  
`ext-core/src/fs.rs` (exists) — `atomic_copy()` and `stale_tmp_cleanup()` live here.

### 1.2 Why `vcs/` is split into two files

`subprocess.rs` and `read.rs` have zero overlap in dependencies and concern:

- `subprocess.rs` uses `std::process::Command` — synchronous, simple, reliable for writes.
- `read.rs` uses the `gix` crate — async-friendly, pure Rust, no C dep, fast for reads.

Keeping them separate means either side can be swapped without touching the other. File names reflect responsibility (`subprocess` = "this is a shell call", `read` = "this reads git state") rather than the library name, so they stay meaningful if dependencies change.

### 1.3 `ext-db` change (one addition)

Add `stashes: HashMap<String, StashEntry>` to `StateFile`:

```rust
// ext-db/src/state/mod.rs — additions only

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StashEntry {
    pub based_on: Option<String>,
    pub stashed_at: DateTime<Utc>,
    pub description: Option<String>,
}

// In StateFile:
pub stashes: HashMap<String, StashEntry>,   // key = branch name
```

Stash metadata lives in `state.json` — no separate `stash/<branch>-meta.json` file. This keeps all runtime state in one place and one atomic write.

The stash `.edb` binary still lives at `.etabs-ext/stash/<branch>.edb` on disk.

---

## 2. VCS Layer (`ext-core/src/vcs/`)

### 2.1 `subprocess.rs` — git write operations

All functions take `repo: &Path` (the `.etabs-ext/` directory, which is the git repo root) and call `git` as a subprocess.

```rust
pub fn git_add(repo: &Path, paths: &[&Path]) -> Result<()>
pub fn git_commit(repo: &Path, message: &str, author: &str, email: &str) -> Result<String>
    // Returns the commit hash (short, 8 chars)
pub fn git_create_branch(repo: &Path, name: &str) -> Result<()>
pub fn git_checkout_branch(repo: &Path, name: &str) -> Result<()>
pub fn git_delete_branch(repo: &Path, name: &str) -> Result<()>
pub fn git_config(repo: &Path, key: &str, value: &str) -> Result<()>
```

**No subprocess for reads.** If you need to read a commit hash, call `read.rs`.

**Error handling:** Capture `stderr`, include it in `EtabsError::GitError`. Never swallow stderr silently.

```rust
fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("Failed to spawn git: {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(EtabsError::GitError(format!(
            "git {} failed: {}", args.join(" "), stderr
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

### 2.2 `read.rs` — git read operations via gix

```rust
pub fn list_commits(
    repo: &Path,
    branch: &str,
    include_internal: bool,   // false = filter "ext:" prefix
) -> Result<Vec<CommitInfo>>

pub fn latest_version_number(repo: &Path, branch: &str) -> Result<u32>
    // Walks commits, finds highest vN in manifest.json committed to git tree
    // Returns 0 if no user-visible commits yet

pub fn next_version_id(repo: &Path, branch: &str) -> Result<String>
    // Returns "v{latest + 1}"

pub fn read_blob(repo: &Path, commit_hash: &str, path: &str) -> Result<String>
    // Read a tracked file at a specific commit (for ext show, ext diff)

pub fn diff_commits(
    repo: &Path,
    from_hash: &str,
    to_hash: &str,
    path_filter: Option<&str>,  // e.g. "*.e2k"
) -> Result<String>
    // Returns raw unified diff text
```

**`CommitInfo` struct:**

```rust
pub struct CommitInfo {
    pub hash: String,           // short 8-char hash
    pub message: String,        // filtered (no "ext:" prefix commits unless include_internal)
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub version_id: Option<String>,  // "v3" parsed from manifest, None for internal
}
```

**`include_internal` filter:** Commits where `message.starts_with("ext:")` are
internal — hidden from `ext log`. Full audit trail preserved in git; never deleted.

**`next_version_id` reads from git, not the filesystem.** Do not scan `ls <branch>/`
directory for existing `vN/` folders — a partial folder from a failed commit would
corrupt the counter. The git log is the source of truth for committed versions.

### 2.3 `read.rs` — version number resolution detail

```rust
pub fn latest_version_number(repo: &Path, branch: &str) -> Result<u32> {
    // Walk commits on branch (include_internal: true — check all commits)
    // For each commit, try to read manifest.json from the git tree
    // Parse manifest.json.id field (e.g. "v3" → 3)
    // Return the maximum found, or 0 if none
}
```

This is the only correct way to determine `vN`. It works even if the filesystem
has partial `vN/` folders from interrupted commits (those folders have no git commit,
so they don't affect the counter).

---

## 3. Version Layer (`ext-core/src/version/`)

### 3.1 `manifest.rs` — manifest.json + summary.json

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub id: String,                  // "v3"
    pub branch: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub parent: Option<String>,      // "v2", None for first commit
    pub edb_size_bytes: u64,
    pub e2k_size_bytes: Option<u64>, // None if --no-e2k
    pub is_analyzed: bool,
    pub e2k_generated: bool,         // false if --no-e2k
    pub materials_extracted: bool,
    pub git_commit_hash: Option<String>,  // filled after git commit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub analyzed_at: DateTime<Utc>,
    pub load_cases: Vec<String>,
    pub modal: ModalSummary,
    pub base_reaction: BaseReactionSummary,
    pub drift: DriftSummary,
}
```

Both read from and write to disk as pretty-printed JSON. Atomic write (tmp+rename).

### 3.2 `snapshot.rs` — atomic copy + sentinel + disk check

**The `.partial` sentinel pattern:**

Every in-progress `vN/` folder has a `.partial` marker written at creation and
deleted only on full success. Any folder with `.partial` present is considered
incomplete and cleaned up on startup.

```rust
pub fn begin_snapshot(version_dir: &Path) -> Result<PartialGuard>
    // 1. Create vN/ directory
    // 2. Write vN/.partial (empty file)
    // 3. Returns a PartialGuard that deletes the directory on drop (RAII rollback)

pub fn complete_snapshot(guard: PartialGuard) -> Result<()>
    // Deletes vN/.partial
    // Calls guard.disarm() so the RAII rollback does not fire

pub fn cleanup_partial_snapshots(branch_dir: &Path) -> Result<Vec<PathBuf>>
    // Scans branch_dir for vN/ folders containing .partial
    // Deletes them, returns list of cleaned-up paths (for logging)
```

**`PartialGuard` is a RAII guard:**

```rust
pub struct PartialGuard {
    version_dir: PathBuf,
    armed: bool,
}

impl Drop for PartialGuard {
    fn drop(&mut self) {
        if self.armed {
            // Best-effort cleanup — log error but do not panic
            let _ = std::fs::remove_dir_all(&self.version_dir);
        }
    }
}
```

This handles both explicit errors (the `?` unwind) and process kills (cleanup on next
startup via `cleanup_partial_snapshots`). The two mechanisms cover complementary failure
modes.

**Disk space check (always called before any `.edb` copy):**

```rust
pub fn check_disk_space(src: &Path, dst_parent: &Path) -> Result<()>
    // required = src file size
    // available = available space on dst_parent's filesystem
    // if available < required + (required / 10):  // 10% buffer
    //     bail!(EtabsError::InsufficientDiskSpace { required, available })
```

Call `check_disk_space` before every `.edb` copy — both in `snapshot.rs` (commit)
and in `branch/copy.rs` (branch creation). Never skip this check.

### 3.3 `version/mod.rs` — public API

```rust
pub struct CommitRequest<'a> {
    pub message: &'a str,
    pub author: &'a str,
    pub email: &'a str,
    pub branch: &'a str,
    pub working_file: &'a Path,
    pub branch_dir: &'a Path,   // .etabs-ext/<branch>/
    pub repo_dir: &'a Path,     // .etabs-ext/
    pub no_e2k: bool,
}

pub struct CommitOutcome {
    pub version_id: String,     // "v3"
    pub git_hash: String,
    pub e2k_size_bytes: Option<u64>,
    pub manifest_path: PathBuf,
}

pub async fn commit(req: CommitRequest<'_>, sidecar: &SidecarClient) -> Result<CommitOutcome>
    // Full commit sequence — see §3.4 below

pub fn list(branch_dir: &Path, repo_dir: &Path, include_internal: bool) -> Result<Vec<CommitInfo>>
pub fn show(version_id: &str, branch_dir: &Path, repo_dir: &Path) -> Result<VersionManifest>
```

### 3.4 Commit sequence in `version/mod.rs`

This is the exact sequence for `ext commit` without `--analyze`.
Steps are numbered to match `workflow.md §2`.

```
1.  check_disk_space(working_file, branch_dir)
2.  let version_id = next_version_id(repo_dir, branch)?
3.  let version_dir = branch_dir.join(&version_id)
4.  let guard = begin_snapshot(&version_dir)?          // creates dir + .partial
5.  atomic_copy(working_file, version_dir/model.edb)?
6.  if !req.no_e2k:
        sidecar.save_snapshot(version_dir/model.edb, &version_dir, false).await?
        // exports model.e2k + materials/takeoff.parquet
    else:
        // skip sidecar entirely — no e2k, no materials
7.  let manifest = VersionManifest { ..., is_analyzed: false, e2k_generated: !no_e2k }
    manifest.write_to(&version_dir/manifest.json)?
8.  git_add(repo_dir, &[version_dir/model.e2k, version_dir/manifest.json])?
        // model.e2k may not exist if --no-e2k — add only what exists
9.  let hash = git_commit(repo_dir, req.message, req.author, req.email)?
10. manifest.git_commit_hash = Some(hash.clone())      // backfill the hash
    manifest.write_to(&version_dir/manifest.json)?     // rewrite with hash
11. complete_snapshot(guard)?                          // deletes .partial — success
```

**If any step from 5–10 returns `Err`:** The `PartialGuard` fires on `guard` drop,
deleting `vN/`. The working file is never touched. The caller (ext-api) sees the
error and surfaces it.

**`--no-e2k` behavior:** Step 6 is skipped entirely. No sidecar call. Manifest is
written with `{ e2kGenerated: false, materialsExtracted: false }`. `ext diff`
against this version returns:

```
⚠ No E2K generated for v3.
  Re-commit without --no-e2k to enable diff.
```

---

## 4. Branch Layer (`ext-core/src/branch/`)

### 4.1 Branch metadata

Branch metadata is stored in `.etabs-ext/<branch>/.branch.json`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchMeta {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_from: Option<String>,  // "main/v3" or None for main
    pub description: Option<String>,
}
```

`main` has a `.branch.json` created during `ext init` with `created_from: None`.

### 4.2 `branch/mod.rs`

```rust
pub fn create(
    name: &str,
    from_version_dir: &Path,    // already-resolved source vN/model.edb parent
    from_ref: &str,             // human-readable "main/v3" for metadata
    ext_dir: &Path,             // .etabs-ext/
) -> Result<BranchMeta>
    // 1. Validate name (no slashes, no spaces, not "main" if exists)
    // 2. Check branch does not already exist
    // 3. check_disk_space(source_edb, ext_dir/<n>/)
    // 4. Create ext_dir/<n>/working/
    // 5. atomic_copy(from_version_dir/model.edb → ext_dir/<n>/working/model.edb)
    // 6. Write .branch.json
    // Returns BranchMeta

pub fn list(ext_dir: &Path) -> Result<Vec<BranchInfo>>
    // Reads all <n>/.branch.json files
    // Returns sorted by created_at

pub fn delete(name: &str, ext_dir: &Path, force: bool) -> Result<()>
    // Refuse if name == "main" (ever)
    // Refuse if name == current_branch (read state.json)
    // If working file MODIFIED and !force: return Err with --force suggestion
    // Delete ext_dir/<n>/ recursively
    // Note: git branch delete is handled at ext-api level (after ext-core succeeds)

pub fn exists(name: &str, ext_dir: &Path) -> bool

pub fn branch_dir(name: &str, ext_dir: &Path) -> PathBuf
```

**`BranchInfo`** (for `ext branch` list output):

```rust
pub struct BranchInfo {
    pub name: String,
    pub version_count: u32,
    pub latest_version: Option<String>,
    pub created_from: Option<String>,
    pub is_active: bool,           // name == state.current_branch
}
```

---

## 5. Switch + Checkout (`ext-core/src/version/`)

These live in `version/mod.rs` since they operate on the working file within a branch context.

### 5.1 `SwitchResult` — structured return for all callers

```rust
pub struct SwitchResult {
    pub branch: String,
    pub arrival_status: WorkingFileStatus,
    pub departure_warning: Option<String>,   // set when leaving MODIFIED/ANALYZED/LOCKED
    pub arrival_warning: Option<String>,     // set when arriving at MODIFIED/MISSING/ORPHANED
}
```

The CLI prints both warnings. The Tauri app surfaces them as toasts. The agent
includes them in its response text. No terminal I/O inside `ext_api::switch()`.

### 5.2 `CheckoutConflictResolution` — enum parameter for MODIFIED prompt

```rust
pub enum CheckoutConflictResolution {
    CommitFirst { message: String },
    Stash,
    Discard,
    Cancel,
}

pub struct CheckoutOptions {
    pub conflict_resolution: Option<CheckoutConflictResolution>,
    // None = detect only, do not execute — returns Err(CheckoutConflict) for
    // the caller to re-call with the user's chosen resolution
}

// Returned when conflict_resolution is None and working file is MODIFIED
pub struct CheckoutConflict {
    pub current_version: String,         // "v3"
    pub current_status: WorkingFileStatus,
    pub target_version: String,          // "v1"
    pub stash_exists: bool,              // caller can warn if stash would overwrite
}
```

**Two-phase flow for CLI:**

```rust
// First call — detect conflict
match ext_api::checkout(&ctx, "v1", CheckoutOptions { conflict_resolution: None }).await {
    Err(e) if e.is::<CheckoutConflict>() => {
        let conflict = e.downcast::<CheckoutConflict>()?;
        let resolution = prompt_user(&conflict)?;  // [c/s/d/x] in terminal
        // Second call — execute with chosen resolution
        ext_api::checkout(&ctx, "v1", CheckoutOptions {
            conflict_resolution: Some(resolution),
        }).await?;
    }
    other => other?,
}
```

Tauri and agent pass the resolution in one call — no two-phase needed for them.

**`--force` flag maps to `CheckoutConflictResolution::Discard`** — CLI passes it
directly in the first call, bypassing the terminal prompt.

### 5.3 Cross-branch checkout

`ext checkout main/v1` (while on `steel-columns`):

1. Apply `ext_api::switch("main")` — if hard-blocked (ETABS open): abort entire checkout.
2. Apply single-branch `ext_api::checkout("v1")` on the now-active `main`.

The switch step uses the same `SwitchResult` path. Its departure warning is
surfaced before the checkout executes.

---

## 6. Stash (`ext-core/src/stash/`)

### 6.1 One slot per branch

```rust
pub fn save(
    branch: &str,
    working_file: &Path,
    ext_dir: &Path,
    description: Option<&str>,
    state: &mut StateFile,
) -> Result<()>
    // 1. ETABS running check (caller's responsibility — already checked in ext-api)
    // 2. Stash already exists? → return Err(StashExists { branch, description, stashed_at })
    //    (caller prompts [o]verwrite / [x]cancel)
    // 3. Create ext_dir/stash/ if not exists
    // 4. check_disk_space(working_file, ext_dir/stash/)
    // 5. atomic_copy(working_file → ext_dir/stash/<branch>.edb)
    // 6. state.stashes.insert(branch, StashEntry { based_on, stashed_at, description })
    //    (caller saves state)

pub fn pop(
    branch: &str,
    working_file: &Path,
    ext_dir: &Path,
    state: &mut StateFile,
) -> Result<()>
    // 1. Stash exists? → return Err(NoStash { branch }) if not
    // 2. atomic_copy(ext_dir/stash/<branch>.edb → working_file)
    // 3. state.stashes.remove(branch)
    //    state.working_file.based_on_version = stash entry's based_on
    //    state.working_file.status = Modified
    //    (caller saves state)

pub fn drop(branch: &str, ext_dir: &Path, state: &mut StateFile) -> Result<()>
    // Deletes stash file + removes from state.stashes (caller saves state)

pub fn list(state: &StateFile) -> Vec<StashListEntry>
```

**`StashExists` error carries enough info for the prompt:**

```rust
pub struct StashExists {
    pub branch: String,
    pub description: Option<String>,
    pub stashed_at: DateTime<Utc>,
}
```

---

## 7. Guard Layer (`ext-api` — centralized)

### 7.1 `check_state_guard` — permission matrix in one place

All `ext-api` functions call this at entry. It encodes `workflow.md §15` exactly.

```rust
// ext-api/src/guards.rs  (new file)

pub enum GuardOutcome {
    Allow,
    Warn(String),            // proceed, but surface this message
    Block(String),           // return Err with this message
}

pub fn check_state_guard(command: Command, status: &WorkingFileStatus) -> GuardOutcome {
    use Command::*;
    use WorkingFileStatus::*;
    use GuardOutcome::*;

    match (command, status) {
        // Commit
        (Commit, OpenClean | OpenModified) =>
            Block("✗ Close ETABS before committing\n  Run: ext etabs close".into()),
        (Commit, Orphaned) =>
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into()),
        (Commit, Missing) =>
            Block("✗ Working file missing\n  Run: ext checkout vN".into()),
        (Commit, Analyzed | Locked) =>
            Warn("⚠ Working file has analysis results. \
                  Consider: ext commit --analyze to capture them.".into()),
        (Commit, _) => Allow,

        // Switch
        (Switch, OpenClean | OpenModified) =>
            Block("✗ Close ETABS before switching branches\n  Run: ext etabs close".into()),
        (Switch, Orphaned) =>
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into()),
        (Switch, Modified | Analyzed | Locked) =>
            Warn("⚠ Leaving branch with uncommitted changes".into()),
        (Switch, Missing) =>
            Warn("⚠ Working file is missing on this branch".into()),
        (Switch, _) => Allow,

        // Checkout
        (Checkout, OpenClean | OpenModified) =>
            Block("✗ Close ETABS before checking out\n  Run: ext etabs close".into()),
        (Checkout, Analyzed | Locked) =>
            Block("✗ Close ETABS and commit analysis results first\n  \
                   Run: ext commit --analyze".into()),
        (Checkout, Orphaned) =>
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into()),
        (Checkout, _) => Allow,
        // Note: MODIFIED is handled via CheckoutConflictResolution, not blocked here

        // Stash save
        (StashSave, Untracked | Clean | Analyzed | Locked | Missing) =>
            Block("✗ Nothing to stash (working file is not modified)".into()),
        (StashSave, OpenClean | OpenModified) =>
            Block("✗ Close ETABS before stashing\n  Run: ext etabs close".into()),
        (StashSave, _) => Allow,

        // Analyze (operates on snapshot only — never on working file)
        (Analyze, OpenClean | OpenModified) =>
            Block("✗ Close ETABS before running analysis\n  Run: ext etabs close".into()),
        (Analyze, _) => Allow,
        // Note: MISSING state is Allow — analyze vN does not touch working file

        // EtabsOpen
        (EtabsOpen, OpenClean | OpenModified) =>
            Block("✗ ETABS is already running\n  Run: ext etabs close".into()),
        (EtabsOpen, Missing) =>
            Block("✗ Working file missing\n  Run: ext checkout vN".into()),
        (EtabsOpen, Orphaned) =>
            Block("✗ ETABS crashed previously\n  Run: ext etabs recover".into()),
        (EtabsOpen, _) => Allow,

        // Always allowed (status, log, show, diff, push, pull, report)
        (Status | Log | Show | Diff | Push | Report | ConfigGet | ConfigList, _) => Allow,
    }
}
```

This is testable independent of any command. Every state × command combination
in the permission matrix has a corresponding match arm.

### 7.2 Usage pattern in every `ext-api` function

```rust
pub async fn commit_version(ctx: &AppContext, message: &str, opts: CommitOptions)
    -> Result<CommitResult>
{
    let state = ctx.load_state()?;
    let status = resolve_working_file_status(&state, &ctx.project_root);

    match check_state_guard(Command::Commit, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(msg) => {
            // Surface warning — attach to result so caller can display it
            // CommitResult carries an Option<String> warning field
        }
        GuardOutcome::Allow => {}
    }

    // ... proceed with commit
}
```

---

## 8. `ext-api` Functions

### 8.1 Return types — named structs on every function

No function returns `()`. Every result carries enough information to render a
useful message in all three output modes (human, shell, JSON) and for the agent
to include in its tool result.

```rust
// ext-api/src/commit.rs
pub struct CommitResult {
    pub version_id: String,
    pub branch: String,
    pub git_hash: String,
    pub message: String,
    pub e2k_generated: bool,
    pub e2k_size_bytes: Option<u64>,
    pub materials_extracted: bool,
    pub analyzed: bool,
    pub elapsed_ms: u64,
    pub warning: Option<String>,    // from GuardOutcome::Warn
}

// ext-api/src/branch.rs
pub struct CreateBranchResult {
    pub name: String,
    pub created_from: String,
    pub working_model_path: PathBuf,
}

pub struct ListBranchesResult {
    pub branches: Vec<BranchInfo>,
    pub current_branch: String,
}

// ext-api/src/switch.rs
pub struct SwitchResult {
    pub branch: String,
    pub arrival_status: WorkingFileStatus,
    pub departure_warning: Option<String>,
    pub arrival_warning: Option<String>,
}

// ext-api/src/checkout.rs
pub struct CheckoutResult {
    pub version_id: String,
    pub branch: String,
    pub working_model_path: PathBuf,
}

// ext-api/src/stash.rs
pub struct StashSaveResult {
    pub branch: String,
    pub based_on: Option<String>,
    pub stash_path: PathBuf,
}

pub struct StashPopResult {
    pub branch: String,
    pub restored_based_on: Option<String>,
}

pub struct StashListResult {
    pub stashes: Vec<StashListEntry>,
}
```

### 8.2 Full function signatures

```rust
// ext-api/src/commit.rs
pub async fn commit_version(
    ctx: &AppContext,
    message: &str,
    opts: CommitOptions,      // { no_e2k: bool }
) -> Result<CommitResult>

// ext-api/src/log.rs
pub async fn list_versions(
    ctx: &AppContext,
    branch: Option<&str>,     // None = current branch
    include_internal: bool,
) -> Result<ListVersionsResult>

pub async fn show_version(
    ctx: &AppContext,
    version_ref: &str,        // "v3" or "main/v3"
) -> Result<VersionDetail>

// ext-api/src/branch.rs
pub async fn create_branch(
    ctx: &AppContext,
    name: &str,
    from_ref: Option<&str>,   // None = latest committed of current branch
) -> Result<CreateBranchResult>

pub async fn list_branches(ctx: &AppContext) -> Result<ListBranchesResult>

pub async fn delete_branch(
    ctx: &AppContext,
    name: &str,
    force: bool,
) -> Result<DeleteBranchResult>

// ext-api/src/switch.rs
pub async fn switch_branch(
    ctx: &AppContext,
    name: &str,
) -> Result<SwitchResult>

pub async fn switch_and_create(
    ctx: &AppContext,
    name: &str,
    from_ref: Option<&str>,
) -> Result<SwitchResult>

// ext-api/src/checkout.rs
pub async fn checkout_version(
    ctx: &AppContext,
    version_ref: &str,
    opts: CheckoutOptions,
) -> Result<CheckoutResult>

// ext-api/src/stash.rs
pub async fn stash_save(
    ctx: &AppContext,
    description: Option<&str>,
    overwrite: bool,
) -> Result<StashSaveResult>

pub async fn stash_pop(ctx: &AppContext) -> Result<StashPopResult>
pub async fn stash_drop(ctx: &AppContext, force: bool) -> Result<StashDropResult>
pub async fn stash_list(ctx: &AppContext) -> Result<StashListResult>

// ext-api/src/diff.rs
pub async fn diff_versions(
    ctx: &AppContext,
    from_ref: &str,
    to_ref: &str,
) -> Result<DiffResult>

pub struct DiffResult {
    pub from_ref: String,
    pub to_ref: String,
    pub diff_text: String,
    pub no_e2k_warning: Option<String>,
}
```

---

## 9. CLI Wiring (`ext` binary)

### 9.1 Commands to add this week

```
ext commit "message" [--analyze] [--no-e2k]
ext log [--branch <n>] [--all] [--json]
ext show <ref> [--json]
ext branch [name] [--from <ref>] [-d <n>] [--force] [--json]
ext switch <branch>
ext switch -c <branch> [--from <ref>]
ext checkout <version> [--force]
ext stash [--message <text>]
ext stash list [--json]
ext stash pop
ext stash drop [--force]
ext diff <from> <to>
```

### 9.2 Handler pattern — under 20 lines each

```rust
// crates/ext/src/commands/commit.rs
pub async fn execute(ctx: &AppContext, args: &CommitArgs, out: &mut OutputChannel) -> Result<()> {
    let result = ext_api::commit_version(
        ctx,
        &args.message,
        CommitOptions { no_e2k: args.no_e2k },
    ).await?;

    if let Some(ref warn) = result.warning {
        out.warn(warn);
    }

    out.human(|o| {
        writeln!(o, "✓ Version {} saved", result.version_id)?;
        writeln!(o, "  Branch: {}  |  {}", result.branch, result.git_hash)?;
        if result.e2k_generated {
            writeln!(o, "  E2K: {} KB", result.e2k_size_bytes.unwrap_or(0) / 1024)?;
        }
        Ok(())
    })?;
    out.shell(|o| writeln!(o, "{}", result.version_id))?;
    out.json(|o| o.write_value(&result))?;
    Ok(())
}
```

### 9.3 `ext checkout` — two-phase prompt in CLI

```rust
pub async fn execute(ctx: &AppContext, args: &CheckoutArgs, out: &mut OutputChannel) -> Result<()> {
    let opts = if args.force {
        CheckoutOptions {
            conflict_resolution: Some(CheckoutConflictResolution::Discard),
            force: true,
        }
    } else {
        CheckoutOptions { conflict_resolution: None, force: false }
    };

    let result = match ext_api::checkout_version(ctx, &args.version, opts).await {
        Ok(r) => r,
        Err(e) if let Some(conflict) = e.downcast_ref::<CheckoutConflict>() => {
            let resolution = tui::prompt_checkout_conflict(conflict)?;
            if matches!(resolution, CheckoutConflictResolution::Cancel) {
                return Ok(());
            }
            ext_api::checkout_version(ctx, &args.version, CheckoutOptions {
                conflict_resolution: Some(resolution),
                force: false,
            }).await?
        }
        Err(e) => return Err(e),
    };

    out.human(|o| writeln!(o, "✓ Checked out {}/{}", result.branch, result.version_id))?;
    out.shell(|o| writeln!(o, "{}", result.version_id))?;
    out.json(|o| o.write_value(&result))?;
    Ok(())
}
```

---

## 10. State Transitions This Week

| Operation | Before | After |
|---|---|---|
| `commit_version` | UNTRACKED / CLEAN / MODIFIED | CLEAN (`basedOnVersion=vN`, `lastKnownMtime=now`) |
| `switch_branch` | any | `currentBranch` updated; working file state re-resolved |
| `switch_and_create` | any | new branch created, `currentBranch` = new branch |
| `checkout_version` (discard) | MODIFIED | CLEAN (`basedOnVersion=vN`, `lastKnownMtime=now`) |
| `checkout_version` (commit first) | MODIFIED → commit → CLEAN | CLEAN on target vN |
| `checkout_version` (stash) | MODIFIED → stash → file unchanged | CLEAN on target vN; stash entry in state |
| `stash_save` | MODIFIED | MODIFIED (file unchanged); stash entry added to state |
| `stash_pop` | CLEAN | MODIFIED (`basedOnVersion=stash.basedOn`); stash entry removed |
| `stash_drop` | any | stash entry removed; working file unchanged |

Every `ext-api` function that modifies working file state calls
`ctx.load_state()` at the start and `ctx.save_state()` at the end.
State is never cached — always fresh from disk.

---

## 11. `ext-db` state.json schema change

Add `stashes` map and bump `schema_version` from 1 → 2.

```json
{
  "schemaVersion": 2,
  "workingFile": { "..." : "..." },
  "stashes": {
    "main": {
      "basedOn": "v3",
      "stashedAt": "2026-03-28T10:00:00Z",
      "description": "WIP: trying larger columns"
    }
  },
  "updatedAt": "2026-03-28T10:00:00Z"
}
```

Migration in `StateFile::load()` — backfill `stashes` as empty `HashMap` when loading a v1 file:

```rust
if state.schema_version == 1 {
    state.stashes = HashMap::new();
    state.schema_version = STATE_SCHEMA_VERSION; // 2
}
```

---

## 12. Tests Required

### 12.1 Unit tests (in `ext-core`, isolated with `tempfile::TempDir`)

**`vcs/read.rs`**
- `next_version_id` returns `"v1"` on empty repo
- `next_version_id` returns `"v4"` after 3 user commits + 1 internal commit
- `list_commits` filters `"ext:"` prefix when `include_internal=false`
- `list_commits` includes all commits when `include_internal=true`

**`version/snapshot.rs`**
- `begin_snapshot` creates `.partial` file
- `complete_snapshot` removes `.partial` file
- `PartialGuard` rollback deletes directory on drop (simulate error path)
- `cleanup_partial_snapshots` finds and deletes partial folders
- `check_disk_space` returns `Err` when space insufficient

**`guards.rs`** (new)
- `GuardOutcome::Block` for every blocked cell in permission matrix (15+ tests)
- `GuardOutcome::Allow` for every allowed cell
- `GuardOutcome::Warn` for every warn cell

**`branch/mod.rs`**
- `create` refuses name with slash
- `create` refuses name `"main"` when `main` exists
- `delete` refuses `"main"`
- `delete` refuses active branch

**`stash/mod.rs`**
- `save` returns `StashExists` when stash already present
- `pop` restores working file and updates state
- `drop` removes stash entry from state

### 12.2 Integration tests (in `ext-api`)

Full cycle (sidecar skipped via `--no-e2k`):

```rust
#[tokio::test]
async fn test_full_vcs_cycle() {
    // init → commit v1 → create branch → switch → commit v1 →
    // checkout → stash → stash pop
    // Assert: state.json correct at each step, files in correct paths
}
```

Permission matrix (one test per blocked cell):

```rust
#[tokio::test]
async fn test_commit_blocked_in_open_clean() {
    let ctx = AppContext::for_test_with_state(WorkingFileStatus::OpenClean, ...);
    let err = ext_api::commit_version(&ctx, "msg", Default::default()).await.unwrap_err();
    assert!(err.to_string().contains("Close ETABS"));
}
```

### 12.3 CLI snapshot tests

For every new command: happy-path human output, `--json` (all fields present), `--shell` (one value per line), key error cases.

---

## 13. Acceptance Criteria

- [ ] All unit tests pass: `cargo test -p ext-core`
- [ ] All integration tests pass: `cargo test -p ext-api`
- [ ] All CLI snapshot tests pass: `cargo test -p ext`
- [ ] `cargo clippy --all-targets` — zero warnings
- [ ] `cargo fmt --check --all` passes
- [ ] Full cycle smoke test: `init → commit --no-e2k → branch → switch → commit --no-e2k → checkout → stash → stash pop → log → show → diff`
- [ ] Every blocked cell in `workflow.md §15` has a test
- [ ] No `unwrap()` or `expect()` in any command path
- [ ] `--json` output is stable (snapshot-tested)
- [ ] `ext log` never shows `"ext:"` prefixed commits
- [ ] Partial `vN/` folders cleaned up on next startup
- [ ] `state.json` schema version = 2 and migration from v1 works

---

## 14. What Comes Next — Week 5–6 Preview

Deferred manual validation for the real sidecar + ETABS flow is captured in
`2026-03-29-week3-4-vcs-visual-test-spec.md`.

Week 3–4 delivers the full VCS cycle without live ETABS interaction. Week 5–6
wires the remaining four states:

- `OPEN_CLEAN`, `OPEN_MODIFIED`, `ANALYZED`, `LOCKED`, `ORPHANED` — all exercised
  with a live ETABS instance (Week 3–4 only covers `UNTRACKED`, `CLEAN`, `MODIFIED`, `MISSING`)
- `ext etabs open/close/status/validate/unlock/recover` commands fully wired
- All state guards connected to sidecar `get-status` for real PID checks
- `ext diff` producing real E2K diffs (requires sidecar-generated `.e2k` files from actual commits)
- Full permission matrix smoke-tested against a live ETABS process

After Week 5–6, the state machine is validated under all 9 states and the
Week 7–8 analysis pipeline can start on solid ground.
