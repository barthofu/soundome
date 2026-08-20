use domain::ports::repositories::SyncScheduleRepository;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::SyncSchedule, types::SoundomeResult};

use crate::{
    entities::{NewSyncScheduleEntity, SyncScheduleEntity, UpdateSyncScheduleEntity},
    mappers::map_error,
    schema,
};

#[derive(Default)]
pub struct DieselSyncScheduleRepository {}

impl DieselSyncScheduleRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl SyncScheduleRepository for DieselSyncScheduleRepository {
    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        let entities = schema::sync_schedule::table
            .order(schema::sync_schedule::id.asc())
            .load::<SyncScheduleEntity>(conn)
            .map_err(map_error)?;
        Ok(entities
            .into_iter()
            .map(SyncScheduleEntity::convert_to_domain)
            .collect())
    }

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<SyncSchedule> {
        let entity = schema::sync_schedule::table
            .filter(schema::sync_schedule::id.eq(id))
            .first::<SyncScheduleEntity>(conn)
            .map_err(map_error)?;
        Ok(SyncScheduleEntity::convert_to_domain(entity))
    }

    fn create(
        &self,
        conn: &mut SqliteConnection,
        schedule: &SyncSchedule,
    ) -> SoundomeResult<SyncSchedule> {
        let new_entity = NewSyncScheduleEntity::convert_from_domain(schedule);
        diesel::insert_into(schema::sync_schedule::table)
            .values(&new_entity)
            .execute(conn)
            .map_err(map_error)?;
        let created = schema::sync_schedule::table
            .order(schema::sync_schedule::id.desc())
            .first::<SyncScheduleEntity>(conn)
            .map_err(map_error)?;
        Ok(SyncScheduleEntity::convert_to_domain(created))
    }

    fn update(
        &self,
        conn: &mut SqliteConnection,
        id: i32,
        schedule: &SyncSchedule,
    ) -> SoundomeResult<SyncSchedule> {
        let changeset = UpdateSyncScheduleEntity {
            label: schedule.label.clone(),
            enabled: Some(if schedule.enabled { 1 } else { 0 }),
            last_run: schedule.last_run,
        };
        diesel::update(schema::sync_schedule::table.filter(schema::sync_schedule::id.eq(id)))
            .set(&changeset)
            .execute(conn)
            .map_err(map_error)?;
        self.get_by_id(conn, id)
    }

    fn delete(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        diesel::delete(schema::sync_schedule::table.filter(schema::sync_schedule::id.eq(id)))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn get_enabled(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        let entities = schema::sync_schedule::table
            .filter(schema::sync_schedule::enabled.eq(1))
            .load::<SyncScheduleEntity>(conn)
            .map_err(map_error)?;
        Ok(entities
            .into_iter()
            .map(SyncScheduleEntity::convert_to_domain)
            .collect())
    }

    fn mark_ran(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        let now = chrono::Utc::now().naive_utc();
        diesel::update(schema::sync_schedule::table.filter(schema::sync_schedule::id.eq(id)))
            .set(schema::sync_schedule::last_run.eq(Some(now)))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn find_artist_subscription(
        &self,
        conn: &mut SqliteConnection,
        artist_id: i32,
        reference_id: i32,
    ) -> SoundomeResult<Option<SyncSchedule>> {
        let entity = schema::sync_schedule::table
            .filter(schema::sync_schedule::entity_type.eq("artist"))
            .filter(schema::sync_schedule::artist_id.eq(artist_id))
            .filter(schema::sync_schedule::reference_id.eq(reference_id))
            .first::<SyncScheduleEntity>(conn)
            .optional()
            .map_err(map_error)?;
        Ok(entity.map(SyncScheduleEntity::convert_to_domain))
    }

    fn find_by_url(
        &self,
        conn: &mut SqliteConnection,
        entity_type: shared::models::SyncEntityType,
        url: &str,
    ) -> SoundomeResult<Option<SyncSchedule>> {
        let entity = schema::sync_schedule::table
            .filter(schema::sync_schedule::entity_type.eq(entity_type.as_str()))
            .filter(schema::sync_schedule::url.eq(url))
            .first::<SyncScheduleEntity>(conn)
            .optional()
            .map_err(map_error)?;
        Ok(entity.map(SyncScheduleEntity::convert_to_domain))
    }
}
