---
name: ext
version: 0.1.0
description: Version control, branching, comparison, and analysis for ETABS models. Use for managing ETABS projects, creating design alternatives, committing versions, running analysis, comparing structural behavior, and generating reports. Replaces ad-hoc ETABS file copies with structured version control.
author: ETABS Extension Team
---

# ETABS Extension CLI Skill

Help users work with the ETABS Extension CLI (`ext` command) to manage ETABS models using a Git-like, engineer-friendly workflow.

---

## Proactive Agent Workflow

**CRITICAL:** Follow this pattern for **every** task involving ETABS model changes:

1. **Check state** → `ext status --json`
2. **Create work context** → `ext branch new &lt;task-name&gt;`
3. **Make changes** → Edit ETABS model (via ETABS UI or automation)
4. **Commit work** → `ext commit "message"` (optionally `--analyze`)
5. **Refine / compare** → Use `ext diff`, `ext compare`, or `ext analyze`
6. **Report** → Use `ext report`

**Commit early, commit often.**  
ETABS model versions are cheap. You can always compare, squash, or discard later. Small, meaningful commits are better than large untracked changes.

---

## After Modifying an ETABS Model

When ready to save work:

1. Run `ext status --json` to confirm working file state.
2. Commit the model:

   ```bash
   ext commit "Describe engineering intent"

   # Optionally run analysis:
   ext commit "Update column sections" --analyze
   ```

You do not need to wait for a “final” design. ETABS Extension is designed for iterative engineering.

---

## Critical Concept: Model-Centric Version Control

**ETABS Extension ≠ Traditional Git**

- **Traditional Git:** Text files, manual staging, source-code focus  
- **ETABS Extension:** Binary ETABS models, structural intent, analysis-aware  

This means:

- ❌ Do not copy `.edb` files manually.
- ❌ Do not manage versions by filenames (`final_v3_real_final.edb`).  
- ✅ Use `ext commit`, `ext branch`, `ext diff`, `ext compare`.  
- ✅ Let the CLI manage E2K exports and metadata.

---

## Quick Start

**Initialize a project:**

```bash
ext init "HighRise Tower"
```

**Typical workflow:**

```bash
ext status --json
ext branch new steel-columns
# Modify model in ETABS
ext commit "Switch columns to steel option" --analyze
ext compare main/v3 steel-columns/v1 --forces
```

---

## Essential Commands

> **IMPORTANT for AI agents:**  
> Always use `--json` for machine-readable output when available.

### Understanding State

- `ext status --json` — Overview of project, branch, working file, ETABS state (**START HERE**)
- `ext show project --json` — Project details
- `ext show branch &lt;name&gt; --json` — Branch details
- `ext show commit &lt;id&gt; --json` — Commit details

### Organizing Work

- `ext branch new &lt;name&gt;` — Create a new design alternative
- `ext branch switch &lt;name&gt;` — Switch active branch
- `ext branch merge &lt;name&gt;` — Merge branch into `main`
- `ext branch merge &lt;name&gt; --into &lt;branch&gt;` — Advanced merge
- `ext branch delete &lt;name&gt;` — Remove obsolete alternatives

### Saving Work (Version Control)

- `ext commit "message"` — Create a new version snapshot
- `ext commit "message" --analyze` — Commit and run ETABS analysis
- `ext save "message"` — Alias for `ext commit`
- `ext log --json` — View version history
- `ext restore &lt;id&gt;` — Restore working file to a commit

### Comparison & Analysis

- `ext diff v1 v2` — Fast E2K / geometry comparison (no ETABS)
- `ext compare v1 v2 --forces` — Structural behavior comparison
- `ext compare v1 v2 --materials` — Quantity comparison
- `ext compare v1 v2 --costs` — Cost estimation deltas
- `ext compare v1 v2 --all` — Full analytical comparison
- `ext analyze &lt;version&gt;` — Explicit ETABS analysis run

### ETABS Integration

- `ext etabs open [version]` — Open model in ETABS
- `ext etabs close` — Close ETABS safely
- `ext etabs status --json` — ETABS runtime status
- `ext etabs validate &lt;file&gt;` — Validate `.edb` file
- `ext etabs export e2k &lt;file&gt;` — Export E2K

### Reports

- `ext report generate comparison --v1 v1 --v2 v2`
- `ext report generate analysis --version v3`
- `ext report generate bom --version v3`
- `ext report list`
- `ext report template &lt;type&gt; --edit`

---

## Key Concepts

### Commits = Engineering Decisions

Each commit represents:

- A design intent  
- A structural assumption  
- A checkable state  

This makes reviews, comparisons, and rollback meaningful.

### Branches = Design Alternatives

Branches are not temporary:

- Each branch is a valid structural option.  
- Branches can be compared quantitatively.  
- Merging is a conscious engineering decision.  

### Diff vs Compare

| Command     | Purpose                             |
|------------|--------------------------------------|
| `ext diff` | Fast, textual / geometric comparison |
| `ext compare` | Structural behavior &amp; quantities |

- Use `diff` early and often.  
- Use `compare` when engineering decisions matter.

---

## Guidelines

- Always start with `ext status --json`.
- Create a branch for each design alternative.
- Commit small, meaningful changes.
- Use `--analyze` intentionally (analysis is expensive).
- Prefer `ext compare` over screenshots or gut feeling.
- Delete branches only after decisions are finalized.
- Treat the CLI as the source of truth, not file copies.

---

## Philosophy

ETABS Extension is **Git-like version control for structural models**, with ETABS as the execution engine.

It is designed to:

- Reduce design ambiguity  
- Enable quantitative comparison  
- Encourage exploratory engineering  
- Replace manual file chaos with intent-driven workflows  
