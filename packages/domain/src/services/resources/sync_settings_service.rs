use std::sync::Arc;

use diesel::SqliteConnection;
use shared::{models::SyncSettings, types::SoundomeResult};

use crate::ports::repositories::SyncSettingsRepository;

/// Manages the single global cron configuration that drives every scheduled
/// sync subscription (see `SyncScheduleService`).
pub struct SyncSettingsService {
    repo: Arc<dyn SyncSettingsRepository + Send + Sync>,
}

impl SyncSettingsService {
    pub fn new(repo: Arc<dyn SyncSettingsRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub fn get(&self, conn: &mut SqliteConnection) -> SoundomeResult<SyncSettings> {
        self.repo.get(conn)
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        settings: &SyncSettings,
    ) -> SoundomeResult<SyncSettings> {
        self.repo.update(conn, settings)
    }

    pub fn mark_ran(&self, conn: &mut SqliteConnection) -> SoundomeResult<()> {
        self.repo.mark_ran(conn)
    }
}
