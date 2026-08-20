use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A sync "subscription": something to keep in sync on every global cron
/// pass. Individual subscriptions no longer carry their own interval/cron —
/// scheduling is driven entirely by the single global `SyncSettings` row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSchedule {
    pub id: Option<i32>,
    pub entity_type: SyncEntityType,
    /// Set when `entity_type == Artist`.
    pub artist_id: Option<i32>,
    /// Set when `entity_type == Artist`: the specific `Source` reference
    /// chosen to sync from.
    pub reference_id: Option<i32>,
    /// Resolved target URL: a playlist's `source_url`, or the artist
    /// reference's `external_url`.
    pub url: String,
    pub label: Option<String>,
    pub enabled: bool,
    pub last_run: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncEntityType {
    Playlist,
    Artist,
}

impl SyncEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncEntityType::Playlist => "playlist",
            SyncEntityType::Artist => "artist",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "artist" => SyncEntityType::Artist,
            _ => SyncEntityType::Playlist,
        }
    }
}

/// Singleton global cron configuration driving every scheduled sync
/// subscription in one pass. There is always exactly one row (`id == 1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: Option<NaiveDateTime>,
}
