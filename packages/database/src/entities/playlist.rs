use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{playlist, playlist_tracks};

// ================================================================================================
// Playlist
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize)]
#[diesel(table_name = playlist)]
pub struct PlaylistEntity {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub cover: Option<String>,
    pub last_sync: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = playlist)]
pub struct NewPlaylistEntity {
    pub name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub cover: Option<String>,
    pub last_sync: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = playlist)]
pub struct UpdatePlaylistEntity {
    pub name: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub cover: Option<String>,
    pub last_sync: Option<chrono::NaiveDateTime>,
}

// ================================================================================================
// PlaylistTrack (junction table)
// ================================================================================================

#[derive(Debug, Clone, Queryable, Insertable, Serialize)]
#[diesel(table_name = playlist_tracks)]
pub struct PlaylistTrackEntity {
    pub track_id: i32,
    pub playlist_id: i32,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = playlist_tracks)]
pub struct NewPlaylistTrackEntity {
    pub track_id: i32,
    pub playlist_id: i32,
    pub position: Option<i32>,
}
