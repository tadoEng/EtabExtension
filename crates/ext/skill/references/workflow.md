# ETABS Extension — Developer Workflow Map

Complete state-by-state guide to expected system behavior.
Use this as the implementation contract for `ext-api` and `ext-core`.

---

## Working File States

```
UNTRACKED     No version committed yet (fresh init)
CLEAN         working/model.edb matches basedOnVersion (mtime unchanged)
MODIFIED      working/model.edb differs from basedOnVersion (mtime changed)
OPEN_CLEAN    ETABS has file open, no Ctrl+S since open
OPEN_MODIFIED ETABS has file open, user saved changes
ANALYZED      ETABS closed, analysis results embedded in working file
LOCKED        ETABS model lock set post-analysis (must unlock before editing)
MISSING       working/model.edb does not exist on disk
ORPHANED      ETABS PID in state.json but process is gone (crash/kill)
```

---

## State Detection on Every Command

Run this before every command that needs state. Runs in milliseconds — no hashing, no COM calls.

```
1. Read state.json
2. Does working/model.edb exist?
     NO  → MISSING (overrides all else)
3. Is ETABS PID in state.json still alive? (OS process check)
     YES → OPEN_CLEAN or OPEN_MODIFIED (compare mtime to distinguish)
     NO, but PID was set → ORPHANED
4. Is basedOnVersion set?
     NO → UNTRACKED
5. Compare mtime:
     stat(working/model.edb).mtime > state.json lastKnownMtime → MODIFIED
     else → CLEAN
6. Analysis lock: checked on-demand only (ext etabs status / ext etabs open)
```

---

## 1. `ext init`

**Precondition:** No `.etabs-ext/` present.

**Sequence:**
1. Validate `--edb` path exists and is `.edb`
2. Check if `--path` or `--edb` is inside a OneDrive-synced folder:
    - Detect OneDrive paths: ancestors containing `OneDrive`, `OneDrive - `, `SharePoint`
    - If detected: warn + prompt `[c] Continue anyway / [x] Cancel`
3. Create `.etabs-ext/` and `main/working/`
4. Atomic copy: `--edb` → `main/working/model.edb`
5. `git init`, `git config core.autocrlf false`, set user.name/email
6. Write `config.toml` (from bundled template via `include_str!()`)
7. Write `config.local.toml` (author, email, reportsDir from flags or interactive prompt)
8. Write `state.json` `{ currentBranch: "main", workingFile: { status: "UNTRACKED" } }`
9. Write `.gitignore` (from bundled template)
10. `git add config.toml .gitignore && git commit "ext: init project"` (internal)

**Postcondition:** State = `UNTRACKED`. No versions yet.

**Errors:**
- `--edb` not found or not `.edb` → error
- `.etabs-ext/` already exists → `✗ Project already initialized`
- Insufficient disk space → `✗ Insufficient disk space (need Xmb, have Ymb)`

---

## 2. `ext commit "message"`

**Precondition:** State is `UNTRACKED`, `MODIFIED`, or `CLEAN`. ETABS NOT running.

**Sequence:**
1. Check state — if `OPEN_*`, `ORPHANED`, `MISSING`: hard stop with message
2. If state is `ANALYZED` or `LOCKED` without `--analyze`: warn and suggest `--analyze`
3. Sidecar `get-status` → confirm ETABS not running
4. Determine `vN` = latest version + 1 (scan branch dir)
5. Create `vN/` directory
6. Atomic copy: `working/model.edb` → `vN/model.edb.tmp` → rename
7. Sidecar `save-snapshot --file vN/model.edb --output-dir vN/`
    - stderr progress forwarded to terminal
    - exports `vN/model.e2k`, extracts `vN/materials/takeoff.parquet`
8. Write `vN/manifest.json` `{ id, branch, message, author, timestamp, parent, isAnalyzed: false }`
9. `git add vN/model.e2k vN/manifest.json`
10. `git commit -m "<message>"`
11. Update `manifest.json.gitCommitHash`
12. Update `state.json` `{ basedOnVersion: vN, status: CLEAN, lastKnownMtime: now }`

**With `--analyze`** (continues after step 12):

13. Sidecar `open-model --file vN/model.edb --hidden`
14. Sidecar `run-analysis --file vN/model.edb` (blocks)
15. Sidecar `extract-results --file vN/model.edb --output-dir vN/results/`
16. Sidecar `close-model`
17. Write `vN/summary.json`
18. Update `vN/manifest.json` `{ isAnalyzed: true }`
19. `git add vN/summary.json vN/manifest.json`
20. `git commit -m "ext: analysis results vN"` (internal — filtered from `ext log`)

