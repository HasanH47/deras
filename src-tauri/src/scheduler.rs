use chrono::{Local, NaiveTime};
use tauri::Manager;
use tokio::time::{sleep, Duration};

use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    /// Format: "HH:MM" (e.g., "00:00")
    pub start_time: String,
    /// Format: "HH:MM" (e.g., "06:00")
    pub end_time: String,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start_time: "00:00".to_string(), // 12:00 AM
            end_time: "06:00".to_string(),   // 06:00 AM
        }
    }
}

pub fn is_active_window(now: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
    if start < end {
        // Normal window, e.g., 08:00 to 17:00
        now >= start && now < end
    } else {
        // Overnight window, e.g., 22:00 to 06:00
        now >= start || now < end
    }
}

pub async fn start_scheduler_loop(app_handle: tauri::AppHandle) {
    loop {
        // Sleep for 1 minute
        sleep(Duration::from_secs(60)).await;

        let state = app_handle.state::<AppState>();

        // Wait, to safely access state and potentially pause/resume downloads,
        // we'll need to lock the downloads list and invoke commands via the engine or commands module.
        // Let's first read the global config from the state.

        let config = {
            let config_lock = state.schedule_config.lock().unwrap();
            config_lock.clone()
        };

        if !config.enabled {
            continue;
        }

        let start_res = NaiveTime::parse_from_str(&config.start_time, "%H:%M");
        let end_res = NaiveTime::parse_from_str(&config.end_time, "%H:%M");

        if let (Ok(start), Ok(end)) = (start_res, end_res) {
            let now = Local::now().time();
            let is_active = is_active_window(now, start, end);

            // In active window: Auto-Resume downloads that were paused BY SCHEDULE?
            // Outside active window: Auto-Pause all active downloads!

            // To be safe and simple for MVP:
            // If outside active window, we pause EVERYTHING that is Downloading/Pending.
            // If inside active window, we ONLY resume if we add a new State variant or flag to remember
            // what was paused "by schedule" versus "by user".
            // Let's implement that logic via AppState.

            crate::commands::process_schedule(&app_handle, is_active).await;
        }
    }
}
