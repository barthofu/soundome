use chrono::{DateTime, Utc};

use super::track::Track;

#[derive(Debug, Clone)]
pub struct PlaylistTrack {
    pub track: Option<Track>,
    pub added_at: Option<DateTime<Utc>>,
    pub position: Option<u32>,
}