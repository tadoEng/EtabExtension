
# ETABS Extension CLI — Development Guidelines

This document defines conventions, architecture rules, and development order for the `ext` CLI crate and the **ext agent**. The CLI is a **first-class frontend**, equal in importance to the desktop (Tauri) app. The agent (`ext chat`) is a **first-class frontend** as well, but **only** as a caller of `ext-api`.

Read the full planning documents before implementing anything:

- `references/concepts.md` — mental model, state machine, all core concepts
- `references/reference.md` — complete command surface and flags
- `references/architecture.md` — crate map, data flows, technology stack
- `references/workflow.md` — implementation contract: exact sequences, permission matrix, error standards
- `references/examples.md` — real-world usage scenarios
- `references/ai.md` — AI agent integration: providers, privacy, tool surface, phase rollout

---

## Architecture Overview

```text
ext (CLI binary)          ← thin clap layer, zero business logic
ext-agent (via `ext chat`)← conversation loop, calls ext-api as tools
    ↓ calls
ext-api                   ← SINGLE SOURCE OF TRUTH for all operations
    ↓ calls
ext-core                  ← pure domain logic (no I/O frameworks)
ext-db                    ← storage (state.json, config.toml, SQLite)
ext-error                 ← shared error types
    ↓ calls
etab-cli.exe (C# sidecar) ← ETABS COM APIs
git subprocess            ← VCS writes
gix crate                 ← VCS reads

ext-agent-llm             ← LlmClient trait + provider backends (separate concern)
```

The CLI binary itself contains **no business logic**. Every command is:

1. Parse args via `clap`
2. Call the corresponding `ext-api` function
3. Format and write the result via `OutputChannel`

The **agent** (`ext chat`) follows the same rule: it calls `ext-api` functions as tools. The agent **cannot bypass** any state guard the CLI cannot bypass.

---

## Crate Usage Rules

| Layer          | Allowed in CLI | Purpose |
|----------------|----------------|---------|
| `ext-core`     | ✅ yes         | Domain types only — never call directly for operations |
| `ext-api`      | ✅ yes         | All application workflows and orchestration |
| `ext-db`       | ⚠️ sparingly  | Direct access only for migrations, diagnostics, maintenance commands |
| `ext-agent`    | ✅ yes         | Only for `ext chat` command handler — nothing else |
| `ext-agent-llm`| ❌ never       | Provider backends — CLI never touches these directly |
| `ext-tauri`    | ❌ never       | UI-specific code, window management |

**Rule (CLI):** If you find yourself writing business logic in a CLI command handler, it belongs in `ext-api` instead. The CLI handler should be **under 20 lines**.

**AI Rule (agent):** If you find yourself writing tool dispatch or conversation logic in the `ext chat` handler, it belongs in `ext-agent` instead. The chat handler should be **under 15 lines**.

---

## Command Naming — Git-Mimic Convention

Commands mirror modern git (post-2019 split of `switch` and `restore`). Do **not** use old git naming.

```bash
# ✅ Correct command names
ext init "Project"
ext status
ext log
ext show v3
ext diff v2 v3
ext branch
ext branch steel-columns --from main/v3
ext branch -d steel-columns
ext switch steel-columns
ext switch -c steel-columns --from main/v3   # create + switch
ext checkout v1
ext checkout main/v1
ext stash
ext stash pop
ext stash drop
ext stash list
ext commit "message"
ext commit "message" --analyze
ext commit "message" --no-e2k
ext analyze v3
ext etabs open
ext etabs close
ext etabs status
ext etabs validate --file <path>
ext etabs unlock
ext etabs recover
ext report analysis --version v3
ext report bom --version v3
ext report comparison --from main/v3 --to steel/v1
ext push
ext pull
ext clone <onedrive-path> --to <local-path>
ext remote status
ext config get <key>
ext config set <key> <value>
ext config list
ext config edit
ext chat                                      # Phase 1 AI: interactive REPL
ext chat --provider ollama                    # override provider for session
ext chat --non-interactive                    # Phase 2: stdin → stdout

# ❌ Wrong — do not use these
ext branch new steel-columns        # use: ext branch steel-columns
ext branch switch steel-columns     # use: ext switch steel-columns
ext branch delete steel-columns     # use: ext branch -d steel-columns
ext version save "message"          # use: ext commit "message"
ext version restore v3              # use: ext checkout v3
ext restore v3                      # use: ext checkout v3
ext save "message"                  # use: ext commit "message"
ext branch merge                    # out of scope Phase 1
ext ai <anything>                   # use: ext chat
```