**Postcondition:** State = `CLEAN`, `basedOnVersion = vN`. Working file untouched throughout steps 13–20.

**Errors:**
- ETABS running → `✗ Close ETABS before committing`
- ORPHANED → `✗ Run: ext etabs recover`
- Sidecar not found → `✗ etab-cli.exe not found. Run: ext config set etabs.sidecarPath <path>`
- Analysis fails → `✗ Analysis failed: <message>` (commit without results is preserved)

---

## 3. `ext switch <branch>`

**Precondition:** ETABS NOT running.

**Sequence:**
1. ETABS running? → hard stop: `✗ Close ETABS before switching branches`
2. If current branch state is `MODIFIED` or `UNTRACKED`: warn (do NOT block)
   ```
   ⚠ Leaving <branch> with uncommitted changes since <version>
     Changes preserved in <branch>/working/model.edb
   ```
3. Update `state.json` `{ currentBranch: <target> }`
4. Resolve target working file state (mtime check on target branch's working file)
5. Report:
    - `CLEAN` → `✓ Switched to: <branch>` (silent)
    - `MODIFIED` → warn
    - `UNTRACKED` → warn: no commits yet
    - `MISSING` → warn: `Run: ext checkout vN`
    - `ORPHANED` → warn: `Run: ext etabs recover`

**Postcondition:** `currentBranch` = target. Working files unchanged.

**Errors:**
- Branch does not exist → `✗ Branch not found: <n>`
- ETABS running → hard stop

---

## 4. `ext switch -c <branch> [--from <ref>]`

**Sequence:**
1. Resolve `--from` ref (default: latest committed version of current branch)
2. Disk space check: source `.edb` size + 10%
3. Create `<branch>/working/`
4. Atomic copy with progress bar: `source/vN/model.edb` → `<branch>/working/model.edb`
5. Write branch metadata: `{ name, createdFrom, createdAt }`
6. Apply `ext switch <branch>` sequence (section 3)

**Postcondition:** New branch exists and is active. Working file = copy of source. State = `CLEAN` with `basedOnVersion: null` (no commits yet on this branch).

---

## 5. `ext checkout <version>`

**Precondition:** ETABS NOT running.

### 5a. Single-branch (`ext checkout v2`)

**Sequence:**
1. ETABS running? → hard stop
2. Resolve `v2` → `<currentBranch>/v2/model.edb`
3. Does `v2/model.edb` exist?
    - NO → `✗ Snapshot missing. Available: v1, v3, v4`
4. If working file `MODIFIED` or `UNTRACKED` → prompt:
   ```
   [c] Commit current changes first, then checkout
   [s] Stash current changes (restore later: ext stash pop)
   [d] Discard changes and checkout v2
   [x] Cancel
   ```
    - `[c]`: run full commit flow (prompt for message), then proceed
    - `[s]`: run stash save, then proceed
    - `[d]`: proceed directly
    - `[x]`: exit
5. Atomic copy: `v2/model.edb` → `working/model.edb.tmp` → rename
6. Update `state.json` `{ basedOnVersion: v2, status: CLEAN, lastKnownMtime: now }`

### 5b. Cross-branch (`ext checkout main/v2` while on `steel-columns`)

1. Apply `ext switch main` (section 3) — if blocked by ETABS: stop entirely
2. Apply single-branch `ext checkout v2` (section 5a)

**Postcondition:** Working file = exact copy of `vN/model.edb`. State = `CLEAN`.

---

## 6. `ext stash`

**Precondition:** State is `MODIFIED` or `UNTRACKED`. ETABS NOT running.

**Sequence:**
1. ETABS running? → hard stop
2. Stash already exists for this branch?
   ```
   ⚠ Stash exists: "WIP: <desc>" (3d ago)
     [o] Overwrite  [x] Cancel
   ```
3. Create `stash/` if not exists
4. Atomic copy: `working/model.edb` → `stash/<branch>.edb`
5. Write `stash/<branch>-meta.json` `{ basedOn, stashedAt, description }`
6. Update `state.json` stash entry

**Postcondition:** Stash saved. Working file unchanged.

---

## 7. `ext stash pop`

**Precondition:** Stash exists for `currentBranch`. ETABS NOT running.

**Sequence:**
1. ETABS running? → hard stop
2. Working file `MODIFIED`? → prompt overwrite
3. Atomic copy: `stash/<branch>.edb` → `working/model.edb`
4. Restore `state.json` `{ basedOnVersion: stash.basedOn, status: MODIFIED }`
5. Delete stash files, remove stash entry from `state.json`

**Postcondition:** Working file = stashed file. Stash cleared. State = `MODIFIED`.

---

## 8. `ext analyze <version>`

**Precondition:** `vN/model.edb` exists. ETABS NOT running.

**Sequence:**
1. ETABS running? → hard stop
2. Verify `vN/model.edb` exists
3. If already analyzed → prompt: `Re-run? [y/n]`
4. Sidecar: `open-model --file vN/model.edb --hidden`
5. Sidecar: `run-analysis` (blocks)
6. Sidecar: `extract-results → vN/results/*.parquet`
7. Sidecar: `close-model`
8. Write `vN/summary.json`, update `vN/manifest.json { isAnalyzed: true }`
9. `git commit "ext: analysis results vN"` (internal)

**Postcondition:** `vN/results/` populated. Working file and current branch unchanged.

---

## 9. `ext etabs open [version]`

**Precondition:** ETABS NOT running.

**Sequence:**
1. ETABS running? → `✗ ETABS is already running (PID: <n>). Close it first.`
2. Resolve target file:
    - No argument → `<currentBranch>/working/model.edb`
    - Version argument → `<branch>/vN/model.edb` + warn: read-only recommended
3. Sidecar: `open-model --file <path>` (visible window)
4. Update `state.json` `{ etabs: { pid, openFile }, workingFile.status: OPEN_CLEAN }`

**Postcondition:** ETABS visible. State = `OPEN_CLEAN`.

---

## 10. `ext etabs unlock`

**Precondition:** ETABS running. State = `LOCKED`.

**Sequence:**
1. ETABS not running? → error
2. Sidecar: `unlock-model --file <working-file>`
    - Calls `SapModel.SetModelIsLocked(false)`
3. Update `state.json` `{ workingFile.status: OPEN_CLEAN }`

**Note:** Extracted Parquet files in `vN/results/` are unaffected. Only the lock inside the `.edb` is cleared.

---

## 11. `ext etabs recover`

**Precondition:** State = `ORPHANED`.

**Sequence:**
1. Verify ETABS PID is dead (OS check)
2. Check working file mtime vs `lastKnownMtime`
3. Present:
   ```
   ⚠ ETABS closed unexpectedly (PID: <n>)
     File modified: Yes / No  (<n> minutes before crash)
     [k] Keep changes  (mark MODIFIED)
     [r] Restore from <version>  (discard changes)
   ```
4. `[k]`: `state.json { etabs: null, status: MODIFIED }`
5. `[r]`: atomic copy `vN/model.edb → working/model.edb`, `state.json { etabs: null, status: CLEAN }`

---

## 12. `ext push`

**Precondition:** `paths.oneDriveDir` set in `config.local.toml`.

**Sequence:**
1. Resolve `oneDriveDir` → error if not set: `Run: ext config set paths.oneDriveDir <path>`
2. Read `OneDrive/project.json` (if exists)
3. Conflict check: for each local `vN`, does remote have same ID with different `gitCommitHash`?
   ```
   ✗ Conflict: main/v4 already exists on OneDrive
     Remote pushed by: Jane Smith (2h ago)
     Remote message: "Updated shear walls"
     Your message:   "Increased column sizes"
     [r] Rename yours to v5 and push  [v] View diff  [x] Cancel
   ```
    - `[r]`: rename local `v4` folder to `v5`, update `manifest.json id`, re-commit manifest
4. `git bundle create OneDrive/git-bundle --all`
5. For each version not in remote (compare `project.json`):
    - Atomic copy with progress: `vN/model.edb → OneDrive/edb/<branch>-vN.edb`
6. If `--include-working`: copy `working/model.edb → OneDrive/edb/<branch>-working.edb`
7. Write/update `OneDrive/project.json`

**Postcondition:** OneDrive has full git history + all `.edb` snapshots.

---

## 13. `ext pull`

**Precondition:** `paths.oneDriveDir` set. Remote `project.json` exists.

**Sequence:**
1. Read `OneDrive/project.json`
2. Find versions in remote but not local
3. `git fetch OneDrive/git-bundle` → restores text files for new versions
4. For each new version: copy `OneDrive/edb/<branch>-vN.edb → vN/model.edb`
5. Report pulled versions

**Note:** `config.local.toml` is never overwritten by pull — it is machine-specific.

---

## 14. `ext clone <onedrive-path> --to <local-path>`

**Precondition:** Remote `project.json` exists. Local path does not have `.etabs-ext/`.

**Sequence:**
1. Read `OneDrive/project.json`
2. Create local `.etabs-ext/` structure
3. `git clone --local OneDrive/git-bundle .etabs-ext/` → restores all text files including `config.toml`
4. Copy all `.edb` files from `OneDrive/edb/`
5. Interactive prompts for machine-specific settings
6. Write `config.local.toml`
7. Set working file to latest version of main
8. Write `state.json { status: CLEAN, basedOn: latest }`

---

## 15. Command Permission Matrix

| Command | UNTRACKED | CLEAN | MODIFIED | OPEN_CLEAN | OPEN_MOD | ANALYZED | LOCKED | MISSING | ORPHANED |
|---|---|---|---|---|---|---|---|---|---|
| `ext status` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `ext log/show/diff` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `ext commit` | ✓ | ✓* | ✓ | ✗ | ✗ | warn | warn | ✗ | ✗ |
| `ext commit --analyze` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| `ext analyze vN` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ |
| `ext switch` | ✓ | ✓ | warn | ✗ | ✗ | ✗ | ✗ | warn | ✗ |
| `ext checkout` | ✓ | ✓ | prompt | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `ext stash` | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `ext stash pop` | ✗ | ✓ | prompt | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| `ext etabs open` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ |
| `ext etabs close` | ✗ | ✗ | ✗ | ✓ | prompt | ✓ | ✓ | ✗ | ✗ |
| `ext etabs unlock` | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ |
| `ext etabs recover` | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| `ext push` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `ext pull` | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ |
| `ext report` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Legend: `✓` allowed, `✗` blocked with error, `warn` allowed with warning, `prompt` prompts user, `*` allowed but no diff (no e2k change if model unchanged)

---

## 16. Internal vs User-Visible Git Commits

**User commits** (shown in `ext log`):
```
git commit -m "Updated column sections"
```

**Internal commits** (hidden from `ext log`):
```
git commit -m "ext: init project"
git commit -m "ext: analysis results v3"
```

`ext log` filters commits where `message.starts_with("ext:")`. Full audit trail preserved in git.

---

## 17. Config Resolution

```rust
// config.local.toml has highest priority for all keys
// config.toml provides shared project defaults
// ext hardcoded defaults are final fallback

pub fn author(&self) -> &str {
    self.local.git.author.as_deref()
        .or(self.shared.git.git.as_deref())
        .unwrap_or("Unknown")
}

pub fn reports_dir(&self) -> Option<&Path> {
    // local always wins — each machine has its own OneDrive path
    self.local.paths.reports_dir.as_deref()
}

pub fn onedrive_dir(&self) -> Option<&Path> {
    self.local.paths.onedrive_dir.as_deref()
}

pub fn sidecar_path(&self) -> &str {
    self.local.etabs.sidecar_path.as_deref()
        .or(self.shared.etabs.sidecar_path.as_deref())
        .unwrap_or("etab-cli.exe")
}
```

`config.local.toml` is never pushed to OneDrive and never overwritten by `ext pull` or `ext clone` (clone creates it fresh via wizard).

---

## 18. OneDrive Path Detection

```rust
fn is_onedrive_path(path: &Path) -> bool {
    let markers = ["OneDrive", "OneDrive - ", "SharePoint"];
    path.ancestors().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| markers.iter().any(|m| n.starts_with(m)))
            .unwrap_or(false)
    })
}
```

Called in `ext init` for both `--path` and `--edb`. If detected: warn + prompt. Record acknowledgment in `config.local.toml { onedrive.acknowledgedSync: true }` to suppress future warnings.

`ext status` checks on every run and shows a persistent warning until `acknowledgedSync = true`.

---

## 19. Atomic File Operations

All `.edb` copies use write-to-temp-then-rename to prevent partial writes:

```rust
fn atomic_copy(src: &Path, dst: &Path) -> Result<()> {
    let tmp = dst.with_extension("edb.tmp");
    fs::copy(src, &tmp)?;
    fs::rename(&tmp, dst)?;   // atomic on same filesystem
    Ok(())
}
```

On startup, clean up any stray `.edb.tmp` files left by interrupted operations.

---

## 20. Error Message Standards

```
✗ <what failed>
  <why it failed>
  Run: <command to fix it>

⚠ <what the user should know>
  <consequence if ignored>
  Run: <command to address it>

✓ <what was accomplished>
→ Next: <suggested next command>   (only on first-time flows)
```

Never use generic messages like "Something went wrong." Always tell the user exactly what failed and exactly what to run.