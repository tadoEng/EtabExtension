# ETABS Extension CLI – Development Guidelines

This document defines conventions and best practices for the `ext` CLI crate.  
The CLI is a **first-class frontend**, equal in importance to the desktop (Tauri) app.

---

## Architecture Overview

The ETABS Extension CLI provides version control and workflow management for ETABS structural engineering projects. It operates on a workspace model similar to GitButler but tailored for ETABS files (.edb, .e2k).

### Core Concepts

**Projects**: Root containers for ETABS structural models with version history
**Branches**: Design alternatives (e.g., steel-columns, foundation-redesign)  
**Versions**: Snapshots of the model at specific points in time  
**Working File**: The active .edb file being edited in ETABS  
**E2K Files**: Text-based export format for diff/comparison

---

## API Usage

- **Never depend on UI crates** (`ext-tauri`) from the CLI.
- Prefer calling **`ext-api`** for application logic and workflows.
- Use **`ext-core`** for pure domain logic (no I/O, no side effects).
- Use **`ext-db`** directly only for:
  - migrations  
  - diagnostics  
  - maintenance commands  

### Rule of Thumb

| Layer      | Allowed in CLI | Purpose                                    |
|------------|----------------|--------------------------------------------|
| ext-core   | ✅ yes         | Domain types, business logic               |
| ext-api    | ✅ yes         | Application workflows, orchestration       |
| ext-db     | ⚠️ sparingly   | Direct database access (maintenance only)  |
| ext-tauri  | ❌ never       | UI-specific code, window management        |

---

## Output

All user-facing output must go through an **output abstraction**, not directly to
`stdout` or `stderr`.

Commands receive:

```rust
out: &mut OutputChannel
```

### Output Modes

**Human-readable** (default for interactive terminals)

```rust
if let Some(out) = out.for_human() {
    writeln!(out, "Created branch '{}' from main/v3", branch_name)?;
}
```

**Shell-friendly** (for scripting, `--shell` flag)

```rust
if let Some(out) = out.for_shell() {
    writeln!(out, "{}", branch_id)?;  // Just the value, no decoration
}
```

**JSON / machine-readable** (`--json` flag)

```rust
if let Some(out) = out.for_json() {
    out.write_value(CreateBranchResponse {
        branch_name: "steel-columns",
        branch_id: "bu",
        parent_branch: "main",
        parent_version: "v3",
    })?;
}
```

### Rules

- Never mix output formats in a single command.
- JSON output must be stable and version-tolerant.
- Errors go to `stderr`; structured results go to `stdout`.
- Progress indicators only appear in human mode (use `progress_channel()`).

### Examples

**Good - Format-aware output:**

```rust
pub fn execute(out: &mut OutputChannel, project: &Project) -> Result<()> {
    if let Some(out) = out.for_human() {
        writeln!(out, "🏗️  Project: {}", project.name)?;
        writeln!(out, "📍 Path: {}", project.path)?;
    }
    
    if let Some(out) = out.for_shell() {
        writeln!(out, "{}", project.path)?;
    }
    
    if let Some(out) = out.for_json() {
        out.write_value(project)?;
    }
    
    Ok(())
}
```

**Bad - Direct stdout/stderr:**

```rust
// ❌ Never do this
println!("Created branch: {}", name);
eprintln!("Error: {}", error);
```

---

## Context & Determinism

- **Do not implicitly discover state.**
  - Projects must be passed explicitly via `--project-path` or current directory.
  - Do not scan filesystem unless command is explicitly about discovery.
- **Avoid implicit global state:**
  - ❌ `std::env::current_dir()` - pass path explicitly
  - ❌ `std::time::SystemTime::now()` - pass time as argument
  - ❌ `std::env::var()` - pass config explicitly
- **Make commands deterministic and testable:**
  - Same input → Same output
  - No hidden side effects
  - Reproducible in CI/tests

### Example: Good vs Bad

**Bad:**

```rust
// ❌ Implicit current directory
pub fn open_project() -> Result<Project> {
    let cwd = env::current_dir()?;
    Project::open(cwd)
}
```

**Good:**

```rust
// ✅ Explicit path parameter
pub fn open_project(path: &Path) -> Result<Project> {
    Project::open(path)
}
```

This ensures the CLI is:
- test-friendly  
- script-friendly  
- CI-friendly  

---

## Testing

### Snapshot Testing

Use **snapbox** for CLI assertions:

```rust
use snapbox::str;

#[test]
fn test_branch_list() {
    let mut cmd = Command::cargo_bin("ext").unwrap();
    cmd.arg("branch")
        .arg("list")
        .arg("--json");
    
    cmd.assert()
       .success()
       .stdout_eq(str![[r#"
{
  "branches": [
    {
      "name": "main",
      "latest_version": "v3",
      "versions": 3
    }
  ]
}
"#]]);
}
```

### Updating Snapshots

```bash
SNAPSHOTS=overwrite cargo test -p ext
```

### Testing with Color/Formatted Output

