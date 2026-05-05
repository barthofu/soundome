use domain::ports::repositories::TaskRepository;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use shared::{models::Task, types::SoundomeResult};

use crate::{
    entities::{NewTaskEntity, TaskEntity},
    mappers::map_error,
    schema,
};

pub struct DieselTaskRepository {}

impl DieselTaskRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl TaskRepository for DieselTaskRepository {
    fn create(&self, conn: &mut SqliteConnection, task: &Task) -> SoundomeResult<Task> {
        let new_entity = NewTaskEntity::convert_from_domain(task);
        diesel::insert_into(schema::task::table)
            .values(&new_entity)
            .execute(conn)
            .map_err(map_error)?;
        let created = schema::task::table
            .order(schema::task::id.desc())
            .first::<TaskEntity>(conn)
            .map_err(map_error)?;
        Ok(TaskEntity::convert_to_domain(created))
    }

    fn get_by_id(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<Task> {
        let entity = schema::task::table
            .filter(schema::task::id.eq(id))
            .first::<TaskEntity>(conn)
            .map_err(map_error)?;
        Ok(TaskEntity::convert_to_domain(entity))
    }

    fn get_all(&self, conn: &mut SqliteConnection) -> SoundomeResult<Vec<Task>> {
        let entities = schema::task::table
            .order(schema::task::id.desc())
            .load::<TaskEntity>(conn)
            .map_err(map_error)?;
        Ok(entities.into_iter().map(TaskEntity::convert_to_domain).collect())
    }

    fn set_running(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::status.eq("Running"),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn update_progress(&self, conn: &mut SqliteConnection, id: i32, progress: i32, total: i32) -> SoundomeResult<()> {
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::progress.eq(progress),
                schema::task::total.eq(Some(total)),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn set_completed(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        // Set progress = total on completion
        let current_total: Option<i32> = schema::task::table
            .filter(schema::task::id.eq(id))
            .select(schema::task::total)
            .first::<Option<i32>>(conn)
            .unwrap_or(None);
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::status.eq("Completed"),
                schema::task::progress.eq(current_total.unwrap_or(0)),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn set_failed(&self, conn: &mut SqliteConnection, id: i32, error: &str) -> SoundomeResult<()> {
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::status.eq("Failed"),
                schema::task::error.eq(Some(error)),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn set_cancelled(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::status.eq("Cancelled"),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn get_by_status(&self, conn: &mut SqliteConnection, status: &str) -> SoundomeResult<Vec<Task>> {
        let entities = schema::task::table
            .filter(schema::task::status.eq(status))
            .order(schema::task::id.asc())
            .load::<TaskEntity>(conn)
            .map_err(map_error)?;
        Ok(entities.into_iter().map(TaskEntity::convert_to_domain).collect())
    }

    fn reset_for_retry(&self, conn: &mut SqliteConnection, id: i32) -> SoundomeResult<()> {
        diesel::update(schema::task::table.filter(schema::task::id.eq(id)))
            .set((
                schema::task::status.eq("Pending"),
                schema::task::progress.eq(0),
                schema::task::error.eq(None::<String>),
                schema::task::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(map_error)?;
        Ok(())
    }

    fn count_by_status(&self, conn: &mut SqliteConnection, status: &str) -> SoundomeResult<i64> {
        schema::task::table
            .filter(schema::task::status.eq(status))
            .count()
            .get_result(conn)
            .map_err(map_error)
    }
}
