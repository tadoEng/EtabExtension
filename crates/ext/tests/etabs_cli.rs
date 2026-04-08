#[path = "../../test-support/fake_sidecar.rs"]
mod fake_sidecar;

use chrono::{Duration, Utc};
use ext_api::init::{InitRequest, init_project};
use snapbox::str;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn ext_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_ext")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_ext not set")
}

fn run_ext(args: &[&str]) -> Output {
    Command::new(ext_bin()).args(args).output().unwrap()
}

fn run_ext_with_input(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(ext_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

async fn init_fixture(temp: &TempDir) -> PathBuf {
    let project_root = temp.path().join("project");
    let edb = temp.path().join("source").join("model.edb");
    std::fs::create_dir_all(edb.parent().unwrap()).unwrap();
    std::fs::write(&edb, b"dummy-edb-content").unwrap();

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

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn normalize_output(text: &str, project_root: &std::path::Path) -> String {
    let native = project_root.display().to_string();
    let slash = native.replace('\\', "/");
    let double_slash = native.replace('\\', "//");
    let escaped = native.replace('\\', "\\\\");
    let mut normalized = text
        .replace(&native, "<PROJECT>")
        .replace(&slash, "<PROJECT>")
        .replace(&double_slash, "<PROJECT>")
        .replace(&escaped, "<PROJECT>")
        .replace(&std::process::id().to_string(), "<PID>")
        .replace("4242", "<PID>");
    normalized = normalized.replace('\\', "/");
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    normalized
}

#[tokio::test]
async fn cli_etabs_open_human_json_and_shell() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    fake_sidecar::set_fake_sidecar_state(
        &sidecar,
        &fake_sidecar::FakeSidecarState {
            pid: Some(std::process::id()),
            ..Default::default()
        },
    );
    let project = project_root.to_str().unwrap();

    let commit = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(
        commit.status.success(),
        "{}",
        String::from_utf8_lossy(&commit.stderr)
    );

    let human = run_ext(&["--project-path", project, "etabs", "open"]);
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    snapbox::assert_data_eq!(
        normalize_output(&stdout_text(&human), &project_root),
        str![[r#"
✓ ETABS opened (PID: <PID>)
  File: <PROJECT>/.etabs-ext/main/working/model.edb

"#]]
    );
    let fake_state_text = std::fs::read_to_string(fake_sidecar::sidecar_state_path(&sidecar)).unwrap();
    assert!(fake_state_text.contains("last_open_new_instance=1"));

    let close_working = run_ext(&["--project-path", project, "etabs", "close", "--no-save"]);
    assert!(
        close_working.status.success(),
        "{}",
        String::from_utf8_lossy(&close_working.stderr)
    );

    let json = run_ext(&["--json", "--project-path", project, "etabs", "open", "v1"]);
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let json_value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(
        normalize_output(json_value["openedFile"].as_str().unwrap(), &project_root),
        "<PROJECT>/.etabs-ext/main/v1/model.edb"
    );
    assert_eq!(json_value["pid"], std::process::id());
    assert_eq!(json_value["isSnapshot"], true);
    assert_eq!(
        json_value["warning"],
        "Opening a snapshot — changes will be discarded"
    );

    let close_snapshot = run_ext(&["--project-path", project, "etabs", "close", "--no-save"]);
    assert!(
        close_snapshot.status.success(),
        "{}",
        String::from_utf8_lossy(&close_snapshot.stderr)
    );

    let shell = run_ext(&["--shell", "--project-path", project, "etabs", "open"]);
    assert!(
        shell.status.success(),
        "{}",
        String::from_utf8_lossy(&shell.stderr)
    );
    snapbox::assert_data_eq!(
        normalize_output(&stdout_text(&shell), &project_root),
        str![[r#"
<PID>

"#]]
    );
}

#[tokio::test]
async fn cli_etabs_close_and_status_snapshots() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();
    let commit = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(
        commit.status.success(),
        "{}",
        String::from_utf8_lossy(&commit.stderr)
    );
    let ctx = ext_api::AppContext::new(&project_root).unwrap();
    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    let working_path = working.path.clone();
    working.etabs_pid = Some(std::process::id());
    working.last_known_mtime = working.last_known_mtime.map(|t| t - Duration::seconds(60));
    working.status = ext_core::state::WorkingFileStatus::OpenModified;
    ctx.save_state(&state).unwrap();

    fake_sidecar::set_fake_sidecar_state(
        &sidecar,
        &fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            open_file_path: Some(working_path),
            is_model_open: true,
            is_locked: Some(false),
            is_analyzed: Some(false),
            ..Default::default()
        },
    );

    let close = run_ext(&["--project-path", project, "etabs", "close", "--save"]);
    assert!(
        close.status.success(),
        "{}",
        String::from_utf8_lossy(&close.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&close),
        str![[r#"
✓ ETABS model closed
  Saved: true
  Working Status: Modified

"#]]
    );

    let status = run_ext(&["--json", "--project-path", project, "etabs", "status"]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_json: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status_json["sidecarAvailable"], true);
    assert_eq!(status_json["workingFileStatus"], "clean");
}

#[tokio::test]
async fn cli_etabs_unlock_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();

    let output = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(output.status.success());

    let ctx = ext_api::AppContext::new(&project_root).unwrap();
    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.status = ext_core::state::WorkingFileStatus::Locked;
    working.etabs_pid = None;
    ctx.save_state(&state).unwrap();

    fake_sidecar::set_fake_sidecar_state(
        &sidecar,
        &fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            is_model_open: false,
            is_locked: Some(true),
            is_analyzed: Some(true),
            ..Default::default()
        },
    );

    let unlock = run_ext(&["--project-path", project, "etabs", "unlock"]);
    assert!(
        unlock.status.success(),
        "{}",
        String::from_utf8_lossy(&unlock.stderr)
    );
    snapbox::assert_data_eq!(
        normalize_output(&stdout_text(&unlock), &project_root),
        str![[r#"
✓ Model unlocked
  File: <PROJECT>/.etabs-ext/main/working/model.edb
  Reopened: true
  Working Status: Analyzed

"#]]
    );
}

#[tokio::test]
async fn cli_etabs_close_prompt_and_recover_prompt() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();
    let ctx = ext_api::AppContext::new(&project_root).unwrap();

    let commit = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(
        commit.status.success(),
        "{}",
        String::from_utf8_lossy(&commit.stderr)
    );

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.etabs_pid = Some(std::process::id());
    working.last_known_mtime = working.last_known_mtime.map(|t| t - Duration::seconds(60));
    working.status = ext_core::state::WorkingFileStatus::OpenModified;
    let working_path = working.path.clone();
    ctx.save_state(&state).unwrap();

    fake_sidecar::set_fake_sidecar_state(
        &sidecar,
        &fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            open_file_path: Some(working_path.clone()),
            is_model_open: true,
            is_locked: Some(false),
            is_analyzed: Some(false),
            ..Default::default()
        },
    );

    let close = run_ext_with_input(&["--project-path", project, "etabs", "close"], "d\n");
    assert!(
        close.status.success(),
        "{}",
        String::from_utf8_lossy(&close.stderr)
    );
    assert!(stdout_text(&close).contains("ETABS has unsaved changes"));
    assert!(stdout_text(&close).contains("Working Status: Clean"));

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    std::fs::write(&working.path, b"after-crash").unwrap();
    working.etabs_pid = Some(u32::MAX);
    working.status = ext_core::state::WorkingFileStatus::Orphaned;
    working.based_on_version = Some("v1".to_string());
    working.status_changed_at = Utc::now();
    ctx.save_state(&state).unwrap();

    let recover = run_ext_with_input(&["--project-path", project, "etabs", "recover"], "k\n");
    assert!(
        recover.status.success(),
        "{}",
        String::from_utf8_lossy(&recover.stderr)
    );
    assert!(stdout_text(&recover).contains("ETABS appears to have crashed"));
    assert!(stdout_text(&recover).contains("Recovery completed"));
}

#[tokio::test]
async fn cli_analyze_json_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();

    let commit = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(commit.status.success());

    let analyze = run_ext(&[
        "--json",
        "--project-path",
        project,
        "analyze",
        "v1",
        "--cases",
        "DEAD,LIVE",
    ]);
    assert!(
        analyze.status.success(),
        "{}",
        String::from_utf8_lossy(&analyze.stderr)
    );

    let json_value: serde_json::Value = serde_json::from_slice(&analyze.stdout).unwrap();
    assert_eq!(json_value["versionId"], "v1");
    assert_eq!(json_value["branch"], "main");
    assert_eq!(
        normalize_output(json_value["resultsDir"].as_str().unwrap(), &project_root),
        "<PROJECT>/.etabs-ext/main/v1/results"
    );
    assert!(json_value["elapsedMs"].as_u64().is_some());
    assert!(json_value["warning"].is_null());
    assert_eq!(json_value["alreadyAnalyzed"], false);
}