When ANSI or formatted output is involved:

```rust
cmd.assert()
   .stdout_eq(snapbox::file![
     "snapshots/branch_list/default.stdout.term.svg"
   ])
```

Update with:

```bash
SNAPSHOTS=overwrite cargo test -p ext
```

### Integration Tests

Test actual ETABS operations in isolated environments:

```rust
#[test]
fn test_create_branch_integration() {
    let temp = TempDir::new().unwrap();
    let project = Project::init(temp.path(), "TestProject").unwrap();
    
    let result = create_branch(
        &project,
        "steel-columns",
        "main",
        "v3",
        None,
    );
    
    assert!(result.is_ok());
    assert!(project.branches().contains_key("steel-columns"));
}
```

---

## CLI Design Principles

Commands should be:

- **Composable**: Output of one command feeds into another
- **Scriptable**: Easy to use in shell scripts and automation
- **Idempotent**: Running twice produces same result (where possible)
- **Self-documenting**: `--help` is comprehensive and accurate

### Command Structure

Prefer noun-verb structure:

```bash
# ✅ Good
ext branch new steel-columns --from main/v3
ext commit "Updated columns"
ext init "MyProject"

# ❌ Avoid
ext createBranch steel-columns
ext saveCurrentVersion
```

### Flags and Options

- Use long flags for clarity: `--from-version` not `-f`
- Provide short aliases for common flags: `-m` for `--message`
- Boolean flags should not require values: `--force` not `--force=true`
- Required arguments should be positional when unambiguous

### Interactive vs Non-Interactive

Avoid interactive prompts unless explicitly requested:

```bash
# ✅ Non-interactive (default)
ext branch delete feature-x --confirm

# ✅ Interactive (explicit)
ext branch delete feature-x --interactive

# ❌ Always prompts
ext branch delete feature-x
# Delete branch 'feature-x'? [y/N]:  # <- Blocks automation
```

---

## ETABS-Specific Considerations

### File Handling

- **Always check if ETABS is running** before modifying .edb files
- **Lock files** when ETABS has them open
- **Generate E2K** automatically on save (unless `--no-e2k`)
- **Preserve ETABS metadata** in project state

### Version Control Integration

- Use Git internally but hide Git complexity from users
- Store .edb files using Git LFS or custom storage
- Keep E2K files in Git for human-readable diffs
- Track analysis results as metadata, not binary files

### Performance

- Large .edb files (100MB+) require streaming
- Cache E2K generation results
- Use incremental diffs when possible
- Show progress for long operations (analysis, E2K generation)

---

## Error Handling

### Error Types

Use domain-specific error types from `ext-error`:

```rust
use ext_error::AppError;

pub fn validate_etabs_file(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Err(AppError::FileSystem(
            format!("ETABS file not found: {}", path.display())
        ));
    }
    
    if !path.extension().map_or(false, |e| e == "edb") {
        return Err(AppError::Validation(
            "Expected .edb file".to_string()
        ));
    }
    
    Ok(())
}
```

### User-Facing Errors

- Be specific and actionable
- Suggest fixes when possible
- Show relevant context

**Good:**

```
Error: ETABS file is currently open in ETABS
→ Close ETABS and try again, or use --force to override

Path: D:\Projects\HighRise\main\working\model.edb
PID:  12345
```

**Bad:**

```
Error: File locked
```

---

## Linting & Formatting

These must always pass before merging:

```bash
cargo fmt --check --all
cargo clippy --all-targets --fix --allow-dirty
```

### Guidelines

- Prefer clarity over cleverness
- Avoid `unwrap()` in command paths - use `?` with proper error context
- Use `anyhow::Context` for actionable error messages
- Document public APIs with examples
- Keep functions small and focused (< 100 lines)

### Example with Context

**Bad:**

```rust
let project = Project::open(path).unwrap();  // ❌ Will panic
```

**Good:**

```rust
use anyhow::Context;

let project = Project::open(path)
    .with_context(|| format!("Failed to open project at {}", path.display()))?;
```

---

## CLI Command Categories

### Project Management

```bash
ext init <name>               # Initialize new project
ext open <path>               # Open existing project
ext status                    # Show project state
ext config                    # View/edit configuration
```

### Branch Operations

```bash
ext branch                    # List branches
ext branch new <name>         # Create new branch
ext branch switch <name>      # Switch to branch
ext branch delete <name>      # Delete branch
ext branch merge <name>       # Merge branch (default → main)
```

### Version Control

```bash
ext commit <message>          # Commit new version (save alias exists)
ext log [branch]              # List commit history
ext show commit <id>          # Show commit details
ext restore <id>              # Restore to commit
```

### ETABS Integration

```bash
ext etabs open [version]      # Open in ETABS
ext etabs close [--save]      # Close ETABS
ext etabs status              # Check ETABS status
ext etabs validate <file>     # Validate ETABS file
ext etabs export e2k <file>   # Generate E2K file
```

### Comparison & Analysis

