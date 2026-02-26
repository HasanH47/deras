use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

use crate::models::{ChunkState, DownloadState, DownloadTask};
use crate::state::AppState;

const DEFAULT_CONNECTIONS: u32 = 4;
const MAX_CONCURRENT_DOWNLOADS: usize = 3;
const MAX_RETRIES: u32 = 3;

/// Payload emitted to the frontend for progress updates.
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub state: DownloadState,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Global registry of active download cancellation tokens.
pub struct ActiveDownloads {
    pub tokens: std::sync::Mutex<HashMap<String, CancellationToken>>,
}

impl ActiveDownloads {
    pub fn new() -> Self {
        Self {
            tokens: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn active_count(&self) -> usize {
        self.tokens.lock().unwrap().len()
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

async fn fetch_metadata(client: &reqwest::Client, url: &str) -> Result<(u64, bool), String> {
    let resp = client
        .head(url)
        .send()
        .await
        .map_err(|e| format!("HEAD request failed: {}", e))?;

    let content_length = resp.content_length().unwrap_or(0);
    let supports_range = resp
        .headers()
        .get(reqwest::header::ACCEPT_RANGES)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("bytes"))
        .unwrap_or(false);

    Ok((content_length, supports_range))
}

fn calculate_chunks(total_bytes: u64, num_connections: u32) -> Vec<ChunkState> {
    let chunk_size = total_bytes / num_connections as u64;
    let mut chunks = Vec::new();
    for i in 0..num_connections {
        let start = i as u64 * chunk_size;
        let end = if i == num_connections - 1 {
            total_bytes - 1
        } else {
            (i as u64 + 1) * chunk_size - 1
        };
        chunks.push(ChunkState {
            id: i,
            start_byte: start,
            end_byte: end,
            downloaded: 0,
            is_complete: false,
        });
    }
    chunks
}

fn part_file_path(save_dir: &PathBuf, filename: &str, chunk_id: u32) -> PathBuf {
    save_dir.join(format!("{}.deras-part-{}", filename, chunk_id))
}

/// Check the queue and start pending downloads if below the concurrency limit.
pub fn process_queue(app_handle: &AppHandle) {
    let active_downloads = app_handle.state::<ActiveDownloads>();
    let app_state = app_handle.state::<AppState>();

    loop {
        let active_count = active_downloads.active_count();
        if active_count >= MAX_CONCURRENT_DOWNLOADS {
            break;
        }

        // Find the next pending download
        let next_task = {
            let downloads = app_state.downloads.lock().unwrap();
            downloads
                .iter()
                .find(|d| matches!(d.state, DownloadState::Pending))
                .cloned()
        };

        match next_task {
            Some(task) => {
                // Update state to Downloading so it's not picked again
                {
                    let mut downloads = app_state.downloads.lock().unwrap();
                    if let Some(t) = downloads.iter_mut().find(|d| d.id == task.id) {
                        t.state = DownloadState::Downloading;
                    }
                    drop(downloads);
                    let _ = app_state.save();
                }

                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    start_download(handle, task, false).await;
                });
            }
            None => break,
        }
    }
}

/// Start downloading a file with queue integration.
pub async fn start_download(app_handle: AppHandle, task: DownloadTask, resume: bool) {
    let id = task.id.clone();
    let app_state = app_handle.state::<AppState>();

    // Register cancellation token
    let cancel_token = CancellationToken::new();
    {
        let active = app_handle.state::<ActiveDownloads>();
        active
            .tokens
            .lock()
            .unwrap()
            .insert(id.clone(), cancel_token.clone());
    }

    let save_dir = expand_tilde(&task.save_path);
    if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
        set_error(
            &app_handle,
            &app_state,
            &id,
            &format!("Failed to create directory: {}", e),
        );
        cleanup_and_process_queue(&app_handle, &id);
        return;
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let (total_bytes, supports_range) = if resume && task.total_bytes > 0 {
        (task.total_bytes, task.supports_range)
    } else {
        match fetch_metadata(&client, &task.url).await {
            Ok(meta) => meta,
            Err(msg) => {
                set_error(&app_handle, &app_state, &id, &msg);
                cleanup_and_process_queue(&app_handle, &id);
                return;
            }
        }
    };

    // Update state with metadata
    {
        let mut downloads = app_state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.total_bytes = total_bytes;
            t.supports_range = supports_range;
            t.state = DownloadState::Downloading;
        }
        drop(downloads);
        let _ = app_state.save();
    }
    emit_progress(&app_handle, &id, DownloadState::Downloading, 0, total_bytes);

