mod commands;

use tauri::Manager;
use ext_api::AppState;
use ext_db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize database synchronously using block_on
            let db = match tauri::async_runtime::block_on(async { initialize_database().await }) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                    panic!("Could not initialize database: {}", e);
                }
            };

            let state = AppState::new(db);
            app_handle.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::create_project,
            commands::get_projects
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_database() -> Result<Database, Box<dyn std::error::Error>> {
    // Get app data directory
    let app_dir = dirs::data_local_dir()
        .ok_or("Failed to get app data directory")?
        .join("etab-extension");
    
    let db_dir = app_dir.join("db");
    let projects_dir = app_dir.join("projects");
    
    // Create directories
    std::fs::create_dir_all(&db_dir)?;
    std::fs::create_dir_all(&projects_dir)?;
    
    let db_path = db_dir.join("app.db");
    // Use proper SQLite URL format with mode=rwc to create file if needed
    let db_url = format!(
        "sqlite://{}?mode=rwc",
        db_path.to_string_lossy().replace("\\", "/")
    );
    
    eprintln!("Database URL: {}", db_url);
    eprintln!("Projects dir: {}", projects_dir.display());
    
    let db = Database::new(&db_url, projects_dir.to_str().unwrap()).await?;
    Ok(db)
}