#[path = "../../test-support/fake_sidecar.rs"]
mod fake_sidecar;

use chrono::Utc;
use ext_api::AppContext;
use ext_api::analyze::{AnalyzeOptions, analyze_version};
use ext_api::commit::{self, CommitOptions};
use ext_api::init::{InitRequest, init_project};
use ext_core::version::VersionManifest;
use ext_db::config::{Config, TableConfig};
use std::process::Command;
use tempfile::TempDir;

fn write_dummy_edb(path: &std::path::Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

async fn init_fixture(temp: &TempDir) -> std::path::PathBuf {
    let project_root = temp.path().join("project");
    let edb = temp.path().join("source").join("model.edb");
    write_dummy_edb(&edb, b"initial-edb");

    init_project(InitRequest {
        name: "Fixture".to_string(),
        edb_path: edb,
        project_path: project_root.clone(),
        author: Some("Tester".to_string()),
        email: Some("tester@example.com".to_string()),
        one_drive_dir: None,
        reports_dir: None,
        allow_onedrive: true,
    })
    .await
    .unwrap();

    project_root
}

fn git_log_messages(ext_dir: &std::path::Path, branch: &str) -> Vec<String> {
    let branch_ref = format!("refs/heads/{branch}");
    let output = Command::new("git")
        .args(["log", "--format=%s", &branch_ref])
        .current_dir(ext_dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[tokio::test]
async fn analyze_version_blocked_when_etabs_open() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let ctx = AppContext::new(&project_root).unwrap();

    commit::commit_version(
        &ctx,
        "Initial",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.etabs_pid = Some(std::process::id());
    working.status_changed_at = Utc::now();
    ctx.save_state(&state).unwrap();

    let err = analyze_version(&ctx, "v1", AnalyzeOptions::default())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Close ETABS"));
}

#[tokio::test]
async fn analyze_version_allowed_when_working_file_missing() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let ctx = AppContext::new(&project_root).unwrap();

    commit::commit_version(
        &ctx,
        "Initial",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    let working_path = working.path.clone();
    std::fs::remove_file(&working_path).unwrap();
    ctx.save_state(&state).unwrap();

    let result = analyze_version(&ctx, "v1", AnalyzeOptions::default())
        .await
        .unwrap();
    assert_eq!(result.version_id, "v1");
    assert!(!result.already_analyzed);
    assert!(
        project_root
            .join(".etabs-ext")
            .join("main")
            .join("v1")
            .join("summary.json")
            .exists()
    );
}

#[tokio::test]
async fn analyze_version_returns_already_analyzed_without_force() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let ctx = AppContext::new(&project_root).unwrap();
    let ext_dir = project_root.join(".etabs-ext");

    commit::commit_version(
        &ctx,
        "Initial analyzed",
        CommitOptions {
            no_e2k: true,
            analyze: true,
        },
    )
    .await
    .unwrap();

    let before = git_log_messages(&ext_dir, "main");
    let result = analyze_version(&ctx, "v1", AnalyzeOptions::default())
        .await
        .unwrap();
    let after = git_log_messages(&ext_dir, "main");

    assert!(result.already_analyzed);
    assert!(
        result
            .warning
            .as_deref()
            .unwrap_or_default()
            .contains("--force")
    );
    assert_eq!(before, after);
}

#[tokio::test]
async fn analyze_version_force_reruns_and_commits_metadata() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let ctx = AppContext::new(&project_root).unwrap();
    let ext_dir = project_root.join(".etabs-ext");

    commit::commit_version(
        &ctx,
        "Initial",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();

    let result = analyze_version(
        &ctx,
        "v1",
        AnalyzeOptions {
            force: true,
            cases: Some(vec!["DEAD".to_string(), "LIVE".to_string()]),
        },
    )
    .await
    .unwrap();

    let manifest =
        VersionManifest::read_from(&project_root.join(".etabs-ext").join("main").join("v1"))
            .unwrap();
    let messages = git_log_messages(&ext_dir, "main");

    assert_eq!(result.version_id, "v1");
    assert!(!result.already_analyzed);
    assert!(manifest.is_analyzed);
    assert!(messages.iter().any(|msg| msg == "ext: analysis results v1"));
}

#[tokio::test]
async fn analyze_version_forwards_new_extract_table_selectors_to_sidecar() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);

    let mut config = Config::load(&project_root).unwrap();
    config.extract.tables.group_assignments = Some(TableConfig {
        groups: Some(vec!["Core".to_string()]),
        ..TableConfig::default()
    });
    config.extract.tables.material_properties_concrete_data = Some(TableConfig {
        field_keys: Some(vec!["Fc".to_string(), "Ec".to_string()]),
        ..TableConfig::default()
    });
    config.extract.tables.material_list_by_story = Some(TableConfig {
        field_keys: Some(vec!["Story".to_string()]),
        ..TableConfig::default()
    });
    Config::write_shared(&project_root, &config).unwrap();

    let ctx = AppContext::new(&project_root).unwrap();

    commit::commit_version(
        &ctx,
        "Initial",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();

    analyze_version(&ctx, "v1", AnalyzeOptions::default())
        .await
        .unwrap();

    let request = fake_sidecar::read_extract_results_request(&sidecar);

    assert!(request.contains("groupAssignments"));
    assert!(request.contains("materialPropertiesConcreteData"));
    assert!(request.contains("materialListByStory"));
    assert!(request.contains("Core"));
    assert!(request.contains("Fc"));
    assert!(request.contains("Ec"));
    assert!(request.contains("Story"));
}
