use std::sync::Arc;

use diesel::SqliteConnection;

use crate::ports::repositories::TaskRepository;
use shared::models::{Task, TaskStatus, TaskType};

pub struct TaskService {
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskService {
    pub fn new(task_repo: Arc<dyn TaskRepository + Send + Sync>) -> Self {
        Self { task_repo }
    }

    pub fn get_by_id(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<Task> {
        self.task_repo.get_by_id(conn, id)
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> shared::types::SoundomeResult<Vec<Task>> {
        self.task_repo.get_all(conn)
    }

    /// Create a new pending task for a playlist sync.
    pub fn create_playlist_sync(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
        label: Option<String>,
    ) -> shared::types::SoundomeResult<Task> {
        let task = Task {
            id: None,
            task_type: TaskType::SyncPlaylist,
            status: TaskStatus::Pending,
            payload: serde_json::json!({ "url": url }).to_string(),
            label,
            progress: 0,
            total: None,
            error: None,
            stats: None,
            created_at: None,
            updated_at: None,
        };
        self.task_repo.create(conn, &task)
    }

    /// Create a new pending task for an artist sync.
    pub fn create_artist_sync(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
        label: Option<String>,
    ) -> shared::types::SoundomeResult<Task> {
        let task = Task {
            id: None,
            task_type: TaskType::SyncArtist,
            status: TaskStatus::Pending,
            payload: serde_json::json!({ "url": url }).to_string(),
            label,
            progress: 0,
            total: None,
            error: None,
            stats: None,
            created_at: None,
            updated_at: None,
        };
        self.task_repo.create(conn, &task)
    }

    /// Create a new pending task for an album sync.
    pub fn create_album_sync(
        &self,
        conn: &mut SqliteConnection,
        url: &str,
        label: Option<String>,
    ) -> shared::types::SoundomeResult<Task> {
        let task = Task {
            id: None,
            task_type: TaskType::SyncAlbum,
            status: TaskStatus::Pending,
            payload: serde_json::json!({ "url": url }).to_string(),
            label,
            progress: 0,
            total: None,
            error: None,
            stats: None,
            created_at: None,
            updated_at: None,
        };
        self.task_repo.create(conn, &task)
    }

    /// Create a new pending task for a batch ingest of all files in `ingest_dir`.
    pub fn create_ingest_dir(
        &self,
        conn: &mut SqliteConnection,
        ingest_dir: &str,
    ) -> shared::types::SoundomeResult<Task> {
        let task = Task {
            id: None,
            task_type: TaskType::IngestDir,
            status: TaskStatus::Pending,
            payload: serde_json::json!({ "ingest_dir": ingest_dir }).to_string(),
            label: Some(format!("Ingest: {ingest_dir}")),
            progress: 0,
            total: None,
            error: None,
            stats: None,
            created_at: None,
            updated_at: None,
        };
        self.task_repo.create(conn, &task)
    }

    pub fn set_running(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.set_running(conn, id)
    }

    pub fn update_progress(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        progress: i32,
        total: i32,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.update_progress(conn, id, progress, total)
    }

    pub fn set_completed(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.set_completed(conn, id)
    }

    pub fn set_failed(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        error: &str,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.set_failed(conn, id, error)
    }

    pub fn set_cancelled(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.set_cancelled(conn, id)
    }

    /// Return all tasks currently stuck in `Running` status (e.g. after a crash).
    pub fn get_stale_running(
        &self,
        conn: &mut SqliteConnection,
    ) -> shared::types::SoundomeResult<Vec<Task>> {
        self.task_repo
            .get_by_status(conn, TaskStatus::Running.as_ref())
    }

    /// Reset a task (Running or Failed) back to Pending so it can be re-executed.
    /// Progress is zeroed — already-processed tracks will be skipped automatically.
    pub fn reset_for_retry(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
    ) -> shared::types::SoundomeResult<Task> {
        self.task_repo.reset_for_retry(conn, id)?;
        self.task_repo.get_by_id(conn, id)
    }

    pub fn count_by_status(
        &self,
        conn: &mut SqliteConnection,
        status: &str,
    ) -> shared::types::SoundomeResult<i64> {
        self.task_repo.count_by_status(conn, status)
    }

    /// Update the task label in-place (e.g. set to the fetched playlist/album/artist name).
    pub fn update_label(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        label: &str,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.update_label(conn, id, label)
    }

    /// Persist live per-category stats (downloaded / to_validate / skipped / per-track errors).
    pub fn update_stats(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        stats: &shared::models::TaskStats,
    ) -> shared::types::SoundomeResult<()> {
        self.task_repo.update_stats(conn, id, stats)
    }
}
