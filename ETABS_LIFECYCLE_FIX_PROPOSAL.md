# Fix ETABS Open Lifecycle Issue - Review Proposal

## Problem Statement

### Current Behavior (Broken)

User runs:
```bash
ext.exe etabs open --new-instance
✓ ETABS opened (PID: 13448)
```

Then immediately:
```bash
ext.exe etabs status
ETABS Running: false
Working File: Orphaned  ← Should be OpenClean!
```

Next command fails:
```bash
ext.exe switch steel-columns
Error: Command failed
Caused by:
    ✗ Working file state unknown
      Run: ext etabs recover
```

### Root Cause

1. **C# Sidecar (Mode B - new instance):**
   - Launches ETABS via COM: `ETABSWrapper.CreateNew(startApplication: true)`
   - Opens file
   - **Disposes COM object** in finally block: `app?.Dispose()`
   - Sidecar exits

2. **COM behavior:**
   - When COM proxy is disposed, ETABS process terminates
   - ETABS becomes a dead process immediately

3. **Rust side:**
   - Stores PID from sidecar response
   - On next command, checks `is_pid_alive(pid)`
   - PID is dead → Status becomes `ORPHANED`
   - Guards block all operations

### Why This Is Wrong

- **PID is valid when returned** (ETABS is running)
- **PID is dead moments later** (sidecar kills it on exit)
- **Rust's PID monitoring becomes useless**
- **User cannot proceed without `ext etabs recover`**
- **Shared ETABS licenses are wasted** if we force close

---

## Proposed Solution

### Phase 1: C# Sidecar Fix

**File:** `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\OpenModelService.cs`

**Change:** Conditional COM disposal based on mode

**Current code (both modes):**
```csharp
private static async Task<Result<OpenModelData>> OpenInNewInstanceAsync(string filePath)
{
    ETABSApplication? app = null;
    try
    {
        // ... launch ETABS, open file ...
        return Result.Ok(new OpenModelData { FilePath = filePath, Pid = pid, ... });
    }
    catch (Exception ex)
    {
        return Result.Fail<OpenModelData>($"ETABS COM error: {ex.Message}");
    }
    finally
    {
        app?.Dispose();  // ← PROBLEM: Always disposes, kills Mode B ETABS
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
        // ... launch ETABS, open file ...
        return Result.Ok(new OpenModelData { FilePath = filePath, Pid = pid, ... });
    }
    catch (Exception ex)
    {
        return Result.Fail<OpenModelData>($"ETABS COM error: {ex.Message}");
    }
    finally
    {
        // NEW: Don't dispose for new instances
        // app?.Dispose();  // ← REMOVE for Mode B
        // Sidecar exits but ETABS stays alive as independent process
        // COM object released by garbage collection (safe for out-of-process server)
    }
}
```

**For Mode A (attach to existing)** - keep disposal:
```csharp
private static async Task<Result<OpenModelData>> OpenInRunningInstanceAsync(string filePath, bool save)
{
    ETABSApplication? app = null;
    try
    {
        // ... attach to existing ETABS, open file ...
        return Result.Ok(new OpenModelData { ... });
    }
    catch (Exception ex)
    {
        return Result.Fail<OpenModelData>($"ETABS COM error: {ex.Message}");
    }
    finally
    {
        app?.Dispose();  // ← KEEP: Safe, user controls ETABS lifetime
    }
}
```

### Phase 2: Rust CLI Verification

**File:** `crates/ext-core/src/sidecar/client.rs`

**Current timeout:** 120 seconds for `open-model` (fine as-is)

**Why:** Sidecar now returns JSON **immediately**, so 120s is sufficient. No monitoring overhead.

**Verification needed:**
- Confirm `--new-instance` flag is properly wired through CLI commands
- If missing, add to `crates/ext/src/commands/etabs_open.rs`

### Phase 3: Workflow Guarantee

**Guards remain unchanged** (current behavior is correct):

- `ext etabs open` → Records PID, sets status `OpenClean`
- User works in ETABS (keeps it open, edits file)
- **User must call** `ext etabs close` before any state-changing command (`commit`, `switch`, `checkout`)
  - This ensures explicit lifecycle management
  - Prevents accidental conflicts
  - Makes user intent clear

