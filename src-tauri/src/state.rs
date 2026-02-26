use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::DownloadTask;
use crate::scheduler::ScheduleConfig;
use crate::speed_limiter::SpeedLimiter;

pub struct AppState {
    pub downloads: Mutex<Vec<DownloadTask>>,
    pub data_path: PathBuf,
    pub credentials_path: PathBuf,
    pub credentials: Mutex<HashMap<String, crate::models::Credential>>,
    pub global_limiter: Arc<SpeedLimiter>,
    pub task_limiters: Mutex<HashMap<String, Arc<SpeedLimiter>>>,
    pub schedule_config: Mutex<ScheduleConfig>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        let data_path = data_dir.join("downloads.json");
        let credentials_path = data_dir.join("credentials.json");

        let downloads: Vec<DownloadTask> = if data_path.exists() {
            match fs::read_to_string(&data_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        let credentials: HashMap<String, crate::models::Credential> = if credentials_path.exists() {
            match fs::read_to_string(&credentials_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        let mut task_limiters = HashMap::new();
        for task in &downloads {
            if let Some(limit) = task.speed_limit_bytes {
                task_limiters.insert(task.id.clone(), Arc::new(SpeedLimiter::new(limit)));
            }
        }

        AppState {
            downloads: Mutex::new(downloads),
            data_path,
            credentials_path,
            credentials: Mutex::new(credentials),
            global_limiter: Arc::new(SpeedLimiter::new(0)),
            task_limiters: Mutex::new(task_limiters),
            schedule_config: Mutex::new(ScheduleConfig::default()),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        // Save downloads
        {
            let downloads = self.downloads.lock().map_err(|e| e.to_string())?;
            let json = serde_json::to_string_pretty(&*downloads).map_err(|e| e.to_string())?;
            fs::write(&self.data_path, json).map_err(|e| e.to_string())?;
        }

        // Save credentials
        {
            let credentials = self.credentials.lock().map_err(|e| e.to_string())?;
            let json = serde_json::to_string_pretty(&*credentials).map_err(|e| e.to_string())?;
            fs::write(&self.credentials_path, json).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
