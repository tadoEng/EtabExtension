# ETABS Extension CLI Key Concepts

Deep dive into the ETABS Extension conceptual model and philosophy.

## Version Control for ETABS Models

### Traditional ETABS Workflow: File-Based Versions

```md
Project Folder
  ├─ HighRise_v1.edb
  ├─ HighRise_v2.edb
  ├─ HighRise_v3_steel.edb
  ├─ HighRise_v3_concrete.edb
  └─ HighRise_FINAL_actually_final_v2.edb
```

**Problems:**

- No clear relationship between versions
- Difficult to compare changes
- Manual file naming and organization
- No change history or commit messages
- Hard to collaborate
- Space inefficient (duplicate data)

### ETABS Extension: Git-Like Version Control

```md
Project: HighRise Tower
  └─ main branch
      ├─ v1: Initial structural layout
      ├─ v2: Added seismic loads
      └─ v3: Updated column sections
  └─ steel-columns branch (from main/v2)
      ├─ v1: Changed to steel W-sections
      └─ v2: Optimized member sizes
  └─ foundation-redesign branch (from main/v3)
      └─ v1: Deep foundations for poor soil
```

**Advantages:**

- Clear version lineage and relationships
- Descriptive commit messages
- Easy comparison between any two versions
- Branch-based design alternatives
- Collaboration-ready
- Efficient storage (Git internally)
- E2K files for human-readable diffs

## Core Concepts

### 1. Projects

A **project** is the root container for an ETABS structural model with version control.

**Anatomy:**

```md
HighRise Tower/
  ├─ .etabs-ext/           # Extension metadata
  │   ├─ config.toml       # Project configuration
  │   ├─ state.json        # Current state
  │   └─ .git/             # Git repository (internal)
  ├─ main/                 # Main branch directory
  │   ├─ working/          # Working files
  │   │   └─ model.edb     # Active working file
  │   └─ versions/         # Version history
  │       ├─ v1/
  │       │   ├─ model.edb
  │       │   └─ model.e2k
  │       ├─ v2/
  │       └─ v3/
  ├─ steel-columns/        # Branch directory
  │   ├─ working/
  │   └─ versions/
  └─ foundation-redesign/
      ├─ working/
      └─ versions/
```

**Initialization:**

```bash
ext init "HighRise Tower"
# Creates directory structure
# Initializes Git repository
# Creates main branch
# Configures defaults
```

### 2. Branches

**Branches** represent design alternatives or parallel work streams.

**Types:**

**Independent Branches** (most common):

```md
main ──┬── steel-columns (different structural system)
       └── concrete-hsc   (different concrete strength)
```

Use for:

- Exploring different structural systems
- Comparing material alternatives
- Client-requested variants
- Value engineering studies

**Dependent Branches** (less common):

```md
main ── foundation-upgrade ── superstructure-adjustment
        (base redesign)       (depends on foundation)
```

Use for:

- Sequential design changes
- When work must build on previous changes
- Incremental major redesigns

**Creating Branches:**

```bash
# Independent branch (default)
ext branch new steel-columns --from main/v3

# Dependent branch (if needed in future)
ext branch new phase2 --from phase1/v2 
```

### 3. Versions

A **version** is a snapshot of the model at a specific point in time.

**Version Anatomy:**

```bash
Version v3:
  - ID: v3
  - Branch: main
  - Message: "Updated column sections per analysis"
  - Timestamp: 2024-02-05 14:30:00
  - Author: John Doe
  - Files:
    - model.edb (45.2 MB)
    - model.e2k (2.3 MB)
  - Analysis: Completed
  - Parent: v2
```

**Version Identification:**

```bash
v3              # Short form (current branch)
main/v3         # Fully qualified (any branch)
bu              # CLI ID (from ext status)
```

**Saving Versions:**

```bash
# Basic save
ext version commit "Updated beam sizes"

# With options
ext version commit "Final design" --analyze --no-e2k
```

### 4. Working File

The **working file** is the active `.edb` file being edited.

**States:**

**Clean:**

