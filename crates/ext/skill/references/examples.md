# ETABS Extension — Workflow Examples

Real-world examples of common structural engineering workflows.

---

## Example 1: Starting a New Project

```bash
ext init "HighRise Tower" \
  --edb "C:\Models\highrise.edb" \
  --path "D:\ETABSProjects\HighRise" \
  --author "John Doe" \
  --email "john@firm.com" \
  --reports "C:\Users\John\OneDrive\Structural\HighRise\reports"

# ✓ Created project: HighRise Tower
# ✓ Copied model → main/working/model.edb (45.2 MB)
# ✓ Wrote config.local.toml
# → Next: ext commit "Initial model"

ext commit "Initial structural layout"
# ✓ Version v1 saved

ext push
# ✓ Pushed git history
# ✓ Pushed main/v1.edb (45.2 MB)
# ✓ OneDrive updated
```

---

## Example 2: Daily Design Iteration

```bash
ext status

ext etabs open
# ✓ ETABS opened: main/working/model.edb

# ... modify beam B45 to W21x93, Ctrl+S, close ETABS ...

ext commit "Increased beam B45 to W21x93"
# ✓ Version v2 saved

ext log
# v2  Increased beam B45 to W21x93    5m ago
# v1  Initial structural layout        2d ago
```

---

## Example 3: Capturing Analysis Results

```bash
ext etabs open
# ... run analysis in ETABS, review results, close ETABS ...

ext commit "Initial analysis complete" --analyze
# ✓ Saving v3...
#   ✓ Exported model.e2k             (42s)
#   ✓ Extracted materials
#   ✓ git commit
#   ✓ Running analysis on v3...      (2m 14s)
#   ✓ Extracted all results
# ✓ Version v3 saved  (total: 3m 12s)

ext report analysis --version v3
# ✓ Generated: C:\Users\John\OneDrive\Structural\HighRise\reports\main-v3-analysis.pdf
```

---

## Example 4: Post-Commit Analysis

```bash
ext log
# v3  Updated sections    1h ago        ← no analysis yet
# v2  Added seismic loads  3d ago  ✓ analyzed

ext analyze v3
# ✓ Running analysis on v3...  (2m 8s)
# ✓ Version v3 now analyzed

ext report analysis --version v3
```

---

## Example 5: Design Alternatives

```bash
ext switch -c steel-columns --from main/v3
# ✓ Created branch: steel-columns  (from main/v3)
# ✓ Switched to: steel-columns

ext etabs open
# ... change columns to W14x120 steel, run analysis, close ...
ext commit "W14x120 steel columns" --analyze

ext switch main
ext switch -c concrete-hsc --from main/v3
ext etabs open
# ... fc=8000 psi, run analysis, close ...
ext commit "fc=8000 psi HSC columns" --analyze

ext report comparison \
  --from steel-columns/v1 \
  --to concrete-hsc/v1
# ✓ Generated: .../reports/steel-columns-v1-vs-concrete-hsc-v1-comparison.pdf
```

---

## Example 6: Switching Branches with Uncommitted Work

```bash
ext status
# Working: Modified (since v3, 2h ago)

ext switch steel-columns
# ⚠ Leaving main with uncommitted changes since v3
#   Changes preserved in main/working/model.edb
#
# ✓ Switched to: steel-columns
# ⚠ steel-columns has uncommitted changes since v1

ext commit "Refined connections"

ext switch main
# ✓ Switched to: main
# ⚠ Working file has uncommitted changes since v3
#   Run: ext commit "msg"   to save them
#        ext checkout v3    to discard them
```

---

## Example 7: Going Back to a Specific Version

```bash
ext log
# v4  Optimized sections   today
# v3  Updated columns      2d ago  ✓ analyzed
# v2  Added seismic loads  5d ago  ✓ analyzed
# v1  Initial layout       2w ago

ext checkout v2
# ⚠ Working file has changes since v4.
#   [c] Commit first  [s] Stash  [d] Discard  [x] Cancel

# Choose [s]
# ✓ Stashed: main working file (since v4)
# ✓ Restored v2 → main/working/model.edb

ext etabs open
# ... review v2, close ...

ext stash pop
# ✓ Restored stash  (was based on v4)
```

