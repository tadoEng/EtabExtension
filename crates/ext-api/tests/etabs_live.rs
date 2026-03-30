//! Live ETABS integration tests.
//!
//! These tests are opt-in and require a real ETABS install plus the external
//! sidecar binary. They never run as part of the default `cargo test`.
//!
//! Run with:
//!   cargo test -p ext-api --test etabs_live -- --include-ignored --test-threads=1

use ext_api::AppContext;
use ext_api::commit::{self, CommitOptions};
use ext_api::etabs::{CloseMode, etabs_close, etabs_open, etabs_status};
use ext_api::init::{InitRequest, init_project};
use ext_db::config::Config;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tempfile::TempDir;

#[derive(Debug, Clone)]
struct LiveEnv {
    sidecar_path: PathBuf,
    source_edb: PathBuf,
    test_output: PathBuf,
    units: String,
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

fn load_live_env() -> Option<LiveEnv> {
    Some(LiveEnv {
        sidecar_path: PathBuf::from(env_value("EXT_ETABS_SIDECAR_PATH")?),
        source_edb: PathBuf::from(env_value("EXT_ETABS_SOURCE_EDB")?),
        test_output: PathBuf::from(env_value("EXT_ETABS_TEST_OUTPUT")?),
        units: env_value("EXT_ETABS_UNITS").unwrap_or_else(|| "kip-ft-F".to_string()),
    })
}

async fn init_live_project(env: &LiveEnv) -> AppContext {
    std::fs::create_dir_all(&env.test_output).expect("create live test output");
    let temp = TempDir::new_in(&env.test_output).expect("tempdir in live test output");
    let temp_path = temp.keep();

    let project_root = temp_path.join("project");
    let copied_edb = temp_path.join("source").join("model.edb");
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

    AppContext::new(&project_root).expect("app context")
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live ETABS install and EXT_ETABS_* environment"]
async fn live_open_and_close_cycle() {
    let Some(env) = load_live_env() else {
        eprintln!("SKIP: EXT_ETABS_* environment not configured");
        return;
    };

    let ctx = init_live_project(&env).await;

    let open_result = etabs_open(&ctx, None).await.expect("open working model");
    assert!(open_result.pid > 0);
    assert!(!open_result.is_snapshot);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let status = etabs_status(&ctx).await.expect("status");
    assert!(status.sidecar_available);
    assert!(status.is_running);

    let close_result = etabs_close(&ctx, CloseMode::NoSave)
        .await
        .expect("close working model");
    assert!(!close_result.saved);

    let state = ctx.load_state().expect("state");
    assert!(state.working_file.as_ref().unwrap().etabs_pid.is_none());
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live ETABS install and EXT_ETABS_* environment"]
async fn live_commit_and_open_snapshot_cycle() {
    let Some(env) = load_live_env() else {
        eprintln!("SKIP: EXT_ETABS_* environment not configured");
        return;
    };

    let ctx = init_live_project(&env).await;

    let commit = commit::commit_version(
        &ctx,
        "Live test commit",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .expect("commit version");
    assert_eq!(commit.version_id, "v1");

    let open_result = etabs_open(&ctx, Some("v1")).await.expect("open snapshot");
    assert!(open_result.is_snapshot);

    let close_result = etabs_close(&ctx, CloseMode::NoSave)
        .await
        .expect("close snapshot");
    assert!(!close_result.saved);

    let state = ctx.load_state().expect("state");
    assert!(state.working_file.as_ref().unwrap().etabs_pid.is_none());
}
