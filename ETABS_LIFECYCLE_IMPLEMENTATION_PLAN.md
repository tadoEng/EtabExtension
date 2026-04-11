# ETABS Lifecycle Fix - Implementation Plan

Based on tech lead review of ETABS_LIFECYCLE_FIX_PROPOSAL.md

## Tech Lead's Feedback - Key Corrections

### What the Proposal Got Right
✅ Root cause diagnosis (app?.Dispose() in Mode B) is correct  
✅ Fix (remove disposal for Mode B) is correct  
✅ Keep disposal in Mode A (correct and safe)  
✅ Don't change guard logic (correct)  

### Critical Gap: Issue #2 Was Incomplete
❌ Original proposal said "verify --new-instance flag is properly wired"  
❌ **Reality:** The flag doesn't exist. EtabsOpenArgs has no --new-instance field  
❌ **Consequence:** Mode A (attach to existing ETABS) is completely unreachable from the CLI  
❌ **The proposal's own Scenario 2 example is impossible with current code**  

**This is not a verification task — it's building a missing feature.**

### Other Corrections
- **Risk table was backwards:** Mode A disposing is correct and safe (client proxy disposal doesn't kill server)
- **Timeline was optimistic:** Issue #2 needs 25 min (full build), not 5 min (verification)
- **Code style:** Empty finally block looks like dead code; add comment explaining it's intentional

## Complete Issue Summary (From Tech Lead Review)

| # | Severity | Component | Issue | Impact | Fix |
|---|----------|-----------|-------|--------|-----|
| 1 | **CRITICAL** | C# Sidecar | app?.Dispose() kills ETABS in Mode B | PID is dead moments after open, blocks all commands | Remove app?.Dispose() from finally block (1 line) |
| 2 | **HIGH** | Rust CLI | --new-instance flag missing; Mode A unreachable | Users can only spawn new instance, cannot attach to existing ETABS | Add --new-instance flag to EtabsOpenArgs, wire through pipeline |
| 3 | MEDIUM | Rust | Redundant preflight get_status + confusing error | Misleading message when ETABS running outside ext | Better error message (is it ext-managed or manual?) |
| 4 | LOW | Rust | Missing comment on Analyzed/Locked close logic | Code unclear why Interactive mode doesn't prompt | Add explanatory comment |
| 5 | LOW | C# | Mode A has no PID retry loop (race condition risk) | Rare case where open fails due to timing | Add retry loop matching Mode B pattern |
| 6 | LOW | UX | Error message "Working file state unknown" not actionable | Users don't know it means ETABS crashed | Better message: "ETABS may have crashed; run ext etabs recover" |

---

## Phase 1: Critical Fix (Issue #1)

### C# Sidecar: Remove COM dispose in Mode B

**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelService.cs`

**Current code:**
```csharp
private static async Task<Result<OpenModelData>> OpenInNewInstanceAsync(string filePath)
{
    ETABSApplication? app = null;
    try
    {
        Console.Error.WriteLine("ℹ Starting new ETABS instance...");
        app = ETABSWrapper.CreateNew(startApplication: true);
        // ... open file ...
        return Result.Ok(new OpenModelData { ... });
    }
    catch (Exception ex)
    {
        return Result.Fail<OpenModelData>($"ETABS COM error: {ex.Message}");
    }
    finally
    {
        // ❌ PROBLEM: Disposes COM proxy → kills ETABS
        app?.Dispose();
    }
}
```

**Fixed code:**
```csharp
private static async Task<Result<OpenModelData>> OpenInNewInstanceAsync(string filePath)
{
    ETABSApplication? app = null;
    try
    {
        Console.Error.WriteLine("ℹ Starting new ETABS instance...");
        app = ETABSWrapper.CreateNew(startApplication: true);
        // ... open file ...
        return Result.Ok(new OpenModelData { ... });
    }
    catch (Exception ex)
    {
        return Result.Fail<OpenModelData>($"ETABS COM error: {ex.Message}");
    }
    finally
    {
        // ✅ FIX: Don't dispose for new instances (Mode B)
        // When sidecar exits without calling Dispose(), the COM proxy is garbage-collected
        // but ETABS (out-of-process server) stays running independently.
        // The user controls the ETABS window lifetime going forward.
        // app?.Dispose();  // ← REMOVED
    }
}
```

**Mode A stays unchanged (keeps disposal):**
```csharp
private static async Task<Result<OpenModelData>> OpenInRunningInstanceAsync(
    string filePath, bool save)
{
    // ...
    finally
    {
        app?.Dispose();  // ✅ KEEP - User controls this instance
    }
}
```

---

## Phase 2: High Priority Fix (Issue #2)

### Add --new-instance flag to Rust CLI

**Status:** Flag exists in C# sidecar but is unreachable from Rust CLI.  
**Problem:** EtabsOpenArgs has no flag, etabs_open() hardcodes new_instance=true. Mode A is dead code.  
**Solution:** Wire the flag through the full pipeline.

---

**File 1:** `crates/ext/src/args/mod.rs`

**Current code:**
```rust
pub struct EtabsOpenArgs {
    pub version: Option<String>,
    // ❌ No --new-instance flag
}
```

**Fixed code:**
```rust
pub struct EtabsOpenArgs {
    pub version: Option<String>,
    /// Launch ETABS in a new instance instead of attaching to existing
    #[arg(long)]
    pub new_instance: bool,
}
```

---

**File 2:** `crates/ext-api/src/etabs.rs` - etabs_open signature

**Current code:**
```rust
pub async fn etabs_open(ctx: &AppContext, version_ref: Option<&str>) -> Result<EtabsOpenResult> {
    // ...
    let opened = sidecar
        .open_model(&target_file, false, true)  // ❌ Hardcoded: new_instance = true
        .await?;
    // ...
}
```

**Fixed code:**
```rust
pub async fn etabs_open(ctx: &AppContext, version_ref: Option<&str>, new_instance: bool) -> Result<EtabsOpenResult> {
    // ...
    let opened = sidecar
        .open_model(&target_file, false, new_instance)  // ✅ Pass through user's choice
        .await?;
    // ...
}
```

---

**File 3:** `crates/ext/src/commands/etabs_open.rs` - wire the flag through

Find where EtabsOpenArgs is used and pass new_instance:
```rust
let result = etabs_open(
    &ctx,
    args.version.as_deref(),
    args.new_instance,  // ✅ NEW: Pass the flag
).await?;
```

**Expected Usage After Fix:**
```bash
# Mode A (attach to existing ETABS) - default without flag
ext.exe etabs open
ext.exe etabs open main/v2

# Mode B (launch new ETABS) - explicit with flag
ext.exe etabs open --new-instance
ext.exe etabs open main/v2 --new-instance
```

---

## Phase 3: Medium Priority Fix (Issue #3)

### Improve get_status preflight error message

**File:** `crates/ext-api/src/etabs.rs` - etabs_open function

**Current code:**
```rust
let sidecar = ctx.require_sidecar()?;
let preflight = sidecar
    .get_status()
    .await
    .context("Failed to check ETABS status before opening")?;
if preflight.is_running {
    let detail = preflight
        .open_file_path
        .as_deref()
        .map(|path| format!("\n  Open file: {}", normalize_display(Path::new(path)).display()))
        .unwrap_or_default();
    bail!(
        "✗ ETABS is already running\n  Close the existing session first with: ext etabs close{}",
        detail
    );
}
```

**Fixed code:**
```rust
let sidecar = ctx.require_sidecar()?;
let preflight = sidecar
    .get_status()
    .await
    .context("Failed to check ETABS status before opening")?;
if preflight.is_running {
    // Distinguish between in-band (ext-managed) and out-of-band (manual) ETABS
    let is_ext_managed = state.working_file
        .as_ref()
        .and_then(|wf| wf.etabs_pid)
        .is_some();
    
    if is_ext_managed {
        let detail = preflight
            .open_file_path
            .as_deref()
            .map(|path| format!("\n  Open file: {}", normalize_display(Path::new(path)).display()))
            .unwrap_or_default();
        bail!(
            "✗ ETABS is already running in this project\n  Close it with: ext etabs close{}",
            detail
        );
    } else {
        bail!(
            "✗ ETABS is already running (started outside ext)\n  Close ETABS manually and try again"
        );
    }
}
```

---

## Phase 4: Low Priority Fixes (Issues #4, #5, #6)

### Issue #4: Add comment to etabs_close

**File:** `crates/ext-api/src/etabs.rs` - etabs_close function

Add comment before the save match:
```rust
// Determine save behavior. For Interactive mode, only prompt if status is OpenModified.
// Analyzed/Locked states (result of prior analysis run) don't prompt because the contract
// is that the user should commit the analysis results first — closing just discards them.
let save = match (status, mode) {
    (WorkingFileStatus::OpenModified, CloseMode::Interactive) => {
        return Err(anyhow::Error::new(EtabsCloseConflict { ... }));
    }
    // ... rest unchanged
};
```

---

### Issue #5: Add PID retry loop to Mode A

**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelService.cs`

**Current OpenInRunningInstanceAsync:**
```csharp
private static async Task<Result<OpenModelData>> OpenInRunningInstanceAsync(
    string filePath, bool save)
{
    // ...
    var pid = ETABSWrapper.GetAllRunningInstances().FirstOrDefault()?.ProcessId;
    Console.Error.WriteLine($"✓ Opened: {Path.GetFileName(filePath)}");
    
    return Result.Ok(new OpenModelData
    {
        FilePath = filePath,
        PreviousFilePath = hasCurrentFile ? currentPath : null,
        Pid = pid,  // ❌ Could be null due to race condition
        OpenedInNewInstance = false
    });
}
```

**Fixed code (add retry for Mode A like Mode B):**
```csharp
private static async Task<Result<OpenModelData>> OpenInRunningInstanceAsync(
    string filePath, bool save)
{
    // ...
    var pid = await WaitForPidAsync(newestFirst: false);  // ✅ Use retry logic
    if (pid is null)
    {
        // Fallback if WaitForPidAsync fails (shouldn't happen in normal case)
        pid = ETABSWrapper.GetAllRunningInstances().FirstOrDefault()?.ProcessId;
    }
    
    Console.Error.WriteLine($"✓ Opened: {Path.GetFileName(filePath)}");
    
    return Result.Ok(new OpenModelData
    {
        FilePath = filePath,
        PreviousFilePath = hasCurrentFile ? currentPath : null,
        Pid = pid,  // ✅ Now has retry safety
        OpenedInNewInstance = false
    });
}
```

---

### Issue #6: Improve Orphaned error message

**File:** `crates/ext-api/src/guards.rs`

Update the Orphaned block message:
```rust
(Commit, Orphaned) => {
    Block(
        "✗ Working file state unknown (ETABS may have crashed)\n  Run: ext etabs recover"
            .into()
    )
},
(CommitAnalyze, Orphaned) => {
    Block(
        "✗ Working file state unknown (ETABS may have crashed)\n  Run: ext etabs recover"
            .into()
    )
},
(Switch, Orphaned) => {
    Block(
        "✗ Working file state unknown (ETABS may have crashed)\n  Run: ext etabs recover"
            .into()
    )
},
// ... etc
```

---

## Implementation Order (Updated Timeline)

### Critical Path (Must do first)
1. **Issue #1 (C# - 5 min):** Remove app?.Dispose() in OpenInNewInstanceAsync
2. **Issue #2 (Rust - 25 min):** Add --new-instance flag, wire through full pipeline
   - Add field to EtabsOpenArgs (args/mod.rs)
   - Add parameter to etabs_open() signature (ext-api/src/etabs.rs)
   - Wire through CLI command (commands/etabs_open.rs)
   - Pass to sidecar.open_model() call
   - **Note:** Current code hardcodes new_instance=true, making Mode A unreachable. This is not a verification task — it's building a missing feature.

### Polish (After critical path)
3. **Issue #3 (Rust - 10 min):** Improve error messages for preflight
4. **Issue #4 (Rust - 2 min):** Add comment to etabs_close
5. **Issue #5 (C# - 5 min):** Add retry loop to Mode A
6. **Issue #6 (Rust - 3 min):** Improve Orphaned error messages

**Total: ~50 minutes** (corrected from 40, mostly due to Issue #2 being a full build not a verification)

---

## Testing Checklist

- [ ] Build C# sidecar: `dotnet build`
- [ ] Build Rust: `cargo build --release`
- [ ] Test Mode B (new instance):
  ```bash
  ext.exe etabs open --new-instance
  # ETABS launches and stays open
  ext.exe etabs status
  # Shows: ETABS Running: true, Working File: OpenClean
  ```
- [ ] Test Mode A (attach to existing):
  ```bash
  # Open ETABS manually first
  ext.exe etabs open  # (no --new-instance)
  # File opens in existing ETABS
  ext.exe etabs status
  # Shows: ETABS Running: true, Working File: OpenClean
  ```
- [ ] Test error messages:
  ```bash
  ext.exe etabs open  # when ETABS already running from ext
  # Shows: "Close it with: ext etabs close"
  
  ext.exe etabs open  # when ETABS running outside ext
  # Shows: "Close ETABS manually and try again"
  ```
- [ ] Test workflow:
  ```bash
  ext.exe etabs open --new-instance
  # Edit file in ETABS
  ext.exe commit "message"
  # Blocked: "Close ETABS before committing"
  ext.exe etabs close
  # Success
  ext.exe commit "message"
  # Success
  ```

---

## Rollback Plan

If any issue arises:
- **Issue #1:** Restore app?.Dispose() (restore from git)
- **Issue #2:** Remove --new-instance flag (restore from git)
- Rust changes are all additive; safe to revert individually

