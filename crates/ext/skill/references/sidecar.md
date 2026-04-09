# ETABS Extension — Sidecar CLI Development Guide

> Current note: the sections below contain older design exploration. The
> active Rust-side contract has moved on in a few important ways. Use the
> summary in this note as the source of truth unless you are intentionally
> reviewing historical ideas.

## Current Contract Summary

- Rust does not use a composite `save-snapshot` sidecar command today.
- `ext commit` currently orchestrates `generate-e2k` and `extract-materials`
  directly.
- `ext analyze vN` currently orchestrates `run-analysis` and
  `extract-results` directly.
- `ext etabs open` now requests `open-model --new-instance` and requires a
  confirmed PID before Rust writes OPEN state.
- `open-model` payload shape is:

```json
{
  "filePath": "C:\\path\\model.edb",
  "previousFilePath": null,
  "pid": 12345,
  "openedInNewInstance": true
}
```

- `generate-e2k` now uses `outputFile` as the canonical field name. Rust still
  accepts legacy `outputPath` for backward compatibility.
- `extract-materials` now uses `outputFile`, `tableKey`, `rowCount`,
  `discardedRowCount`, `units`, and `extractionTimeMs`. Rust must not assume
  `materials/takeoff.parquet`; it stages every generated parquet file under
  `materials/`.
- `extract-results` request defaults now cover the full current `ext-calc`
  parquet input set:
  - `storyDefinitions`
  - `pierSectionProperties`
  - `baseReactions`
  - `storyForces`
  - `jointDrifts`
  - `pierForces`
  - `modalParticipatingMassRatios`
  - `groupAssignments`
  - `materialPropertiesConcreteData`
  - `materialListByStory`
- `ext analyze` still means ETABS analysis plus parquet extraction only. It
  does not run calc, render, or report.

## Read This File Carefully

Anything below that still mentions `save-snapshot`, `takeoff.parquet`, or the
older 7-table result set is historical and should not be implemented as-is.

Architecture, connection strategy, command contracts, and implementation
patterns for `etab-cli.exe` — the C# .NET 10 sidecar that owns all ETABS
COM interaction.

---

## Purpose and Boundaries

The sidecar is the **only component in the entire system that talks to
ETABS**. No Rust code, no Tauri, no agent ever calls ETABS COM directly.

```
Rust (ext-core / ext-api)
    │
    └── spawns ──► etab-cli.exe [stdin=nothing, stdout=JSON, stderr=progress]
                       │
                       └── COM ──► ETABS.exe (user's running instance, Mode A)
                                   OR
                                   ETABS.exe (hidden instance, Mode B)
```

The sidecar is a **single-shot process**: one command, one job, one exit.
It is never a daemon. It is never kept alive between Rust calls. Rust spawns
it, reads stdout JSON, reads stderr for live progress, waits for exit.

---

## IPC Contract (never changes)

```
stdin:   nothing — sidecar never reads stdin
stdout:  ONE JSON object, written once, at the very end — the Result<T>
stderr:  human-readable progress lines, written freely during execution
         Rust forwards these live to the terminal for the user to see
exit:    0 = success (Result.Success == true)
         1 = failure (Result.Success == false, Result.Error set)
```

**stdout rule:** Nothing is written to stdout until the operation completes.
The JSON is written exactly once, then the process exits. Rust reads full
stdout after process exit.

**stderr rule:** Write progress freely — the engineer sees these in real time:
```
✓ Connected to ETABS 22.0.0
ℹ Opening model.edb...
ℹ Running analysis... (this may take several minutes)
✓ Analysis complete (2m 14s)
ℹ Extracting results...
✓ Extracted 7 result tables
```

**`Program.cs` enforces stdout discipline globally:**
```csharp
// ALL Console.WriteLine → stderr (progress visible to user)
Console.SetOut(new StreamWriter(Console.OpenStandardError()) { AutoFlush = true });
// Only explicit stdout writes carry the JSON result
```

---

## ETABS Connection Strategy

### License Reality

ETABS licenses are expensive. Users may have only one seat. The sidecar
**never starts a visible ETABS instance on its own** — doing so would
confuse the user or burn a license slot unexpectedly.

The intended workflow is:

```
1. User manually opens ETABS (their license, their responsibility)
2. ext etabs open  → sidecar attaches to that running instance and calls
                     OpenFile() to load the .edb into ETABS
3. User works in ETABS
4. ext etabs close → sidecar attaches, saves if needed, "closes" the model
5. ext commit      → sidecar starts a SEPARATE HIDDEN instance to do
                     snapshot work (E2K export, analysis, extraction),
                     then exits that hidden instance cleanly —
                     never touching the user's main ETABS instance
```

### Two Connection Modes

#### Mode A — Attach to User's Running ETABS

Used by: `get-status`, `open-model`, `close-model`, `unlock-model`, `validate`.

The raw ETABS COM API for attaching to a running instance:
```vb
' VB equivalent
Dim myHelper As cHelper = New Helper
EtabsObject = myHelper.GetObject("CSI.ETABS.API.ETABSObject")
SapModel = EtabsObject.SapModel
```

In C# via EtabSharp:
```csharp
var app = ETABSWrapper.Connect(); // wraps GetObject("CSI.ETABS.API.ETABSObject")
```

After the command finishes, **release COM refs without exiting ETABS**:
```csharp
// ✅ Release COM — ETABS stays running for the user
ComCleanup.Release(sapModel, etabsObject);
// ETABS.exe continues running — user is unaffected
```

**Why releasing COM refs is enough:** `ComCleanup.Release()` drops the
.NET-side RCW (Runtime Callable Wrapper) proxy objects. ETABS is a native
Win32 app — the currently open file lives entirely in the ETABS process.
Releasing the COM proxy just disconnects the control channel. ETABS keeps
running, the file stays loaded, the user sees nothing change. There is no
"keep-alive" problem.

**Never call `ApplicationExit()` on a Mode A connection.** That exits the
user's ETABS, losing their unsaved work.