**Aliases** (acceptable shorthands):

```bash
ext ci  → ext commit
ext co  → ext checkout
ext sw  → ext switch
```

---

## Output Abstraction

All user-facing output goes through `OutputChannel`. Never write directly to `stdout` or `stderr`.

```rust
pub fn execute(out: &mut OutputChannel, result: &BranchCreated) -> Result<()> {
    if let Some(out) = out.for_human() {
        writeln!(out, "✓ Created branch '{}'", result.branch_name)?;
        writeln!(out, "  Based on: {}", result.created_from)?;
    }

    if let Some(out) = out.for_shell() {
        writeln!(out, "{}", result.branch_name)?;  // just the value
    }

    if let Some(out) = out.for_json() {
        out.write_value(result)?;
    }

    Ok(())
}
```

### Output Mode Rules

- **Human** (default): rich text, icons (`✓` `✗` `⚠`), progress bars, colour
- **Shell** (`--shell`): one value per line, no decoration — for scripting
- **JSON** (`--json`): stable, versioned structs — for Tauri IPC and automation

Rules:

- Never mix output modes in a single command.
- Progress indicators only in **human** mode.
- Errors always go to `stderr`.
- Structured results always go to `stdout`.

`ext chat` is **always human mode**. The chat REPL never uses `--json` or `--shell`. The agent’s **internal tool calls** use the `ext-api` return types directly — they never go through `OutputChannel`.

---

## Error Message Format

Follow the standard from `workflow.md §20` exactly:

```text
✗ <what failed>
  <why it failed>
  Run: <command to fix it>
```

```text
⚠ <what the user should know>
  <consequence if ignored>
  Run: <command to address it>
```

```text
✓ <what was accomplished>
→ Next: <suggested next command>   (only on first-time / init flows)
```

**Good:**

```text
✗ ETABS file is currently open
  Close ETABS before committing
  Run: ext etabs close

File: D:\Projects\HighRise\main\working\model.edb
PID:  12345
```

**Bad:**

```text
Error: File locked
```

---

## Context and Determinism

**Do not implicitly discover state.** Everything needed must be passed explicitly.

```rust
// ❌ Bad — implicit state
pub fn open_project() -> Result<Project> {
    let cwd = env::current_dir()?;
    Project::open(cwd)
}

// ✅ Good — explicit parameter
pub fn open_project(path: &Path) -> Result<Project> {
    Project::open(path)
}
```

**Never use implicit globals** in command handlers or **agent tools**:

| Bad | Good |
|---|---|
| `std::env::current_dir()` | Pass path explicitly via `--project-path` or arg |
| `std::time::SystemTime::now()` | Pass time as an argument |
| `std::env::var()` | Pass config via `AppContext` |

Same input → same output. Commands and agent tools must be **deterministic and CI-safe**.

---

## Config Files — Two-Tier System

There are exactly two config files. Do not add a third level.

| File | Tracked | Purpose |
|---|---|---|
| `.etabs-ext/config.toml`        | ✅ git-tracked | Shared project settings (pushed to OneDrive) |
| `.etabs-ext/config.local.toml`  | ❌ git-ignored | Machine-specific: author, email, OneDrive path, reports path, AI provider/model/key |

**Resolution order:** `config.local.toml` → `config.toml` → ext defaults.

`config.local.toml` is created interactively on `ext init` and `ext clone`. It is **never** overwritten by `ext pull`. It is never pushed to OneDrive.

**Keys that belong in `config.local.toml`:**

- `git.author`, `git.email`
- `paths.oneDriveDir`, `paths.reportsDir`
- `onedrive.acknowledgedSync`
- `ai.provider`, `ai.model`, `ai.apiKey`, `ai.baseUrl`, `ai.autoConfirm`

**Keys that belong in `config.toml`:**

- `project.name`
- `etabs.sidecarPath`
- `behavior.confirmDestructive`, `behavior.pushWorking`
- `paths.reportNaming`

**AI keys always go to `config.local.toml`.** API keys must never appear in `config.toml` — it is git-tracked and pushed to OneDrive. This is enforced in `ext config set` routing logic.

