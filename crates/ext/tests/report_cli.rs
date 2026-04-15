use ext_api::AppContext;
use ext_api::commit::{CommitOptions, commit_version};
use ext_api::init::{InitRequest, init_project};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn ext_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_ext")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_ext not set")
}

fn run_ext(args: &[&str]) -> Output {
    Command::new(ext_bin())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

fn fixture_results_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ext-calc/tests/fixtures/results_realistic")
}

fn copy_fixture_config(project_root: &Path) {
    let fixture_root = fixture_results_dir();
    let fixture_config_dir = fixture_root.join(".etabs-ext");
    let project_config_dir = project_root.join(".etabs-ext");

    std::fs::copy(
        fixture_config_dir.join("config.toml"),
        project_config_dir.join("config.toml"),
    )
    .unwrap();
    std::fs::copy(
        fixture_config_dir.join("config.local.toml"),
        project_config_dir.join("config.local.toml"),
    )
    .unwrap();
}

fn copy_fixture_results(project_root: &Path) {
    let source = fixture_results_dir();
    let target = project_root
        .join(".etabs-ext")
        .join("main")
        .join("v1")
        .join("results");
    std::fs::create_dir_all(&target).unwrap();

    for entry in std::fs::read_dir(&source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_string_lossy();
        if file_name == "calc_output.json" {
            continue;
        }
        std::fs::copy(&path, target.join(path.file_name().unwrap())).unwrap();
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

    let ctx = AppContext::new(&project_root).unwrap();
    runtime
        .block_on(commit_version(
            &ctx,
            "Initial",
            CommitOptions {
                no_e2k: true,
                analyze: false,
            },
        ))
        .unwrap();

    copy_fixture_config(&project_root);
    copy_fixture_results(&project_root);
    project_root
}

#[test]
fn cli_calc_render_report_json_outputs() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp);
    let project = project_root.to_str().unwrap();

    let calc = run_ext(&["--json", "--project-path", project, "calc", "v1"]);
    assert!(
        calc.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&calc.stderr)
    );
    let calc_json: serde_json::Value = serde_json::from_slice(&calc.stdout).unwrap();
    let calc_output = PathBuf::from(calc_json["calcOutputPath"].as_str().unwrap());
    assert!(calc_output.exists());

    let report = run_ext(&["--json", "--project-path", project, "report", "v1"]);
    assert!(
        report.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&report.stderr)
    );
    let report_json: serde_json::Value = serde_json::from_slice(&report.stdout).unwrap();
    let pdf_path = PathBuf::from(report_json["pdfPath"].as_str().unwrap());
    assert!(pdf_path.exists());
    assert_eq!(report_json["theme"].as_str(), Some("tabloid"));

    let report_a4 = run_ext(&[
        "--json",
        "--project-path",
        project,
        "report",
        "v1",
        "--theme",
        "a4",
        "--name",
        "report_a4",
    ]);
    assert!(
        report_a4.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&report_a4.stderr)
    );
    let report_a4_json: serde_json::Value = serde_json::from_slice(&report_a4.stdout).unwrap();
    assert_eq!(report_a4_json["theme"].as_str(), Some("a4"));
    let pdf_a4_path = PathBuf::from(report_a4_json["pdfPath"].as_str().unwrap());
    assert!(pdf_a4_path.exists());

    let render = run_ext(&["--json", "--project-path", project, "render", "v1"]);
    assert!(
        render.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&render.stderr)
    );
    let render_json: serde_json::Value = serde_json::from_slice(&render.stdout).unwrap();
    let asset_dir = PathBuf::from(render_json["assetDir"].as_str().unwrap());
    assert!(asset_dir.exists());
    assert!(!render_json["assets"].as_array().unwrap().is_empty());
}

#[test]
fn cli_calc_supports_results_dir_mode() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp);
    let project = project_root.to_str().unwrap();
    let results_dir = project_root
        .join(".etabs-ext")
        .join("main")
        .join("v1")
        .join("results");

    let calc = run_ext(&[
        "--json",
        "--project-path",
        project,
        "calc",
        "--results-dir",
        results_dir.to_str().unwrap(),
    ]);
    assert!(
        calc.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&calc.stderr)
    );
    let calc_json: serde_json::Value = serde_json::from_slice(&calc.stdout).unwrap();
    assert!(calc_json["versionId"].is_null());
    assert!(calc_json["branch"].is_null());
    let calc_output = PathBuf::from(calc_json["calcOutputPath"].as_str().unwrap());
    assert!(calc_output.exists());
}
