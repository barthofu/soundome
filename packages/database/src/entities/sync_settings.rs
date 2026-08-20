use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::sync_settings;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = sync_settings)]
pub struct SyncSettingsEntity {
    pub id: i32,
    pub cron_expression: String,
    pub enabled: i32,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub next_run: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = sync_settings)]
pub struct UpdateSyncSettingsEntity {
    pub cron_expression: Option<String>,
    pub enabled: Option<i32>,
    pub last_run: Option<chrono::NaiveDateTime>,
    pub next_run: Option<chrono::NaiveDateTime>,
}
