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
    /// Per-category breakdown updated live during sync.
    pub stats: Option<TaskStats>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

/// Live per-category counters persisted as JSON in the `stats` column.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskStats {
    /// Tracks fully downloaded and moved to library.
    pub downloaded: i32,
    /// Tracks saved as "needs_validation" (staged, awaiting approval).
    pub to_validate: i32,
    /// Tracks already in library — linked to playlist/artist but not re-downloaded.
    pub skipped: i32,
    /// Per-track failures that did not abort the whole sync.
    pub errors: Vec<TaskTrackError>,
}

/// One entry per track that failed during a sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTrackError {
    /// Human-readable display name of the track (e.g. "Artist - Title").
    pub track: String,
    /// Error message returned by the orchestrator workflow.
    pub reason: String,
    /// Track ID if available, for linking to the library.
    pub track_id: Option<i32>,
    /// Provider URL (audio source) for external link if track failed before DB save.
    pub provider_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr)]
pub enum TaskType {
    SyncPlaylist,
    SyncArtist,
    SyncAlbum,
    DownloadTrack,
}

impl TaskType {
    #[allow(clippy::should_implement_trait)]
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
    Cancelled,
}

impl TaskStatus {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "Pending" => TaskStatus::Pending,
            "Running" => TaskStatus::Running,
            "Completed" => TaskStatus::Completed,
            "Failed" => TaskStatus::Failed,
            "Cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
}
