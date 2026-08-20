use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::sync_schedule;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = sync_schedule)]
pub struct SyncScheduleEntity {
    pub id: i32,
    pub entity_type: String,
    pub artist_id: Option<i32>,
    pub reference_id: Option<i32>,
    pub url: String,
    pub label: Option<String>,
    pub enabled: i32,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = sync_schedule)]
pub struct NewSyncScheduleEntity {
    pub entity_type: String,
    pub artist_id: Option<i32>,
    pub reference_id: Option<i32>,
    pub url: String,
    pub label: Option<String>,
    pub enabled: i32,
    pub last_run: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = sync_schedule)]
pub struct UpdateSyncScheduleEntity {
    pub label: Option<String>,
    pub enabled: Option<i32>,
    pub last_run: Option<chrono::NaiveDateTime>,
}
