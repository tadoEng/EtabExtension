
# Etab Extension CLI – Guidelines

This document defines conventions and best practices for the `ext` CLI crate.  
The CLI is a **first-class frontend**, equal in importance to the desktop (Tauri) app.

---

## API usage

- **Never depend on UI crates** (`ext-tauri`) from the CLI.
- Prefer calling **`ext-api`** for application logic and workflows.
- Use **`ext-core`** for pure domain logic (no I/O, no side effects).
- Use **`ext-db`** directly only for:
  - migrations  
  - diagnostics  
  - maintenance commands  

### Rule of thumb

| Layer    | Allowed in CLI |
|----------|----------------|
| ext-core | ✅ yes         |
| ext-api  | ✅ yes         |
| ext-db   | ⚠️ sparingly   |
| ext-tauri| ❌ never       |

---

## Output

All user-facing output must go through an **output abstraction**, not directly to
`stdout` or `stderr`.

Commands receive:

```rust
out: &mut OutputChannel
```

### Output modes

**Human-readable**

```rust
if let Some(out) = out.for_human() {
    writeln!(out, "{message}")?;
}
```

**Shell-friendly**

```rust
if let Some(out) = out.for_shell() {
    writeln!(out, "{value}")?;
}
```

**JSON / machine-readable**

```rust
if let Some(out) = out.for_json() {
    out.write_value(json_value)?;
}
```

### Rules

- Never mix output formats in a single command.
- JSON output must be stable and version-tolerant.
- Errors go to `stderr`; structured results go to `stdout`.

### Context & determinism

- **Do not implicitly discover state.**
  - Repositories, projects, or workspaces must be passed explicitly.
  - Do not scan the filesystem unless the command is explicitly about discovery.
- **Avoid implicit global state:**
  - ❌ `std::env::current_dir()`
  - ❌ `std::time::SystemTime::now()`
- Instead:
  - Pass the current time as an argument.
  - Pass resolved paths or repository handles explicitly.
  - Make commands deterministic and testable.

This ensures the CLI is:

- test-friendly  
- script-friendly  
- CI-friendly  

---

## Testing

### Snapshot testing

Use **snapbox** for CLI assertions:

```rust
use snapbox::str;

cmd.assert()
   .stdout_eq(str![["expected stdout"]])
   .stderr_eq(str![["expected stderr"]]);
```

### Updating snapshots

```bash
SNAPSHOTS=overwrite cargo test -p ext
```

### Color / formatted output

When ANSI or formatted output is involved:

```rust
.stdout_eq(snapbox::file![
  "snapshots/<test-name>/<invocation>.stdout.term.svg"
])
```

Update with:

```bash
SNAPSHOTS=overwrite cargo test -p ext
```

---

## Linting & formatting

These must always pass before merging:

```bash
cargo fmt --check --all
cargo clippy --all-targets --fix --allow-dirty
```

Guidelines:

- Prefer clarity over cleverness.
- Avoid `unwrap()` in command paths.
- Use `anyhow::Context` for actionable error messages.

---

## CLI design principles

Commands should be:

- composable  
- scriptable  
- idempotent where possible  

Prefer:

```bash
ext project list --json
```

over:

```bash
ext listProjects
```

Avoid interactive prompts unless explicitly requested (`--interactive`).

---

## Skill / automation awareness

When CLI commands, flags, or workflows change:

- Update any CLI capability descriptions used by automation or AI tooling.
- Ensure `--help` output is accurate and complete.
- Keep examples current.

This ensures users, scripts, and future agents remain aligned with the real CLI
surface.

---

## Philosophy

The `ext` CLI is **not** a second-class interface.  
It is a **first-class frontend**, equal to the desktop app.

If it is clean in the CLI, it will be:

- easier to test  
- easier to automate  
- easier to trust  
