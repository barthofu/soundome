use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSchedule {
    pub id: Option<i32>,
    pub playlist_url: String,
    pub label: Option<String>,
    /// Sync interval in seconds (mutually exclusive with cron_expression).
    pub interval_seconds: Option<i32>,
    /// Cron expression for scheduling (mutually exclusive with interval_seconds).
    pub cron_expression: Option<String>,
    pub enabled: bool,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}