```md
Working file matches latest version (v3)
No uncommitted changes
Ready to open in ETABS
```

**Modified:**

```md
Working file has changes since v3
Changes detected: beam sections modified
Ready to save as v4
```

**Open in ETABS:**

```md
Working file locked by ETABS (PID: 12345)
Cannot save version while ETABS is open
Must close ETABS first
```

**Checking Status:**

```bash
ext status

# Output:
# Working File: Modified
#   Based on: v3
#   Changes: Yes
#   ETABS: Not running
```

### 5. E2K Files

**E2K files** are text-based ETABS exports used for human-readable diffs.

**Why E2K?**

- `.edb` files are binary (can't diff)
- `.e2k` files are text (can diff)
- Shows structural elements in plain text
- Enables version comparison
- Git-friendly format

**E2K Content Example:**

```md
$ FRAME SECTION PROPERTIES
Frame=C1 Material=CONC Shape=Rectangular Depth=30 Width=30

$ LOAD PATTERNS
LoadPat=DEAD Type=DEAD SelfWtMult=1.0
LoadPat=LIVE Type=LIVE SelfWtMult=0.0

$ LOAD COMBINATIONS
Combo=COMB1 Type=Linear
  LoadCombItem=DEAD SF=1.4
  LoadCombItem=LIVE SF=1.7
```

**Automatic Generation:**

```bash
# Generated automatically on save (unless --no-e2k)
ext version commit "message"

# Generate manually
ext etabs generate-e2k model.edb
```

### 6. Comparison and Diffs

**Three types of comparisons:**

**1. E2K Diff (default):**
Shows line-by-line changes in E2K format

```bash
ext diff v2 v3

# Output:
# Frame Section:
#   - C1: 30x30 (fc=4000 psi)
#   + C1: 36x36 (fc=5000 psi)
```

**2. Geometry Diff:**
Shows 3D model changes

```bash
ext diff v2 v3 --type geometry

# Output:
# Members Modified: 12 (C1-C12)
# Section changes: 30x30 → 36x36
```

**3. Full Comparison:**
Comprehensive analysis including material quantities

```bash
ext compare v2 v3

# Output includes:
# - E2K differences
# - Geometry changes
# - Material quantities
# - Cost impact
# - Performance metrics
```

### 7. ETABS Integration

The CLI integrates directly with ETABS application.

**ETABS Operations:**

**Opening Files:**

```bash
ext etabs open           # Open working file
ext etabs open v3        # Open specific version
```

Behind the scenes:

1. Check if ETABS is already running
2. Launch ETABS if needed
3. Open specified `.edb` file
4. Lock file to prevent conflicts
5. Update project state

**Closing ETABS:**

```bash
ext etabs close          # Prompt to save
ext etabs close --save   # Save and close
```

**Status Check:**

```bash
ext etabs status

# Output:
# ETABS Status:
#   Running: Yes
#   Version: 22.0.0
#   File: D:\Projects\HighRise\main\working\model.edb
#   PID: 12345
```

**File Locking:**

```md
While ETABS has file open:
  ✓ Can view status
  ✓ Can compare versions
  ✓ Can create branches
  ✗ Cannot save versions
  ✗ Cannot switch branches
  ✗ Cannot restore versions
```

### 8. Git Integration (Internal)

ETABS Extension uses **Git internally** but hides complexity.

**What Git Manages:**

- Version history
- Branch relationships
- E2K files (for diff)
- Project metadata
- Configuration

**What Git Doesn't Manage:**

- `.edb` files (too large, stored separately)
- Temporary files
- ETABS lock files
- Analysis output files

**Git Storage Strategy:**

```md
Git Repository:
  ✓ E2K files (text, diffable)
  ✓ Project metadata (JSON)
  ✓ Configuration files
  ✓ Version manifests

Separate Storage:
  ✓ EDB files (binary, large)
  ✓ Analysis results
  ✓ Generated reports
```

**User Perspective:**
Users **never interact with Git directly**. All Git operations are abstracted:

```bash
ext version commit "message"   # → git commit
ext branch new name       # → git branch
ext diff v1 v2              # → git diff
ext version restore v1      # → git checkout
```

### 9. Configuration Hierarchy

Configuration is resolved in priority order:

**1. Project Config** (highest priority)

```bash
.etabs-ext/config.toml
```

**2. User Config**

```bash
~/.config/etabs-ext/config.toml
```

**3. System Config**

```bash
/etc/etabs-ext/config.toml  # Linux/macOS
C:\ProgramData\etabs-ext\   # Windows
```

**4. Defaults** (lowest priority)
Built-in defaults in code

**Example Configuration:**

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

### 10. CLI IDs: Short Identifiers

Every object gets a short, memorable CLI ID:

**Why CLI IDs?**

- Version IDs can be long: `main/v3`
- Full paths are verbose: `D:\Projects\HighRise\main\versions\v3\`
- UUIDs are unreadable: `550e8400-e29b-41d4-a716-446655440000`

**CLI ID Format:**

```md
Projects:    proj-a, proj-b
Branches:    br-m (main), br-s (steel), br-c (concrete)
Versions:    v1, v2, v3, v4, v5
Files:       file-a, file-b, file-c
```

**Usage in Commands:**

```bash
ext status                    # Shows CLI IDs
ext branch switch br-s        # Use CLI ID
ext diff v2 v3               # Use version IDs
ext version show v3          # Use version ID
```

**Stability:**
CLI IDs are **session-stable** but not permanent:

- Same IDs within a session
- May change between sessions
- Use full names for scripts

### 11. Output Formats

Three output modes for different use cases:

**1. Human Format (default):**

```bash
ext status

# Output:
# Project: HighRise Tower
# Branch: main (3 versions)
# 
# Working File: Modified
#   Based on: v3
#   ETABS: Not running
```

Pretty-printed, colored, formatted for readability.

**2. Shell Format:**

```bash
ext status --shell

# Output:
# HighRise Tower
# main
# v3
# modified
```

Minimal output for parsing in shell scripts.

**3. JSON Format:**

```bash
ext status --json

# Output:
# {
#   "project_name": "HighRise Tower",
#   "current_branch": "main",
#   "working_file": {
#     "status": "modified",
#     "based_on": "v3"
#   }
# }
```

Structured output for AI agent use.

### 12. Workspace Model

Unlike Git's checkout-based model, ETABS Extension uses a **single workspace**:

**Git Workflow:**

```bash
git checkout main           # Switch to main
# All files now show main
git checkout feature        # Switch to feature
# All files now show feature
```

**ETABS Extension Workflow:**

```bash
ext branch switch main      # Load main's working file
# Working file: main/working/model.edb

ext branch switch steel     # Load steel's working file
# Working file: steel/working/model.edb
```

**Key Differences:**

- Each branch has its own `working/` directory
- No file deletion/recreation on switch
- Working files persist independently
- ETABS must be closed before switching

### 13. Analysis Integration

ETABS Extension tracks analysis status:

**Analysis States:**

**Not Analyzed:**

```md
Version v3:
  Status: Not analyzed
  Can run analysis in ETABS
```

**Analyzed:**

```md
Version v3:
  Status: Analyzed ✓
  Results: Available
  Timestamp: 2024-02-05 14:30:00
```

**Outdated:**

```md
Version v3:
  Status: Analyzed (outdated)
  Reason: Model modified since analysis
  Need: Re-analyze
```

**Commands:**

```bash
# Check analysis status
ext version show v3

# Analyze version
ext analyze v3

# Save with analysis
ext version save "message" --analyze
```

### 14. File Validation

Before operations, the CLI validates ETABS files:

**Validation Checks:**

- File exists
- File extension is `.edb`
- File is readable
- File format is valid ETABS
- ETABS version compatibility
- File not corrupted

**Validation Command:**

```bash
ext etabs validate model.edb

# Output:
# ✓ File exists
# ✓ Extension: .edb
# ✓ Format: Valid ETABS 22
# ✓ Readable: Yes
# ✓ Size: 45.2 MB
# ✓ Analysis: Complete
```

**Error Handling:**

```bash
ext etabs validate broken.edb

# Output:
# ✓ File exists
# ✓ Extension: .edb
# ✗ Format: Corrupted or invalid
# → Cannot open file
# → Try recovering from backup
```

### 15. Reports

The CLI can generate various reports:

**Report Types:**

**1. Comparison Report:**

```bash
ext report generate comparison --v1 v2 --v2 v3 --pdf --excel
```

Detailed comparison with charts and tables in both pdf and excel file.

**2. Analysis Report:**

```bash
ext report generate analysis --version v3
```

Structural analysis postprocessing results summary.

**3. Bill of Materials:**

```bash
ext report generate bom --version v3
```

Material quantities and costs.

**4. Progress Report:**

```bash
ext report generate progress --branch main
```

Timeline of design development.

**Report Outputs:**

- PDF (default)
- Excel (for BOM)
- HTML (for web viewing)
- Markdown (for documentation)

### 16. Collaboration Model

ETABS Extension enables team collaboration:

**Workflow:**

**Designer A:**

```bash
ext branch new steel-frame
ext commit "Initial steel design"
```

**Designer B:**

```bash
ext branch switch steel-frame
# Review and make changes
ext commit  "Refine steel connections"
```

**Lead Engineer:**

```bash
ext compare main/v3 steel-frame/v2 --forces --materials
ext branch merge steel-frame --confirm   # If approved
```

**Behind the Scenes:**

- Git handles synchronization
- `.edb` files synced via LFS or cloud
- E2K files show clear diffs
- Conflict resolution when needed

### 17. Recovery and Undo

Multiple safety mechanisms:

**Version History:**
Every version is preserved:

```bash
ext log
ext restore v2        # Restore working file to commit v2
```

**Snapshots:**
Automatic snapshots before operations:

```bash
#Branch switches
#Restores
#Merges
#ETABS-close with unsaved changes
```

**Manual Backup:**

```bash
ext backup
# Creates timestamped backup
```

**Undo Last Operation:**

```bash
ext undo
# Reverts last change
```

### 18. Performance Considerations

**Large Files:**

- `.edb` files can be 100MB+
- Streaming for large operations
- Progress indicators
- Cancellable operations

**E2K Generation:**

- Can take 30-120 seconds
- Runs in background when possible
- Cached results
- Skip with `--no-e2k` if needed

**Storage:**

- E2K files tracked in Git (~10% of .edb size)
- `.edb` files in separate storage
- Automatic compression
- Cleanup old versions

## Philosophy

### Version Control Is Essential

Structural engineering projects are:

- **Iterative**: Multiple design cycles
- **Collaborative**: Teams working together
- **Complex**: Many interdependent decisions
- **Regulated**: Audit trail required
- **Long-lived**: Projects span months/years

Traditional file-based versioning fails at scale. ETABS Extension brings software engineering practices to structural engineering.

### Transparency and Trust

The CLI is transparent about what it does:

- Clear status messages
- Verbose mode available
- Dry-run options
- Confirmation prompts for destructive operations

Engineers can trust the tool because they can see exactly what's happening.

### Integration, Not Replacement

ETABS Extension **integrates with** ETABS, not replaces it:

- ETABS remains the modeling environment
- Extension adds version control layer
- Engineers work in familiar tools
- Minimal workflow disruption

### CLI-First Design

The CLI is not an afterthought—it's the **primary interface**:

- Desktop app is built on CLI
- API calls CLI commands
- Scripts use CLI
- Documentation focuses on CLI

This ensures:

- Testability
- Scriptability
- Consistency
- Reliability

## Summary

ETABS Extension CLI provides Git-like version control for structural engineering:

**Core Value:**

- Track every design decision
- Compare alternatives easily
- Collaborate effectively
- Maintain complete history
- Audit trail for regulations

**Key Innovations:**

- E2K files for human-readable diffs
- Branch-based design alternatives
- ETABS integration
- Git benefits without Git complexity

**Result:**
Engineers can focus on **design** instead of **file management**.