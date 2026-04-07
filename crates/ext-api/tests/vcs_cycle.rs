#[path = "../../test-support/fake_sidecar.rs"]
mod fake_sidecar;

use chrono::{Duration, Utc};
use ext_api::AppContext;
use ext_api::branch;
use ext_api::checkout::{self, CheckoutOptions};
use ext_api::commit::{self, CommitOptions};
use ext_api::diff;
use ext_api::init::{InitRequest, init_project};
use ext_api::log;
use ext_api::stash::{self, StashPopConflict, StashPopConflictResolution, StashPopOptions};
use ext_api::switch;
use ext_core::state::WorkingFileStatus;
use ext_core::version::VersionManifest;
use std::process::Command;
use tempfile::TempDir;

fn write_dummy_edb(path: &std::path::Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
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

#[tokio::test]
async fn full_vcs_cycle_no_e2k() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let ctx = AppContext::new(&project_root).unwrap();
    let ext_dir = project_root.join(".etabs-ext");

    let commit_v1 = commit::commit_version(
        &ctx,
        "Initial model",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(commit_v1.version_id, "v1");
    assert_eq!(commit_v1.branch, "main");
    let main_v1 = log::show_version(&ctx, "main/v1").await.unwrap();
    assert_eq!(
        main_v1.manifest.git_commit_hash.as_deref(),
        Some(commit_v1.git_hash.as_str())
    );
    let main_messages = git_log_messages(&ext_dir, "main");
    assert_eq!(main_messages[0], "Initial model");
    assert!(
        !main_messages
            .iter()
            .any(|msg| msg.starts_with("ext: finalize manifest"))
    );

    let create_branch = branch::create_branch(&ctx, "steel-columns", Some("main/v1"))
        .await
        .unwrap();
    assert_eq!(create_branch.name, "steel-columns");
    assert!(create_branch.working_model_path.exists());
    let branch_list = branch::list_branches(&ctx).await.unwrap();
    let steel = branch_list
        .branches
        .iter()
        .find(|branch| branch.name == "steel-columns")
        .unwrap();
    assert_eq!(steel.version_count, 1);
    assert_eq!(steel.latest_version.as_deref(), Some("v1"));

    let switch_result = switch::switch_branch(&ctx, "steel-columns").await.unwrap();
    assert_eq!(switch_result.branch, "steel-columns");
    assert_eq!(switch_result.arrival_status, WorkingFileStatus::Clean);

    let commit_v2 = commit::commit_version(
        &ctx,
        "Steel option",
        CommitOptions {
            no_e2k: true,
            analyze: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(commit_v2.version_id, "v2");
    assert_eq!(commit_v2.branch, "steel-columns");
    let steel_messages = git_log_messages(&ext_dir, "steel-columns");
    assert_eq!(steel_messages[0], "Steel option");
    assert!(steel_messages.iter().any(|msg| msg == "Initial model"));
    assert!(
        !steel_messages
            .iter()
            .any(|msg| msg.starts_with("ext: finalize manifest"))
    );

    let steel_v2 = log::show_version(&ctx, "steel-columns/v2").await.unwrap();
    assert_eq!(steel_v2.manifest.parent.as_deref(), Some("v1"));
    assert_eq!(
        steel_v2.manifest.git_commit_hash.as_deref(),
        Some(commit_v2.git_hash.as_str())
    );
    let branch_list = branch::list_branches(&ctx).await.unwrap();
    let steel = branch_list
        .branches
        .iter()
        .find(|branch| branch.name == "steel-columns")
        .unwrap();
    assert_eq!(steel.version_count, 2);
    assert_eq!(steel.latest_version.as_deref(), Some("v2"));

    let checkout_main = checkout::checkout_version(&ctx, "main/v1", CheckoutOptions::default())
        .await
        .unwrap();
    assert_eq!(checkout_main.branch, "main");
    assert_eq!(checkout_main.version_id, "v1");
    assert!(checkout_main.working_model_path.exists());

    let working_file = checkout_main.working_model_path.clone();
    std::fs::write(&working_file, b"main-modified").unwrap();

    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.last_known_mtime = wf.last_known_mtime.map(|t| t - Duration::seconds(60));
    wf.status_changed_at = Utc::now();
    ctx.save_state(&state).unwrap();

    let stash_save = stash::stash_save(&ctx, Some("WIP"), false).await.unwrap();
    assert_eq!(stash_save.branch, "main");
    assert!(stash_save.stash_path.exists());

    let state = ctx.load_state().unwrap();
    assert!(state.stashes.contains_key("main"));

    std::fs::write(&working_file, b"other-content").unwrap();
    let mut state = ctx.load_state().unwrap();
    let wf = state.working_file.as_mut().unwrap();
    wf.last_known_mtime = wf.last_known_mtime.map(|t| t - Duration::seconds(60));
    ctx.save_state(&state).unwrap();

    let stash_pop_err = stash::stash_pop(&ctx, StashPopOptions::default())
        .await
        .unwrap_err();
    let conflict = stash_pop_err.downcast::<StashPopConflict>().unwrap();
    assert_eq!(conflict.branch, "main");
    assert_eq!(conflict.current_status, WorkingFileStatus::Modified);

    let stash_pop = stash::stash_pop(
        &ctx,
        StashPopOptions {
            conflict_resolution: Some(StashPopConflictResolution::Overwrite),
        },
    )
    .await
    .unwrap();
    assert_eq!(stash_pop.branch, "main");
    assert_eq!(stash_pop.restored_based_on.as_deref(), Some("v1"));
    assert_eq!(std::fs::read(&working_file).unwrap(), b"main-modified");

    let state = ctx.load_state().unwrap();
    assert!(!state.stashes.contains_key("main"));
    let wf = state.working_file.as_ref().unwrap();
    assert_eq!(wf.based_on_version.as_deref(), Some("v1"));
    assert_eq!(wf.status, WorkingFileStatus::Modified);

    let main_log = log::list_versions(&ctx, Some("main"), false).await.unwrap();
    assert_eq!(main_log.branch, "main");
    assert_eq!(main_log.commits.len(), 1);
    assert_eq!(main_log.commits[0].version_id.as_deref(), Some("v1"));

    let main_v1 = log::show_version(&ctx, "main/v1").await.unwrap();
    assert_eq!(main_v1.manifest.id, "v1");
    assert!(!main_v1.manifest.e2k_generated);

    let diff_result = diff::diff_versions(&ctx, "main/v1", "steel-columns/v2")
        .await
        .unwrap();
    assert_eq!(diff_result.from_ref, "main/v1");
    assert_eq!(diff_result.to_ref, "steel-columns/v2");
    assert!(diff_result.no_e2k_warning.is_some());
    assert!(
        diff_result.diff_text.is_empty(),
        "Expected empty diff text for --no-e2k commits"
    );
}

#[tokio::test]
async fn commit_analyze_preserves_version_when_analysis_fails() {
    let temp = TempDir::new().unwrap();
    let project_root = init_fixture(&temp).await;
    let sidecar =
        fake_sidecar::write_fake_sidecar(&temp, fake_sidecar::FakeSidecarMode::AnalysisFail);
    fake_sidecar::configure_fake_sidecar(&project_root, &sidecar);
    let ctx = AppContext::new(&project_root).unwrap();
    let ext_dir = project_root.join(".etabs-ext");

    let result = commit::commit_version(
        &ctx,
        "Initial analyzed model",
        CommitOptions {
            no_e2k: true,
            analyze: true,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.version_id, "v1");
    assert!(!result.analyzed);
    assert!(
        result
            .warning
            .as_deref()
            .unwrap_or_default()
            .contains("Version saved, but analysis failed")
    );

    let version_dir = project_root.join(".etabs-ext").join("main").join("v1");
    assert!(version_dir.exists());
    assert!(!version_dir.join("summary.json").exists());

    let manifest = VersionManifest::read_from(&version_dir).unwrap();
    assert_eq!(manifest.id, "v1");
    assert!(!manifest.is_analyzed);

    let state = ctx.load_state().unwrap();
    let working = state.working_file.as_ref().unwrap();
    assert_eq!(working.based_on_version.as_deref(), Some("v1"));
    assert_eq!(working.status, WorkingFileStatus::Clean);

    let visible_log = log::list_versions(&ctx, Some("main"), false).await.unwrap();
    assert_eq!(visible_log.commits.len(), 1);
    assert_eq!(visible_log.commits[0].version_id.as_deref(), Some("v1"));
    let main_messages = git_log_messages(&ext_dir, "main");
    assert_eq!(main_messages[0], "Initial analyzed model");
    assert!(
        !main_messages
            .iter()
            .any(|msg| msg.starts_with("ext: finalize manifest"))
    );

    let shown = log::show_version(&ctx, "main/v1").await.unwrap();
    assert_eq!(
        shown.manifest.git_commit_hash.as_deref(),
        Some(result.git_hash.as_str())
    );
}