#### Mode B — Hidden Instance for Background Work

Used by: `save-snapshot`, `run-analysis` (standalone), `generate-e2k`
(when no visible ETABS is running).

The raw ETABS COM API for starting a new instance:
```vb
' VB equivalent — from official ETABS API example
Dim myHelper As cHelper = New Helper
EtabsObject = myHelper.CreateObjectProgID("CSI.ETABS.API.ETABSObject")
ret = EtabsObject.ApplicationStart()
ret = SapObject.Hide()   ' hide immediately — user never sees it
SapModel = EtabsObject.SapModel
```

In C# via EtabSharp (after you add `Hide()`):
```csharp
var etabsObject = ETABSWrapper.CreateNew(startApplication: true);
etabsObject.Hide(); // you add Hide() to EtabSharp — see EtabSharp section
var sapModel = etabsObject.SapModel;
```

This starts a **second, separate** ETABS process running hidden. When
work is done, exit it cleanly:
```csharp
// ✅ Exit the hidden instance WE started — safe, no user impact
try { etabsObject.ApplicationExit(false); } catch { } // false = don't save
ComCleanup.Release(sapModel, etabsObject);
```

**Why a separate process and not the user's instance?**
Running analysis on the user's open file would:
- Lock them out of ETABS for 2–5 minutes
- Potentially corrupt their in-progress work
- Switch their open file to a snapshot they don't own

The snapshot `.edb` lives in `vN/` — it belongs to ext, not the user.

### Connection Decision Tree

```
Command received
│
├── User-facing commands
│   (get-status, open-model, close-model, unlock-model, validate)
│       → Mode A: attach to user's running ETABS
│       → Error if ETABS not running (user must start it first)
│       → Release COM on exit, NEVER ApplicationExit
│
└── Everything else
    (save-snapshot, run-analysis, generate-e2k, extract-results,
     extract-materials)
        → Mode B: always start a new hidden instance
        → Even if user's ETABS is running: Mode B is a SEPARATE process
        → ApplicationExit(false) + Release COM on exit, always in finally
```

**Why `generate-e2k` and `extract-*` are Mode B, not dual-capable:**

The tempting optimization is "if ETABS is running and the right file is
open, attach to it — saves the 15–30s startup." It breaks down because:

- The user may have unsaved changes. Switching their open file to the
  snapshot path would trigger a "Save changes?" dialog that blocks the
  COM call indefinitely — your background command hangs waiting for user
  input on a dialog they don't know is there.
- Even if the right file is open, running `ExportFile()` on the user's
  active session has side effects (changes the "modified" flag, can
  interfere with undo history).
- Mode B startup overhead is a one-time fixed cost per command. For
  interactive use it feels slow; for automated `ext commit` pipelines
  it's irrelevant. Design for the common case.

If startup time becomes a real pain point, the right fix is to make
`save-snapshot` even more composite (it already combines E2K + materials
+ results in one hidden session), not to add fragile Mode A fallback to
  individual commands.

### The `close-model` Problem

**There is no confirmed ETABS COM API for closing the currently open file
while keeping ETABS open.** The raw API does not expose a `File.Close()`
that leaves ETABS running with an empty state.

**Workaround — open a blank model:**
```csharp
// Opening a new blank effectively "closes" the current file
// ETABS window remains open, now showing an empty model
int ret = sapModel.File.NewBlank();
```

**Risk:** `NewBlank()` may show a "Save changes?" dialog on a modified model.
Mitigate by checking modification state first:
```csharp
// Guard before NewBlank to avoid surprise Save dialog
bool isModified = false;
sapModel.GetModelIsModified(ref isModified);

if (isModified && !saveFirst)
{
    // Force-clear modification flag to suppress the Save dialog
    // Then open blank
    sapModel.SetModelIsModified(false);
}
int ret = sapModel.File.NewBlank();
```

Verify this behavior in integration tests on both saved and unsaved models.

### COM Cleanup Helper

Add `Shared/Infrastructure/Etabs/ComCleanup.cs`:

```csharp
using System.Runtime.InteropServices;

namespace EtabExtension.CLI.Shared.Infrastructure.Etabs;

internal static class ComCleanup
{
    /// <summary>
    /// Release COM references safely. Always call in finally blocks.
    /// Does NOT call ApplicationExit — caller is responsible for that.
    /// </summary>
    internal static void Release(params object?[] comObjects)
    {
        foreach (var obj in comObjects)
        {
            if (obj is null) continue;
            try { Marshal.ReleaseComObject(obj); }
            catch { /* ignore — may already be released */ }
        }
        GC.Collect();
        GC.WaitForPendingFinalizers();
    }
}
```

**Usage pattern — Mode A (attach, do NOT exit):**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;
try
{
    etabsObject = helper.GetObject("CSI.ETABS.API.ETABSObject");
    sapModel = etabsObject.SapModel;
    // ... do work ...
}
finally
{
    ComCleanup.Release(sapModel, etabsObject); // ETABS keeps running
}
```

**Usage pattern — Mode B (hidden, DO exit):**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;
try
{
    etabsObject = helper.CreateObjectProgID("CSI.ETABS.API.ETABSObject");
    etabsObject.ApplicationStart();
    etabsObject.Hide();
    sapModel = etabsObject.SapModel;
    // ... do work ...
}
finally
{
    try { etabsObject?.ApplicationExit(false); } catch { } // exit hidden instance
    ComCleanup.Release(sapModel, etabsObject);
}
```

---

## Result Pattern (Rust-Inspired, C# Idiomatic)

All service methods return `Result<T>`. Command handlers call
`ExitWithResult()` which writes JSON to stdout and returns the exit code.

```csharp
// Every service method signature
Task<Result<TData>> DoSomethingAsync(...);

// Every command handler — the only place Environment.Exit is called
var result = await service.DoSomethingAsync(...);
Environment.Exit(result.ExitWithResult());
```

