use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSchedule {
    pub id: Option<i32>,
    pub playlist_url: String,
    pub label: Option<String>,
    /// Sync interval in seconds.
    pub interval_seconds: i32,
    pub enabled: bool,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}
