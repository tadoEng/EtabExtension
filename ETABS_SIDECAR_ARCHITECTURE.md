# ETABS Sidecar Integration Architecture

## Overview

The ETABS integration uses a **two-component architecture**:
1. **Rust CLI + API Layer** (`crates/ext` and `crates/ext-api`) — VCS and state management
2. **C# Sidecar (`EtabExtension.CLI`)** — Direct ETABS COM interop and process control

The sidecar is a separate executable (`etab-cli.exe`) that acts as the IPC bridge to ETABS.

---

## 1. Command Implementation: `etabs open`

### Rust CLI Entry Point
**File:** [crates/ext/src/commands/etabs_open.rs](crates/ext/src/commands/etabs_open.rs#L1)

```rust
pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: EtabsOpenArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = etabs_open(&ctx, args.version.as_deref()).await?;
    // Returns: { pid, opened_file, is_snapshot, warning? }
}
```

### Core Implementation
**File:** [crates/ext-api/src/etabs.rs](crates/ext-api/src/etabs.rs#L248)

```rust
pub async fn etabs_open(ctx: &AppContext, version_ref: Option<&str>) -> Result<EtabsOpenResult>
```

**High-level flow:**

1. **Load state** — get the current working file path and status
2. **Fast status check** — via [resolve_working_file_status()](#etabs-running-status-tracking):
   - Missing files, Open/Closed states are detected from:
     - `state.json` (stored etabs_pid and last_known_mtime)
     - File existence and mtime
     - Is process alive check (via sysinfo)
3. **Guard check** — blocks if ETABS already running or file missing
4. **Sidecar LOCKED detection** — when CLEAN status alone isn't definitive
5. **Call sidecar** — `sidecar.open_model(file_path, false, true)`
   - `false` = don't save
   - `true` = open in new instance (always visible)
6. **Confirm PID** — verify ETABS actually started
7. **Update state.json**:
   ```rust
   wf.etabs_pid = Some(confirmed_pid);
   wf.last_known_mtime = mtime(&target_file);
   wf.status = WorkingFileStatus::OpenClean;
   wf.status_changed_at = Utc::now();
   ```

---

## 2. Sidecar Communication Layer

### Rust Sidecar Client
**File:** [crates/ext-core/src/sidecar/client.rs](crates/ext-core/src/sidecar/client.rs#L1)

The `SidecarClient` spawns `etab-cli.exe` as a **synchronous subprocess**:

```rust
pub async fn run<T>(&self, args: &[&str]) -> ExtResult<SidecarResponse<T>>
```

**Key behavior:**
- Spawns the sidecar executable with args: `["open-model", "--file", "<path>", "--new-instance"]`
- Captures stdout (single JSON response)
- Streams stderr live to terminal (progress messages: ℹ ✓ ✗ ⚠)
- **Waits for completion**: `child.wait().await` — blocks until sidecar exits
- **Timeouts**: 120 seconds for `open-model` (configurable per command)

Once the sidecar returns JSON, the Rust process continues and ETABS is left running.

### Sidecar Command Interface
**File:** [crates/ext-core/src/sidecar/commands.rs](crates/ext-core/src/sidecar/commands.rs#L1)

**Mode A (attach) — commands:**
- `get_status` — query running ETABS instance
- `open_model` — open file in ETABS
- `close_model` — close ETABS
- `unlock_model` — unlock a file

**Mode B (hidden) — commands:**
- `generate_e2k` — headless model generation
- `run_analysis` — headless analysis
- `extract_materials`, `extract_results` — headless extraction

---

## 3. C# Sidecar Implementation

### OpenModelCommand
**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelCommand.cs`

Accepts flags:
- `--file <path>` — the .edb file to open
- `--save` — save current file before switching (Mode A only)
- `--no-save` — discard without prompting
- `--new-instance` — launch a new visible ETABS instance

### OpenModelService
**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelService.cs`

**Mode B (new instance) — spawn and forget:**

```csharp
private static async Task<Result<OpenModelData>> OpenInNewInstanceAsync(string filePath)
{
    // Spawn new ETABS with startApplication: true (visible)
    app = ETABSWrapper.CreateNew(startApplication: true);
    
    // Open the file
    int openRet = app.Model.Files.OpenFile(filePath);
    
    // Get the PID
    var pid = await WaitForPidAsync(newestFirst: true);
    
    // CRITICAL: No ApplicationExit() call!
    // User controls the visible ETABS window — we just return the PID
    // app?.Dispose();  // Release COM proxy only
}
```

**Mode A (existing instance):**

```csharp
private static async Task<Result<OpenModelData>> OpenInRunningInstanceAsync(
    string filePath, bool save)
{
    // Connect to running ETABS
    app = ETABSWrapper.Connect();
    
    // Open file
    int openRet = app.Model.Files.OpenFile(filePath);
    
    // Release COM proxy but ETABS keeps running
    // app?.Dispose();  // Release COM only — ETABS keeps running
}
```

**Key insight:** Both modes release the COM proxy but do NOT call `ApplicationExit()`. ETABS continues running on its own.

---

## 4. Process Lifetime Management

### Spawn Model: Spawn-and-Forget

```
User runs: ext etabs open
  ↓
Rust calls: sidecar.open_model("path/to/model.edb", false, true)
  ↓
Sidecar spawns ETABS via COM: ETABSWrapper.CreateNew(startApplication: true)
  ↓
Sidecar gets PID: Process.GetProcessesByName("ETABS").FirstOrDefault()?.ProcessId
  ↓
Sidecar exits, returns JSON: { "pid": 12345, "filePath": "...", ... }
  ↓
Rust stores PID in state.json and returns to user
  ↓
ETABS continues running independently
  ↓
User closes ETABS manually (or via ext etabs close)
```

### Process Monitoring (Not Active Monitoring)

The extension **does NOT actively monitor** the ETABS process. Instead:

1. **PID stored in state.json** — when running `ext` commands
2. **Alive check via sysinfo** — when needed (status, guards)
3. **Sidecar queries** — `get-status` checks via COM if ETABS is still running

So the process model is: **fire-and-forget, check-on-demand**.

---

## 5. "ETABS Running" Status Tracking

### State.json Structure
**File:** [crates/ext-db/src/state.rs](crates/ext-db/src/state.rs) (in ext-db crate)

```rust
pub struct WorkingFileInfo {
    pub etabs_pid: Option<u32>,
    pub last_known_mtime: Option<DateTime<Utc>>,
    pub status: WorkingFileStatus,
    pub status_changed_at: DateTime<Utc>,
    // ...
}
```

### WorkingFileStatus Enum
**File:** [crates/ext-core/src/state.rs](crates/ext-core/src/state.rs#L1)

```rust
pub enum WorkingFileStatus {
    Missing,           // File doesn't exist
    OpenClean,         // ETABS running, no changes (pid_alive && !is_modified)
    OpenModified,      // ETABS running, changes made (pid_alive && is_modified)
    Orphaned,          // pid stored but process dead
    Clean,             // Closed, no changes from version
    Modified,          // Closed, changes from version
    Analyzed,          // Closed, was analyzed
    Locked,            // Closed, model is locked
    Untracked,         // No version history
}
```

### State Resolution Logic
**File:** [crates/ext-api/src/status.rs](crates/ext-api/src/status.rs#L1)

The resolver takes these inputs:
```rust
pub struct ResolveInput {
    pub file_exists: bool,
    pub etabs_pid: Option<u32>,
    pub pid_alive: bool,           // ← Key field
    pub based_on_version: Option<String>,
    pub last_known_mtime: Option<DateTime<Utc>>,
    pub current_mtime: Option<DateTime<Utc>>,
}
```

And applies precedence:
```rust
if !input.file_exists {
    return WorkingFileStatus::Missing;
}

if input.etabs_pid.is_some() {
    if input.pid_alive {
        if is_modified(input.last_known_mtime, input.current_mtime) {
            return WorkingFileStatus::OpenModified;
        }
        return WorkingFileStatus::OpenClean;
    }
    return WorkingFileStatus::Orphaned;  // ← Crash detected!
}
// ... other statuses for closed file
```

### PID Alive Check
**File:** [crates/ext-api/src/status.rs](crates/ext-api/src/status.rs#L70)

```rust
fn is_pid_alive(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
    system.process(sysinfo::Pid::from_u32(pid)).is_some()
}
```

Uses **sysinfo crate** to poll Windows process table.

### When Status is Checked

1. **Fast path** (resolve_working_file_status):
   - On every command that needs state
   - Uses stored `etabs_pid` + `is_pid_alive()` check
   - Compares file mtime with `last_known_mtime`
   - **NO sidecar call**

2. **Full path** (resolve_with_sidecar):
   - Called by `etabs_open` only (before guard check)
   - Calls `sidecar.get_status()` to detect LOCKED/ANALYZED
   - COM connection required

3. **Live sidecar** (etabs_status command):
   - Dedicated command to query current state
   - Always calls sidecar for live data
   - Returns: is_running, pid, open_file_path, etabs_version, is_locked, is_analyzed

---

## 6. Expected Flow: Visible ETABS Open

### Scenario: `ext etabs open` (visible new instance)

```
$ ext etabs open
✓ ETABS opened (PID: 18924)
  File: model.edb
{"pid": 18924, "openedFile": "D:\\..\\model.edb", "isSnapshot": false}
```

**What happens:**

1. CLI loads state.json
2. Checks fast status (no ETABS running yet)
3. Calls sidecar with: `open-model --file D:\\..\\model.edb --new-instance`
4. Sidecar:
   - Creates `ETABSWrapper.CreateNew(startApplication: true)` → ETABS window appears
   - Calls `OpenFile()` → model loads in ETABS
   - Gets PID: 18924
   - Writes JSON to stdout: `{ "success": true, "data": { "pid": 18924, ... } }`
   - **Exits** (sidecar process termination)
5. Rust confirms PID by querying sidecar again (or trusts the returned value)
6. Updates state.json:
   - `etabs_pid = 18924`
   - `status = OpenClean`
   - `last_known_mtime = current_file_mtime`
7. Returns control to user
8. **ETABS remains running independently** — user can work, save, etc.
9. Next `ext` command:
   - Checks `is_pid_alive(18924)` → true
   - Status becomes `OpenClean` (still running, no changes)
   - Or `OpenModified` if file mtime changed

---

## 7. Expected Flow: Close ETABS

### Scenario: `ext etabs close`

**File:** [crates/ext-api/src/etabs.rs](crates/ext-api/src/etabs.rs#L330)

```rust
pub async fn etabs_close(ctx: &AppContext, mode: CloseMode) -> Result<EtabsCloseResult>
```

1. Load state, get stored PID
2. Call sidecar: `close-model --save` or `close-model --no-save`
3. Sidecar calls COM: `app.Model.Files.SaveFile()` (if --save)
4. Sidecar calls COM: `app.ApplicationExit(true)` → closes ETABS window
5. Sidecar exits with JSON: `{ "was_saved": true, "closed_file_path": "..." }`
6. Rust updates state.json:
   - `etabs_pid = None` (clear the PID)
   - `status = Clean` or `Modified` (depending on arrival mtime)
   - Record whether it was analyzed/locked for later
7. Returns success to user

---

## 8. Crash Recovery: ORPHANED State

### Detection

When next `ext` command runs:
1. State.json has `etabs_pid = 12345`
2. Call `is_pid_alive(12345)` → returns false
3. Status resolves to `WorkingFileStatus::Orphaned`

### What Happens

**File:** [crates/ext-api/src/guards.rs](crates/ext-api/src/guards.rs#L136)

Guards block certain operations:
```rust
(EtabsOpen, Orphaned) => {
    Block("✗ ETABS crashed previously\n  Run: ext etabs recover".into())
}
```

### Recovery Flow

**File:** [crates/ext-api/src/etabs.rs](crates/ext-api/src/etabs.rs#L480)

Two-phase dialog:
1. User runs: `ext etabs recover`
2. Prompt: "Keep unsaved changes or restore from version?"
3. User chooses → Phase 2 executes recovery
4. Clear PID or restore file from version

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│ ext CLI                                                          │
│ (etabs_open command)                                            │
│                                                                 │
│  1. Load state.json                                             │
│  2. resolve_working_file_status() {                             │
│       check is_pid_alive() from sysinfo                         │
│     }                                                           │
│  3. Guard check (block if already running)                      │
│  4. Spawn sidecar: etab-cli.exe                                 │
│     └─ args: ["open-model", "--file", "...", "--new-instance"] │
└─────────────────────────────┬──────────────────────────────────┘
                              │
                    Request/Response
                   (stdout/stderr pipes)
                              │
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ C# Sidecar (etab-cli.exe)                                        │
│ (OpenModelService)                                              │
│                                                                 │
│  1. ETABSWrapper.CreateNew(startApplication: true)              │
│     └─ ETABS.exe spawned by Windows via COM                     │
│  2. app.Model.Files.OpenFile(filePath)                          │
│  3. Get PID from Process.GetProcessesByName("ETABS")            │
│  4. Write JSON to stdout and exit                               │
│     └─ app?.Dispose() // Release COM proxy only                 │
└─────────────────────────────┬──────────────────────────────────┘
                              │
                  JSON Response
                              │
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ ext CLI (continued)                                              │
│                                                                 │
│  5. Confirm PID from sidecar response                           │
│  6. Update state.json: { etabs_pid, status: OpenClean }        │
│  7. Return to user with PID                                    │
│                                                                 │
│  >>> CONTROL RELEASED — ETABS RUNNING INDEPENDENTLY <<<         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ↓
                    ┌─────────────────┐
                    │  ETABS.exe      │
                    │ (running on its │
                    │  own, user can  │
                    │  edit, save)    │
                    └─────────────────┘
```

---

## Key Architectural Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Process Model** | Spawn-and-forget | User controls ETABS lifetime; doesn't block Rust |
| **Status Tracking** | PID + mtime + process polling | Cheap, reliable, doesn't require active monitoring |
| **PID Alive Check** | sysinfo crate polling | Fast, cross-platform, no COM needed |
| **Sidecar Lifetime** | Synchronous (waits for response) | Simple request/response semantics |
| **ETABS Lifetime** | Independent after open | User can work at their own pace |
| **Crash Detection** | ORPHANED state (lazy detection) | No active watchdog; detected on next command |

---

## Files Summary

| File | Purpose |
|------|---------|
| [crates/ext/src/commands/etabs_open.rs](crates/ext/src/commands/etabs_open.rs) | CLI command entry point |
| [crates/ext-api/src/etabs.rs](crates/ext-api/src/etabs.rs) | Core workflow: open, close, status, unlock, recover |
| [crates/ext-core/src/sidecar/client.rs](crates/ext-core/src/sidecar/client.rs) | Spawn and manage sidecar subprocess |
| [crates/ext-core/src/sidecar/commands.rs](crates/ext-core/src/sidecar/commands.rs) | SidecarClient method signatures for all 8 commands |
| [crates/ext-api/src/status.rs](crates/ext-api/src/status.rs) | State resolution + PID alive check |
| [crates/ext-core/src/state.rs](crates/ext-core/src/state.rs) | WorkingFileStatus enum + resolution logic |
| `EtabExtension.CLI/src/Features/OpenModel/` | C# sidecar: Mode A (attach) vs Mode B (spawn) |
