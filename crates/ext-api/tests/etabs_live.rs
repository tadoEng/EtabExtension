//! Live ETABS integration tests.
//!
//! These tests are opt-in and require a real ETABS install plus the external
//! sidecar binary. They never run as part of the default `cargo test`.
//!
//! Run with:
//!   cargo test -p ext-api --test etabs_live -- --include-ignored --test-threads=1 --nocapture
//!
//! Required environment via workspace-root `.env` or process env:
//!   EXT_ETABS_SIDECAR_PATH
//!   EXT_ETABS_SOURCE_EDB
//!   EXT_ETABS_TEST_OUTPUT
//!   EXT_ETABS_UNITS

use chrono::{Duration, Utc};
use ext_api::AppContext;
use ext_api::analyze::{AnalyzeOptions, analyze_version};
use ext_api::branch;
use ext_api::checkout::{CheckoutOptions, checkout_version};
use ext_api::commit::{self, CommitOptions};
use ext_api::diff;
use ext_api::etabs::{CloseMode, etabs_close, etabs_open, etabs_status};
use ext_api::init::{InitRequest, init_project};
use ext_api::log;
use ext_api::stash::{self, StashPopConflictResolution, StashPopOptions};
use ext_api::switch;
use ext_core::state::WorkingFileStatus;
use ext_core::version::VersionManifest;
use ext_db::config::Config;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

#[derive(Debug, Clone)]
struct LiveEnv {
    sidecar_path: PathBuf,
    source_edb: PathBuf,
    test_output: PathBuf,
    units: String,
    enable_hidden_mode: bool,
}

struct LiveProject {
    _temp: TempDir,
    project_root: PathBuf,
    ctx: AppContext,
}

impl LiveProject {
    fn ext_dir(&self) -> PathBuf {
        self.project_root.join(".etabs-ext")
    }
}

