use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{
    models::{SyncEntityType, SyncSchedule},
    types::SoundomeResult,
};

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

    /// Subscribe a URL to the global scheduled sync (playlist or artist,
    /// auto-detected/decided by the caller). Idempotent: if a subscription
    /// for this exact (entity_type, url) already exists, it is returned as-is.
    pub fn subscribe_url(
        &self,
        conn: &mut SqliteConnection,
        entity_type: SyncEntityType,
        url: String,
        label: Option<String>,
    ) -> SoundomeResult<SyncSchedule> {
        if let Some(existing) = self.repo.find_by_url(conn, entity_type, &url)? {
            return Ok(existing);
        }
        let schedule = SyncSchedule {
            id: None,
            entity_type,
            artist_id: None,
            reference_id: None,
            url,
            label,
            enabled: true,
            last_run: None,
            created_at: None,
        };
        self.repo.create(conn, &schedule)
    }

    /// Subscribe a specific artist source (reference) to the global scheduled
    /// sync. Idempotent: if this (artist_id, reference_id) pair is already
    /// subscribed, it is returned as-is.
    pub fn subscribe_artist_source(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
        reference_id: i32,
        url: String,
        label: Option<String>,
    ) -> SoundomeResult<SyncSchedule> {
        if let Some(existing) = self
            .repo
            .find_artist_subscription(conn, artist_id, reference_id)?
        {
            return Ok(existing);
        }
        let schedule = SyncSchedule {
            id: None,
            entity_type: SyncEntityType::Artist,
            artist_id: Some(artist_id),
            reference_id: Some(reference_id),
            url,
            label,
            enabled: true,
            last_run: None,
            created_at: None,
        };
        self.repo.create(conn, &schedule)
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        schedule: &SyncSchedule,
    ) -> SoundomeResult<SyncSchedule> {
        self.repo.update(conn, id, schedule)
    }

    pub fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.repo.delete(conn, id)
    }

    pub fn get_enabled(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        self.repo.get_enabled(conn)
    }

    pub fn mark_ran(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        self.repo.mark_ran(conn, id)
    }

    pub fn find_artist_subscription(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
        reference_id: i32,
    ) -> SoundomeResult<Option<SyncSchedule>> {
        self.repo
            .find_artist_subscription(conn, artist_id, reference_id)
    }

    pub fn find_by_url(
        &self,
        conn: &mut SqliteConnection,
        entity_type: SyncEntityType,
        url: &str,
    ) -> SoundomeResult<Option<SyncSchedule>> {
        self.repo.find_by_url(conn, entity_type, url)
    }
}
