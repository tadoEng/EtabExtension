# ETABS Extension CLI Workflow Examples

Real-world examples of common structural engineering workflows.

## Contents

- [ETABS Extension CLI Workflow Examples](#etabs-extension-cli-workflow-examples)
  - [Contents](#contents)
  - [Example 1: Starting a New Project](#example-1-starting-a-new-project)
  - [Example 2: Creating Design Alternatives](#example-2-creating-design-alternatives)
  - [Example 3: Comparing Column Designs](#example-3-comparing-column-designs)
  - [Example 4: Foundation Redesign Workflow](#example-4-foundation-redesign-workflow)
  - [Example 5: Iterative Design Refinement](#example-5-iterative-design-refinement)
  - [Example 6: Collaborative Design Review](#example-6-collaborative-design-review)
  - [Example 7: Recovering from Mistakes](#example-7-recovering-from-mistakes)
  - [Example 8: Batch Analysis Workflow](#example-8-batch-analysis-workflow)
  - [Example 9: Cost Optimization Study](#example-9-cost-optimization-study)
  - [Example 10: Complete Project Lifecycle](#example-10-complete-project-lifecycle)
  - [Tips and Tricks](#tips-and-tricks)
    - [Quick Status Check](#quick-status-check)
    - [Automation with Scripts](#automation-with-scripts)
    - [Branch Naming Conventions](#branch-naming-conventions)
    - [Commit Message Best Practices](#commit-message-best-practices)
    - [Diffing Specific Files](#diffing-specific-files)
    - [Exporting for Reports](#exporting-for-reports)
  - [Common Workflows Summary](#common-workflows-summary)

---

## Example 1: Starting a New Project

**Scenario:** You're starting a new high-rise tower project in ETABS and want to use version control from the beginning.

```bash
# 1. Initialize project
ext init "HighRise Tower" --path D:\Projects\HighRise

# Output:
# ✓ Created project: HighRise Tower
# ✓ Initialized Git repository
# ✓ Created main branch
# → Path: D:\Projects\HighRise

# 2. Configure ETABS settings
ext config set etabs.executable "C:\Program Files\ETABS 22\ETABS.exe"
ext config set etabs.auto_generate_e2k true
ext config set git.author "John Doe"

# 3. Create initial model in ETABS
ext etabs open

# Work in ETABS: Create geometry, define loads, etc.
# Save in ETABS as: D:\Projects\HighRise\main\working\model.edb

# 4. Save first version
ext commit "Initial structural layout"

# Output:
# ✓ Saved version v1
# ✓ Generated E2K file
# ✓ Committed to main branch
#   Files: model.edb (45.2 MB), model.e2k (2.3 MB)

# 5. Check status
ext status

# Output:
# Project: HighRise Tower
# Branch: main (1 version)
# Working File: Clean (matches v1)
# ETABS: Running (PID: 12345)
```

**Result:** Project is initialized with version control and first commit.

---

## Example 2: Creating Design Alternatives

**Scenario:** You want to explore two different column designs: concrete vs steel.

```bash
# 1. Check current state
ext status

# Output shows you're on main branch with v3 as latest

# 2. Create branch for steel columns
ext branch new steel-columns --from main/v3 \
  --description "Explore steel wide-flange sections for columns"

# Output:
# ✓ Created branch: steel-columns
# ✓ Based on: main/v3
# ✓ Working file: Ready for editing

# 3. Switch to new branch
ext branch switch steel-columns

# Output:
# ✓ Switched to branch: steel-columns
# ✓ Working file updated from v3
# ✓ Ready to open in ETABS

# 4. Make changes in ETABS
ext etabs open

# Modify column sections to steel W-shapes in ETABS

# 5. Save the steel version
ext commit "Changed columns to W14x120 steel sections"

# 6. Create second alternative for concrete
ext branch new concrete-hsc --from main/v3 \
  --description "High-strength concrete columns"

# 7. Switch and modify
ext branch switch concrete-hsc
ext etabs open

# Change to HSC columns in ETABS

ext commit "Updated columns to fc=8000 psi concrete"

# 8. Compare the alternatives
ext diff steel-columns/v1 concrete-hsc/v1

# Output:
# E2K Changes:
#   Frame Section:
#     - Concrete: 12 columns, C30x30, fc=4000 psi
#     + Steel: 12 columns, W14x120
#     + Concrete HSC: 12 columns, C24x24, fc=8000 psi
```

**Result:** Two design alternatives created and easily comparable.

---

## Example 3: Comparing Column Designs

**Scenario:** You need to compare structural performance between steel and concrete columns.

```bash
# 1. Generate comparison report
ext report generate comparison \
  --v1 steel-columns/v1 \
  --v2 concrete-hsc/v1 \
  --format pdf \
  --output column-comparison.pdf \
  --include-images

# Output:
# Analyzing steel-columns/v1...
# Analyzing concrete-hsc/v1...
# Comparing structures...
# ✓ Generated report: column-comparison.pdf (12 pages)

# 2. Get detailed diff
ext compare steel-columns/v1 concrete-hsc/v1 --all --json > comparison.json
# 3. View E2K differences
ext diff steel-columns/v1 concrete-hsc/v1 --e2k

# Output:
# E2K Changes:
#   Material:
#     - Removed: CONC C4000 (Concrete fc=4000 psi)
#     + Added: STEEL A992 (Steel Fy=50 ksi)
#     + Added: CONC C8000 (Concrete fc=8000 psi)
#   
#   Frame Section:
#     Steel version:
#       12 columns: W14x120
#     Concrete HSC version:
#       12 columns: C24x24 (fc=8000 psi)
#   
#   Weight:
#     Steel: 245 tons
#     Concrete HSC: 312 tons

# 4. View 3D geometry differences
ext diff steel-columns/v1 concrete-hsc/v1 --geometry

# Output:
# 3D Geometry Changes:
#   Members Modified: 12 (all columns)
#   Section changes: C30x30 → W14x120 or C24x24
#   Total Changes: 12
```

**Result:** Comprehensive comparison of two design alternatives with visual report.

---

## Example 4: Foundation Redesign Workflow

**Scenario:** Poor soil conditions require redesigning from spread footings to deep foundations.

```bash
# 1. Create branch for foundation redesign
ext branch new deep-foundation --from main/v3 \
  --description "Redesign for poor soil: drilled piers"

# 2. Switch and open in ETABS
ext branch switch deep-foundation
ext etabs open

# In ETABS: Replace spread footings with drilled piers

# 3. Save incremental versions
ext commit "Removed spread footings"
ext commit "Added drilled pier layout"
ext commit "Updated pier reinforcement"

# 4. Check version history
ext log

# Output:
# Branch: deep-foundation
#   v3  Updated pier reinforcement        10m ago
#   v2  Added drilled pier layout         25m ago
#   v1  Removed spread footings           45m ago

# 5. Compare with original design
ext diff main/v3 deep-foundation/v3

# Output:
# Foundation Changes:
#   Removed: 24 spread footings (F1: 8'x8'x2')
#   Added: 24 drilled piers (DP1: 4' dia x 45' deep)
#   
# Material Quantities:
#   Concrete: +125 CY
#   Rebar: +14.2 tons
#   
# Cost Impact: +$185,000 (estimated)

# 6. Generate comparison report for client
ext report generate comparison \
  --v1 main/v3 \
  --v2 deep-foundation/v3 \
  --format pdf \
  --output foundation-comparison.pdf \
  --include-images \
  --include-costs
```

**Result:** Documented foundation redesign with clear comparison to original.

---

## Example 5: Iterative Design Refinement

**Scenario:** Iteratively refining design based on analysis results.

```bash
# 1. Check current status
ext status

# Output shows main branch, v3 is latest

# 2. Open in ETABS and run analysis
ext etabs open
# Run analysis in ETABS: Analyze → Run Analysis

# 3. Save version with analysis
ext commit "Ran initial analysis" --analyze

# Output:
#✓ Created commit v4
#✓ Analysis results captured
#✓ Generated E2K file

# 4. Review analysis (in ETABS, find issues: beam B45 overstressed)

# 5. Fix issue and validate in one step
# In ETABS: Increase beam B45 to W21x93
# Run analysis again

ext commit "Increase beam B45 to W21x93 and reanalyze" --analyze

# 6. Compare performance
ext analyze v4 --results > initial-results.xlsx
ext analyze v5 --results > final-results.xlsx

# 8. View version history
ext log

# Output:
# Branch: main
#   v6  Reanalyzed with updated sections  5m ago   ✓ analyzed
#   v5  Increased beam B45 to W21x93      10m ago
#   v4  Ran initial analysis               20m ago  ✓ analyzed
#   v3  Updated column sections            2d ago
#   v2  Added seismic loads                5d ago
#   v1  Initial layout                     2w ago

# 9. Generate progress report
ext report generate progress \
  --branch main \
  --from v4 \
  --to v5 \
  --format pdf
```

**Result:** Clear record of iterative design process with analysis checkpoints.

---

## Example 6: Collaborative Design Review

**Scenario:** Senior engineer reviews your design and suggests changes.

```bash
# 1. Create review branch
ext branch create review-jsmith --from main/v6 \
  --description "John Smith design review comments"

ext branch switch review-jsmith

# 2. Implement review comments
ext etabs open

# Make changes based on reviewer's comments

ext version save "Addressed J.Smith comment #1: Increase lateral bracing"
ext version save "Addressed J.Smith comment #2: Update connection details"
ext version save "Addressed J.Smith comment #3: Add edge beams"

# 3. Compare before/after review
ext diff main/v6 review-jsmith/v3

# 4. Generate review response document
ext report generate comparison \
  --v1 main/v6 \
  --v2 review-jsmith/v3 \
  --format pdf \
  --output review-response.pdf

# 5. If approved, merge back to main
ext branch merge review-jsmith --confirm

# Output:
# ✓ Merged review-jsmith into main
# ✓ Created merge version: main/v7
# ✓ Review branch: retained for history

# 6. Clean up (optional - keep for record)
# ext branch delete review-jsmith
```

**Result:** Documented review process with clear change tracking.

---

## Example 7: Recovering from Mistakes

**Scenario:** Made changes you want to undo.

```bash
# Situation: Made bad changes in v8, want to go back to v7

# 1. Check what changed
ext diff v7 v8

# Output shows you removed critical lateral bracing (oops!)

# 2. Restore to previous version
ext restore v7 --confirm

# Output:
# ⚠ Working file has unsaved changes
# Restore will overwrite current work
# Continue? [y/N]: y
# ✓ Restored working file from v7

# 3. Verify restoration
ext diff v7  # Shows no changes (working file matches v7)

# 4. Open and verify in ETABS
ext etabs open

# 5. If you want to keep v8 as a warning
ext branch new bad-design --from main/v8
ext branch switch main
ext restore v7 --confirm

# Now v8 is preserved in bad-design branch but main is back to v7
```

**Alternative: If you already saved v8 but want to undo:**

```bash
# Don't delete v8, just create new version from v7
ext restore v7 --confirm
ext commit "Revert to v7 – v8 removed critical lateral bracing"

# Now you have:
# v9 - Reverted back (same as v7)
# v8 - Bad version (kept for record)
# v7 - Good version

# v8 is still in history for reference
```

**Result:** Safely recovered from mistakes with full audit trail.

---

## Example 8: Batch Analysis Workflow

**Scenario:** Run analysis on multiple design alternatives for comparison.

```bash
# 1. List all branches
ext branch

# Output:
# main               v6  (active)
# steel-columns      v3
# concrete-hsc       v3
# deep-foundation    v3

# 2. Analyze each alternative (script)
#!/bin/bash
branches=("main" "steel-columns" "concrete-hsc" "deep-foundation")

for branch in "${branches[@]}"; do
    echo "Analyzing $branch..."
    ext branch switch "$branch"
    ext etabs open
    # Run ETABS analysis (API or manual)
    ext commit "Batch analysis run for $branch" --analyze
done

# 3. Extract analysis results (JSON for tooling)
ext analyze main/v7 --results --json > results-main.json
ext analyze steel-columns/v4 --results --json > results-steel.json
ext analyze concrete-hsc/v4 --results --json > results-concrete.json
ext analyze deep-foundation/v4 --results --json > results-foundation.json

# 4. Generate multi-version comparison report
ext report generate analysis-comparison \
  --versions main/v7,steel-columns/v4,concrete-hsc/v4,deep-foundation/v4 \
  --format pdf \
  --output analysis-comparison.pdf

# Output includes:
# - Max displacements
# - Base shear
# - Member utilization ratios
# - Material quantities
# - Cost estimates
```

**Result:** Systematic comparison of multiple design alternatives.

---

## Example 9: Cost Optimization Study

**Scenario:** Explore cost-saving measures while maintaining structural integrity.

```bash
# 1. Create optimization branch
ext branch new cost-optimization --from main/v6 \
  --description "Value engineering study"

ext branch switch cost-optimization

# 2. Try different cost-saving measures
# Iteration 1: Reduce concrete strength
ext etabs open
# Change fc=5000 to fc=4000 where possible
ext commit "Reduce concrete strength to fc=4000 and reanalyze" --analyze

# Iteration 2: Optimize member sizes
# Reduce oversized members
ext commit "Optimize oversized beams and reanalyze" --analyze

# Iteration 3: Simplify connections
# Standardize connection details
ext commit "Standardize connection types and reanalyze" --analyze

# 3. Compare each iteration
ext diff main/v6 cost-optimization/v1 --json > cost-iter1.json
ext diff main/v6 cost-optimization/v2 --json > cost-iter2.json
ext diff main/v6 cost-optimization/v3 --json > cost-iter3.json

# 4. Generate cost summary report
ext report generate bom --version main/v6 --output original-bom.xlsx
ext report generate bom --version cost-optimization/v3 --output optimized-bom.xlsx

# 5. Compare material quantities
# Parse CSV files or:
ext compare main/v6 cost-optimization/v3 --report

# Output:
# Cost Optimization Summary:
# Original Design (main/v6): $2,450,000
# Optimized Design (v3): $2,180,000
# Savings: $270,000 (11%)
#
# Material Changes:
#   Concrete: -85 CY
#   Rebar: -8.5 tons
#   Steel: -12 tons
#
# Performance Changes:
#   Max Drift: 0.018 → 0.019 (still < 0.020 limit ✓)
#   Max Util: 0.87 → 0.92 (still < 0.95 limit ✓)

# 6. If acceptable, merge
ext branch merge cost-optimization --confirm
```

**Result:** Documented cost optimization with performance verification.

---

## Example 10: Complete Project Lifecycle

**Scenario:** Full project from start to construction documents.

```bash
# === PHASE 1: SCHEMATIC DESIGN ===

# 1. Initialize project
ext init "Downtown Office Tower" --path D:\Projects\DowntownOffice

# 2. Create initial design
ext etabs open
# Create basic geometry in ETABS
ext commit "SD-01: Initial structural concept"

# 3. Explore alternatives
ext branch new alt-steel --from main/v1
ext branch switch alt-steel
ext commit "SD-02: Steel frame alternative"

# Refined concrete option
ext branch switch main
ext commit "SD-03: Refined concrete frame"

# 4. Compare alternatives
ext report generate comparison \
  --v1 main/v3 \
  --v2 alt-steel/v1 \
  --format pdf \
  --output schematic-comparison.pdf

# === PHASE 2: DESIGN DEVELOPMENT ===

# 5. Continue with selected system (concrete)
ext branch switch main

ext commit "DD-01: Developed member sizes"
ext commit "DD-02: Added lateral system details"
ext commit "DD-03: Refined foundation design"

# === PHASE 3: CONSTRUCTION DOCUMENTS ===

# 6. Final design
ext commit "CD-01: Final member sizes" --analyze
ext commit "CD-02: Final connection details"
ext commit "CD-03: Construction sequencing"

# 7. Review cycle
ext branch new cd-review --from main/v9
ext branch switch cd-review
ext commit "CD-04: Addressed review comments"

# Merge review back to main
ext branch merge cd-review --confirm

# 8. Final deliverables
ext report generate analysis \
  --version main/v10 \
  --format pdf \
  --output analysis-report.pdf

ext report generate bom \
  --version main/v10 \
  --format xlsx \
  --output material-quantities.xlsx

ext report generate progress \
  --branch main \
  --format pdf \
  --output design-narrative.pdf

# 9. Archive final design
ext etabs export e2k main/v10 \
  --output FINAL-CD-SET.e2k

# 10. Project summary
ext status

# Output:
# Project: Downtown Office Tower
# Branch: main (10 versions)
# 
# Version History:
#   v10  CD-04: Review comments addressed     3d ago   ✓ final
#   v9   CD-03: Construction sequencing       5d ago
#   v8   CD-02: Final connection details      1w ago
#   v7   CD-01: Final member sizes            1w ago   ✓ analyzed
#   ...
#   v1   SD-01: Initial concept               6w ago
#
# Alternative Branches:
#   alt-steel (not merged)
#   cd-review (merged to main)
```

**Result:** Complete project history from concept to construction documents with full traceability.

---

## Tips and Tricks

### Quick Status Check

```bash
# Fast overview
ext status

# Detailed with file sizes
ext status --verbose

# JSON for scripting
ext status --json | jq '.working_file.has_unsaved_changes'
```

### Automation with Scripts

```bash
#!/bin/bash
# daily-snapshot.sh — Create daily analysis checkpoint

DATE=$(date +%Y-%m-%d)
ext commit "Daily snapshot: $DATE" --analyze
```

### Branch Naming Conventions

```bash
# Good branch names:
ext branch new steel-columns-w14
ext branch new foundation-drilled-piers
ext branch new review-2024-01-15
ext branch new cost-reduction-phase2

# Avoid generic names:
# alternative, test, new, temp
```

### Commit Message Best Practices

```bash
# ✅ Good: Specific and descriptive
ext commit "Increase columns C1–C6 from W14x90 to W14x120 per analysis"

# ✅ Good: Reference review/issue
ext commit "Fix connection detail per review comment #3"

# ❌ Bad: Vague
# ext commit "Updated model"
# ext commit "Changes"
```

### Diffing Specific Files

```bash
# Focus on specific changes
ext diff v2 v3 --e2k | grep "Frame Section"
ext diff v2 v3 --e2k | grep "Load"
```

### Exporting for Reports

```bash
# Get data for external analysis
ext compare v1 v2 --json > changes.json
ext analyze v2 --results --json > results.json

# Process with Python / R / Excel
python analyze-results.py changes.json results.json
```

---

## Common Workflows Summary

**Daily Development:**

```bash
ext status                          # Morning: Check state
ext etabs open                      # Work in ETABS
ext commit "Daily progress"   # Afternoon: Save work
```

**Design Alternatives:**

```bash
ext branch new alternative          # Create design alternative
ext branch switch alternative       # Switch to it
ext commit "Alternative design"     # Record design decision
ext diff main/v3 alternative/v1     # Compare with baseline
ext branch merge alternative --confirm   # Merge if approved
```

**Review Process:**

```bash
ext branch new review               # Create review branch
ext branch switch review

ext commit "Address review comment #1"
ext commit "Address review comment #2"

ext report generate comparison \
  --v1 main/v6 \
  --v2 review/v2

ext branch merge review --confirm   # Merge if approved
```

**Recovery:**

```bash
ext log                             # Inspect history
ext restore v5 --confirm            # Restore good state
ext commit "Revert to v5 due to design issue"
```