Never throw across the service boundary. Catch at service level:
```csharp
try
{
    // ... ETABS operation ...
    return Result.Ok(data);
}
catch (COMException ex)
{
    return Result.Fail<TData>($"ETABS COM error: {ex.Message}")
        with { Data = partialData };
}
catch (Exception ex)
{
    return Result.Fail<TData>($"Unexpected error: {ex.Message}")
        with { Data = partialData };
}
finally
{
    ComCleanup.Release(sapModel, etabsObject); // always runs
}
```

### JSON Output Shape

Success:
```json
{
  "success": true,
  "error": null,
  "timestamp": "2024-02-05T14:30:00Z",
  "data": { ... }
}
```

Failure (with partial data for Rust diagnostics):
```json
{
  "success": false,
  "error": "ETABS is not running",
  "timestamp": "2024-02-05T14:30:00Z",
  "data": {
    "isRunning": false,
    "messages": [
      "✗ ETABS is not running",
      "Start ETABS manually before running this command"
    ]
  }
}
```

Rust reads `success` first. On `false`: reads `error` for the message,
`data.messages` for user-facing lines, any typed fields for state info.

---

## EtabSharp — What Exists and What You Need to Add

### Already available (inferred from existing codebase)

```csharp
ETABSWrapper.IsRunning()                               // check ETABS process
ETABSWrapper.Connect()                                 // Mode A: attach
ETABSWrapper.CreateNew(startApplication: true)         // Mode B: start new

// On ETABSApplication / cOAPI:
etabsObject.ApplicationStart()                         // start ETABS
etabsObject.ApplicationExit(bool saveModel)            // exit ETABS
etabsObject.SapModel                                   // → cSapModel
etabsObject.FullVersion                                // version string

// On cSapModel / app.Model:
sapModel.File.OpenFile(path)                           // open .edb
sapModel.File.Save(path)                               // save in place
sapModel.File.NewBlank()                               // new blank model
sapModel.File.NewSteelDeck(...)                        // template (not used)
sapModel.File.ExportFile(path, eFileTypeIO.TextFile)   // export .e2k
sapModel.Files.GetFilePath()                           // path of open file
sapModel.Analyze.RunAnalysis()                         // run analysis
sapModel.Analyze.RunCompleteAnalysis()                 // run all cases
sapModel.Analyze.SetRunCaseFlag(name, run, all)        // configure cases
sapModel.Analyze.GetCaseStatus()                       // case results
sapModel.ModelIsLocked                                 // bool property
sapModel.SetModelIsLocked(bool)                        // unlock
sapModel.ModelInfo.GetModelFilepath()                  // open file path
```

### You need to add to EtabSharp

```csharp
// From VB API:  ret = SapObject.Hide()
//               ret = SapObject.Unhide()
// These are on cOAPI (the ETABSObject), not on cSapModel.

// On ETABSApplication wrapper:
int Hide()      // hide the ETABS window immediately after start
int Unhide()    // make a hidden ETABS window visible (ext etabs open uses this)
```

**Where they sit:** In the official VB example, `SapObject` is the `cOAPI`
reference — the same object as `EtabsObject` / your `ETABSApplication`.
`SapModel` is `EtabsObject.SapModel` (one level deeper). `Hide`/`Unhide`
are on the outer object.

**`Unhide()` use case:** When `ext etabs open` is called, you could start
a pre-hidden ETABS and then unhide it to show the user the window. Or you
could attach to an already-running hidden instance and unhide it. This gives
a smoother open experience than always starting a new visible ETABS.

---

## Command Catalogue

### `get-status`

Returns ETABS running state, PID, open file path, lock state, analysis state.
This is called by Rust on every `ext status` — it must be fast and never fail.

```bash
etab-cli get-status
```

**No flags.**

**Connection:** Mode A (attach). If ETABS not running → `Result.Ok` with
`isRunning: false`. Not an error — returning `success: false` here would
break `ext status` in normal use.

**Implementation:**
```csharp
public async Task<Result<GetStatusData>> GetStatusAsync()
{
    await Task.CompletedTask;
    var messages = new List<string>();

    // Check process first — gives PID without needing COM
    var processes = Process.GetProcessesByName("ETABS");
    var isRunning = processes.Length > 0;
    var pid = processes.FirstOrDefault()?.Id;

    if (!isRunning)
    {
        return Result.Ok(new GetStatusData
        {
            IsRunning = false,
            Messages = ["ℹ ETABS is not running"]
        });
    }

    cOAPI? etabsObject = null;
    cSapModel? sapModel = null;

    try
    {
        var helper = new Helper();
        etabsObject = helper.GetObject("CSI.ETABS.API.ETABSObject");
        sapModel = etabsObject.SapModel;

        var openFilePath = sapModel.GetModelFilename(false); // full path
        bool isLocked = false;
        sapModel.GetModelIsLocked(ref isLocked);

        // Check analysis status
        var caseStatuses = sapModel.Analyze.GetCaseStatus();
        var isAnalyzed = caseStatuses.Any(cs => cs.IsFinished);

        messages.Add($"✓ ETABS {etabsObject.FullVersion} is running");
        if (!string.IsNullOrEmpty(openFilePath))
            messages.Add($"✓ Open: {Path.GetFileName(openFilePath)}");

        return Result.Ok(new GetStatusData
        {
            IsRunning = true,
            Pid = pid,
            EtabsVersion = etabsObject.FullVersion,
            OpenFilePath = string.IsNullOrEmpty(openFilePath) ? null : openFilePath,
            IsModelOpen = !string.IsNullOrEmpty(openFilePath),
            IsLocked = isLocked,
            IsAnalyzed = isAnalyzed,
            Messages = messages
        });
    }
    catch (Exception ex)
    {
        // COM failed but ETABS is running — return partial data
        messages.Add($"⚠ Connected to ETABS but could not read model state: {ex.Message}");
        return Result.Ok(new GetStatusData
        {
            IsRunning = true,
            Pid = pid,
            Messages = messages
        });
    }
    finally
    {
        ComCleanup.Release(sapModel, etabsObject); // NOT ApplicationExit
    }
}
```

