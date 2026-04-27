use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{reference::Platform, track::Track};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Option<i32>,
    pub name: String,
    pub source: Platform,
    pub source_url: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub id: Option<i32>,
    pub track: Track,
    pub added_at: Option<DateTime<Utc>>,
    pub position: Option<u32>,
}