    if supports_range && total_bytes > 0 {
        let chunks = if resume {
            let downloads = app_state.downloads.lock().unwrap();
            downloads
                .iter()
                .find(|d| d.id == id)
                .and_then(|t| t.chunks.clone())
                .unwrap_or_else(|| calculate_chunks(total_bytes, DEFAULT_CONNECTIONS))
        } else {
            let chunks = calculate_chunks(total_bytes, DEFAULT_CONNECTIONS);
            let mut downloads = app_state.downloads.lock().unwrap();
            if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
                t.chunks = Some(chunks.clone());
            }
            drop(downloads);
            let _ = app_state.save();
            chunks
        };

        download_chunked(
            &app_handle,
            &app_state,
            &client,
            &id,
            &task.url,
            &task.filename,
            &save_dir,
            total_bytes,
            chunks,
            cancel_token,
        )
        .await;
    } else {
        download_single(
            &app_handle,
            &app_state,
            &client,
            &id,
            &task.url,
            &task.filename,
            &save_dir,
            total_bytes,
            cancel_token,
        )
        .await;
    }

    // Clean up and process queue for next pending downloads
    cleanup_and_process_queue(&app_handle, &id);
}

/// Remove cancellation token and trigger queue processing.
fn cleanup_and_process_queue(app_handle: &AppHandle, id: &str) {
    {
        let active = app_handle.state::<ActiveDownloads>();
        active.tokens.lock().unwrap().remove(id);
    }
    process_queue(app_handle);
}

// ─── Chunked Download ────────────────────────────────────────────────────────

async fn download_chunked(
    app_handle: &AppHandle,
    app_state: &AppState,
    client: &reqwest::Client,
    id: &str,
    url: &str,
    filename: &str,
    save_dir: &PathBuf,
    total_bytes: u64,
    chunks: Vec<ChunkState>,
    cancel_token: CancellationToken,
) {
    let progress = Arc::new(TokioMutex::new(chunks.clone()));
    let mut handles = Vec::new();

    for chunk in chunks.iter() {
        if chunk.is_complete {
            continue;
        }
        let client = client.clone();
        let url = url.to_string();
        let part_path = part_file_path(save_dir, filename, chunk.id);
        let chunk_clone = chunk.clone();
        let progress = Arc::clone(&progress);
        let cancel = cancel_token.clone();
        let app_handle_clone = app_handle.clone();
        let id_owned = id.to_string();

        let handle = tokio::spawn(async move {
            download_chunk_with_retry(
                &client,
                &url,
                &part_path,
                chunk_clone,
                progress,
                cancel,
                &app_handle_clone,
                &id_owned,
                total_bytes,
            )
            .await
        });
        handles.push(handle);
    }

    let mut all_ok = true;
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => {}
            Ok(Err(_)) | Err(_) => {
                all_ok = false;
            }
        }
    }

    if cancel_token.is_cancelled() {
        let chunks_state = progress.lock().await.clone();
        let mut downloads = app_state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.chunks = Some(chunks_state);
        }
        drop(downloads);
        let _ = app_state.save();
        return;
    }

    if !all_ok {
        set_error(
            app_handle,
            app_state,
            id,
            "One or more chunks failed after retries",
        );
        return;
    }

    let final_path = save_dir.join(filename);
    if let Err(e) = merge_parts(save_dir, filename, chunks.len() as u32, &final_path).await {
        set_error(app_handle, app_state, id, &format!("Merge failed: {}", e));
        return;
    }

    {
        let mut downloads = app_state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Completed;
            t.downloaded_bytes = total_bytes;
            t.chunks = None;
        }
        drop(downloads);
        let _ = app_state.save();
    }
    emit_progress(
        app_handle,
        id,
        DownloadState::Completed,
        total_bytes,
        total_bytes,
    );
}

