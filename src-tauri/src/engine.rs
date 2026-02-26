use std::path::PathBuf;
use std::time::Instant;

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};

use crate::models::{DownloadState, DownloadTask};
use crate::state::AppState;

/// Payload emitted to the frontend for progress updates.
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub state: DownloadState,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Start downloading a file from `task.url` and stream it to disk.
/// Emits `download_progress` events to the Tauri frontend.
pub async fn start_download(app_handle: AppHandle, task: DownloadTask) {
    let id = task.id.clone();
    let state = app_handle.state::<AppState>();

    // Resolve save directory and full file path
    let save_dir = expand_tilde(&task.save_path);
    if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
        update_state_error(&state, &id, &format!("Failed to create directory: {}", e));
        emit_progress(
            &app_handle,
            &id,
            DownloadState::Error(format!("Failed to create directory: {}", e)),
            0,
            0,
        );
        return;
    }
    let file_path = save_dir.join(&task.filename);

    // Issue the HTTP GET request
    let client = reqwest::Client::new();
    let response = match client.get(&task.url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            let msg = format!("Network error: {}", e);
            update_state_error(&state, &id, &msg);
            emit_progress(&app_handle, &id, DownloadState::Error(msg), 0, 0);
            return;
        }
    };

    if !response.status().is_success() {
        let msg = format!("HTTP error: {}", response.status());
        update_state_error(&state, &id, &msg);
        emit_progress(&app_handle, &id, DownloadState::Error(msg), 0, 0);
        return;
    }

    let total_bytes = response.content_length().unwrap_or(0);

    // Update state to Downloading and set total_bytes
    {
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Downloading;
            t.total_bytes = total_bytes;
        }
        drop(downloads);
        let _ = state.save();
    }
    emit_progress(&app_handle, &id, DownloadState::Downloading, 0, total_bytes);

    // Open the destination file
    let mut file = match tokio::fs::File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            let msg = format!("Failed to create file: {}", e);
            update_state_error(&state, &id, &msg);
            emit_progress(&app_handle, &id, DownloadState::Error(msg), 0, total_bytes);
            return;
        }
    };

    // Stream chunks to disk
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_emit = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await {
                    let msg = format!("Write error: {}", e);
                    update_state_error(&state, &id, &msg);
                    emit_progress(
                        &app_handle,
                        &id,
                        DownloadState::Error(msg),
                        downloaded,
                        total_bytes,
                    );
                    return;
                }
                downloaded += chunk.len() as u64;

                // Throttle progress events to ~100ms interval
                if last_emit.elapsed().as_millis() >= 100 {
                    // Update in-memory state
                    {
                        let mut downloads = state.downloads.lock().unwrap();
                        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
                            t.downloaded_bytes = downloaded;
                        }
                    }
                    emit_progress(
                        &app_handle,
                        &id,
                        DownloadState::Downloading,
                        downloaded,
                        total_bytes,
                    );
                    last_emit = Instant::now();
                }
            }
            Err(e) => {
                let msg = format!("Download stream error: {}", e);
                update_state_error(&state, &id, &msg);
                emit_progress(
                    &app_handle,
                    &id,
                    DownloadState::Error(msg),
                    downloaded,
                    total_bytes,
                );
                return;
            }
        }
    }

    // Download complete
    {
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Completed;
            t.downloaded_bytes = downloaded;
            t.total_bytes = if total_bytes > 0 {
                total_bytes
            } else {
                downloaded
            };
        }
        drop(downloads);
        let _ = state.save();
    }

    let final_total = if total_bytes > 0 {
        total_bytes
    } else {
        downloaded
    };
    emit_progress(
        &app_handle,
        &id,
        DownloadState::Completed,
        downloaded,
        final_total,
    );
}

fn emit_progress(
    app_handle: &AppHandle,
    id: &str,
    state: DownloadState,
    downloaded_bytes: u64,
    total_bytes: u64,
) {
    let _ = app_handle.emit(
        "download_progress",
        DownloadProgressPayload {
            id: id.to_string(),
            state,
            downloaded_bytes,
            total_bytes,
        },
    );
}

fn update_state_error(state: &AppState, id: &str, msg: &str) {
    let mut downloads = state.downloads.lock().unwrap();
    if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
        t.state = DownloadState::Error(msg.to_string());
    }
    drop(downloads);
    let _ = state.save();
}
