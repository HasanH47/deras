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

    // Let the queue processor decide when to start this download
    engine::process_queue(&app_handle);

    Ok(task)
}

#[tauri::command]
pub fn pause_download(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let active = app_handle.state::<ActiveDownloads>();
    {
        let tokens = active.tokens.lock().unwrap();
        if let Some(token) = tokens.get(&id) {
            token.cancel();
        }
    }

    // Update state to Paused
    let (downloaded_bytes, total_bytes) = {
        let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
        let result = downloads
            .iter()
            .find(|d| d.id == id)
            .map(|d| (d.downloaded_bytes, d.total_bytes))
            .unwrap_or((0, 0));
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Paused;
        }
        drop(downloads);
        state.save()?;
        result
    };

    let _ = tauri::Emitter::emit(
        &app_handle,
        "download_progress",
        engine::DownloadProgressPayload {
            id: id.clone(),
            state: DownloadState::Paused,
            downloaded_bytes,
            total_bytes,
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
    let active = app_handle.state::<ActiveDownloads>();
    {
        let tokens = active.tokens.lock().unwrap();
        if let Some(token) = tokens.get(&id) {
            token.cancel();
        }
    }

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

#[tauri::command]
pub fn move_download(
    state: State<'_, AppState>,
    id: String,
    direction: String,
) -> Result<(), String> {
    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    let pos = downloads
        .iter()
        .position(|d| d.id == id)
        .ok_or_else(|| "Download not found".to_string())?;

    match direction.as_str() {
        "up" => {
            if pos > 0 {
                downloads.swap(pos, pos - 1);
            }
        }
        "down" => {
            if pos < downloads.len() - 1 {
                downloads.swap(pos, pos + 1);
            }
        }
        _ => return Err("Invalid direction. Use 'up' or 'down'.".to_string()),
    }
    drop(downloads);
    state.save()?;
    Ok(())
}

#[tauri::command]
pub fn force_start(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let task = {
        let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
        let task = downloads
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .ok_or_else(|| "Download not found".to_string())?;
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Downloading;
        }
        drop(downloads);
        state.save()?;
        task
    };

    // Bypass queue limit
    tauri::async_runtime::spawn(async move {
        engine::start_download(app_handle, task, false).await;
    });

    Ok(())
}
