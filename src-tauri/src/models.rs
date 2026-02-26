use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "message")]
pub enum DownloadState {
    Pending,
    Downloading,
    Paused,
    Scheduled, // Paused by the scheduler (will auto-resume)
    Completed,
    Error(String),
}

impl DownloadState {
    pub fn is_active(&self) -> bool {
        matches!(self, DownloadState::Downloading | DownloadState::Pending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileCategory {
    Video,
    Audio,
    Document,
    Archive,
    Application,
    Image,
    Other,
}

impl Default for FileCategory {
    fn default() -> Self {
        FileCategory::Other
    }
}

impl FileCategory {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" => FileCategory::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => FileCategory::Audio,
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "md" | "xls" | "xlsx" | "csv"
            | "ppt" | "pptx" => FileCategory::Document,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => FileCategory::Archive,
            "exe" | "msi" | "apk" | "app" | "dmg" | "deb" | "rpm" | "sh" | "bin" => {
                FileCategory::Application
            }
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => FileCategory::Image,
            _ => FileCategory::Other,
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(FileCategory::Other)
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
    #[serde(default)]
    pub category: FileCategory,
    pub date_added: String,
    #[serde(default)]
    pub supports_range: bool,
    #[serde(default)]
    pub chunks: Option<Vec<ChunkState>>,
    #[serde(default)]
    pub speed_limit_bytes: Option<usize>,
    #[serde(default)]
    pub is_torrent: bool,
    #[serde(default)]
    pub info_hash: Option<String>,
}
