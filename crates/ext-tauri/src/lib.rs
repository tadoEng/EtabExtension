mod commands;

use tauri::{Manager};
use tauri_plugin_log::{Target, TargetKind};
use ext_api::AppState;
use ext_db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ─── Log directory ───────────────────────────────────────────────
    let app_log_dir = dirs::data_local_dir()
        .expect("Failed to resolve data directory")
        .join("etab-extension")
        .join("logs");

    std::fs::create_dir_all(&app_log_dir)
        .expect("failed to create app log dir");

    // ─── Log plugin (GitButler-style) ─────────────────────────────────
    let log_plugin = tauri_plugin_log::Builder::default()
        .target(Target::new(TargetKind::LogDir {
            file_name: Some("ui-logs".to_string()),
        }))
        .level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Error
        })
        .build();

    tauri::Builder::default()
        // ─── Plugins ──────────────────────────────────────────────────
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(log_plugin)

        // ─── Setup ────────────────────────────────────────────────────
        .setup(|app| {
            let app_handle = app.handle().clone();

            let db = tauri::async_runtime::block_on(async {
                initialize_database().await
            })
            .expect("Failed to initialize database");

            app_handle.manage(AppState::new(db));
            Ok(())
        })

        // ─── Commands ─────────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::create_project,
            commands::get_projects,
        ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



async fn initialize_database() -> Result<Database, Box<dyn std::error::Error>> {
    let app_dir = dirs::data_local_dir()
        .ok_or("Failed to get app data directory")?
        .join("etab-extension");

    let db_dir = app_dir.join("db");
    let projects_dir = app_dir.join("projects");

    std::fs::create_dir_all(&db_dir)?;
    std::fs::create_dir_all(&projects_dir)?;

    let db_path = db_dir.join("app.db");

    let db_url = format!(
        "sqlite://{}?mode=rwc",
        db_path.to_string_lossy().replace('\\', "/")
    );

    let db = Database::new(&db_url, projects_dir.to_str().unwrap()).await?;
    Ok(db)
}
