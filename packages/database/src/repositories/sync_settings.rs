use domain::ports::repositories::SyncSettingsRepository;
use domain::schedule::calculate_next_run;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::SyncSettings, types::SoundomeResult};

use crate::{
    entities::{SyncSettingsEntity, UpdateSyncSettingsEntity},
    mappers::map_error,
    schema,
};

/// Singleton row (`id == 1`) holding the global cron configuration.
const SETTINGS_ID: i32 = 1;

#[derive(Default)]
pub struct DieselSyncSettingsRepository {}

impl DieselSyncSettingsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl SyncSettingsRepository for DieselSyncSettingsRepository {
    fn get(&self, conn: &mut SqliteConnection) -> SoundomeResult<SyncSettings> {
        let entity = schema::sync_settings::table
            .filter(schema::sync_settings::id.eq(SETTINGS_ID))
            .first::<SyncSettingsEntity>(conn)
            .map_err(map_error)?;
        Ok(SyncSettingsEntity::convert_to_domain(entity))
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        settings: &SyncSettings,
    ) -> SoundomeResult<SyncSettings> {
        let changeset = UpdateSyncSettingsEntity {
            cron_expression: Some(settings.cron_expression.clone()),
            enabled: Some(if settings.enabled { 1 } else { 0 }),
            last_run: settings.last_run,
            next_run: settings.next_run,
        };
        diesel::update(
            schema::sync_settings::table.filter(schema::sync_settings::id.eq(SETTINGS_ID)),
        )
        .set(&changeset)
        .execute(conn)
        .map_err(map_error)?;
        self.get(conn)
    }

    fn mark_ran(&self, conn: &mut SqliteConnection) -> SoundomeResult<()> {
        let settings = self.get(conn)?;
        let now = chrono::Utc::now().naive_utc();
        let next = calculate_next_run(now, &settings.cron_expression)?;
        diesel::update(
            schema::sync_settings::table.filter(schema::sync_settings::id.eq(SETTINGS_ID)),
        )
        .set((
            schema::sync_settings::last_run.eq(Some(now)),
            schema::sync_settings::next_run.eq(Some(next)),
        ))
        .execute(conn)
        .map_err(map_error)?;
        Ok(())
    }
}