Example:

- When a user runs `ext config set git.author "Jane"`, the CLI must automatically write to `config.local.toml`, not `config.toml`.
- When a user runs `ext config set ai.apiKey "sk-..."`, the key must be routed to `config.local.toml` and never written to any git-tracked file.

---

## State Machine

Every command (and agent tool) begins by resolving working file state. The resolution order is defined in `workflow.md §State Detection` and must be followed exactly:

```text
1. Does working/model.edb exist?     → MISSING if no
2. Is ETABS PID alive?               → OPEN_* or ORPHANED
3. Is basedOnVersion set?            → UNTRACKED if no
4. mtime vs lastKnownMtime           → MODIFIED or CLEAN
```

Before executing any command — including agent tool calls — check the **permission matrix** in `workflow.md §15`. If a command is not permitted in the current state, return the appropriate error with a remediation command.

```rust
// Example: guard for ext commit (same guard used by agent commit tool)
    match state {
    WorkingFileState::OpenClean | WorkingFileState::OpenModified =>
        bail!("✗ Close ETABS before committing\n  Run: ext etabs close"),
    WorkingFileState::Orphaned =>
        bail!("✗ Working file state unknown\n  Run: ext etabs recover"),
    WorkingFileState::Missing =>
        bail!("✗ Working file missing\n  Run: ext checkout v1"),
    _ => {} // UNTRACKED, CLEAN, MODIFIED, ANALYZED — proceed
}
```

---

## Sidecar Integration

The sidecar (`etab-cli.exe`) is the **only** component that can call ETABS COM APIs. The Rust CLI — **and the agent** — never call COM directly.

**Always access the sidecar via `ctx.require_sidecar()`** — never via `ctx.sidecar.as_ref()` directly.

```rust
// ✅ Correct — require_sidecar() gives a clear error if not configured
pub async fn etabs_open(ctx: &AppContext, file: &Path) -> Result<OpenModelData> {
    let sidecar = ctx.require_sidecar()?;
    sidecar.open_model(file, false, false).await.map_err(Into::into)
}

// ❌ Wrong — silent Option handling, no user-facing error
pub async fn etabs_open(ctx: &AppContext, file: &Path) -> Result<OpenModelData> {
    if let Some(sidecar) = ctx.sidecar.as_ref() {   // NEVER do this
        sidecar.open_model(file, false, false).await.map_err(Into::into)
    } else {
        Ok(...)  // silently does nothing — user gets no error
    }
}
```

**The one permitted exception** is `ext status --verbose`, which degrades gracefully with a warning instead of failing hard when the sidecar is not configured. That call site carries an explicit comment. No other call site may access `ctx.sidecar` directly.

**Always find the sidecar via `AppContext::new(root)`** — it checks:

1. `config.toml` / `config.local.toml` → `project.sidecar-path`
2. `ETABS_SIDECAR_PATH` env var
3. `PATH`

Never hardcode the path.

**All sidecar operations are single-shot:** one command, one job, exit. No daemon, no persistent connection.

**IPC contract:**

- `stdin`: nothing
- `stdout`: `Result<T>` JSON (always, even on failure)
- `stderr`: human-readable progress — forwarded live to terminal via `tracing::info!`
- `exit`: `0` = success, `1` = failure


---

## File Operations — Atomic Copies

All `.edb` file copies use write-to-temp-then-rename. This prevents partial writes if the process is killed mid-copy.

```rust
fn atomic_copy(src: &Path, dst: &Path) -> Result<()> {
    let tmp = dst.with_extension("edb.tmp");
    fs::copy(src, &tmp)?;
    fs::rename(&tmp, dst)?;   // atomic on same filesystem
    Ok(())
}
```

Always check disk space before copying large `.edb` files (require **10% buffer**):

```rust
let required = fs::metadata(src)?.len();
let available = available_space(dst.parent().unwrap())?;
if available < required + (required / 10) {  // require 10% buffer
    bail!("✗ Insufficient disk space ...");
}
```

On startup, clean up any stray `.edb.tmp` files.

---

## AI Agent Rules

The agent is a **caller of `ext-api`, nothing more.**

