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

#[tokio::test]
async fn cli_config_set_get_and_list_route_values() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let project = project_root.to_str().unwrap();

    let set_shared = run_ext(&[
        "--project-path",
        project,
        "config",
        "set",
        "project.name",
        "\"Proof Tower\"",
    ]);
    assert!(
        set_shared.status.success(),
        "{}",
        String::from_utf8_lossy(&set_shared.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&set_shared),
        str![[r#"
✓ Updated shared config: project.name = "Proof Tower"

"#]]
    );

    let set_local = run_ext(&[
        "--project-path",
        project,
        "config",
        "set",
        "project.sidecar-path",
        "\"C:/tools/etab-cli.exe\"",
    ]);
    assert!(
        set_local.status.success(),
        "{}",
        String::from_utf8_lossy(&set_local.stderr)
    );
    snapbox::assert_data_eq!(
        stdout_text(&set_local),
        str![[r#"
✓ Updated local config: project.sidecar_path = "C:/tools/etab-cli.exe"

"#]]
    );

    let get = run_ext(&[
        "--json",
        "--project-path",
        project,
        "config",
        "get",
        "project.sidecar-path",
    ]);
    assert!(
        get.status.success(),
        "{}",
        String::from_utf8_lossy(&get.stderr)
    );
    let get_json: serde_json::Value = serde_json::from_slice(&get.stdout).unwrap();
    assert_eq!(get_json["scope"], "local");
    assert_eq!(get_json["key"], "project.sidecar_path");
    assert_eq!(get_json["value"], "C:/tools/etab-cli.exe");

    let list = run_ext(&["--shell", "--project-path", project, "config", "list"]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let list_text = stdout_text(&list);
    assert!(list_text.contains("project.name"));
    assert!(list_text.contains("project.sidecar_path"));

    let shared_text =
        std::fs::read_to_string(project_root.join(".etabs-ext").join("config.toml")).unwrap();
    let local_text =
        std::fs::read_to_string(project_root.join(".etabs-ext").join("config.local.toml")).unwrap();
    assert!(shared_text.contains("name = \"Proof Tower\""));
    assert!(!shared_text.contains("sidecar-path"));
    assert!(local_text.contains("sidecar-path = \"C:/tools/etab-cli.exe\""));
}
