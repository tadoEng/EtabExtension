use chrono::{DateTime, Duration, Utc};
use ext_api::AppContext;
use ext_api::init::{InitRequest, init_project};
use ext_api::status::{StatusOptions, project_status};
use ext_core::state::WorkingFileStatus;
use tempfile::TempDir;

fn write_dummy_edb(path: &std::path::Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"dummy-edb-content").unwrap();
}

fn file_mtime(path: &std::path::Path) -> DateTime<Utc> {
    let modified = std::fs::metadata(path).unwrap().modified().unwrap();
    modified.into()
}

async fn init_fixture(temp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let project_root = temp.path().join("project");
    let edb = temp.path().join("source").join("model.edb");
    write_dummy_edb(&edb);

    init_project(InitRequest {
        name: "Fixture".to_string(),
        edb_path: edb.clone(),
        project_path: project_root.clone(),
        author: Some("Tester".to_string()),
        email: Some("tester@example.com".to_string()),
        one_drive_dir: None,
        reports_dir: None,
        allow_onedrive: true,
    })
    .await
    .unwrap();

    (project_root, edb)
}

#[tokio::test]
async fn init_creates_project_structure() {
    let temp = TempDir::new().unwrap();
    let (project_root, _) = init_fixture(&temp).await;
    let ext_dir = project_root.join(".etabs-ext");

    assert!(ext_dir.join("config.toml").exists());
    assert!(ext_dir.join("config.local.toml").exists());
    assert!(ext_dir.join("state.json").exists());
    assert!(
        ext_dir
            .join("main")
            .join("working")
            .join("model.edb")
            .exists()
    );
    assert!(ext_dir.join(".git").exists());
}

#[tokio::test]
async fn init_rejects_invalid_edb() {
    let temp = TempDir::new().unwrap();
    let project_root = temp.path().join("project");
    let missing = temp.path().join("missing.edb");

    let err = init_project(InitRequest {
        name: "Invalid".to_string(),
        edb_path: missing,
        project_path: project_root,
        author: None,
        email: None,
        one_drive_dir: None,
        reports_dir: None,
        allow_onedrive: false,
    })
    .await
    .unwrap_err();

    assert!(err.to_string().contains("EDB file not found"));
}

#[tokio::test]
async fn init_rejects_already_initialized_project() {
    let temp = TempDir::new().unwrap();
    let (project_root, edb) = init_fixture(&temp).await;

    let err = init_project(InitRequest {
        name: "Duplicate".to_string(),
        edb_path: edb,
        project_path: project_root,
        author: None,
        email: None,
        one_drive_dir: None,
        reports_dir: None,
        allow_onedrive: true,
    })
    .await
    .unwrap_err();

    assert!(err.to_string().contains("Project already initialized"));
}

#[tokio::test]
async fn status_reports_untracked_clean_modified_and_missing() {
    let temp = TempDir::new().unwrap();
    let (project_root, _) = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();

    let report = project_status(&ctx, StatusOptions { verbose: false })
        .await
        .unwrap();
    assert_eq!(report.working_status, WorkingFileStatus::Untracked);

    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.based_on_version = Some("v1".to_string());
    wf.last_known_mtime = Some(file_mtime(&wf.path));
    ctx.save_state(&state).unwrap();

    let clean = project_status(&ctx, StatusOptions { verbose: false })
        .await
        .unwrap();
    assert_eq!(clean.working_status, WorkingFileStatus::Clean);

    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.last_known_mtime = wf.last_known_mtime.map(|t| t - Duration::seconds(60));
    ctx.save_state(&state).unwrap();

    let modified = project_status(&ctx, StatusOptions { verbose: false })
        .await
        .unwrap();
    assert_eq!(modified.working_status, WorkingFileStatus::Modified);

    std::fs::remove_file(modified.working_model_path).unwrap();
    let missing = project_status(&ctx, StatusOptions { verbose: false })
        .await
        .unwrap();
    assert_eq!(missing.working_status, WorkingFileStatus::Missing);
}

#[tokio::test]
async fn status_verbose_succeeds_without_sidecar() {
    let temp = TempDir::new().unwrap();
    let (project_root, _) = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();

    let report = project_status(&ctx, StatusOptions { verbose: true })
        .await
        .unwrap();
    if ctx.sidecar.is_none() {
        assert!(report.sidecar_warning.is_some());
    }
}