```rust
// ✅ Correct — agent tool calls ext-api exactly as CLI does
async fn commit_tool(&self, input: Value) -> Result<Value> {
    let msg = input["message"].as_str().unwrap_or_default();
    let result = ext_api::commit_version(&self.ctx, msg, Default::default()).await?;
    Ok(serde_json::to_value(result)?)
}

// ❌ Wrong — agent must never call ext-core or ext-db directly
async fn commit_tool(&self, input: Value) -> Result<Value> {
    let result = ext_core::version::commit(&self.ctx.project, msg).await?;  // NEVER
    Ok(serde_json::to_value(result)?)
}
```

### API keys and logging

- API keys are **never** logged or traced.
- Always redact keys in logs:

```rust
// ✅ Safe — redact key in all log output
tracing::debug!("Connecting to Claude API (key: {}...)", &key[..8]);

// ❌ Never log the full key
tracing::debug!("Connecting to Claude API (key: {})", key);
```

### Data privacy — what can reach the LLM

Agent tools must **not embed model data** in prompts:

- The LLM receives **text summaries** of project state, not raw binary content.
- `.edb` bytes, raw Parquet data, and full `.e2k` files are **never** sent to any LLM.

```rust
// ✅ Safe — send diff summary text only
let diff = ext_api::diff(&ctx, "v2", "v3").await?;
// diff is already a human-readable text diff of E2K — send as-is

// ❌ Never — do not read and send raw file bytes
let edb_bytes = fs::read("working/model.edb").await?;
// never include edb_bytes in any LLM message
```

### Default provider and local-first policy

When `ai.provider` is not set, default to `"ollama"` with a clear message:

```text
⚠ No AI provider configured.
  Using Ollama (local) — no data leaves your machine.
  Make sure Ollama is running: https://ollama.com
  Run: ext config set ai.model "qwen2.5-coder:14b"

  To use Claude instead: ext config set ai.provider claude
                         ext config set ai.apiKey "sk-ant-..."
```

### Confirmation gate for write tools

Every **write tool** must go through the confirmation gate before calling `ext-api`.

- `--no-confirm` (for CLI) bypasses the prompt but still prints the action.
- Truly destructive operations (discard, force delete) **always** confirm regardless of `--no-confirm`.
- For the agent, the confirmation gate in `ext-agent` **replaces** `--force`. The agent never passes `--force` without explicit user confirmation.

```rust
// In ext-agent confirmation gate
const ALWAYS_CONFIRM: &[&str] = &[
    "checkout_discard",   // checkout with [d] discard
    "branch_force_delete",
    "stash_drop",
];

async fn dispatch_write(&mut self, name: &str, input: &Value) -> Result<Value> {
    let always = ALWAYS_CONFIRM.contains(&name);
    if always || !self.auto_confirm {
        self.confirm(name, input).await?;
    }
    // ...
}
```

`ext chat` itself is **always human mode** (see Output section). The agent never asks `ext` to use `--json` or `--shell`.

---

## OneDrive Awareness

**Detection on `ext init`:** Check if `--path` or `--edb` is inside a OneDrive-synced folder by scanning path ancestors for `OneDrive`, `OneDrive - `, or `SharePoint`. If detected, warn and prompt.

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

**`ext status` persistent warning:** If project is inside a OneDrive-synced path and `config.local.toml { onedrive.acknowledgedSync }` is not `true`, show the warning on every `ext status`.

**`ext push` requires `paths.oneDriveDir`** in `config.local.toml`. If not set:

```text
✗ OneDrive folder not configured
  Run: ext config set paths.oneDriveDir "C:\Users\...\OneDrive\Structural\HighRise"
```

**Reports auto-route to OneDrive:** If `paths.reportsDir` is set, all `ext report` commands write there by default. `--out` flag overrides for one-off outputs.

---

## VCS Rules

**Writes → git subprocess only:**

```rust
fn git(args: &[&str], cwd: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()?;
    if !status.success() {
        return Err(EtabsError::GitError(args.join(" ")));
    }
    Ok(())
}
```

**Reads → gix crate only.** Never use git subprocess for reads.

**Never expose git to users.**

- No commit hashes in user-facing output (unless `--verbose`).
- No "staging area" concept.
- No "HEAD detached" messages.
- All git concepts are re-expressed as **domain language**.

**Internal commits** are prefixed `ext:` and filtered from `ext log`:

