use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use ext_core::state::WorkingFileStatus;
use ext_db::{
    StateFile, WorkingFileState,
    config::{Config, GitConfig, OneDriveConfig, PathsConfig},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::path_utils::normalize_path;

const GITIGNORE_CONTENT: &str = r#"*.edb
*.parquet
*/working/
state.json
config.local.toml
stash/
*.edb.lock
*.$et
*.mdb
*.OUT
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitRequest {
    pub name: String,
    pub edb_path: PathBuf,
    pub project_path: PathBuf,
    pub author: Option<String>,
    pub email: Option<String>,
    pub one_drive_dir: Option<PathBuf>,
    pub reports_dir: Option<PathBuf>,
    pub allow_onedrive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitResult {
    pub project_root: PathBuf,
    pub ext_dir: PathBuf,
    pub working_model_path: PathBuf,
    pub onedrive_detected: bool,
}

fn to_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn file_mtime_utc(path: &Path) -> Result<DateTime<Utc>> {
    let modified = std::fs::metadata(path)
        .with_context(|| format!("Failed to read metadata: {}", path.display()))?
        .modified()
        .with_context(|| format!("Failed to read mtime: {}", path.display()))?;
    Ok(modified.into())
}

fn run_git(repo: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        if detail.is_empty() {
            bail!("git command failed: git {}", args.join(" "));
        }
        bail!("git command failed: git {}\n  {}", args.join(" "), detail);
    }
    Ok(())
}

fn git_init(repo: &Path) -> Result<()> {
    let preferred = Command::new("git")
        .current_dir(repo)
        .args(["init", "--initial-branch=main"])
        .output();

    match preferred {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) | Err(_) => run_git(repo, &["init"]),
    }
}

fn atomic_copy(src: &Path, dst: &Path) -> Result<()> {
    let tmp = dst.with_extension("edb.tmp");
    std::fs::copy(src, &tmp).with_context(|| {
        format!(
            "Failed to copy model from {} to {}",
            src.display(),
            tmp.display()
        )
    })?;
    std::fs::rename(&tmp, dst)
        .with_context(|| format!("Failed to move {} to {}", tmp.display(), dst.display()))?;
    Ok(())
}

pub fn is_onedrive_path(path: &Path) -> bool {
    let markers = ["OneDrive", "OneDrive - ", "SharePoint"];
    path.ancestors().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| markers.iter().any(|m| n.starts_with(m)))
            .unwrap_or(false)
    })
}

pub async fn init_project(req: InitRequest) -> Result<InitResult> {
    let edb_path = to_absolute(&req.edb_path)?;
    if !edb_path.is_file() {
        bail!("EDB file not found: {}", edb_path.display());
    }
    if edb_path.extension().and_then(|s| s.to_str()) != Some("edb") {
        bail!(
            "Input file must have .edb extension: {}",
            edb_path.display()
        );
    }

    let project_root = to_absolute(&req.project_path)?;
    let ext_dir = Config::config_dir(&project_root);
    if ext_dir.exists() {
        bail!(
            "Project already initialized at {}\n  Run: ext status",
            project_root.display()
        );
    }

    let onedrive_detected = is_onedrive_path(&edb_path) || is_onedrive_path(&project_root);
    if onedrive_detected && !req.allow_onedrive {
        bail!(
            "Project path is inside OneDrive-synced folder\n  \
             Pass --allow-onedrive to continue anyway"
        );
    }

    let working_dir = ext_dir.join("main").join("working");
    std::fs::create_dir_all(&working_dir)
        .with_context(|| format!("Failed to create {}", working_dir.display()))?;

    let working_model_path = working_dir.join("model.edb");
    atomic_copy(&edb_path, &working_model_path)?;

    let mut shared = Config::default();
    shared.project.name = Some(req.name.clone());
    Config::write_shared(&project_root, &shared)?;

    let mut local = Config::default();
    local.git = GitConfig {
        author: req.author.clone(),
        email: req.email.clone(),
    };
    local.paths = PathsConfig {
        one_drive_dir: req.one_drive_dir.as_ref().map(|p| p.display().to_string()),
        reports_dir: req.reports_dir.as_ref().map(|p| p.display().to_string()),
    };
    local.onedrive = OneDriveConfig {
        acknowledged_sync: Some(!onedrive_detected || req.allow_onedrive),
    };
    Config::write_local(&project_root, &local)?;

    std::fs::write(ext_dir.join(".gitignore"), GITIGNORE_CONTENT)
        .with_context(|| "Failed to write .gitignore".to_string())?;

    let now = Utc::now();
    let working_state = WorkingFileState {
        path: normalize_path(
            &working_model_path
                .canonicalize()
                .unwrap_or_else(|_| working_model_path.clone()),
        ),
        status: WorkingFileStatus::Untracked,
        etabs_pid: None,
        last_commit_hash: None,
        based_on_version: None,
        last_known_mtime: Some(file_mtime_utc(&working_model_path)?),
        status_changed_at: now,
    };
    let state = StateFile {
        schema_version: ext_db::state::STATE_SCHEMA_VERSION,
        working_file: Some(working_state),
        updated_at: now,
    };
    state.save(&project_root)?;

    git_init(&ext_dir)?;
    run_git(&ext_dir, &["config", "core.autocrlf", "false"])?;

    let loaded = Config::load(&project_root)?;
    run_git(
        &ext_dir,
        &["config", "user.name", loaded.git.author_or_default()],
    )?;
    run_git(
        &ext_dir,
        &["config", "user.email", loaded.git.email_or_default()],
    )?;
    run_git(&ext_dir, &["add", "config.toml", ".gitignore"])?;
    run_git(&ext_dir, &["commit", "-m", "ext: init project"])?;

    Ok(InitResult {
        project_root: normalize_path(&project_root),
        ext_dir: normalize_path(&ext_dir),
        working_model_path: normalize_path(&working_model_path),
        onedrive_detected,
    })
}