```bash
ext diff <v1> <v2>            # Fast diff (E2K / geometry)
ext compare <v1> <v2>         # Deep analytical comparison
ext analyze <version>         # Run ETABS analysis
```

### Report

```bash
ext report generate <type>    # Generate report
ext report list               # List reports
ext report template <type>    # Edit report template
```

---

## Configuration

### Config Files

Configuration is stored in:

1. **Project level**: `.etabs-ext/config.toml`
2. **User level**: `~/.config/etabs-ext/config.toml`
3. **System level**: `/etc/etabs-ext/config.toml`

Priority: Project > User > System

### Example Config

```toml
[project]
name = "HighRise Tower"
default_branch = "main"

[etabs]
executable = "C:\\Program Files\\ETABS 22\\ETABS.exe"
auto_generate_e2k = true
auto_analyze = false

[git]
author = "John Doe"
email = "john@example.com"

[behavior]
auto_save_interval = 300  # seconds
confirm_destructive = true
```

### Environment Variables

- `ETABS_EXT_PROJECT`: Override project path
- `ETABS_EXT_CONFIG`: Override config file location
- `ETABS_EXECUTABLE`: Override ETABS executable path
- `NO_COLOR`: Disable colored output

---

## Skill / Automation Awareness

When CLI commands, flags, or workflows change:

- Update capability descriptions used by automation or AI tooling
- Ensure `--help` output is accurate and complete
- Keep examples current in documentation
- Update integration tests
- Regenerate shell completions

### Shell Completions

Generate completions for common shells:

```bash
ext completions bash > ~/.bash_completion.d/ext
ext completions zsh > ~/.zsh/completions/_ext
ext completions fish > ~/.config/fish/completions/ext.fish
```

---

## Philosophy

The `ext` CLI is **not** a second-class interface.  
It is a **first-class frontend**, equal to the desktop app.

### Why CLI-First Matters

1. **Scriptability**: Engineers can automate workflows
2. **CI/CD Integration**: Version control in build pipelines
3. **Remote Work**: Manage projects over SSH
4. **Testability**: Easier to test than GUI
5. **Power Users**: Keyboard-driven workflows

### CLI Advantages Over GUI

- Faster for repetitive tasks
- Composable with other tools
- Version-controllable workflows (scripts)
- Accessible over SSH/remote connections
- Easier to document and share

If it is clean in the CLI, it will be:

- easier to test  
- easier to automate  
- easier to trust  
- easier to extend to GUI

---

## Development Workflow

### Adding a New Command

1. **Define types** in `ext-core`
2. **Add API method** in `ext-api`
3. **Create command module** in `crates/ext/src/commands/`
4. **Add args struct** in `crates/ext/src/args/`
5. **Register in main** command enum
6. **Write tests** with snapshots
7. **Update documentation**
8. **Generate completions**

### Example Command Structure

```rust
// crates/ext/src/commands/branch.rs

use anyhow::Result;
use ext_api::AppState;
use ext_core::CreateBranchRequest;

use crate::utils::OutputChannel;

pub async fn create(
    state: &AppState,
    request: CreateBranchRequest,
    out: &mut OutputChannel,
) -> Result<()> {
    // 1. Validate input
    if request.branch_name.is_empty() {
        anyhow::bail!("Branch name cannot be empty");
    }
    
    // 2. Call API
    let result = state.create_branch(request).await?;
    
    // 3. Output results
    if let Some(out) = out.for_human() {
        writeln!(out, "✓ Created branch '{}'", result.branch_name)?;
        writeln!(out, "  Based on: {}/{}", 
                 result.parent_branch, result.parent_version)?;
    }
    
    if let Some(out) = out.for_json() {
        out.write_value(&result)?;
    }
    
    Ok(())
}
```

---

## Metrics & Telemetry

The CLI collects anonymous usage metrics (opt-out) to improve the tool:

- Command usage frequency
- Error rates
- Performance metrics
- Feature adoption

### Privacy

- No personal data collected
- No file contents or project names
- No IP addresses or location data
- All data anonymized

### Opt-Out

```bash
ext config set telemetry.enabled false
```

Or set environment variable:

```bash
export ETABS_EXT_TELEMETRY=0
```

---

## Summary Checklist

Before submitting CLI changes:

- [ ] Commands follow noun-verb structure
- [ ] All output goes through `OutputChannel`
- [ ] JSON output is stable and documented
- [ ] Tests include snapshot assertions
- [ ] Help text is comprehensive (`--help`)
- [ ] Works non-interactively (for CI/CD)
- [ ] Error messages are actionable
- [ ] No implicit state discovery
- [ ] Code formatted and linted
- [ ] Documentation updated

---

## Resources

- **CLI Testing**: [snapbox documentation](https://docs.rs/snapbox)
- **Argument Parsing**: [clap documentation](https://docs.rs/clap)
- **Error Handling**: [anyhow documentation](https://docs.rs/anyhow)
- **ETABS API**: See `docs/etabs-api.md`
- **Git Integration**: See `docs/git-workflow.md`