```rust
// Internal — hidden from user
git(&["commit", "-m", "ext: analysis results v3"], path)?;
git(&["commit", "-m", "ext: init project"], path)?;

// User-visible
git(&["commit", "-m", "Updated column sections"], path)?;
```

`ext log` implementation must filter commits where `message.starts_with("ext:")`.

**Git LFS is not used.**

- `.edb` files are stored beside git and git-ignored.
- Parquet files are git-ignored.
- Only **text** files go in git.

---

## Interactive Prompts Policy

The default for destructive operations is **non-interactive with `--force` override**.

Exception: the `ext checkout` decision tree always prompts when working file is **MODIFIED** — this is intentional and must not be suppressed without explicit `--force`.

```bash
# ✅ Default: interactive prompt on data loss (checkout with MODIFIED working file)
ext checkout v1
# ⚠ Working file has changes since v3.
#   [c] Commit first  [s] Stash  [d] Discard  [x] Cancel

# ✅ CI/automation: force flag bypasses prompt
ext checkout v1 --force           # implies [d] discard

# ✅ Destructive branch delete: non-interactive by default
ext branch -d steel-columns
# ✗ Branch has uncommitted work. Use --force to delete anyway.

ext branch -d steel-columns --force   # ✅ bypasses protection
```

**`ext etabs recover` always prompts** — it is a recovery operation and should never be automated silently.

**`ext push` conflict prompt always shows** — renaming a version is not reversible without pushing again.

For the agent:

- The **confirmation gate** in `ext-agent` replaces `--force`.
- The agent never passes `--force` without explicit user confirmation.

---

## Error Handling

Use `ext-error::EtabsError` for **domain errors**. Use `anyhow::Context` to add context to I/O and external errors.

```rust
use anyhow::Context;

// ✅ Domain error — use EtabsError directly
return Err(EtabsError::SnapshotMissing { version: "v3".into() });

// ✅ External error — add context with anyhow
let project = Project::open(path)
    .with_context(|| format!("Failed to open project at {}", path.display()))?;

// ❌ Never panic in command paths
let project = Project::open(path).unwrap();
```

Never use `unwrap()` or `expect()` in command handlers, agent tools, or any code reachable from a user action. Use `?` with proper error context.

---

## Testing

### Snapshot Testing with snapbox

```rust
use snapbox::str;

#[test]
fn test_branch_list() {
    let mut cmd = Command::cargo_bin("ext").unwrap();
    cmd.arg("branch").arg("--json");

    cmd.assert()
       .success()
       .stdout_eq(str![[r#"
{
  "branches": [
    { "name": "main", "latestVersion": "v3", "versionCount": 3 }
  ]
}
"#]]);
}
```

Update snapshots:

```bash
SNAPSHOTS=overwrite cargo test -p ext
```

### Agent Tool Testing

Test agent tools via `ext-agent` directly — not by running `ext chat` end-to-end. Mock the `LlmClient` trait to inject fixed tool call responses:

```rust
#[tokio::test]
async fn test_agent_commit_tool_blocked_when_etabs_open() {
    let ctx = AppContext::for_test_with_state(WorkingFileState::OpenClean);
    let llm = MockLlmClient::responding_with_tool("commit_version", json!({
        "message": "test commit"
    }));
    let mut session = AgentSession::new(&ctx, llm);
    let response = session.chat("commit my changes").await.unwrap();
    assert!(response.contains("Close ETABS before committing"));
}
```

### Integration Tests — Use Isolated Projects

```rust
#[tokio::test]
async fn test_commit_creates_version() {
    let temp = TempDir::new().unwrap();
    // Copy a test .edb fixture into temp/main/working/model.edb
    // Run ext-api::commit_version() directly — not the CLI binary
    let ctx = AppContext::for_test(temp.path());
    let result = ext_api::commit_version(&ctx, "Test commit", Default::default()).await;
    assert!(result.is_ok());
    assert!(temp.path().join(".etabs-ext/main/v1/model.e2k").exists());
}
```

Test `ext-api` directly in integration tests, not the CLI binary. The CLI is a thin layer — snapshot test the output format, integration test the business logic via `ext-api`.

### Testing State Machine Transitions

Each state transition in `workflow.md §Command Permission Matrix` needs a test. For blocked states:

