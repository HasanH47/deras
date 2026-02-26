mod commands;
mod engine;
mod models;
mod state;

use engine::ActiveDownloads;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let app_state = AppState::new(data_dir);
            app.manage(app_state);
            app.manage(ActiveDownloads::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_downloads,
            commands::add_download,
            commands::remove_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
