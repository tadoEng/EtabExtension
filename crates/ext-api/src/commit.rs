// ext-api::commit — commit_version() orchestration.
//
// Implements the 11-step commit sequence from the Week 3-4 spec §3.4:
//   1.  Disk-space check
//   2.  Resolve next version id from git log
//   3.  Create version directory path
//   4.  begin_snapshot (dir + .partial sentinel, RAII rollback on error)
//   5.  atomic_copy working file → vN/model.edb
//   6.  Sidecar call (save_snapshot) unless --no-e2k
//   7.  Write manifest.json
//   8.  git add staged files
//   9.  git commit → returns hash
//   10. Backfill git_commit_hash in manifest.json
//   11. complete_snapshot (delete .partial → success)
//
// If any step 5-10 fails, PartialGuard fires on drop and cleans up vN/.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_core::{
    fs::{atomic_copy, check_disk_space},
    vcs::{current_branch, git_add, git_amend_no_edit, git_commit, next_version_id},
    version::{
        VersionManifest,
        snapshot::{begin_snapshot, complete_snapshot},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    analyze,
    context::AppContext,
    guards::{Command, GuardOutcome, check_state_guard},
    status::resolve_working_file_status,
};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CommitOptions {
    /// Skip E2K export and sidecar call entirely.
    pub no_e2k: bool,
    /// Run analysis on the committed snapshot and capture summary/results metadata.
    pub analyze: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResult {
    pub version_id: String,
    pub branch: String,
    pub git_hash: String,
    pub message: String,
    pub e2k_generated: bool,
    pub e2k_size_bytes: Option<u64>,
    pub materials_extracted: bool,
    pub analyzed: bool,
    pub elapsed_ms: u64,
    /// Set when the guard emitted GuardOutcome::Warn.
    pub warning: Option<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn commit_version(
    ctx: &AppContext,
    message: &str,
    opts: CommitOptions,
) -> Result<CommitResult> {
    let t0 = std::time::Instant::now();

    // Load state + resolve status
    let mut state = ctx.load_state()?;
    let status = resolve_working_file_status(&state, &ctx.project_root);
    let guard_command = if opts.analyze {
        Command::CommitAnalyze
    } else {
        Command::Commit
    };

    // Permission check
    let mut warning: Option<String> = None;
    match check_state_guard(guard_command, &status) {
        GuardOutcome::Block(msg) => bail!("{msg}"),
        GuardOutcome::Warn(msg) => push_warning(&mut warning, msg),
        GuardOutcome::Allow => {}
    }

    if opts.analyze {
        let _ = ctx.require_sidecar()?;
    }

    let ext_dir = ctx.ext_dir();
    let branch = current_branch(&ext_dir)?;
    let branch_dir = ext_dir.join(&branch);
    let working_file = branch_dir.join("working").join("model.edb");

    if !working_file.exists() {
        bail!("✗ Working file not found: {}", working_file.display());
    }

    // Step 1: disk space check
    check_disk_space(&working_file, &branch_dir)
        .with_context(|| "Disk space check failed before commit")?;

    // Step 2: next version id (reads from git log — never filesystem)
    let version_id = next_version_id(&ext_dir, &branch)?;

    // Step 3: version directory
    let version_dir = branch_dir.join(&version_id);

    // Step 4: begin snapshot (creates dir + .partial; RAII rollback on error)
    let guard = begin_snapshot(&version_dir)?;

    // Step 5: copy working file
    let edb_dst = version_dir.join("model.edb");
    atomic_copy(&working_file, &edb_dst)
        .with_context(|| format!("Failed to copy working file to {}", edb_dst.display()))?;

    // Step 6: sidecar snapshot export (skip if --no-e2k)
    let sidecar_units = if !opts.no_e2k || opts.analyze {
        Some(analyze::resolve_sidecar_units(ctx)?)
    } else {
        None
    };

    let (e2k_size_bytes, materials_extracted) = if opts.no_e2k {
        (None, false)
    } else {
        match ctx.sidecar.as_ref() {
            Some(sidecar) => {
                // Generate E2K text file
                let e2k_out = version_dir.join("model.e2k");
                let e2k_result = sidecar.generate_e2k(&edb_dst, &e2k_out, true).await;

                let e2k_bytes = match e2k_result {
                    Ok(data) => Some(data.file_size_bytes),
                    Err(e) => {
                        push_warning(
                            &mut warning,
                            format!("⚠ E2K export failed (commit saved without E2K): {e}"),
                        );
                        None
                    }
                };

                // Extract materials takeoff (non-fatal if it fails)
                let units = sidecar_units
                    .as_deref()
                    .context("Missing sidecar units for material extraction")?;
                let mats_dir = version_dir.join("materials");
                let _ = std::fs::create_dir_all(&mats_dir);
                let mats_ok = sidecar
                    .extract_materials(&edb_dst, &mats_dir, units, None, None)
                    .await
                    .is_ok();

                (e2k_bytes, mats_ok)
            }
            None => {
                // No sidecar configured — treat as --no-e2k silently.
                (None, false)
            }
        }
    };

    let e2k_generated = !opts.no_e2k && e2k_size_bytes.is_some();

    // Step 7: write manifest.json
    let edb_size_bytes = std::fs::metadata(&edb_dst).map(|m| m.len()).unwrap_or(0);

    let parent = state
        .working_file
        .as_ref()
        .and_then(|w| w.based_on_version.clone());

    let (author, email) = {
        let a = ctx
            .config
            .git
            .author
            .as_deref()
            .unwrap_or("Unknown")
            .to_string();
        let e = ctx
            .config
            .git
            .email
            .as_deref()
            .unwrap_or("unknown@local")
            .to_string();
        (a, e)
    };

    let mut manifest = VersionManifest {
        id: version_id.clone(),
        branch: branch.clone(),
        message: message.to_string(),
        author: author.clone(),
        timestamp: Utc::now(),
        parent,
        edb_size_bytes,
        e2k_size_bytes,
        is_analyzed: false,
        e2k_generated,
        materials_extracted,
        git_commit_hash: None,
    };
    manifest
        .write_to(&version_dir)
        .with_context(|| "Failed to write manifest.json")?;

    // Step 8: git add
    let manifest_path = version_dir.join("manifest.json");
    let e2k_path = version_dir.join("model.e2k");
    let mut paths_to_add: Vec<&std::path::Path> = vec![&manifest_path];
    if e2k_path.exists() {
        paths_to_add.push(&e2k_path);
    }

    let material_paths = collect_material_paths(&version_dir);
    for path in &material_paths {
        paths_to_add.push(path.as_path());
    }

    git_add(&ext_dir, &paths_to_add).with_context(|| "Failed to stage files for commit")?;

    // Step 9: git commit
    let initial_git_hash =
        git_commit(&ext_dir, message, &author, &email).with_context(|| "git commit failed")?;

    // Step 10: backfill git_commit_hash and amend the user commit in place.
    // The manifest file cannot stably contain the final amended hash without a
    // second rewrite, so user-facing reads resolve the visible commit hash from
    // git history when needed.
    manifest.git_commit_hash = Some(initial_git_hash.clone());
    manifest
        .write_to(&version_dir)
        .with_context(|| "Failed to rewrite manifest.json with git hash")?;
    git_add(&ext_dir, &[&manifest_path])
        .with_context(|| "Failed to stage finalized manifest.json")?;
    let git_hash = git_amend_no_edit(&ext_dir, &author, &email)
        .with_context(|| "Failed to amend finalized manifest.json into commit")?;
    manifest.git_commit_hash = Some(git_hash.clone());

    // Step 11: complete snapshot (deletes .partial — commits to success)
    complete_snapshot(guard).with_context(|| "Failed to complete snapshot")?;

    // Update state.json
    let mtime = std::fs::metadata(&working_file)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(chrono::DateTime::<Utc>::from);

    if let Some(ref mut wf) = state.working_file {
        wf.based_on_version = Some(version_id.clone());
        wf.last_commit_hash = Some(git_hash.clone());
        wf.last_known_mtime = mtime;
        wf.status = ext_core::state::WorkingFileStatus::Clean;
        wf.status_changed_at = Utc::now();
    }
    state.updated_at = Utc::now();
    ctx.save_state(&state)?;

    let analyzed = if opts.analyze {
        match analyze::analyze_snapshot(ctx, &version_dir, None).await {
            Ok(outcome) => {
                if let Some(extract_warning) = outcome.extract_warning {
                    push_warning(&mut warning, extract_warning);
                }

                match finalize_analysis(
                    &ext_dir,
                    &version_dir,
                    &version_id,
                    &author,
                    &email,
                    &mut manifest,
                    &outcome.summary_path,
                ) {
                    Ok(()) => true,
                    Err(err) => {
                        push_warning(
                            &mut warning,
                            format!(
                                "⚠ Analysis finished, but failed to finalize git metadata: {err}"
                            ),
                        );
                        false
                    }
                }
            }
            Err(err) => {
                push_warning(
                    &mut warning,
                    format!("⚠ Version saved, but analysis failed: {err}"),
                );
                false
            }
        }
    } else {
        false
    };

    Ok(CommitResult {
        version_id,
        branch,
        git_hash,
        message: message.to_string(),
        e2k_generated,
        e2k_size_bytes,
        materials_extracted,
        analyzed,
        elapsed_ms: t0.elapsed().as_millis() as u64,
        warning,
    })
}

fn collect_material_paths(version_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let materials_dir = version_dir.join("materials");
    let Ok(entries) = std::fs::read_dir(&materials_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect()
}

fn push_warning(warning: &mut Option<String>, next: impl Into<String>) {
    let next = next.into();
    match warning {
        Some(existing) => {
            existing.push('\n');
            existing.push_str(&next);
        }
        None => *warning = Some(next),
    }
}

fn finalize_analysis(
    ext_dir: &std::path::Path,
    version_dir: &std::path::Path,
    version_id: &str,
    author: &str,
    email: &str,
    manifest: &mut VersionManifest,
    summary_path: &std::path::Path,
) -> Result<()> {
    manifest.is_analyzed = true;
    manifest
        .write_to(version_dir)
        .with_context(|| "Failed to rewrite manifest.json after analysis")?;

    let manifest_path = version_dir.join("manifest.json");
    git_add(ext_dir, &[&manifest_path, summary_path])
        .with_context(|| "Failed to stage analysis summary")?;
    git_commit(
        ext_dir,
        &format!("ext: analysis results {}", version_id),
        author,
        email,
    )
    .with_context(|| "Failed to commit analysis summary")?;
    Ok(())
}