**Why this is good for shared licenses:**
- ETABS stays open indefinitely after `ext etabs open`
- No forced close
- User calls `ext etabs close` when done
- No license waste

---

## Expected Behavior After Fix

### Scenario 1: New Instance (Mode B)
```bash
PS> ext.exe etabs open --new-instance
✓ ETABS opened (PID: 13448)
[ETABS window appears and stays open independently]

[User edits file in ETABS for 30 minutes]

PS> ext.exe etabs status
ETABS Running: true          ← ✅ NOW CORRECT
Working File: OpenClean      ← ✅ FILE UNCHANGED

PS> ext.exe etabs close
✓ Closed ETABS

PS> ext.exe commit "Updated model"
✓ Version v2 saved           ← ✅ WORKS (guards allow it now)
```

### Scenario 2: Attach to Existing (Mode A)
```bash
[User opens ETABS manually first]

PS> ext.exe etabs open
✓ ETABS opened (file switched to working file)

PS> ext.exe etabs close
✓ Closed file (ETABS still running for user's other work)

PS> ext.exe commit "Updated"
✓ Version created
```

---

## Changes Summary

| Component | File | Change | Impact |
|-----------|------|--------|--------|
| **C# Sidecar** | `OpenModelService.cs` | Remove `app?.Dispose()` in Mode B finally block | ETABS stays alive when sidecar exits |
| **C# Mode A** | `OpenModelService.cs` | Keep `app?.Dispose()` | User controls ETABS lifetime (safe) |
| **Rust** | `sidecar/client.rs` | Verify timeout (no change needed) | Sidecar returns fast enough |
| **Rust CLI** | `commands/etabs_open.rs` | Verify `--new-instance` flag exists | Pass flag to sidecar |
| **Guards** | `guards.rs` | No changes (keep current) | Workflow: open → close → commit |

---

## Testing Plan

### Manual Testing
1. `ext etabs open --new-instance` 
   - ✅ ETABS launches
   - ✅ JSON returns immediately with PID
   - ✅ ETABS window stays open after sidecar exits

2. `ext etabs status` (while ETABS open)
   - ✅ Shows `ETABS Running: true`
   - ✅ Shows `Working File: OpenClean` (no edits)

3. Edit file in ETABS, call `ext etabs status`
   - ✅ Shows `Working File: OpenModified`

4. `ext commit` (while ETABS still open)
   - ✅ Blocked: "Close ETABS before committing"

5. `ext etabs close`
   - ✅ Closes ETABS
   - ✅ Sidecar returns success

6. `ext commit "message"`
   - ✅ Creates new version
   - ✅ No "Orphaned" errors

### Automated Testing
- Unit tests: Verify `is_pid_alive()` correctly detects running ETABS
- Integration: Test open → status → commit cycle

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| COM object garbage collection delays ETABS exit | ETABS is out-of-process; GC doesn't affect it. Safe. |
| Mode A (attach) still disposes → user's ETABS dies | Only Mode A disposes. User's ETABS continues. Safe. |
| User forgets to call `ext etabs close` | Guards block subsequent commands → forces explicit close. By design. |
| Sidecar process hangs if ETABS crashes | Sidecar exits normally. No waiting or monitoring. Not an issue. |

---

## Open Questions for Tech Lead

1. **License management:** Should we add metrics to track ETABS open duration per user?
2. **Orphaned recovery:** Keep current `ext etabs recover` command for crash scenarios?
3. **Mode A default:** Do we want `ext etabs open` (without `--new-instance`) to work, or require explicit flag?
4. **Future:** Should `ext etabs close` prompt "Save changes?" if file was modified?

---

## Timeline

- **C# Sidecar:** ~5 minutes (remove 1 line + test)
- **Rust verification:** ~5 minutes (check flag wiring)
- **Testing:** ~15 minutes
- **Total:** ~30 minutes

---

## References

- Current Sidecar: `D:\Work\EtabExtension.CLI\src\EtabExtension.CLI\Features\OpenModel\`
- Rust Guards: `crates/ext-api/src/guards.rs`
- Rust CLI: `crates/ext/src/commands/etabs_open.rs`
- Architecture doc: `ETABS_SIDECAR_ARCHITECTURE.md`