```rust
#[tokio::test]
async fn test_commit_blocked_when_etabs_open() {
    let ctx = AppContext::for_test_with_state(WorkingFileState::OpenClean, ...);
    let err = ext_api::commit_version(&ctx, "msg", Default::default()).await.unwrap_err();
    assert!(err.to_string().contains("Close ETABS before committing"));
}
```

---

## Adding a New Command — Step Order

Follow this order exactly. Do not skip steps.

1. **Define domain types** in `ext-core` (request/response structs, any new enums).
2. **Add error variants** to `ext-error::EtabsError` if needed.
3. **Implement business logic** in `ext-core` (pure, no I/O).
4. **Add API function** in `ext-api` (orchestration: calls `ext-core` + `ext-db` + sidecar).
5. **Write unit tests** for `ext-core` logic.
6. **Write integration tests** targeting `ext-api` directly.
7. **Create command module** in `crates/ext/src/commands/<name>.rs`.
8. **Create args struct** in `crates/ext/src/args/<name>.rs` (clap derive).
9. **Register in main** command enum.
10. **Add agent tool** in `ext-agent/src/tools/` if the command should be accessible to the AI (most commands should be).
11. **Write snapshot tests** for CLI output format (all three output modes).
12. **Update `--help` text** — must be accurate and complete.
13. **Update docs** (`reference.md`, `examples.md`, and `ai.md` if tool added).
14. **Generate shell completions** (run completion generation script).

Command module pattern:

```rust
// crates/ext/src/commands/branch.rs

use anyhow::Result;
use ext_api::AppContext;
use crate::args::BranchArgs;
use crate::output::OutputChannel;

pub async fn execute(ctx: &AppContext, args: &BranchArgs, out: &mut OutputChannel) -> Result<()> {
    // 1. Parse and validate args (fail fast — no side effects yet)
    let branch_name = args.name.as_deref()
        .ok_or_else(|| anyhow::anyhow!("Branch name required"))?;

    // 2. Call ext-api — all business logic lives there
    let result = ext_api::create_branch(ctx, branch_name, args.from.as_deref()).await?;

    // 3. Format output — nothing else
    if let Some(out) = out.for_human() {
        writeln!(out, "✓ Created branch '{}'", result.branch_name)?;
        writeln!(out, "  Based on: {}", result.created_from)?;
    }
    if let Some(out) = out.for_shell() {
        writeln!(out, "{}", result.branch_name)?;
    }
    if let Some(out) = out.for_json() {
        out.write_value(&result)?;
    }

    Ok(())
}
```

---

## Phase 1 Build Order

Build in this order. Each week's output is a working, tested increment. Do not start a week until the prior week's tests pass.

### Week 1–2: Foundation

**Goal:** `ext init` and `ext status` work end-to-end.

```text
ext-error crate
  └── All EtabsError variants for Phase 1 (including OneDriveConflict, OneDriveNotConfigured)

ext-db crate
  └── config.toml + config.local.toml read/write with resolution order
  └── state.json read/write

ext-core/sidecar
  └── SidecarClient: spawn etab-cli, read stdout JSON, forward stderr
  └── locate(): config → env var → PATH

Sidecar (C# etab-cli):
  └── validate --file should not be Required
  └── get-status (running, PID, open file, lock state, analyzed)
  └── open-model --file [--hidden]
  └── close-model [--save|--no-save]
  └── unlock-model --file

ext-api:
  └── init(): OneDrive detection, folder creation, git init, config write
  └── status(): state detection (mtime, PID, MISSING check)

ext CLI:
  └── ext init (with OneDrive warning + prompt)
  └── ext status (human + json output)
  └── OutputChannel implementation
  └── AppContext construction from project path
```

**Tests:** Snapshot tests for `ext status` in each of: UNTRACKED, CLEAN, MODIFIED, MISSING states.

---

### Week 3–4: Version Control Core

**Goal:** Full branch/switch/checkout/stash/commit cycle works without analysis.