**Data shape:**
```json
{
  "isRunning": true,
  "pid": 12345,
  "etabsVersion": "22.0.0.1234",
  "openFilePath": "C:\\Projects\\main\\working\\model.edb",
  "isModelOpen": true,
  "isLocked": false,
  "isAnalyzed": true,
  "messages": ["✓ ETABS 22.0.0 is running", "✓ Open: model.edb"]
}
```

---

### `open-model`

Attaches to the user's running ETABS and calls `OpenFile()`.

```bash
etab-cli open-model --file <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to `.edb` file to open |

**Connection:** Mode A only. Hard error if ETABS not running — user must
start ETABS manually first.

**Implementation:**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;

try
{
    var helper = new Helper();
    etabsObject = helper.GetObject("CSI.ETABS.API.ETABSObject");
    sapModel = etabsObject.SapModel;

    Console.Error.WriteLine($"ℹ Opening {Path.GetFileName(filePath)}...");

    int ret = sapModel.File.OpenFile(filePath);
    if (ret != 0)
        return Result.Fail<OpenModelData>($"OpenFile failed (ret={ret})");

    var pid = Process.GetProcessesByName("ETABS").FirstOrDefault()?.Id;

    Console.Error.WriteLine($"✓ Opened {Path.GetFileName(filePath)}");

    return Result.Ok(new OpenModelData
    {
        FilePath = filePath,
        Pid = pid,
        Messages = [$"✓ Opened {Path.GetFileName(filePath)}"]
    });
}
finally
{
    ComCleanup.Release(sapModel, etabsObject); // NOT ApplicationExit
}
```

**Data shape:**
```json
{
  "filePath": "C:\\...\\main\\working\\model.edb",
  "pid": 12345,
  "messages": ["✓ Opened model.edb"]
}
```

---

### `close-model`

Closes the currently open model in the user's ETABS. ETABS itself stays
running. Uses the `NewBlank()` workaround — see the close-model problem
discussion above.

```bash
etab-cli close-model
etab-cli close-model --save
etab-cli close-model --no-save
```

| Flag | Required | Description |
|---|---|---|
| `--save` | no | Save before closing |
| `--no-save` | no | Close without saving |

**Default (no flag):** Do not save.

**Connection:** Mode A. Hard error if ETABS not running.

**Implementation:**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;

try
{
    var helper = new Helper();
    etabsObject = helper.GetObject("CSI.ETABS.API.ETABSObject");
    sapModel = etabsObject.SapModel;

    var currentPath = sapModel.GetModelFilename(false);

    if (save)
    {
        Console.Error.WriteLine("ℹ Saving...");
        int saveRet = sapModel.File.Save(currentPath);
        if (saveRet != 0)
            return Result.Fail<CloseModelData>("Save failed");
        Console.Error.WriteLine("✓ Saved");
    }

    // Suppress Save dialog on modified models when not saving
    if (!save)
    {
        bool isModified = false;
        sapModel.GetModelIsModified(ref isModified);
        if (isModified)
            sapModel.SetModelIsModified(false); // suppress Save dialog
    }

    // Workaround: NewBlank() "closes" the current file
    int ret = sapModel.File.NewBlank();
    if (ret != 0)
        return Result.Fail<CloseModelData>("Could not close model");

    Console.Error.WriteLine("✓ Model closed");

    return Result.Ok(new CloseModelData
    {
        ClosedFilePath = currentPath,
        WasSaved = save,
        Messages = [$"✓ Model closed{(save ? " (saved)" : "")}"]
    });
}
finally
{
    ComCleanup.Release(sapModel, etabsObject); // NOT ApplicationExit
}
```

**Data shape:**
```json
{
  "closedFilePath": "C:\\...\\model.edb",
  "wasSaved": false,
  "messages": ["✓ Model closed"]
}
```

---

### `unlock-model`

Clears the ETABS post-analysis model lock (`SetModelIsLocked(false)`).

```bash
etab-cli unlock-model --file <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to the locked `.edb` file |

**Connection:** Mode A. The file must already be open in ETABS — do not
open it silently, as the user's working context matters.

**Implementation:**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;

try
{
    var helper = new Helper();
    etabsObject = helper.GetObject("CSI.ETABS.API.ETABSObject");
    sapModel = etabsObject.SapModel;

    // Verify the correct file is open
    var currentPath = sapModel.GetModelFilename(false);
    if (!PathsAreEqual(currentPath, filePath))
        return Result.Fail<UnlockData>(
            $"File not open in ETABS. Open it first with: ext etabs open");

    bool wasLocked = false;
    sapModel.GetModelIsLocked(ref wasLocked);

    if (wasLocked)
    {
        int ret = sapModel.SetModelIsLocked(false);
        if (ret != 0)
            return Result.Fail<UnlockData>("Failed to clear model lock");
        Console.Error.WriteLine("✓ Model lock cleared");
    }
    else
    {
        Console.Error.WriteLine("ℹ Model was not locked");
    }

    return Result.Ok(new UnlockData
    {
        FilePath = filePath,
        WasLocked = wasLocked,
        Messages = [wasLocked ? "✓ Lock cleared" : "ℹ Model was not locked"]
    });
}
finally
{
    ComCleanup.Release(sapModel, etabsObject); // NOT ApplicationExit
}
```

**Data shape:**
```json
{
  "filePath": "C:\\...\\model.edb",
  "wasLocked": true,
  "messages": ["✓ Model lock cleared"]
}
```

---

### `validate`

Validates file existence, type, and analysis status.

```bash
etab-cli validate --file <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to `.edb` or `.e2k` file |

**Connection:** Dual-capable. Attaches to running ETABS if available.
If not running, returns `isAnalyzed: null` — unknown without ETABS.

**Already implemented** — add `ComCleanup.Release()` to the `finally` block.
Everything else is correct.

---

### `generate-e2k`

Exports a `.edb` to `.e2k` text format.

