use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use url::Url;

/// Public tilde expander for use in other modules (e.g. checksum).
pub fn expand_tilde_public(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Last URL we detected from clipboard, to avoid re-emitting.
struct LastClipboardUrl(Mutex<String>);

/// Polls the clipboard every 1.5 seconds looking for new HTTP(S) URLs.
pub async fn start_clipboard_monitor(app_handle: AppHandle) {
    app_handle.manage(LastClipboardUrl(Mutex::new(String::new())));

    loop {
        tokio::time::sleep(Duration::from_millis(1500)).await;

        let text = match app_handle.clipboard().read_text() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        // Check if it's a valid HTTP(S) URL or Magnet link
        if text.starts_with("magnet:?") {
            // Valid magnet link
        } else if let Ok(parsed) = Url::parse(&text) {
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                continue;
            }
        } else {
            continue;
        }

        // Check if it's the same URL we already emitted
        let last = app_handle.state::<LastClipboardUrl>();
        {
            let mut last_url = last.0.lock().unwrap();
            if *last_url == text {
                continue;
            }
            *last_url = text.clone();
        }

        // Emit event to frontend
        let _ = app_handle.emit("clipboard_url_detected", text);
    }
}