---

## Example 8: Editing After Analysis (Unlock Workflow)

```bash
ext status
# Working: Analyzed

ext etabs unlock
# ✓ Model lock cleared — ETABS model is now editable
# ⚠ Embedded analysis results cleared (extracted Parquet files are preserved)

# ... make edits, Ctrl+S, close ETABS ...

ext commit "Refined after reviewing results" --analyze
# ✓ Version v4 saved with fresh analysis
```

---

## Example 9: ETABS Crash Recovery

```bash
ext status
# Working: Orphaned

ext etabs recover
# ⚠ ETABS closed unexpectedly (PID: 12345)
#   File modified: Yes (14 minutes before crash)
#
#   [k] Keep changes  [r] Restore from v3

# Choose [k]
# ✓ State set to: MODIFIED

ext commit "Recovery: pre-crash changes"
```

---

## Example 10: Setting Up on a New Machine

John has been working on the project. Jane joins and needs a local copy.

**John pushes everything first:**

```bash
ext push --include-working
# ✓ Pushed git history
# ✓ Pushed main/v1.edb through main/v4.edb
# ✓ Pushed steel-columns/v1.edb
# ✓ Pushed main-working.edb
```

**Jane sets up on her machine:**

```bash
ext clone "C:\Users\Jane\OneDrive\Structural\HighRise" --to "C:\ETABSProjects\HighRise"

# Setting up HighRise Tower on this machine...
#
# Author name:    [Jane Smith        ]
# Author email:   [jane@firm.com     ]
# OneDrive folder:[C:\Users\Jane\OneDrive\Structural\HighRise]
# Reports folder: [C:\Users\Jane\OneDrive\Structural\HighRise\reports]
#
# ✓ Restored git history (2 branches, 5 versions)
# ✓ Copied main/v1 through v4 (.edb files)
# ✓ Copied steel-columns/v1.edb
# ✓ Set working file: main/v4
# ✓ Wrote config.local.toml
# → Project ready. Run: ext status

ext status
# Project: HighRise Tower
# Branch:  main (4 versions)
# Working: Clean (matches v4)
# Author:  Jane Smith
```

---

## Example 11: Team Workflow — Person-per-Branch Convention

Jane works on a foundation redesign independently, then John adopts it into main.

**Jane's machine:**

```bash
ext switch -c jane/foundation-v2 --from main/v4
ext etabs open
# ... redesign mat foundation, run analysis, close ...
ext commit "Mat foundation redesign" --analyze

ext report analysis --version v1
# saved to OneDrive reports folder automatically

ext push --branch jane/foundation-v2
# ✓ Pushed jane/foundation-v2/v1
```

**John's machine:**

```bash
ext pull --branch jane/foundation-v2
# ✓ Pulled jane/foundation-v2/v1 — "Mat foundation redesign" (Jane Smith, 1h ago)

ext report comparison --from main/v4 --to jane/foundation-v2/v1
# review comparison PDF...

ext switch main
ext checkout jane/foundation-v2/v1
# ✓ Restored jane/foundation-v2/v1 → main/working/model.edb

ext commit "Adopt Jane's mat foundation redesign" --analyze
# ✓ Version v5 saved with analysis

ext push
# ✓ Pushed main/v5
```

**Jane pulls the update:**

```bash
ext pull
# ✓ Pulled main/v5 — "Adopt Jane's mat foundation redesign" (John Doe, 30m ago)
```

---

## Example 12: Conflict Resolution on Push

Two engineers pushed to OneDrive at the same time.

```bash
ext push
# ✗ Conflict: main/v4 already exists on OneDrive
#   Pushed by: Jane Smith  (2h ago)
#   Her v4:   "Updated shear walls"
#   Your v4:  "Increased column sizes"
#
#   [r] Rename yours to v5 and push  (recommended)
#   [v] View diff between versions
#   [x] Cancel

# Choose [r]
# ✓ Local v4 renamed to v5
# ✓ Pushed main/v5 — "Increased column sizes"
# → Inform Jane: run ext pull to get v5
```

---

## Example 13: BOM Report for Cost Estimation