```bash
etab-cli generate-e2k --file <path> --output <path>
etab-cli generate-e2k --file <path> --output <path> --overwrite
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to input `.edb` |
| `--output` / `-o` | no | Output `.e2k` path (default: same dir as input) |
| `--overwrite` | no | Overwrite if output exists |

**Connection:** Mode B always. Start hidden ETABS, open the file, export,
`ApplicationExit(false)`.

This avoids the fragile "reuse user's open session" path where a "Save
changes?" dialog on an unsaved model could block the COM call indefinitely.
Mode B is clean, isolated, and predictable.

**Export call (unchanged from existing implementation):**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;

try
{
    var helper = new Helper();
    etabsObject = helper.CreateObjectProgID("CSI.ETABS.API.ETABSObject");
    etabsObject.ApplicationStart();
    etabsObject.Hide();
    sapModel = etabsObject.SapModel;

    int openRet = sapModel.File.OpenFile(inputFilePath);
    if (openRet != 0)
        return Result.Fail<GenerateE2KData>($"OpenFile failed (ret={openRet})");

    int exportRet = sapModel.File.ExportFile(e2kOutputPath, eFileTypeIO.TextFile);
    if (exportRet != 0 || !File.Exists(e2kOutputPath))
        return Result.Fail<GenerateE2KData>("ExportFile failed");

    // ...build and return success data
}
finally
{
    try { etabsObject?.ApplicationExit(false); } catch { }
    ComCleanup.Release(sapModel, etabsObject);
}
```

**Already partially implemented** — the existing `EtabsApiGenerateE2KFile.cs`
has the correct export call. Refactor it to: remove the Mode A attach logic,
add the hidden-instance start, add `ApplicationExit(false)` in `finally`.

---

### `run-analysis`

Runs complete analysis on a model snapshot. Always Mode B (hidden).

```bash
etab-cli run-analysis --file <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to `.edb` snapshot (`vN/model.edb`) |

**Connection:** Mode B **always**. Never attach to the user's running ETABS.
This is background work on a snapshot file.

**Requires `EtabSharp.Hide()` to be added first.**

**Implementation:**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;
var stopwatch = Stopwatch.StartNew();

try
{
    Console.Error.WriteLine("ℹ Starting ETABS (hidden)...");
    var helper = new Helper();
    etabsObject = helper.CreateObjectProgID("CSI.ETABS.API.ETABSObject");
    etabsObject.ApplicationStart();
    etabsObject.Hide(); // EtabSharp — add this
    sapModel = etabsObject.SapModel;
    Console.Error.WriteLine($"✓ ETABS started (hidden)");

    Console.Error.WriteLine($"ℹ Opening {Path.GetFileName(filePath)}...");
    int openRet = sapModel.File.OpenFile(filePath);
    if (openRet != 0)
        return Result.Fail<RunAnalysisData>($"OpenFile failed (ret={openRet})");

    Console.Error.WriteLine("ℹ Running analysis... (this may take several minutes)");

    sapModel.Analyze.SetRunCaseFlag("", true, true); // all cases
    int analysisRet = sapModel.Analyze.RunAnalysis();
    stopwatch.Stop();

    if (analysisRet != 0)
        return Result.Fail<RunAnalysisData>($"Analysis failed (ret={analysisRet})")
            with { Data = new RunAnalysisData { AnalysisTimeMs = stopwatch.ElapsedMilliseconds } };

    Console.Error.WriteLine($"✓ Analysis complete ({FormatDuration(stopwatch.Elapsed)})");

    // Save so analysis results persist in the .edb after hidden ETABS exits
    sapModel.File.Save(filePath);

    var caseStatuses = sapModel.Analyze.GetCaseStatus();
    var finished = caseStatuses.Count(cs => cs.IsFinished);

    return Result.Ok(new RunAnalysisData
    {
        FilePath = filePath,
        AnalysisTimeMs = stopwatch.ElapsedMilliseconds,
        CaseCount = caseStatuses.Length,
        FinishedCaseCount = finished,
        Messages = [
            $"✓ Analysis complete ({FormatDuration(stopwatch.Elapsed)})",
            $"✓ {finished}/{caseStatuses.Length} load cases finished"
        ]
    });
}
finally
{
    try { etabsObject?.ApplicationExit(false); } catch { } // exit hidden instance
    ComCleanup.Release(sapModel, etabsObject);
}
```

**Data shape:**
```json
{
  "filePath": "C:\\...\\vN\\model.edb",
  "analysisTimeMs": 134210,
  "caseCount": 12,
  "finishedCaseCount": 12,
  "messages": ["✓ Analysis complete (2m 14s)", "✓ 12/12 load cases finished"]
}
```

---

### `extract-results`

Extracts all 7 analysis result tables to Parquet files.

```bash
etab-cli extract-results --file <path> --output-dir <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to analyzed `.edb` snapshot |
| `--output-dir` | yes | Directory to write Parquet files |

**Connection:** Mode B always. Same rationale as `generate-e2k` — the
analyzed snapshot belongs to ext, not the user. Start hidden, extract,
`ApplicationExit(false)`.

**Output schemas — 7 Parquet files:**

| File | ETABS API call |
|---|---|
| `modal.parquet` | `sapModel.Results.ModalParticipatingMassRatios(...)` |
| `base_reactions.parquet` | `sapModel.Results.BaseReact(...)` |
| `story_forces.parquet` | `sapModel.Results.StoryForces(...)` |
| `story_drifts.parquet` | `sapModel.Results.StoryDrifts(...)` |
| `joint_displacements.parquet` | `sapModel.Results.JointDispl(...)` |
| `wall_pier_forces.parquet` | `sapModel.Results.PierForce(...)` |
| `shell_stresses.parquet` | `sapModel.Results.AreaStressShell(...)` |

**Parquet writing (add `Parquet.Net` NuGet `5.*`):**
```csharp
using Parquet;
using Parquet.Data;
using Parquet.Schema;

