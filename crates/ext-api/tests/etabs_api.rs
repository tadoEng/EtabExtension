#[path = "../../test-support/fake_sidecar.rs"]
mod fake_sidecar;

use chrono::Duration;
use ext_api::AppContext;
use ext_api::commit::{self, CommitOptions};
use ext_api::etabs::{
    CloseMode, EtabsRecoverConflict, RecoveryChoice, etabs_close, etabs_open, etabs_recover,
    etabs_status, etabs_unlock,
};
use ext_api::init::{InitRequest, init_project};
use ext_core::state::WorkingFileStatus;
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

fn configure_sidecar(project_root: &std::path::Path, temp: &TempDir) -> std::path::PathBuf {
    let sidecar = fake_sidecar::write_fake_sidecar(temp, fake_sidecar::FakeSidecarMode::Success);
    fake_sidecar::configure_fake_sidecar(project_root, &sidecar);
    sidecar
}

fn set_sidecar_state(
    sidecar: &std::path::Path,
    mut state: fake_sidecar::FakeSidecarState,
    working_file: &std::path::Path,
) {
    if state.open_file_path.is_none() && state.is_model_open {
        state.open_file_path = Some(working_file.to_path_buf());
    }
    fake_sidecar::set_fake_sidecar_state(sidecar, &state);
}

#[tokio::test]
async fn etabs_open_updates_state_to_open_clean() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
    let ctx = AppContext::new(&project_root).unwrap();

    let result = etabs_open(&ctx, None).await.unwrap();
    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();

    assert_eq!(result.pid, 4242);
    assert!(!result.is_snapshot);
    assert_eq!(working.etabs_pid, Some(4242));
    assert_eq!(working.status, WorkingFileStatus::OpenClean);

    let fake_state_path = fake_sidecar::sidecar_state_path(&sidecar);
    assert!(fake_state_path.exists());
}

#[tokio::test]
async fn etabs_open_snapshot_sets_warning_and_snapshot_flag() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
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

    let result = etabs_open(&ctx, Some("v1")).await.unwrap();
    assert!(result.is_snapshot);
    assert!(
        result
            .warning
            .as_deref()
            .unwrap_or_default()
            .contains("snapshot")
    );

    let fake_state_text =
        std::fs::read_to_string(fake_sidecar::sidecar_state_path(&sidecar)).unwrap();
    assert!(fake_state_text.contains("is_model_open=1"));
}

#[tokio::test]
async fn etabs_close_interactive_returns_conflict_for_open_modified() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.etabs_pid = Some(std::process::id());
    working.last_known_mtime = working.last_known_mtime.map(|t| t - Duration::seconds(60));
    working.status = WorkingFileStatus::OpenModified;
    let working_path = working.path.clone();
    ctx.save_state(&state).unwrap();

    set_sidecar_state(
        &sidecar,
        fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            is_model_open: true,
            is_locked: Some(false),
            is_analyzed: Some(false),
            ..Default::default()
        },
        &working_path,
    );

    let err = etabs_close(&ctx, CloseMode::Interactive).await.unwrap_err();
    let conflict = err.downcast::<ext_api::EtabsCloseConflict>().unwrap();
    assert_eq!(conflict.pid, std::process::id());
}

#[tokio::test]
async fn etabs_close_save_clears_pid_and_updates_arrival_status() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.etabs_pid = Some(std::process::id());
    working.last_known_mtime = working.last_known_mtime.map(|t| t - Duration::seconds(60));
    working.status = WorkingFileStatus::OpenModified;
    let working_path = working.path.clone();
    ctx.save_state(&state).unwrap();

    set_sidecar_state(
        &sidecar,
        fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            open_file_path: Some(working_path.clone()),
            is_model_open: true,
            is_locked: Some(false),
            is_analyzed: Some(true),
            ..Default::default()
        },
        &working_path,
    );

    let result = etabs_close(&ctx, CloseMode::Save).await.unwrap();
    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();

    assert!(result.saved);
    assert_eq!(result.arrival_status, WorkingFileStatus::Analyzed);
    assert!(working.etabs_pid.is_none());
    assert!(working.last_known_mtime.is_some());
    assert_eq!(working.status, WorkingFileStatus::Analyzed);
}

#[tokio::test]
async fn etabs_status_reports_sidecar_fields() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
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

    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();
    set_sidecar_state(
        &sidecar,
        fake_sidecar::FakeSidecarState {
            is_running: true,
            pid: Some(std::process::id()),
            open_file_path: Some(working.path.clone()),
            is_model_open: true,
            is_locked: Some(false),
            is_analyzed: Some(true),
            ..Default::default()
        },
        &working.path,
    );

    let result = etabs_status(&ctx).await.unwrap();
    assert!(result.sidecar_available);
    assert_eq!(result.working_file_status, WorkingFileStatus::Analyzed);
    assert_eq!(result.pid, Some(std::process::id()));
    assert_eq!(
        result
            .unit_system
            .as_ref()
            .map(|units| units.force.as_str()),
        Some("kip")
    );
}

