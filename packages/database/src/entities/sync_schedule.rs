use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::sync_schedule;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = sync_schedule)]
pub struct SyncScheduleEntity {
    pub id: i32,
    pub playlist_url: String,
    pub label: Option<String>,
    pub interval_seconds: i32,
    pub enabled: i32,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub next_run: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = sync_schedule)]
pub struct NewSyncScheduleEntity {
    pub playlist_url: String,
    pub label: Option<String>,
    pub interval_seconds: i32,
    pub enabled: i32,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub next_run: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = sync_schedule)]
pub struct UpdateSyncScheduleEntity {
    pub label: Option<String>,
    pub interval_seconds: Option<i32>,
    pub enabled: Option<i32>,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub next_run: Option<chrono::NaiveDateTime>,
}
