use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::models::{DownloadState, DownloadTask};
use crate::state::AppState;

#[tauri::command]
pub fn get_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    let downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    Ok(downloads.clone())
}

#[tauri::command]
pub fn add_download(
    state: State<'_, AppState>,
    url: String,
    save_path: String,
) -> Result<DownloadTask, String> {
    let filename = url
        .split('/')
        .last()
        .unwrap_or("unknown")
        .split('?')
        .next()
        .unwrap_or("unknown")
        .to_string();

    let task = DownloadTask {
        id: Uuid::new_v4().to_string(),
        url,
        filename,
        save_path,
        state: DownloadState::Pending,
        downloaded_bytes: 0,
        total_bytes: 0,
        date_added: Utc::now().to_rfc3339(),
    };

    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.insert(0, task.clone());
    drop(downloads);

    state.save()?;
    Ok(task)
}

#[tauri::command]
pub fn remove_download(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.retain(|d| d.id != id);
    drop(downloads);

    state.save()?;
    Ok(())
}