/// Download a chunk with automatic retry and exponential backoff.
async fn download_chunk_with_retry(
    client: &reqwest::Client,
    url: &str,
    part_path: &PathBuf,
    chunk: ChunkState,
    progress: Arc<TokioMutex<Vec<ChunkState>>>,
    cancel_token: CancellationToken,
    app_handle: &AppHandle,
    id: &str,
    total_bytes: u64,
) -> Result<(), String> {
    let mut attempts = 0;
    let mut current_chunk = chunk;

    loop {
        match download_chunk(
            client,
            url,
            part_path,
            current_chunk.clone(),
            Arc::clone(&progress),
            cancel_token.clone(),
            app_handle,
            id,
            total_bytes,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(e) if e == "cancelled" => return Err(e),
            Err(e) if is_filesystem_error(&e) => {
                // Don't retry filesystem errors (disk full, permissions, etc.)
                return Err(e);
            }
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_RETRIES {
                    return Err(format!("Failed after {} retries: {}", MAX_RETRIES, e));
                }

                // Refresh current_chunk with latest progress
                {
                    let chunks = progress.lock().await;
                    if let Some(c) = chunks.iter().find(|c| c.id == current_chunk.id) {
                        current_chunk = c.clone();
                    }
                }

                // Exponential backoff: 2s, 4s, 8s
                let backoff = Duration::from_secs(2u64.pow(attempts));
                tokio::select! {
                    _ = cancel_token.cancelled() => return Err("cancelled".to_string()),
                    _ = tokio::time::sleep(backoff) => {
                        // Retry
                    }
                }
            }
        }
    }
}

fn is_filesystem_error(msg: &str) -> bool {
    msg.contains("write error")
        || msg.contains("file create error")
        || msg.contains("file open error")
        || msg.contains("No space left")
        || msg.contains("Permission denied")
}