fn materials_dir_has_parquet(materials_dir: &Path) -> bool {
    std::fs::read_dir(materials_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .any(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("parquet"))
        })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn load_dotenv() -> &'static HashMap<String, String> {
    static DOTENV: OnceLock<HashMap<String, String>> = OnceLock::new();
    DOTENV.get_or_init(|| {
        let env_path = workspace_root().join(".env");
        if !env_path.exists() {
            return HashMap::new();
        }

        std::fs::read_to_string(&env_path)
            .ok()
            .map(|contents| {
                contents
                    .lines()
                    .filter_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') {
                            return None;
                        }
                        trimmed
                            .split_once('=')
                            .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .or_else(|| load_dotenv().get(key).cloned())
}

fn env_bool(key: &str, default: bool) -> bool {
    match env_value(key) {
        Some(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => default,
    }
}

fn load_live_env() -> Option<LiveEnv> {
    Some(LiveEnv {
        sidecar_path: PathBuf::from(env_value("EXT_ETABS_SIDECAR_PATH")?),
        source_edb: PathBuf::from(env_value("EXT_ETABS_SOURCE_EDB")?),
        test_output: PathBuf::from(env_value("EXT_ETABS_TEST_OUTPUT")?),
        units: env_value("EXT_ETABS_UNITS").unwrap_or_else(|| "kip-ft-F".to_string()),
        enable_hidden_mode: env_bool("EXT_ETABS_ENABLE_HIDDEN_MODE", false),
    })
}

async fn init_live_project(env: &LiveEnv) -> LiveProject {
    std::fs::create_dir_all(&env.test_output).expect("create live test output");
    let temp = TempDir::new_in(&env.test_output).expect("tempdir in live test output");
    let project_root = temp.path().join("project");
    let copied_edb = temp.path().join("source").join("model.edb");
    std::fs::create_dir_all(copied_edb.parent().unwrap()).expect("source dir");
    std::fs::copy(&env.source_edb, &copied_edb).expect("copy source edb");

    init_project(InitRequest {
        name: "LiveTest".to_string(),
        edb_path: copied_edb,
        project_path: project_root.clone(),
        author: Some("Live Tester".to_string()),
        email: Some("live@test.local".to_string()),
        one_drive_dir: None,
        reports_dir: None,
        allow_onedrive: true,
    })
    .await
    .expect("init project");

    let mut config = Config::load(&project_root).expect("load config");
    config.project.sidecar_path = Some(env.sidecar_path.display().to_string());
    config.project.units = Some(env.units.clone());
    Config::write_local(&project_root, &config).expect("write local config");

    let ctx = AppContext::new(&project_root).expect("app context");
    ensure_etabs_idle(&ctx).await;

    LiveProject {
        _temp: temp,
        project_root,
        ctx,
    }
}

async fn ensure_etabs_idle(ctx: &AppContext) {
    let sidecar = ctx.require_sidecar().expect("sidecar");
    let status = sidecar.get_status().await.expect("sidecar status");
    if status.is_running && status.is_model_open {
        let _ = sidecar.close_model(false).await;
    }
}

fn git_log_messages(ext_dir: &Path, branch: &str) -> Vec<String> {
    let branch_ref = format!("refs/heads/{branch}");
    let output = Command::new("git")
        .args(["log", "--format=%s", &branch_ref])
        .current_dir(ext_dir)
        .output()
        .expect("git log");
    assert!(output.status.success(), "git log failed");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn mark_working_file_modified(ctx: &AppContext, bytes: &[u8]) -> PathBuf {
    let mut state = ctx.load_state().expect("load state");
    let working = state.working_file.as_mut().expect("working file");
    let path = working.path.clone();
    std::fs::write(&path, bytes).expect("write working file");
    let current_mtime: chrono::DateTime<chrono::Utc> = std::fs::metadata(&path)
        .expect("working metadata")
        .modified()
        .expect("working modified")
        .into();
    working.last_known_mtime = Some(current_mtime - Duration::seconds(60));
    working.etabs_pid = None;
    working.status_changed_at = Utc::now();
    ctx.save_state(&state).expect("save state");
    path
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live ETABS install and EXT_ETABS_* environment"]
async fn live_week34_vcs_and_commit_analyze_cycle() {
    let Some(env) = load_live_env() else {
        eprintln!("SKIP: EXT_ETABS_* environment not configured");
        return;
    };

    let project = init_live_project(&env).await;
    let ctx = &project.ctx;
    let ext_dir = project.ext_dir();
    let hidden_mode = env.enable_hidden_mode;

    let commit_v1 = commit::commit_version(
        ctx,
        "Initial model",
        CommitOptions {
            no_e2k: !hidden_mode,
            analyze: false,
        },
    )
    .await
    .expect("commit v1");
    assert_eq!(commit_v1.version_id, "v1");
    assert_eq!(commit_v1.branch, "main");
    if hidden_mode {
        assert!(
            commit_v1.e2k_generated,
            "hidden-mode export failed: {:?}",
            commit_v1.warning
        );
        assert!(
            commit_v1.materials_extracted,
            "hidden-mode materials extraction failed: {:?}",
            commit_v1.warning
        );
    } else {
        eprintln!("SKIP hidden-mode export/assertions for v1 (EXT_ETABS_ENABLE_HIDDEN_MODE=false)");
        assert!(!commit_v1.e2k_generated);
        assert!(!commit_v1.materials_extracted);
    }
    let main_v1_dir = ext_dir.join("main").join("v1");
    assert!(main_v1_dir.join("manifest.json").exists());
    if hidden_mode {
        assert!(main_v1_dir.join("model.e2k").exists());
        assert!(materials_dir_has_parquet(&main_v1_dir.join("materials")));
    }

    let shown_v1 = log::show_version(ctx, "main/v1")
        .await
        .expect("show main/v1");
    assert_eq!(shown_v1.manifest.id, "v1");
    assert_eq!(shown_v1.manifest.branch, "main");
    assert!(!shown_v1.manifest.is_analyzed);

    let main_log = log::list_versions(ctx, Some("main"), false)
        .await
        .expect("main log");
    assert_eq!(main_log.commits.len(), 1);
    assert_eq!(main_log.commits[0].version_id.as_deref(), Some("v1"));

    let branch_result = branch::create_branch(ctx, "steel-columns", Some("main/v1"))
        .await
        .expect("create branch");
    assert_eq!(branch_result.name, "steel-columns");
    assert!(branch_result.working_model_path.exists());

    let switched = switch::switch_branch(ctx, "steel-columns")
        .await
        .expect("switch branch");
    assert_eq!(switched.branch, "steel-columns");
    assert_eq!(switched.arrival_status, WorkingFileStatus::Clean);

    let commit_v2 = commit::commit_version(
        ctx,
        "Steel option",
        CommitOptions {
            no_e2k: !hidden_mode,
            analyze: hidden_mode,
        },
    )
    .await
    .expect("commit v2 analyze");
    assert_eq!(commit_v2.version_id, "v2");
    if hidden_mode {
        assert!(
            commit_v2.e2k_generated,
            "hidden-mode export failed: {:?}",
            commit_v2.warning
        );
        assert!(
            commit_v2.materials_extracted,
            "hidden-mode materials extraction failed: {:?}",
            commit_v2.warning
        );
        assert!(
            commit_v2.analyzed,
            "hidden-mode analysis failed: {:?}",
            commit_v2.warning
        );
    } else {
        eprintln!(
            "SKIP hidden-mode export/analyze assertions for v2 (EXT_ETABS_ENABLE_HIDDEN_MODE=false)"
        );
        assert!(!commit_v2.e2k_generated);
        assert!(!commit_v2.materials_extracted);
        assert!(!commit_v2.analyzed);
    }

    let steel_v2_dir = ext_dir.join("steel-columns").join("v2");
    assert!(steel_v2_dir.join("manifest.json").exists());
    if hidden_mode {
        assert!(steel_v2_dir.join("model.e2k").exists());
        assert!(materials_dir_has_parquet(&steel_v2_dir.join("materials")));
        assert!(steel_v2_dir.join("summary.json").exists());
        assert!(steel_v2_dir.join("results").exists());
    }

    let shown_v2 = log::show_version(ctx, "steel-columns/v2")
        .await
        .expect("show steel-columns/v2");
    if hidden_mode {
        assert!(shown_v2.manifest.is_analyzed);
        assert!(shown_v2.analysis.is_some());
    } else {
        assert!(!shown_v2.manifest.is_analyzed);
        assert!(shown_v2.analysis.is_none());
    }

    let steel_log = log::list_versions(ctx, Some("steel-columns"), false)
        .await
        .expect("steel log");
    assert!(
        steel_log
            .commits
            .iter()
            .any(|commit| commit.version_id.as_deref() == Some("v2"))
    );
    assert!(
        steel_log
            .commits
            .iter()
            .any(|commit| commit.version_id.as_deref() == Some("v1"))
    );

    let raw_messages = git_log_messages(&ext_dir, "steel-columns");
    assert!(raw_messages.iter().any(|msg| msg == "Steel option"));
    if hidden_mode {
        assert!(
            raw_messages
                .iter()
                .any(|msg| msg == "ext: analysis results v2")
        );
    } else {
        assert!(
            !raw_messages
                .iter()
                .any(|msg| msg == "ext: analysis results v2")
        );
    }

    let diff_result = diff::diff_versions(ctx, "main/v1", "steel-columns/v2")
        .await
        .expect("diff versions");
    if hidden_mode {
        assert!(diff_result.no_e2k_warning.is_none());
    } else {
        assert!(diff_result.no_e2k_warning.is_some());
        assert!(diff_result.diff_text.is_empty());
    }

    let checkout = checkout_version(ctx, "main/v1", CheckoutOptions::default())
        .await
        .expect("checkout main/v1");
    assert_eq!(checkout.branch, "main");
    assert_eq!(checkout.version_id, "v1");
    assert!(checkout.working_model_path.exists());

    let stashed_bytes = b"main-wip";
    let working_file = mark_working_file_modified(ctx, stashed_bytes);
    let stash_save = stash::stash_save(ctx, Some("main WIP"), false)
        .await
        .expect("stash save");
    assert_eq!(stash_save.branch, "main");
    assert!(stash_save.stash_path.exists());

    mark_working_file_modified(ctx, b"main-overwrite-me");
    let stash_pop = stash::stash_pop(
        ctx,
        StashPopOptions {
            conflict_resolution: Some(StashPopConflictResolution::Overwrite),
        },
    )
    .await
    .expect("stash pop");
    assert_eq!(stash_pop.branch, "main");
    assert_eq!(
        std::fs::read(&working_file).expect("read working"),
        stashed_bytes
    );
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live ETABS install and EXT_ETABS_* environment"]
async fn live_week56_etabs_status_and_analyze_version_cycle() {
    let Some(env) = load_live_env() else {
        eprintln!("SKIP: EXT_ETABS_* environment not configured");
        return;
    };

    let project = init_live_project(&env).await;
    let ctx = &project.ctx;
    let hidden_mode = env.enable_hidden_mode;

    let commit_v1 = commit::commit_version(
        ctx,
        "Initial model",
        CommitOptions {
            no_e2k: !hidden_mode,
            analyze: false,
        },
    )
    .await
    .expect("commit v1");
    assert_eq!(commit_v1.version_id, "v1");

    let open_working = etabs_open(ctx, None, true).await.expect("open working");
    assert!(open_working.pid > 0);
    assert!(!open_working.is_snapshot);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let working_status = etabs_status(ctx).await.expect("etabs status");
    assert!(working_status.sidecar_available);
    assert!(working_status.is_running);
    assert!(working_status.pid.is_some());

    let close_working = etabs_close(ctx, CloseMode::NoSave)
        .await
        .expect("close working");
    assert!(!close_working.saved);

    let manifest = VersionManifest::read_from(&project.ext_dir().join("main").join("v1"))
        .expect("manifest v1");
    if hidden_mode {
        let analyze = analyze_version(ctx, "v1", AnalyzeOptions::default())
            .await
            .expect("analyze v1");
        assert_eq!(analyze.version_id, "v1");
        assert!(!analyze.already_analyzed);
        assert!(analyze.results_dir.exists());
        let manifest = VersionManifest::read_from(&project.ext_dir().join("main").join("v1"))
            .expect("manifest v1");
        assert!(manifest.is_analyzed);

        let analyze_again = analyze_version(ctx, "v1", AnalyzeOptions::default())
            .await
            .expect("analyze v1 again");
        assert!(analyze_again.already_analyzed);
        assert!(analyze_again.warning.is_some());
    } else {
        eprintln!(
            "SKIP standalone analyze_version assertions (EXT_ETABS_ENABLE_HIDDEN_MODE=false)"
        );
        assert!(!manifest.is_analyzed);
    }

    let open_snapshot = etabs_open(ctx, Some("v1"), true)
        .await
        .expect("open snapshot");
    assert!(open_snapshot.is_snapshot);

    let close_snapshot = etabs_close(ctx, CloseMode::NoSave)
        .await
        .expect("close snapshot");
    assert!(!close_snapshot.saved);

    let state = ctx.load_state().expect("state");
    assert!(state.working_file.as_ref().unwrap().etabs_pid.is_none());
}
