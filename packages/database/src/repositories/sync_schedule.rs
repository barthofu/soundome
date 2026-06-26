use domain::ports::repositories::SyncScheduleRepository;
use domain::schedule::calculate_next_run;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
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
            interval_seconds: schedule.interval_seconds,
            cron_expression: schedule.cron_expression.clone(),
            enabled: Some(if schedule.enabled { 1 } else { 0 }),
            last_run: schedule.last_run,
            next_run: schedule.next_run,
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

    fn get_due(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<SyncSchedule>> {
        use diesel::BoolExpressionMethods;
        let now = chrono::Utc::now().naive_utc();
        let entities = schema::sync_schedule::table
            .filter(schema::sync_schedule::enabled.eq(1))
            .filter(
                schema::sync_schedule::next_run
                    .is_null()
                    .or(schema::sync_schedule::next_run.le(now)),
            )
            .load::<SyncScheduleEntity>(conn)
            .map_err(map_error)?;
        Ok(entities
            .into_iter()
            .map(SyncScheduleEntity::convert_to_domain)
            .collect())
    }

    fn mark_ran(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        // Fetch schedule to get interval_seconds and cron_expression
        let schedule = self.get_by_id(conn, id)?;

        let now = chrono::Utc::now().naive_utc();
        let next = calculate_next_run(now, schedule.interval_seconds, schedule.cron_expression.as_deref())?;

        diesel::update(schema::sync_schedule::table.filter(schema::sync_schedule::id.eq(id)))
            .set((
                schema::sync_schedule::last_run.eq(Some(now)),
                schema::sync_schedule::next_run.eq(Some(next)),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }
}
