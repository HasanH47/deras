use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum DownloadState {
    Pending,
    Downloading,
    Paused,
    Completed,
    Error(String),
}

impl DownloadState {
    pub fn is_active(&self) -> bool {
        matches!(self, DownloadState::Downloading | DownloadState::Pending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkState {
    pub id: u32,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded: u64,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_path: String,
    pub state: DownloadState,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub date_added: String,
    #[serde(default)]
    pub supports_range: bool,
    #[serde(default)]
    pub chunks: Option<Vec<ChunkState>>,
}