async fn download_chunk(
    client: &reqwest::Client,
    url: &str,
    part_path: &PathBuf,
    chunk: ChunkState,
    progress: Arc<TokioMutex<Vec<ChunkState>>>,
    cancel_token: CancellationToken,
    app_handle: &AppHandle,
    id: &str,
    total_bytes: u64,
) -> Result<(), String> {
    let range_start = chunk.start_byte + chunk.downloaded;
    let range_end = chunk.end_byte;

    if range_start > range_end {
        return Ok(());
    }

    let range_header = format!("bytes={}-{}", range_start, range_end);

    let response = client
        .get(url)
        .header(reqwest::header::RANGE, &range_header)
        .send()
        .await
        .map_err(|e| format!("Chunk {} network error: {}", chunk.id, e))?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        return Err(format!(
            "Chunk {} HTTP error: {}",
            chunk.id,
            response.status()
        ));
    }

    let mut file = if chunk.downloaded > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(part_path)
            .await
            .map_err(|e| format!("Chunk {} file open error: {}", chunk.id, e))?
    } else {
        tokio::fs::File::create(part_path)
            .await
            .map_err(|e| format!("Chunk {} file create error: {}", chunk.id, e))?
    };

    let mut stream = response.bytes_stream();
    let mut chunk_downloaded = chunk.downloaded;
    let mut last_emit = Instant::now();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                let mut chunks = progress.lock().await;
                if let Some(c) = chunks.iter_mut().find(|c| c.id == chunk.id) {
                    c.downloaded = chunk_downloaded;
                }
                return Err("cancelled".to_string());
            }
            chunk_result = stream.next() => {
                match chunk_result {
                    Some(Ok(data)) => {
                        tokio::io::AsyncWriteExt::write_all(&mut file, &data)
                            .await
                            .map_err(|e| format!("Chunk {} write error: {}", chunk.id, e))?;
                        chunk_downloaded += data.len() as u64;

                        if last_emit.elapsed().as_millis() >= 100 {
                            let total_downloaded = {
                                let mut chunks = progress.lock().await;
                                if let Some(c) = chunks.iter_mut().find(|c| c.id == chunk.id) {
                                    c.downloaded = chunk_downloaded;
                                }
                                chunks.iter().map(|c| c.downloaded).sum::<u64>()
                            };
                            emit_progress(app_handle, id, DownloadState::Downloading, total_downloaded, total_bytes);
                            let app_state = app_handle.state::<AppState>();
                            {
                                let mut downloads = app_state.downloads.lock().unwrap();
                                if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
                                    t.downloaded_bytes = total_downloaded;
                                }
                            }
                            last_emit = Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        // Update progress before returning error (for retry)
                        let mut chunks = progress.lock().await;
                        if let Some(c) = chunks.iter_mut().find(|c| c.id == chunk.id) {
                            c.downloaded = chunk_downloaded;
                        }
                        return Err(format!("Chunk {} stream error: {}", chunk.id, e));
                    }
                    None => {
                        let mut chunks = progress.lock().await;
                        if let Some(c) = chunks.iter_mut().find(|c| c.id == chunk.id) {
                            c.downloaded = chunk_downloaded;
                            c.is_complete = true;
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

// ─── Merge ───────────────────────────────────────────────────────────────────

async fn merge_parts(
    save_dir: &PathBuf,
    filename: &str,
    num_parts: u32,
    final_path: &PathBuf,
) -> Result<(), String> {
    let mut final_file = tokio::fs::File::create(final_path)
        .await
        .map_err(|e| format!("Failed to create final file: {}", e))?;

    for i in 0..num_parts {
        let part_path = part_file_path(save_dir, filename, i);
        let mut part_file = tokio::fs::File::open(&part_path)
            .await
            .map_err(|e| format!("Failed to open part {}: {}", i, e))?;
        tokio::io::copy(&mut part_file, &mut final_file)
            .await
            .map_err(|e| format!("Failed to copy part {}: {}", i, e))?;
        let _ = tokio::fs::remove_file(&part_path).await;
    }

    Ok(())
}

// ─── Single-stream fallback ─────────────────────────────────────────────────

async fn download_single(
    app_handle: &AppHandle,
    app_state: &AppState,
    client: &reqwest::Client,
    id: &str,
    url: &str,
    filename: &str,
    save_dir: &PathBuf,
    total_bytes: u64,
    cancel_token: CancellationToken,
) {
    let result = download_single_with_retry(
        app_handle,
        app_state,
        client,
        id,
        url,
        filename,
        save_dir,
        total_bytes,
        cancel_token.clone(),
    )
    .await;

    if let Err(e) = result {
        if e != "cancelled" {
            set_error(app_handle, app_state, id, &e);
        }
    }
}

async fn download_single_with_retry(
    app_handle: &AppHandle,
    app_state: &AppState,
    client: &reqwest::Client,
    id: &str,
    url: &str,
    filename: &str,
    save_dir: &PathBuf,
    total_bytes: u64,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let mut attempts = 0;

    loop {
        match download_single_attempt(
            app_handle,
            app_state,
            client,
            id,
            url,
            filename,
            save_dir,
            total_bytes,
            cancel_token.clone(),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(e) if e == "cancelled" => return Err(e),
            Err(e) if is_filesystem_error(&e) => return Err(e),
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_RETRIES {
                    return Err(format!("Failed after {} retries: {}", MAX_RETRIES, e));
                }
                let backoff = Duration::from_secs(2u64.pow(attempts));
                tokio::select! {
                    _ = cancel_token.cancelled() => return Err("cancelled".to_string()),
                    _ = tokio::time::sleep(backoff) => {}
                }
            }
        }
    }
}

async fn download_single_attempt(
    app_handle: &AppHandle,
    app_state: &AppState,
    client: &reqwest::Client,
    id: &str,
    url: &str,
    filename: &str,
    save_dir: &PathBuf,
    total_bytes: u64,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let file_path = save_dir.join(filename);

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let mut file = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| format!("File create error: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                {
                    let mut downloads = app_state.downloads.lock().unwrap();
                    if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
                        t.downloaded_bytes = downloaded;
                    }
                    drop(downloads);
                    let _ = app_state.save();
                }
                return Err("cancelled".to_string());
            }
            chunk_result = stream.next() => {
                match chunk_result {
                    Some(Ok(chunk)) => {
                        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                            .await
                            .map_err(|e| format!("Write error: {}", e))?;
                        downloaded += chunk.len() as u64;

                        if last_emit.elapsed().as_millis() >= 100 {
                            {
                                let mut downloads = app_state.downloads.lock().unwrap();
                                if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
                                    t.downloaded_bytes = downloaded;
                                }
                            }
                            emit_progress(app_handle, id, DownloadState::Downloading, downloaded, total_bytes);
                            last_emit = Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        return Err(format!("Stream error: {}", e));
                    }
                    None => break,
                }
            }
        }
    }

    // Complete
    {
        let mut downloads = app_state.downloads.lock().unwrap();
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
        let _ = app_state.save();
    }
    let final_total = if total_bytes > 0 {
        total_bytes
    } else {
        downloaded
    };
    emit_progress(
        app_handle,
        id,
        DownloadState::Completed,
        downloaded,
        final_total,
    );

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

fn set_error(app_handle: &AppHandle, app_state: &AppState, id: &str, msg: &str) {
    {
        let mut downloads = app_state.downloads.lock().unwrap();
        if let Some(t) = downloads.iter_mut().find(|d| d.id == id) {
            t.state = DownloadState::Error(msg.to_string());
        }
        drop(downloads);
        let _ = app_state.save();
    }
    emit_progress(app_handle, id, DownloadState::Error(msg.to_string()), 0, 0);
}
