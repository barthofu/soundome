use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i32>,
    pub task_type: TaskType,
    pub status: TaskStatus,
    /// JSON payload (e.g. `{"url": "..."}`)
    pub payload: String,
    pub label: Option<String>,
    /// Number of items processed so far.
    pub progress: i32,
    /// Total number of items, if known.
    pub total: Option<i32>,
    pub error: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr)]
pub enum TaskType {
    SyncPlaylist,
    SyncArtist,
    SyncAlbum,
    DownloadTrack,
}

impl TaskType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "SyncPlaylist" => TaskType::SyncPlaylist,
            "SyncArtist" => TaskType::SyncArtist,
            "SyncAlbum" => TaskType::SyncAlbum,
            "DownloadTrack" => TaskType::DownloadTrack,
            _ => TaskType::DownloadTrack,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Pending" => TaskStatus::Pending,
            "Running" => TaskStatus::Running,
            "Completed" => TaskStatus::Completed,
            "Failed" => TaskStatus::Failed,
            _ => TaskStatus::Pending,
        }
    }
}
