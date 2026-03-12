use chrono::{DateTime, Duration, Utc};
use ext_api::AppContext;
use ext_api::init::{InitRequest, init_project};
use ext_core::state::WorkingFileStatus;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn ext_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_ext")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_ext not set")
}

fn file_mtime(path: &Path) -> DateTime<Utc> {
    std::fs::metadata(path).unwrap().modified().unwrap().into()
}

fn run_ext(args: &[&str], stdin_data: Option<&str>) -> Output {
    let mut cmd = Command::new(ext_bin());
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(input) = stdin_data {
        cmd.stdin(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    } else {
        cmd.output().unwrap()
    }
}

fn init_fixture(temp: &TempDir) -> PathBuf {
    let project_root = temp.path().join("project");
    let edb = temp.path().join("source").join("model.edb");
    std::fs::create_dir_all(edb.parent().unwrap()).unwrap();
    std::fs::write(&edb, b"dummy-edb-content").unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime
        .block_on(init_project(InitRequest {
            name: "Fixture".to_string(),
            edb_path: edb,
            project_path: project_root.clone(),
            author: Some("Tester".to_string()),
            email: Some("tester@example.com".to_string()),
            one_drive_dir: None,
            reports_dir: None,
            allow_onedrive: true,
        }))
        .unwrap();
    project_root
}

fn assert_status_json(project_root: &Path, expected: WorkingFileStatus) {
    let output = run_ext(
        &[
            "--json",
            "--project-path",
            project_root.to_str().unwrap(),
            "status",
        ],
        None,
    );
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["workingStatus"],
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn status_human_and_json_cover_untracked_clean_modified_missing() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp);
    let ctx = AppContext::new(&project_root).unwrap();

    let out = run_ext(
        &["--project-path", project_root.to_str().unwrap(), "status"],
        None,
    );
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Working File: Untracked"));
    assert_status_json(&project_root, WorkingFileStatus::Untracked);

    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.based_on_version = Some("v1".to_string());
    wf.last_known_mtime = Some(file_mtime(&wf.path));
    ctx.save_state(&state).unwrap();
    let out = run_ext(
        &["--project-path", project_root.to_str().unwrap(), "status"],
        None,
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Working File: Clean"));
    assert_status_json(&project_root, WorkingFileStatus::Clean);

    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.last_known_mtime = wf.last_known_mtime.map(|t| t - Duration::seconds(60));
    ctx.save_state(&state).unwrap();
    let out = run_ext(
        &["--project-path", project_root.to_str().unwrap(), "status"],
        None,
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Working File: Modified"));
    assert_status_json(&project_root, WorkingFileStatus::Modified);

    let working = project_root
        .join(".etabs-ext")
        .join("main")
        .join("working")
        .join("model.edb");
    std::fs::remove_file(&working).unwrap();
    let out = run_ext(
        &["--project-path", project_root.to_str().unwrap(), "status"],
        None,
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Working File: Missing"));
    assert_status_json(&project_root, WorkingFileStatus::Missing);
}

#[test]
fn init_blocks_onedrive_path_in_non_human_mode_without_allow_flag() {
    let temp = TempDir::new().unwrap();
    let one_drive_project = temp.path().join("OneDrive - Team").join("project");
    let edb = temp.path().join("source").join("model.edb");
    std::fs::create_dir_all(edb.parent().unwrap()).unwrap();
    std::fs::write(&edb, b"dummy-edb-content").unwrap();

    let output = run_ext(
        &[
            "--json",
            "init",
            "Demo",
            "--edb",
            edb.to_str().unwrap(),
            "--path",
            one_drive_project.to_str().unwrap(),
        ],
        None,
    );

    assert!(
        !output.status.success(),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("OneDrive path detected"));
}

#[test]
fn init_human_prompts_and_allows_continue_for_onedrive_path() {
    let temp = TempDir::new().unwrap();
    let one_drive_project = temp.path().join("OneDrive - Team").join("project");
    let edb = temp.path().join("source").join("model.edb");
    std::fs::create_dir_all(edb.parent().unwrap()).unwrap();
    std::fs::write(&edb, b"dummy-edb-content").unwrap();

    let output = run_ext(
        &[
            "init",
            "Demo",
            "--edb",
            edb.to_str().unwrap(),
            "--path",
            one_drive_project.to_str().unwrap(),
        ],
        Some("c\n"),
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OneDrive path detected"));
    assert!(stdout.contains("Created project: Demo"));
}