#[tokio::test]
async fn etabs_unlock_clears_locked_state() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar = configure_sidecar(&project_root, &temp);
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.based_on_version = Some("v1".to_string());
    working.status = WorkingFileStatus::Locked;
    working.etabs_pid = None;
    let working_path = working.path.clone();
    ctx.save_state(&state).unwrap();

    set_sidecar_state(
        &sidecar,
        fake_sidecar::FakeSidecarState {
            is_running: false,
            pid: Some(std::process::id()),
            open_file_path: None,
            is_model_open: false,
            is_locked: Some(true),
            is_analyzed: Some(true),
            ..Default::default()
        },
        &working_path,
    );

    let result = etabs_unlock(&ctx).await.unwrap();
    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();

    assert!(result.reopened_for_unlock);
    assert_eq!(result.arrival_status, WorkingFileStatus::Analyzed);
    assert_eq!(working.status, WorkingFileStatus::Analyzed);
    assert!(working.etabs_pid.is_none());
}

#[tokio::test]
async fn etabs_recover_keep_marks_modified() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    working.etabs_pid = Some(u32::MAX);
    working.status = WorkingFileStatus::Orphaned;
    ctx.save_state(&state).unwrap();

    let result = etabs_recover(&ctx, Some(RecoveryChoice::KeepChanges))
        .await
        .unwrap();
    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();

    assert_eq!(result.arrival_status, WorkingFileStatus::Modified);
    assert_eq!(working.status, WorkingFileStatus::Modified);
    assert!(working.etabs_pid.is_none());
}

#[tokio::test]
async fn etabs_recover_phase1_surfaces_file_was_modified_true() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    let working_path = working.path.clone();
    std::fs::write(&working_path, b"before-crash").unwrap();
    let current_mtime: chrono::DateTime<chrono::Utc> = std::fs::metadata(&working_path)
        .unwrap()
        .modified()
        .unwrap()
        .into();
    working.etabs_pid = Some(u32::MAX);
    working.status = WorkingFileStatus::Orphaned;
    working.last_known_mtime = Some(current_mtime - Duration::seconds(60));
    working.based_on_version = Some("v1".to_string());
    ctx.save_state(&state).unwrap();

    let err = etabs_recover(&ctx, None).await.unwrap_err();
    let conflict = err.downcast::<EtabsRecoverConflict>().unwrap();

    assert!(conflict.file_was_modified);
    assert_eq!(conflict.based_on_version.as_deref(), Some("v1"));
}

#[tokio::test]
async fn etabs_recover_phase1_surfaces_file_was_modified_false() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();

    let mut state = ctx.load_state().unwrap();
    let working = state.working_file.as_mut().unwrap();
    let working_path = working.path.clone();
    std::fs::write(&working_path, b"before-crash").unwrap();
    let current_mtime: chrono::DateTime<chrono::Utc> = std::fs::metadata(&working_path)
        .unwrap()
        .modified()
        .unwrap()
        .into();
    working.etabs_pid = Some(u32::MAX);
    working.status = WorkingFileStatus::Orphaned;
    working.last_known_mtime = Some(current_mtime + Duration::seconds(60));
    working.based_on_version = Some("v1".to_string());
    ctx.save_state(&state).unwrap();

    let err = etabs_recover(&ctx, None).await.unwrap_err();
    let conflict = err.downcast::<EtabsRecoverConflict>().unwrap();

    assert!(!conflict.file_was_modified);
    assert_eq!(conflict.based_on_version.as_deref(), Some("v1"));
}

#[tokio::test]
async fn etabs_recover_restore_copies_snapshot() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
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
    std::fs::write(&working.path, b"edited-after-crash").unwrap();
    working.etabs_pid = Some(u32::MAX);
    working.status = WorkingFileStatus::Orphaned;
    working.based_on_version = Some("v1".to_string());
    let working_path = working.path.clone();
    ctx.save_state(&state).unwrap();

    let result = etabs_recover(&ctx, Some(RecoveryChoice::RestoreFromVersion))
        .await
        .unwrap();
    let restored = std::fs::read(&working_path).unwrap();

    assert_eq!(result.arrival_status, WorkingFileStatus::Clean);
    assert_eq!(restored, b"initial-edb");
}
