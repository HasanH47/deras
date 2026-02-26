use chrono::Utc;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::engine::{self, ActiveDownloads};
use crate::models::{DownloadState, DownloadTask};
use crate::state::AppState;

#[tauri::command]
pub fn get_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    let downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    Ok(downloads.clone())
}

#[tauri::command]
pub fn add_download(
    app_handle: AppHandle,
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
        supports_range: false,
        chunks: None,
    };

    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.insert(0, task.clone());
    drop(downloads);

    state.save()?;

    // Spawn the download asynchronously
    let task_clone = task.clone();
    tauri::async_runtime::spawn(async move {
        engine::start_download(app_handle, task_clone, false).await;
    });

    Ok(task)
}

#[tauri::command]
pub fn pause_download(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Cancel the running download via its token
    let active = app_handle.state::<ActiveDownloads>();
    {
        let tokens = active.tokens.lock().unwrap();
        if let Some(token) = tokens.get(&id) {
            token.cancel();
        }
    }

    // Update state to Paused
    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
        t.state = DownloadState::Paused;
    }
    drop(downloads);
    state.save()?;

    // Emit paused event to frontend
    let _ = tauri::Emitter::emit(
        &app_handle,
        "download_progress",
        engine::DownloadProgressPayload {
            id: id.clone(),
            state: DownloadState::Paused,
            downloaded_bytes: {
                let downloads = state.downloads.lock().unwrap();
                downloads
                    .iter()
                    .find(|d| d.id == id)
                    .map(|d| d.downloaded_bytes)
                    .unwrap_or(0)
            },
            total_bytes: {
                let downloads = state.downloads.lock().unwrap();
                downloads
                    .iter()
                    .find(|d| d.id == id)
                    .map(|d| d.total_bytes)
                    .unwrap_or(0)
            },
        },
    );

    Ok(())
}

#[tauri::command]
pub fn resume_download(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let task = {
        let downloads = state.downloads.lock().map_err(|e| e.to_string())?;
        downloads
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or_else(|| "Download not found".to_string())?
    };

    // Re-spawn the download with resume=true
    tauri::async_runtime::spawn(async move {
        engine::start_download(app_handle, task, true).await;
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_download(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Cancel the running download
    let active = app_handle.state::<ActiveDownloads>();
    {
        let tokens = active.tokens.lock().unwrap();
        if let Some(token) = tokens.get(&id) {
            token.cancel();
        }
    }

    // Remove from state
    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.retain(|d| d.id != id);
    drop(downloads);
    state.save()?;

    Ok(())
}

#[tauri::command]
pub fn remove_download(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.retain(|d| d.id != id);
    drop(downloads);

    state.save()?;
    Ok(())
}
