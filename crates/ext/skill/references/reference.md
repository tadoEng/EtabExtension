# ETABS Extension CLI Command Reference

Comprehensive reference for all `ext` commands.

## Contents

- [Inspection](#inspection-understanding-state) - `status`, `show`, `log`, `diff`
- [Project Management](#project-management) - `init`, `open`
- [Branching](#branching) - `branch new`, `branch switch`, `branch merge`, `branch delete`
- [Version Control](#version-control) - `commit`, `restore`, `log`, `show`
- [ETABS Integration](#etabs-integration) - `etabs open`, `etabs close`, `etabs status`, `etabs validate`, `etabs export`
- [Comparison](#comparison) - `diff`, `compare`, `analyze`
- [Reports](#reports) - `report generate`, `report template`, `report list`
- [Configuration](#configuration) - `config get`, `config set`
- [Utilities](#utilities) - `completions`, `update`, `help`
- [Global Options](#global-options)

---

## Inspection (Understanding State)

### `ext status`

Overview of project state - this is your entry point.

```bash
ext status              # Human-readable view
ext status --json       # Structured output for parsing
ext status --verbose    # Detailed information
```

Shows:

- Current project and branch
- Uncommitted changes in working file
- ETABS running status
- Recent versions
- Branch relationships

**Example output:**

```
Project: HighRise Tower
Path: D:\Projects\HighRise
Branch: main (3 versions)

Working File:
  Status: Modified
  Based on: v3
  ETABS: Not running

Recent Versions:
  v3  Final review of main design         2d ago  ✓ analyzed
  v2  Updated load combinations            5d ago  ✓ analyzed
  v1  Initial structural design            2w ago  ✓ analyzed
```

### `ext show <target>`

Details about a project, branch, or version.

```bash
ext show project        # Show project details
ext show branch main    # Show branch details
ext show version v3     # Show version details
ext show --json         # JSON output
```

### `ext diff [v1] [v2]`

Display differences between versions.

```bash
ext diff                # Diff working file vs latest version
ext diff v2 v3          # Diff two specific versions
ext diff --e2k          # Show E2K file differences
ext diff --geometry     # Show 3D geometry changes
ext diff --json         # JSON output with change details
```

---

## Project Management

### `ext init <name>`

Initialize a new ETABS project with version control.

```bash
ext init "HighRise Tower"
ext init "HighRise Tower" --path D:\Projects\HighRise
ext init "HighRise Tower" --edb-file D:\model.edb  # Import existing
```

Creates:

- `.etabs-ext/` directory structure
- Git repository (internal)
- `main` branch
- Initial configuration

### `ext project open <path>`

Open an existing ETABS project.

```bash
ext project open .                    # Open current directory
ext project open D:\Projects\HighRise
ext project open --json               # JSON output
```

### `ext project status`

Show current project state.

```bash
ext project status
ext project status --json
ext project status --verbose
```

Alias for `ext status`.

### `ext project config`

View or edit project configuration.

```bash
ext project config              # Show all config
ext project config etabs        # Show ETABS settings
ext project config --edit       # Open editor
ext project config --json       # JSON output
```

### `ext project list`

List all known projects.

```bash
ext project list
ext project list --json
ext project list --recent       # Sort by last accessed
```

---

## Branching

### `ext branch list`

List all branches (default when no subcommand).

```bash
ext branch              # List branches
ext branch list
ext branch list --json
ext branch list --verbose
```

**Example output:**

```
Branches:
  main                3 versions  (active)
  steel-columns       2 versions  from main/v2
  foundation-redesign 1 version   from main/v2
```

### `ext branch create <name>`

Create a new branch.

```bash
# Create branch from current version
ext branch create steel-columns

# Create from specific version
ext branch create steel-columns --from main/v3

# With description
ext branch create steel-columns --from main/v3 \
  --description "Explore steel column alternative"
```

Creates a new design alternative starting from a specific version.

### `ext branch switch <name>`

Switch to a different branch.

```bash
ext branch switch steel-columns
ext branch switch main
```

**Behavior:**

- Closes ETABS if running (with save prompt)
- Updates working file to branch's latest version
- Updates project state

### `ext branch delete <name>`

Delete a branch.

```bash
ext branch delete steel-columns
ext branch delete steel-columns --force  # Skip confirmation
```

**Safety:**

- Cannot delete `main` branch
- Cannot delete current branch (switch first)
- Prompts for confirmation unless `--force`

### `ext branch show <name>`

Show detailed information about a branch.

```bash
ext branch show main
ext branch show steel-columns --json
```

Shows:

- All versions in branch
- Parent branch relationship
- Working file status
- Creation date and description

### `ext branch merge <name>`

Merge a branch into main.

```bash
ext branch merge steel-columns
ext branch merge steel-columns --strategy latest  # Use latest version only
ext branch merge steel-columns --squash            # Combine all versions
```

---

## Version Control

### `ext version save <message>`

Save current working file as a new version.

```bash
ext version save "Updated column sections"
ext version save "Updated column sections" --no-e2k  # Skip E2K generation
ext version save "Updated column sections" --analyze # Auto-analyze in ETABS
```

**Behavior:**

- Creates new version from working file
- Generates E2K file (unless `--no-e2k`)
- Commits to Git internally
- Updates version history

### `ext version list [branch]`

List versions in current or specified branch.

```bash
ext version list                # Current branch
ext version list main           # Specific branch
ext version list --all          # All branches
ext version list --json
```

### `ext version show <id>`

Show detailed information about a version.

```bash
ext version show v3
ext version show main/v3        # Fully qualified
ext version show v3 --json
ext version show v3 --files     # Show changed files
```

Shows:

- Commit message
- Author and timestamp
- File sizes (.edb, .e2k)
- Analysis status
- Changes from previous version

### `ext version restore <id>`

Restore working file to a specific version.

```bash
ext version restore v3
ext version restore main/v2
ext version restore v3 --open   # Open in ETABS after restore
```

**Behavior:**

- Overwrites current working file
- Prompts if unsaved changes exist
- Can optionally open in ETABS

### `ext version diff <v1> <v2>`

Compare two versions.

```bash
ext version diff v2 v3
ext version diff main/v2 steel-columns/v1
ext version diff v2 v3 --e2k       # Show E2K differences
ext version diff v2 v3 --geometry  # Show 3D changes
ext version diff v2 v3 --json
```

---

## ETABS Integration

### `ext etabs open [version]`

Open a file in ETABS.

```bash
ext etabs open                  # Open working file
ext etabs open v3               # Open specific version
ext etabs open main/v2          # Fully qualified version
```

**Behavior:**

- Launches ETABS if not running
- Opens specified .edb file
- Locks file (prevents concurrent edits)
- Updates project state

### `ext etabs close`

Close ETABS application.

```bash
ext etabs close                 # Prompt to save
ext etabs close --save          # Save and close
ext etabs close --no-save       # Discard changes
```

### `ext etabs status`

Check ETABS application status.

```bash
ext etabs status
ext etabs status --json
```

Shows:

- Running/not running
- Open file (if any)
- Process ID
- Version
- Can save (file lock status)

### `ext etabs validate <file>`

Validate an ETABS file.

```bash
ext etabs validate model.edb
ext etabs validate D:\Projects\HighRise\main\working\model.edb
ext etabs validate model.edb --json
```

Checks:

- File exists and readable
- Valid ETABS format
- Analysis status
- ETABS version compatibility

### `ext etabs generate-e2k <file>`

Generate E2K file from EDB.

```bash
ext etabs generate-e2k model.edb
ext etabs generate-e2k model.edb --output model.e2k
ext etabs generate-e2k model.edb --overwrite
ext etabs generate-e2k model.edb --json
```

**Behavior:**

- Launches ETABS (hidden)
- Opens .edb file
- Exports to .e2k format
- Returns file path and metadata

---

## Comparison

### `ext diff <v1> <v2>`

Show differences between versions.

```bash
ext diff v2 v3
ext diff v2 v3 --type e2k       # E2K file diff (default)
ext diff v2 v3 --type geometry  # 3D geometry changes
ext diff v2 v3 --type both      # Both E2K and geometry
ext diff v2 v3 --json
```

**E2K Diff Output:**

```
E2K Changes:
  Added:    12 lines
  Removed:  5 lines
  Modified: 18 lines

Changes by Category:
  Frame Section:  3 modifications
  Load Pattern:   2 additions
  Load Combo:     1 modification
  Joint:          2 additions
```

**Geometry Diff Output:**

```
3D Geometry Changes:
  Members Added:    5 (C45, C46, B78, B79, B80)
  Members Removed:  2 (B45, B67)
  Members Modified: 8
  
Total Changes: 15
```

### `ext compare <v1> <v2>`

Detailed comparison with analysis.

```bash
ext compare v2 v3
ext compare v2 v3 --report      # Generate comparison report
ext compare v2 v3 --json
```

More comprehensive than `ext diff`. Shows:

- E2K differences
- Geometry changes
- Material quantity differences
- Analysis results comparison
- Performance metrics

### `ext analyze <version>`

Analyze structural model.

```bash
ext analyze v3
ext analyze --working           # Analyze working file
ext analyze v3 --results        # Show analysis results
ext analyze v3 --json
```

Triggers ETABS analysis and stores results.

---

## Reports

### `ext report generate <type>`

Generate a report.

```bash
ext report generate comparison --v1 v2 --v2 v3
ext report generate analysis --version v3
ext report generate bom --version v3
ext report generate progress --branch main
```

**Report Types:**

- `comparison` - Compare two versions
- `analysis` - Analysis results summary
- `bom` - Bill of materials
- `progress` - Design progress timeline

**Options:**

```bash
ext report generate comparison --v1 v2 --v2 v3 \
  --format pdf \
  --output comparison-report.pdf \
  --include-images
```

### `ext report list`

List generated reports.

```bash
ext report list
ext report list --json
```

### `ext report template <type>`

View or edit report template.

```bash
ext report template comparison
ext report template comparison --edit
ext report template comparison --reset   # Reset to default
```

---

## Configuration

### `ext config get <key>`

Get configuration value.

```bash
ext config get etabs.executable
ext config get project.default_branch
ext config get --json
```

### `ext config set <key> <value>`

Set configuration value.

```bash
ext config set etabs.executable "C:\Program Files\ETABS 22\ETABS.exe"
ext config set etabs.auto_generate_e2k true
ext config set git.author "John Doe"
```

### `ext config list`

List all configuration.

```bash
ext config list
ext config list etabs           # Show ETABS config only
ext config list --json
ext config list --defaults      # Show defaults
```

### `ext config edit`

Open configuration file in editor.

```bash
ext config edit                 # Open in default editor
ext config edit --editor code   # Open in VS Code
```

### `ext config reset`

Reset configuration to defaults.

```bash
ext config reset                # Reset all
ext config reset etabs          # Reset ETABS config only
ext config reset --confirm      # Skip confirmation
```

---

## Utilities

### `ext completions <shell>`

Generate shell completions.

```bash
ext completions bash > ~/.bash_completion.d/ext
ext completions zsh > ~/.zsh/completions/_ext
ext completions fish > ~/.config/fish/completions/ext.fish
ext completions powershell > $PROFILE\...\ext.ps1
```

**Supported shells:**

- bash
- zsh
- fish
- powershell
- elvish

### `ext update`

Check for and install updates.

```bash
ext update              # Check for updates
ext update --install    # Install latest version
ext update --channel stable   # Update to stable channel
ext update --channel beta     # Update to beta channel
```

### `ext help [command]`

Show help information.

```bash
ext help                # General help
ext help branch         # Help for branch commands
ext help branch create  # Help for specific command
```

### `ext version`

Show CLI version information.

```bash
ext version
ext version --json
```

---

## Global Options

Available on most commands:

- `-j, --json` - Output in JSON format for parsing
- `-q, --quiet` - Suppress non-essential output
- `-v, --verbose` - Show detailed information
- `--project-path <PATH>` - Specify project directory
- `--no-color` - Disable colored output
- `-h, --help` - Show help for command

**Environment Variables:**

- `ETABS_EXT_PROJECT` - Default project path
- `ETABS_EXECUTABLE` - ETABS executable path
- `NO_COLOR` - Disable colored output (set to any value)
- `ETABS_EXT_CONFIG` - Custom config file path

---

## Getting More Help

```bash
ext --help                    # List all commands
ext <subcommand> --help       # Detailed help for specific command
ext <subcommand> <cmd> --help # Help for nested command
```

**Documentation:**

- Full documentation: <https://docs.etabs-ext.com>
- GitHub: <https://github.com/yourorg/etabs-extension>
- Issues: <https://github.com/yourorg/etabs-extension/issues>

---

## Quick Reference

**Most Common Commands:**

```bash
# Start new project
ext project init "MyProject"

# Create branch for alternative design
ext branch create steel-columns --from main/v3

# Save your work
ext version save "Updated column sections"

# Compare designs
ext diff main/v3 steel-columns/v1

# Generate report
ext report generate comparison --v1 main/v3 --v2 steel-columns/v1

# Open in ETABS
ext etabs open
```

**Status Check:**

```bash
ext status                    # Quick overview
ext status --verbose          # Detailed info
ext etabs status              # Check ETABS
```

**Branch Workflow:**

```bash
ext branch create feature     # Create
ext branch switch feature     # Switch
ext version save "message"    # Save work
ext branch merge feature      # Merge back
ext branch delete feature     # Clean up
```