```bash
# Material takeoff is always available (generated on every commit)
ext report bom --version v1
ext report bom --version v4

# Both saved to OneDrive reports folder
ext report comparison --from v1 --to v4
# → shows +12.3t concrete, -0.8t rebar delta
```

---

## Example 14: Checking Remote Status

```bash
ext remote status
# Remote: C:\Users\John\OneDrive\Structural\HighRise
# Last pushed by: John Doe  (2h ago)
#
#   main:              local v5  =  remote v5  ✓ in sync
#   steel-columns:     local v1  =  remote v1  ✓ in sync
#   jane/foundation:   ─────────   remote v1  ↓ not pulled

ext pull --branch jane/foundation
# ✓ Pulled jane/foundation/v1 — "Alternative slab system" (Jane, 3h ago)
```

---

## Example 15: Setting Up Local AI (Ollama, Private)

No data leaves the machine. No API key needed.

```bash
# Step 1: Install Ollama from https://ollama.com
# Step 2: Pull a model (run in any terminal)
ollama pull qwen2.5-coder:14b

# Step 3: Configure ext
ext config set ai.provider ollama
ext config set ai.model "qwen2.5-coder:14b"
ext config set ai.baseUrl "http://localhost:11434/v1"

# Step 4: Start a session
ext chat

# ETABS Agent — HighRise Tower
# Provider: ollama / qwen2.5-coder:14b  ●  local — no data leaves your machine
# Branch: main  ·  v3  ·  Modified  ·  ETABS not running
#
# You> what's the state of this project?
#
# Agent> You're on branch main with 3 committed versions. The working file
#        has been modified since v3 — about 2 hours ago based on the mtime.
#        ETABS is not running. No stash exists. OneDrive is in sync at v3.
#        Want me to commit your changes?
```

---

## Example 16: AI-Assisted Commit Workflow

```bash
ext chat

# You> commit my changes, message is refined beam B45 connection details

# Agent> I'll run: ext commit "Refined beam B45 connection details"
#        Saves current working file as v4 on branch main.
#        Confirm? [y/n] y

# Agent> ✓ Version v4 saved. Working file is now CLEAN.
#        E2K exported and materials extracted.
#        Suggestion: v4 hasn't been analyzed yet. Run ext commit --analyze
#        or ext analyze v4 when you're ready to capture results.
```

---

## Example 17: AI-Assisted Branch Comparison

```bash
ext chat

# You> what's different between the steel and concrete column alternatives?

# Agent> [reads ext diff steel-columns/v1 concrete-hsc/v1 internally]
#
#        Comparing steel-columns/v1 vs concrete-hsc/v1 (both from main/v3):
#
#        Steel alternative (v1):
#        • 12 columns changed to W-section steel: C1–C12
#        • Section depth: 14", flange width: 14.670"
#        • No slab or foundation changes
#
#        Concrete alternative (v1):
#        • Same 12 columns remain rectangular concrete
#        • Section increased: 30×30 cm → 40×40 cm
#        • Concrete strength increased: fc=4000 → fc=8000 psi
#
#        Both versions have analysis results. Want me to generate a
#        comparison report? (Phase 2 feature — I can give you the command now)

# You> yes give me the command

# Agent> Run:
#        ext report comparison --from steel-columns/v1 --to concrete-hsc/v1
#        The PDF will be saved to your configured reports folder on OneDrive.
```

---

## Example 18: AI-Assisted Crash Recovery

```bash
ext chat

# You> ETABS crashed, what do I do?

# Agent> [reads ext status --json — sees ORPHANED state]
#
#        ETABS closed unexpectedly (PID 12345). The working file was
#        modified about 14 minutes before the crash.
#
#        I'll run ext etabs recover for you. It will ask whether to:
#        [k] Keep your changes — marks the file as MODIFIED so you can commit
#        [r] Restore from v3 — discards the 14 minutes of work
#
#        Which do you prefer? (I recommend [k] unless the changes were
#        experimental and you want a clean start.)

# You> keep the changes

# Agent> I'll run: ext etabs recover
#        Confirm? [y/n] y
#
# [agent runs ext etabs recover and selects [k]]
#
# Agent> ✓ Recovery complete. State is now MODIFIED.
#        Your pre-crash changes are preserved.
#        Run ext commit "message" when you're ready to save them.
```