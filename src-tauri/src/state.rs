use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::DownloadTask;

pub struct AppState {
    pub downloads: Mutex<Vec<DownloadTask>>,
    pub data_path: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        let data_path = data_dir.join("downloads.json");

        let downloads = if data_path.exists() {
            match fs::read_to_string(&data_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        AppState {
            downloads: Mutex::new(downloads),
            data_path,
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let downloads = self.downloads.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*downloads).map_err(|e| e.to_string())?;
        fs::write(&self.data_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}