// Example: modal table
var schema = new ParquetSchema(
    new DataField<string>("loadCase"),
    new DataField<int>("modeNumber"),
    new DataField<double>("period"),
    new DataField<double>("ux"), new DataField<double>("uy"), new DataField<double>("uz"),
    new DataField<double>("sumUx"), new DataField<double>("sumUy"), new DataField<double>("sumUz"),
    new DataField<double>("rx"), new DataField<double>("ry"), new DataField<double>("rz")
);

await using var stream = File.Create(outputPath);
await using var writer = await ParquetWriter.CreateAsync(schema, stream);
await using var group = writer.CreateRowGroup();
await group.WriteColumnAsync(new DataColumn(schema.DataFields[0], loadCaseArray));
// ... repeat for each column
```

**Data shape:**
```json
{
  "filePath": "C:\\...\\vN\\model.edb",
  "outputDir": "C:\\...\\vN\\results",
  "tablesExtracted": ["modal", "base_reactions", "story_forces",
                      "story_drifts", "joint_displacements",
                      "wall_pier_forces", "shell_stresses"],
  "rowCounts": {
    "modal": 12,
    "base_reactions": 24,
    "story_forces": 360,
    "story_drifts": 720,
    "joint_displacements": 45600,
    "wall_pier_forces": 180,
    "shell_stresses": 8400
  },
  "extractionTimeMs": 8240,
  "messages": ["✓ Extracted 7 tables", "✓ 55296 total rows"]
}
```

---

### `extract-materials`

Extracts material takeoff to `takeoff.parquet`.

```bash
etab-cli extract-materials --file <path> --output <path>
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Path to `.edb` file |
| `--output` | yes | Output path for `takeoff.parquet` |

**Connection:** Mode B always. Same rationale as above — start hidden, extract,
`ApplicationExit(false)`.
```csharp
int numItems = 0;
string[] storyName = [], matProp = [], matType = [];
double[] dryWeight = [], volume = [];
int ret = sapModel.Results.MaterialTakeoff(
    ref numItems, ref storyName, ref matProp,
    ref matType, ref dryWeight, ref volume);
```

**`takeoff.parquet` schema:**

| Column | Type | Description |
|---|---|---|
| `storyName` | string | Story label |
| `materialName` | string | Material name |
| `materialType` | string | Concrete / Steel / Other |
| `volumeM3` | double | Volume in cubic metres |
| `massKg` | double | Mass in kilograms |

**Data shape:**
```json
{
  "filePath": "C:\\...\\model.edb",
  "outputFile": "C:\\...\\vN\\materials\\takeoff.parquet",
  "rowCount": 147,
  "materials": ["C32/40", "S355"],
  "extractionTimeMs": 1240,
  "messages": ["✓ Material takeoff extracted (147 rows)"]
}
```

---

### `save-snapshot`

**The most important command.** Composite: opens a snapshot `.edb` in a
hidden ETABS instance, exports E2K, extracts materials, optionally runs
analysis and extracts results, then exits the hidden instance.

Called by Rust for `ext commit` (no `--with-results`) and
`ext commit --analyze` (`--with-results`).

```bash
etab-cli save-snapshot --file <path> --output-dir <path>
etab-cli save-snapshot --file <path> --output-dir <path> --with-results
etab-cli save-snapshot --file <path> --output-dir <path> --overwrite
```

| Flag | Required | Description |
|---|---|---|
| `--file` / `-f` | yes | Snapshot `.edb` path (`vN/model.edb`) |
| `--output-dir` | yes | Directory for all outputs |
| `--with-results` | no | Also run analysis + extract all result tables |
| `--overwrite` | no | Overwrite existing outputs |

**Connection:** Mode B **always.** Even if the user has the same file open,
we start a fresh hidden instance for snapshot work. This is non-negotiable:
we must not lock the user's ETABS for 2–5 minutes or switch their open file.

**Full sequence:**
```
1. Validate inputs (no ETABS)
2. Create output directories
3. Start hidden ETABS: CreateObjectProgID → ApplicationStart → Hide
4. OpenFile(--file) in hidden instance
5. ExportFile → output-dir/model.e2k
6. MaterialTakeoff → output-dir/materials/takeoff.parquet
7. [If --with-results]:
     a. SetRunCaseFlag("", true, true)
     b. RunAnalysis() — blocks, log progress to stderr
     c. File.Save(filePath) — persist analysis results in snapshot .edb
     d. Extract 7 result tables → output-dir/results/*.parquet
8. ApplicationExit(false) — always, in finally block
9. ComCleanup.Release
10. Write JSON to stdout → exit
```

