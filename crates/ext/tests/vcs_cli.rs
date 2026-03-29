#[path = "../../test-support/fake_sidecar.rs"]
mod fake_sidecar;

use ext_api::init::{InitRequest, init_project};
use snapbox::str;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

fn ext_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_ext")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_ext not set")
}

fn run_ext(args: &[&str]) -> Output {
    Command::new(ext_bin()).args(args).output().unwrap()
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
    text.replace(&native, "<PROJECT>")
        .replace(&slash, "<PROJECT>")
}

#[tokio::test]
async fn cli_vcs_smoke_json_and_human() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let project = project_root.to_str().unwrap();

    let commit = run_ext(&[
        "--json",
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
    let commit_json: serde_json::Value = serde_json::from_slice(&commit.stdout).unwrap();
    assert_eq!(commit_json["versionId"], "v1");

    let create_branch = run_ext(&[
        "--project-path",
        project,
        "branch",
        "steel-columns",
        "--from",
        "main/v1",
    ]);
    assert!(
        create_branch.status.success(),
        "{}",
        String::from_utf8_lossy(&create_branch.stderr)
    );
    assert!(stdout_text(&create_branch).contains("Created branch steel-columns"));

    let switch = run_ext(&["--project-path", project, "switch", "steel-columns"]);
    assert!(
        switch.status.success(),
        "{}",
        String::from_utf8_lossy(&switch.stderr)
    );
    assert!(stdout_text(&switch).contains("Switched to steel-columns"));

    let log = run_ext(&[
        "--json",
        "--project-path",
        project,
        "log",
        "--branch",
        "main",
    ]);
    assert!(
        log.status.success(),
        "{}",
        String::from_utf8_lossy(&log.stderr)
    );
    let log_json: serde_json::Value = serde_json::from_slice(&log.stdout).unwrap();
    assert_eq!(log_json["branch"], "main");
    assert_eq!(log_json["commits"][0]["versionId"], "v1");

    let show = run_ext(&["--json", "--project-path", project, "show", "main/v1"]);
    assert!(
        show.status.success(),
        "{}",
        String::from_utf8_lossy(&show.stderr)
    );
    let show_json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_json["manifest"]["id"], "v1");
}

#[tokio::test]
async fn cli_commit_human_output_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let project = project_root.to_str().unwrap();

    let output = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Initial model",
        "--no-e2k",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    snapbox::assert_data_eq!(
        stdout_text(&output),
        str![[r#"
✓ Version v1 saved
  Branch: main  |  [..]
  E2K: skipped (--no-e2k)

"#]]
    );
}

#[tokio::test]
async fn cli_branch_json_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
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

    let create_branch = run_ext(&[
        "--project-path",
        project,
        "branch",
        "steel-columns",
        "--from",
        "main/v1",
    ]);
    assert!(
        create_branch.status.success(),
        "{}",
        String::from_utf8_lossy(&create_branch.stderr)
    );

    let output = run_ext(&["--json", "--project-path", project, "branch"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    snapbox::assert_data_eq!(
        stdout_text(&output),
        str![[r#"
{
  "branches": [
    {
      "name": "main",
      "versionCount": 1,
      "latestVersion": "v1",
      "createdFrom": null,
      "isActive": true
    },
    {
      "name": "steel-columns",
      "versionCount": 1,
      "latestVersion": "v1",
      "createdFrom": "main/v1",
      "isActive": false
    }
  ],
  "currentBranch": "main"
}

"#]]
    );
}

#[tokio::test]
async fn cli_shell_outputs_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
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

    let create_branch = run_ext(&[
        "--project-path",
        project,
        "branch",
        "steel-columns",
        "--from",
        "main/v1",
    ]);
    assert!(
        create_branch.status.success(),
        "{}",
        String::from_utf8_lossy(&create_branch.stderr)
    );

    let branch_output = run_ext(&["--shell", "--project-path", project, "branch"]);
    assert!(
        branch_output.status.success(),
        "{}",
        String::from_utf8_lossy(&branch_output.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&branch_output),
        str![[r#"
main
steel-columns

"#]]
    );

    let log_output = run_ext(&[
        "--shell",
        "--project-path",
        project,
        "log",
        "--branch",
        "main",
    ]);
    assert!(
        log_output.status.success(),
        "{}",
        String::from_utf8_lossy(&log_output.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&log_output),
        str![[r#"
v1

"#]]
    );
}

#[tokio::test]
async fn cli_branch_create_human_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
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

    let output = run_ext(&[
        "--project-path",
        project,
        "branch",
        "steel-columns",
        "--from",
        "main/v1",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    snapbox::assert_data_eq!(
        normalize_output(&stdout_text(&output), &project_root),
        str![[r#"
✓ Created branch steel-columns
  From: main/v1
  Working: <PROJECT>/.etabs-ext/steel-columns/working/model.edb

"#]]
    );
}

#[tokio::test]
async fn cli_commit_analyze_human_and_json() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();

    let json_output = run_ext(&[
        "--json",
        "--project-path",
        project,
        "commit",
        "Initial analysis complete",
        "--no-e2k",
        "--analyze",
    ]);
    assert!(
        json_output.status.success(),
        "{}",
        String::from_utf8_lossy(&json_output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    assert_eq!(json["versionId"], "v1");
    assert_eq!(json["analyzed"], true);
    assert_eq!(json["e2kGenerated"], false);
    assert!(
        project_root
            .join(".etabs-ext")
            .join("main")
            .join("v1")
            .join("summary.json")
            .exists()
    );

    let human_output = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Follow-up analysis",
        "--no-e2k",
        "--analyze",
    ]);
    assert!(
        human_output.status.success(),
        "{}",
        String::from_utf8_lossy(&human_output.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&human_output),
        str![[r#"
✓ Version v2 saved
  Branch: main  |  [..]
  E2K: skipped (--no-e2k)
  Analysis: captured

"#]]
    );
}

#[tokio::test]
async fn cli_commit_analyze_warning_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar =
        fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::AnalysisFail);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let project = project_root.to_str().unwrap();

    let output = run_ext(&[
        "--project-path",
        project,
        "commit",
        "Analysis attempt",
        "--no-e2k",
        "--analyze",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    snapbox::assert_data_eq!(
        normalize_output(&stdout_text(&output), &project_root),
        str![[r#"
⚠ Version saved, but analysis failed: run-analysis failed for <PROJECT>/.etabs-ext/main/v1/model.edb
✓ Version v1 saved
  Branch: main  |  [..]
  E2K: skipped (--no-e2k)
  Analysis: requested, but not finalized

"#]]
    );
}
