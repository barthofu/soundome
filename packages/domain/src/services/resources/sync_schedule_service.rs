use std::sync::Arc;

use chrono::Duration;
use diesel::SqliteConnection;
use shared::{models::SyncSchedule, types::SoundomeResult};

use crate::ports::repositories::SyncScheduleRepository;

pub struct SyncScheduleService {
    repo: Arc<dyn SyncScheduleRepository + Send + Sync>,
}

impl SyncScheduleService {
    pub fn new(repo: Arc<dyn SyncScheduleRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        self.repo.get_all(conn)
    }

    pub fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<SyncSchedule> {
        self.repo.get_by_id(conn, id)
    }

    pub fn create(&self, conn: &mut SqliteConnection, playlist_url: String, label: Option<String>, interval_seconds: i32) -> SoundomeResult<SyncSchedule> {
        let now = chrono::Utc::now().naive_utc();
        let next_run = now + Duration::seconds(interval_seconds as i64);
        let schedule = SyncSchedule {
            id: None,
            playlist_url,
            label,
            interval_seconds,
            enabled: true,
            last_run: None,
            next_run: Some(next_run),
            created_at: None,
        };
        self.repo.create(conn, &schedule)
    }

    pub fn update(&self, conn: &mut SqliteConnection, id: i32, schedule: &SyncSchedule) -> SoundomeResult<SyncSchedule> {
        self.repo.update(conn, id, schedule)
    }

    pub fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.repo.delete(conn, id)
    }

    pub fn get_due(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        self.repo.get_due(conn)
    }

    pub fn mark_ran(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.repo.mark_ran(conn, id)
    }
}