**Implementation:**
```csharp
cOAPI? etabsObject = null;
cSapModel? sapModel = null;
var stopwatch = Stopwatch.StartNew();
var messages = new List<string>();

try
{
    // 1. Validate
    if (!File.Exists(filePath))
        return Result.Fail<SaveSnapshotData>("Snapshot file not found");

    // 2. Create directories
    Directory.CreateDirectory(outputDir);
    Directory.CreateDirectory(Path.Combine(outputDir, "materials"));
    if (withResults)
        Directory.CreateDirectory(Path.Combine(outputDir, "results"));

    // 3. Start hidden ETABS
    Console.Error.WriteLine("ℹ Starting ETABS (hidden)...");
    var helper = new Helper();
    etabsObject = helper.CreateObjectProgID("CSI.ETABS.API.ETABSObject");
    int startRet = etabsObject.ApplicationStart();
    if (startRet != 0)
        return Result.Fail<SaveSnapshotData>("Failed to start ETABS");

    etabsObject.Hide(); // requires EtabSharp.Hide()
    sapModel = etabsObject.SapModel;
    Console.Error.WriteLine($"✓ ETABS started (hidden)");

    // 4. Open file
    Console.Error.WriteLine($"ℹ Opening {Path.GetFileName(filePath)}...");
    int openRet = sapModel.File.OpenFile(filePath);
    if (openRet != 0)
        return Result.Fail<SaveSnapshotData>($"OpenFile failed (ret={openRet})");

    // 5. Export E2K
    Console.Error.WriteLine("ℹ Exporting E2K...");
    var e2kPath = Path.Combine(outputDir, "model.e2k");
    var e2kTimer = Stopwatch.StartNew();
    int exportRet = sapModel.File.ExportFile(e2kPath, eFileTypeIO.TextFile);
    e2kTimer.Stop();

    if (exportRet != 0 || !File.Exists(e2kPath))
        return Result.Fail<SaveSnapshotData>("E2K export failed");

    var e2kSize = new FileInfo(e2kPath).Length;
    Console.Error.WriteLine($"✓ E2K exported ({FormatSize(e2kSize)}, {FormatDuration(e2kTimer.Elapsed)})");
    messages.Add($"✓ E2K exported ({FormatSize(e2kSize)})");

    // 6. Extract materials
    Console.Error.WriteLine("ℹ Extracting material takeoff...");
    var materialsPath = Path.Combine(outputDir, "materials", "takeoff.parquet");
    var matRowCount = await ExtractMaterialsAsync(sapModel, materialsPath);
    Console.Error.WriteLine($"✓ Materials extracted ({matRowCount} rows)");
    messages.Add($"✓ Materials extracted");

    // 7. Optional: analysis + results
    bool analysisRun = false;
    long? analysisTimeMs = null;
    Dictionary<string, int>? rowCounts = null;

    if (withResults)
    {
        Console.Error.WriteLine("ℹ Running analysis... (this may take several minutes)");
        var analysisTimer = Stopwatch.StartNew();

        sapModel.Analyze.SetRunCaseFlag("", true, true);
        int analysisRet = sapModel.Analyze.RunAnalysis();
        analysisTimer.Stop();
        analysisTimeMs = analysisTimer.ElapsedMilliseconds;

        if (analysisRet != 0)
            return Result.Fail<SaveSnapshotData>($"Analysis failed (ret={analysisRet})")
                with { Data = BuildPartialData(filePath, outputDir, e2kPath,
                                               e2kSize, materialsPath, messages, stopwatch) };

        Console.Error.WriteLine($"✓ Analysis complete ({FormatDuration(analysisTimer.Elapsed)})");
        messages.Add($"✓ Analysis complete ({FormatDuration(analysisTimer.Elapsed)})");

        // Save so analysis results persist in .edb after hidden ETABS exits
        sapModel.File.Save(filePath);

        Console.Error.WriteLine("ℹ Extracting results (7 tables)...");
        var resultsDir = Path.Combine(outputDir, "results");
        rowCounts = await ExtractAllResultsAsync(sapModel, resultsDir);
        analysisRun = true;

        var totalRows = rowCounts.Values.Sum();
        Console.Error.WriteLine($"✓ Extracted 7 tables ({totalRows} total rows)");
        messages.Add($"✓ Extracted 7 result tables");
    }

    stopwatch.Stop();
    Console.Error.WriteLine($"✓ All done (total: {FormatDuration(stopwatch.Elapsed)})");

    return Result.Ok(new SaveSnapshotData
    {
        FilePath = filePath,
        OutputDir = outputDir,
        E2KFile = e2kPath,
        E2KSizeBytes = e2kSize,
        MaterialsFile = materialsPath,
        AnalysisRun = analysisRun,
        AnalysisTimeMs = analysisTimeMs,
        ResultsDir = analysisRun ? Path.Combine(outputDir, "results") : null,
        RowCounts = rowCounts,
        TotalTimeMs = stopwatch.ElapsedMilliseconds,
        Messages = messages
    });
}
finally
{
    // Always exit the hidden instance and release COM
    try { etabsObject?.ApplicationExit(false); } catch { }
    ComCleanup.Release(sapModel, etabsObject);
}
```

**stderr progress (--with-results):**
```
ℹ Starting ETABS (hidden)...
✓ ETABS 22.0.0 started (hidden)
ℹ Opening vN/model.edb...
ℹ Exporting E2K...
✓ E2K exported (2.3 MB, 18s)
ℹ Extracting material takeoff...
✓ Materials extracted (147 rows)
ℹ Running analysis... (this may take several minutes)
✓ Analysis complete (2m 14s)
ℹ Extracting results (7 tables)...
✓ modal (12 rows)
✓ base_reactions (24 rows)
✓ story_forces (360 rows)
✓ story_drifts (720 rows)
✓ joint_displacements (45600 rows)
✓ wall_pier_forces (180 rows)
✓ shell_stresses (8400 rows)
✓ All done (total: 2m 45s)
```

**Data shape:**
```json
{
  "filePath": "C:\\...\\vN\\model.edb",
  "outputDir": "C:\\...\\vN",
  "e2kFile": "C:\\...\\vN\\model.e2k",
  "e2kSizeBytes": 2415620,
  "materialsFile": "C:\\...\\vN\\materials\\takeoff.parquet",
  "analysisRun": true,
  "analysisTimeMs": 134210,
  "resultsDir": "C:\\...\\vN\\results",
  "rowCounts": {
    "modal": 12,
    "base_reactions": 24,
    "story_forces": 360,
    "story_drifts": 720,
    "jointDisplacements": 45600,
    "wallPierForces": 180,
    "shellStresses": 8400
  },
  "totalTimeMs": 165420,
  "messages": ["✓ E2K exported (2.3 MB)", "✓ Analysis complete (2m 14s)", "✓ Extracted 7 result tables"]
}
```

---

## Project Structure