```text
ext-core:
  └── version/: commit(), list(), show(), manifest.json write
  └── branch/: create(), list(), delete(), copy.rs (atomic copy + disk check)
  └── switch/: decision tree — ETABS check, departure warn, arrival report
  └── checkout/: single-branch + cross-branch, MODIFIED prompt (c/s/d/x)
  └── stash/: save, pop, drop, list (one slot per branch)

ext-api:
  └── commit_version() — without --analyze
  └── log(), show()
  └── create_branch(), list_branches(), delete_branch()
  └── switch_branch(), switch_and_create()
  └── checkout()
  └── stash_save(), stash_pop(), stash_drop(), stash_list()

VCS:
  └── git init, .gitignore write, git config (core.autocrlf false)
  └── git_ops.rs: git subprocess for add/commit/branch/checkout
  └── gix_ops.rs: gix for log/diff/blob reads
  └── Internal commit prefix filtering in ext log

ext CLI:
  └── ext commit, ext log, ext show
  └── ext branch (list/create/delete)
  └── ext switch, ext switch -c
  └── ext checkout (with prompt: c/s/d/x, --force flag)
  └── ext stash (save/list/pop/drop)
```

**Tests:**

- State machine transition tests for every row in `workflow.md §15` permission matrix.
- Snapshot tests for all output formats.
- Integration test for full cycle: `init → commit → branch → switch → checkout → stash`.

---

### Week 5–6: State Machine + ETABS Commands

**Goal:** Full ETABS lifecycle: open, work, close, recover, unlock. All 9 states exercised.

```text
ext-core/state:
  └── Full 9-state machine with mtime detection
  └── State detection algorithm (workflow.md §State Detection) — exact order
  └── ORPHANED detection (PID alive check)
  └── MISSING detection (file existence)

ext-api:
  └── etabs_open(), etabs_close(), etabs_status(), etabs_validate()
  └── etabs_unlock() — sidecar unlock-model
  └── etabs_recover() — ORPHANED recovery with [k/r] prompt
  └── diff() — raw git diff passthrough on E2K files

ext CLI:
  └── ext etabs open/close/status/validate/unlock/recover
  └── ext diff
  └── All state guards wired (permission matrix enforced)
  └── ext etabs recover prompts ([k] keep / [r] restore)
```

**Tests:**

- Test every blocked state in permission matrix.
- ORPHANED recovery both paths.
- Snapshot test for `ext etabs status` JSON output.

---

### Week 7–8: Analysis Pipeline

**Goal:** `ext commit --analyze` and `ext analyze` work end-to-end with Parquet extraction.

```text
Sidecar (C#):
  └── Add Parquet.Net dependency
  └── extract-results: all 7 Parquet schemas
      (modal, base_reactions, story_forces, story_drifts,
       joint_displacements, wall_pier_forces, shell_stresses)
  └── extract-materials: takeoff.parquet
  └── save-snapshot --with-results: composite command
      (open hidden → e2k → materials → run-analysis → extract-results → close)

ext-core:
  └── analyze/: open snapshot, run, extract, close — working file untouched
  └── Polars reads for all 7 result tables + materials
  └── calculations/: modal, drifts, reactions, forces, dcr, materials modules

ext-api:
  └── commit_version() with --analyze option
      (runs on vN/model.edb snapshot, NOT working file)
  └── analyze() standalone

ext CLI:
  └── ext commit --analyze (with progress output)
  └── ext analyze <version>
```

**Critical:** Analysis always runs on the committed snapshot `vN/model.edb`, never on `working/model.edb`. Enforce this in `ext-api`, not just in the CLI.

**Tests:**

- Integration test for full `commit --analyze` cycle.
- Verify working file is unchanged after `--analyze`.
- Test that Parquet files are written to correct paths.

---

### Week 9–10: Reports + Remote + AI (Phase 1)

**Goal:** PDF reports auto-saved to OneDrive. Project shareable across machines. Phase 1 AI agent online (read + basic write tools).

```text
Reports:
  └── Validate: generate hello-world PDF on Windows — DO THIS FIRST on Day 1
  └── TypstWorld implementation: Windows font loading + Liberation Sans bundled
  └── generators/analysis.rs: modal + drifts + base shear + code checks
  └── generators/bom.rs: material quantities + cost summary
  └── generators/comparison.rs: E2K diff summary + result deltas + material delta
  └── ext report analysis/bom/comparison commands
  └── Output path: paths.reportsDir → auto-naming → --out override
  └── Auto-naming: "{branch}-{version}-{type}.pdf"

Remote (OneDrive):
  └── remote/bundle.rs: git bundle create/unbundle wrappers
  └── remote/transfer.rs: .edb file copy to/from OneDrive with progress bar
  └── remote/conflict.rs: version ID conflict detection + rename prompt
  └── remote/project_json.rs: project.json read/write/merge
  └── ext push: bundle + edb copy + conflict check + project.json update
  └── ext pull: bundle fetch + edb copy
  └── ext clone: wizard (author/email/paths prompts) + full restore
  └── ext remote status: local vs OneDrive diff

AI (Phase 1):
  └── ext-agent-llm crate: LlmClient trait + Claude backend
  └── ext-agent crate: read tools + write tools + confirmation gate + system prompt
  └── ext chat CLI subcommand (interactive REPL, human mode only)
  └── Ollama config support (backend stubbed, activated Phase 2)
```

