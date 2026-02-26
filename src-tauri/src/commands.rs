use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager, State};
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
    add_download_internal(&app_handle, &state, url, save_path)
}

pub fn add_download_internal(
    app_handle: &AppHandle,
    state: &AppState,
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

    let category = crate::models::FileCategory::from_filename(&filename);

    // Auto-routing: append category folder to the base save_path
    let final_save_path = {
        let mut path = std::path::PathBuf::from(shellexpand::tilde(&save_path).into_owned());
        // E.g., ~/Downloads/Video
        path.push(format!("{:?}", category));
        // Create the directory if it doesn't exist
        let _ = std::fs::create_dir_all(&path);
        path.to_string_lossy().to_string()
    };

    let task = DownloadTask {
        id: Uuid::new_v4().to_string(),
        url,
        filename,
        save_path: final_save_path,
        state: DownloadState::Pending,
        downloaded_bytes: 0,
        total_bytes: 0,
        category,
        date_added: Utc::now().to_rfc3339(),
        supports_range: false,
        chunks: None,
        speed_limit_bytes: None,
    };

    let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
    downloads.insert(0, task.clone());
    drop(downloads);
    state.save()?;

    // Notify frontend that a new download was added (useful for external/Axum additions)
    let _ = app_handle.emit("download_added", task.clone());

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

#[tauri::command]
pub async fn verify_checksum(
    state: State<'_, AppState>,
    id: String,
    hash_type: String,
    expected_hash: String,
) -> Result<bool, String> {
    // Find the task and build the file path
    let (save_path, filename) = {
        let downloads = state.downloads.lock().map_err(|e| e.to_string())?;
        let task = downloads
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| "Download not found".to_string())?;
        (task.save_path.clone(), task.filename.clone())
    };

    let file_path = crate::clipboard::expand_tilde_public(&save_path).join(&filename);

    // Read the file and compute hash
    let file_bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let computed_hash = match hash_type.to_lowercase().as_str() {
        "md5" => {
            use md5::Digest as _;
            use md5::Md5;
            let mut hasher = Md5::new();
            hasher.update(&file_bytes);
            let result = hasher.finalize();
            format!("{:x}", result)
        }
        "sha256" => {
            use sha2::Digest as _;
            use sha2::Sha256;
            let mut hasher = Sha256::new();
            hasher.update(&file_bytes);
            let result = hasher.finalize();
            format!("{:x}", result)
        }
        _ => return Err("Unsupported hash type. Use 'md5' or 'sha256'.".to_string()),
    };

    Ok(computed_hash.to_lowercase() == expected_hash.trim().to_lowercase())
}

#[tauri::command]
pub fn set_global_speed_limit(
    state: tauri::State<'_, crate::state::AppState>,
    bytes_per_sec: usize,
) {
    state.global_limiter.set_limit(bytes_per_sec);
}

#[tauri::command]
pub fn set_download_speed_limit(
    state: tauri::State<'_, crate::state::AppState>,
    id: String,
    bytes_per_sec: Option<usize>,
) -> Result<(), String> {
    let mut task_limiters = state.task_limiters.lock().unwrap();

    // Update the task definition so it persists
    {
        let mut downloads = state.downloads.lock().map_err(|e| e.to_string())?;
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.speed_limit_bytes = bytes_per_sec;
        } else {
            return Err("Download not found".to_string());
        }
        let _ = state.save();
    }

    if let Some(bps) = bytes_per_sec {
        if let Some(limiter) = task_limiters.get(&id) {
            limiter.set_limit(bps);
        } else {
            task_limiters.insert(
                id,
                std::sync::Arc::new(crate::speed_limiter::SpeedLimiter::new(bps)),
            );
        }
    } else {
        task_limiters.remove(&id);
    }

    Ok(())
}

pub async fn process_schedule(app_handle: &tauri::AppHandle, is_active: bool) {
    let state = app_handle.state::<crate::state::AppState>();
    let mut to_resume = Vec::new();
    let mut to_pause = Vec::new();

    {
        let mut downloads = state.downloads.lock().unwrap();
        for task in downloads.iter_mut() {
            if is_active {
                // If we enter active window, resume tasks that were paused by scheduler
                if let crate::models::DownloadState::Scheduled = task.state {
                    task.state = crate::models::DownloadState::Pending; // Set to pending to trigger queue
                    to_resume.push(task.id.clone());
                }
            } else {
                // If we exit active window, pause active downloads and mark as Scheduled
                if task.state.is_active() {
                    task.state = crate::models::DownloadState::Scheduled;
                    to_pause.push(task.id.clone());
                }
            }
        }
    }

    // Cancel active downloads that needed pausing
    if !is_active && !to_pause.is_empty() {
        let active = app_handle.state::<crate::engine::ActiveDownloads>();
        let mut tokens = active.tokens.lock().unwrap();
        for id in to_pause {
            if let Some(token) = tokens.remove(&id) {
                token.cancel();
            }
        }
    }

    let _ = state.save();

    // Process queue to either start resumed tasks or fill slots of paused tasks
    crate::engine::process_queue(app_handle);
}

#[tauri::command]
pub fn set_schedule_config(
    state: tauri::State<'_, crate::state::AppState>,
    enabled: bool,
    start_time: String,
    end_time: String,
) {
    let mut config = state.schedule_config.lock().unwrap();
    config.enabled = enabled;
    config.start_time = start_time;
    config.end_time = end_time;
}