```
src/EtabExtension.CLI/
  Program.cs
  Features/
    GetStatus/
      GetStatusCommand.cs
      GetStatusService.cs
      IGetStatusService.cs
      GetStatusExtensions.cs
      Models/GetStatusData.cs
    Validate/                          ← exists — add ComCleanup to finally
    OpenModel/
      OpenModelCommand.cs
      OpenModelService.cs
      IOpenModelService.cs
      OpenModelExtensions.cs
      Models/OpenModelData.cs
    CloseModel/
      CloseModelCommand.cs
      CloseModelService.cs
      ICloseModelService.cs
      CloseModelExtensions.cs
      Models/CloseModelData.cs
    UnlockModel/
      UnlockModelCommand.cs
      UnlockModelService.cs
      IUnlockModelService.cs
      UnlockModelExtensions.cs
      Models/UnlockModelData.cs
    GenerateE2K/                       ← exists — refactor connection logic
    RunAnalysis/
      RunAnalysisCommand.cs
      RunAnalysisService.cs
      IRunAnalysisService.cs
      RunAnalysisExtensions.cs
      Models/RunAnalysisData.cs
    ExtractResults/
      ExtractResultsCommand.cs
      ExtractResultsService.cs
      IExtractResultsService.cs
      ExtractResultsExtensions.cs
      Models/ExtractResultsData.cs
    ExtractMaterials/
      ExtractMaterialsCommand.cs
      ExtractMaterialsService.cs
      IExtractMaterialsService.cs
      ExtractMaterialsExtensions.cs
      Models/ExtractMaterialsData.cs
    SaveSnapshot/
      SaveSnapshotCommand.cs
      SaveSnapshotService.cs
      ISaveSnapshotService.cs
      SaveSnapshotExtensions.cs
      Models/SaveSnapshotData.cs
  Shared/
    Common/
      Result.cs
      ResultT.cs
      JsonExtensions.cs
    Infrastructure/
      Etabs/
        ComCleanup.cs                  ← NEW: shared COM release helper
        EtabsExtensions.cs
        EtabsConnection/               ← keep for Validate + existing
        EtabsFileOperations/           ← keep
        GenerateE2KFile/               ← keep, refactor connection
        Validation/                    ← keep, add finally cleanup
        Models/
```

---

## Key Rules for All Service Implementations

**Mode A: never call `ApplicationExit()`.** Only `ComCleanup.Release()`.

**Mode B: always call `ApplicationExit(false)` in `finally`, then `ComCleanup.Release()`.**

**Never write to `Console.WriteLine` in a service.** `Program.cs` redirects
it to stderr, but the intent is that JSON is the only stdout content.
Use `Console.Error.WriteLine` for progress lines explicitly.

**Never call `Environment.Exit()` from a service.** Only command handlers exit.

**Always `Stopwatch` long operations** (`run-analysis`, `save-snapshot`,
`generate-e2k`, `extract-results`). Include `*TimeMs` in data shape.

**Guard COM null returns early:**
```csharp
var app = ETABSWrapper.Connect();
if (app?.SapModel is null)
    return Result.Fail<TData>("ETABS connection lost — model inaccessible");
```

---

## `.csproj` Changes

```xml
<!-- Add for result extraction — pure C#, no native deps -->
<PackageReference Include="Parquet.Net" Version="5.*" />
```

---

## Phase 1 Build Order

### Step 1 — `ComCleanup.cs`
No dependencies. Establishes the cleanup pattern before any COM code.

### Step 2 — `get-status`
Tests full A/B detection. Both running and not-running paths must work.
This is the eyes of Rust's state machine — highest priority.

### Step 3 — `open-model` and `close-model`
The user-facing open/close lifecycle. Validates the `NewBlank()` workaround
in integration tests on both clean and modified models.

### Step 4 — `unlock-model`
One COM call with a path-match guard. Short, easy to verify.

### Step 5 — `generate-e2k` (refactor existing)
The existing `EtabsApiGenerateE2KFile.cs` has the correct `ExportFile` call.
Refactor to: remove the Mode A attach logic entirely, replace with Mode B
hidden-instance start, add `ApplicationExit(false)` + `ComCleanup.Release()`
in `finally`. This is also a good template for how `extract-results` and
`extract-materials` will be structured.

### Step 6 — `run-analysis`
**First command requiring `EtabSharp.Hide()`.** Implement after you have
added `Hide()`/`Unhide()` to EtabSharp. Verify `ApplicationExit(false)` is
called on all paths. Test that user's ETABS is unaffected by a parallel run.

### Step 7 — `extract-results` and `extract-materials`
Add Parquet.Net. Implement one table schema at a time. Test each schema
against a known-good analyzed model.

### Step 8 — `save-snapshot`
Capstone command. Combines Steps 5–7 under a single hidden ETABS session.
Critical requirement: `ApplicationExit(false)` must be called on every path.
Run `save-snapshot --with-results` while the user's ETABS is open and verify
the user's session is completely undisturbed.

---

## Testing Strategy

```
Unit tests (no ETABS required):
  ├── Input validation for all commands (missing --file, wrong extension, etc.)
  ├── Result<T> serialization — success and failure shapes match contract
  ├── ComCleanup.Release does not throw on null or already-released objects
  ├── JSON output fields match documented data shapes
  └── FormatDuration / FormatSize helpers

Integration tests (require ETABS install):
  ├── get-status: ETABS running → isRunning true, PID populated, version present
  ├── get-status: ETABS not running → success=true, isRunning=false
  ├── open-model: file opens, get-status confirms correct openFilePath
  ├── close-model --save: saves, NewBlank, original file no longer reported open
  ├── close-model --no-save: NewBlank without save dialog on modified model
  ├── unlock-model: wasLocked=true after analysis, cleared successfully
  ├── generate-e2k: output .e2k exists, is non-empty text
  ├── run-analysis: small model, finishedCaseCount > 0
  ├── extract-results: 7 parquet files, non-zero row counts
  └── save-snapshot --with-results: full round-trip,
                                    user's ETABS open file unchanged after run
```

Mark integration tests skipped unless env var set:
```csharp
[Fact]
[Trait("Category", "Integration")]
public async Task GetStatus_WhenRunning_ReturnsRunningState()
{
    Skip.IfNot(
        Environment.GetEnvironmentVariable("ETABS_INTEGRATION_TESTS") == "1",
        "Skipped: set ETABS_INTEGRATION_TESTS=1 to run");
    // ...
}
```