**Tests:**

- Snapshot tests for all three report types (mock Parquet data).
- Push/pull round-trip test with temp OneDrive folder.
- Clone wizard test with pre-populated OneDrive folder.
- Conflict detection test.
- Basic agent tool tests (see Testing section).

---

## Phase 2 Build Order (after Phase 1 ships)

After Phase 1, extend AI capabilities and streaming.

```text
ext-agent-llm:
  └── Ollama backend
  └── OpenAI-compatible backend via async-openai

Streaming:
  └── chat_streaming() in LlmClient trait
  └── CLI spinner + Tauri events

Unlock deferred tools:
  └── analyze_version, generate_report, etabs_unlock from agent

Tauri:
  └── Streaming chat panel (agent-token events, confirm dialog)

Agent UX:
  └── suggestion.rs in ext-agent (post-tool suggestions)
  └── ext chat --resume / --clear-history (session persistence in ext-db)
  └── ext chat --non-interactive (stdin/stdout scripting mode)
```

---

## Performance Guidelines

- **Large `.edb` files** (100MB+) require progress bars — use `indicatif` in **human mode only**.
- **Never block the async runtime** on file I/O — use `tokio::fs`.
- **Sidecar operations are long-running** — forward `stderr` progress lines live as they arrive; do not buffer.
- **`ext status` must be fast** — mtime check only, no hashing, no sidecar calls unless `--verbose`.
- **`ext log` must be fast** — use `gix` for reads, never git subprocess.
- **`ext chat` system prompt injection** calls `ext-api::status()` on every turn — this must remain fast (mtime-only, same as `ext status`).

---

## Shell Completions

Generate after any command surface change:

```bash
ext completions bash > ~/.bash_completion.d/ext
ext completions zsh  > ~/.zsh/completions/_ext
ext completions fish > ~/.config/fish/completions/ext.fish
```

---

## Linting and Formatting

Must always pass before merging:

```bash
cargo fmt --check --all
cargo clippy --all-targets --fix --allow-dirty
```

Additional rules:

- No `unwrap()` or `expect()` in command paths — use `?` with `anyhow::Context`.
- Functions under 100 lines.
- Public APIs documented with examples.
- All `EtabsError` variants must have at least one test exercising them.

---

## Pre-Submit Checklist

### Before submitting any CLI change:

- [ ] Command follows the git-mimic naming convention.
- [ ] Handler is under 20 lines (business logic is in `ext-api`).
- [ ] All output goes through `OutputChannel`.
- [ ] JSON output is stable (add fields only, never remove/rename).
- [ ] State machine guard is in place (workflow.md §15 permission matrix).
- [ ] Tests include snapshot assertions for all three output modes.
- [ ] `--help` text is accurate and complete.
- [ ] Works non-interactively with `--force` where applicable.
- [ ] Error messages follow the `✗ / ⚠ / ✓` standard.
- [ ] No `unwrap()` in command paths.
- [ ] No git internals exposed in user-facing output.
- [ ] No LFS, no direct ETABS COM calls from Rust.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy` passes.
- [ ] Documentation updated (`reference.md`, `examples.md`, and `ai.md` if applicable).
- [ ] Shell completions regenerated if command surface changed.

### Before submitting any agent tool change:

- [ ] Tool calls `ext-api` only — never `ext-core` or `ext-db` directly.
- [ ] Write tool is in `WRITE_TOOLS` constant (goes through confirmation gate).
- [ ] No raw `.edb` bytes or full Parquet data in LLM messages.
- [ ] No API key or secret in any log/trace output.
- [ ] Tool has a `MockLlmClient` test covering the blocked-state case.
- [ ] Tool description is clear enough for the LLM to use it correctly.
- [ ] `ai.md` tool surface table updated.

