use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::{entities::AlbumEntity, schema::{track, track_source}};

#[derive(Debug, Clone, Associations, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = track)]
#[diesel(belongs_to(AlbumEntity, foreign_key = album_id))]
pub struct TrackEntity {
    pub id: i32,
    pub title: String,
    pub duration: Option<i32>,
    pub album_id: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub file_path: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub source_id: Option<String>,
    pub provider: Option<String>,
    pub provider_url: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = track)]
pub struct NewTrackEntity {
    pub title: String,
    pub duration: Option<i32>,
    pub album_id: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub file_path: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub source_id: Option<String>,
    pub provider: Option<String>,
    pub provider_url: Option<String>,
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = track)]
pub struct UpdateTrackEntity {
    pub title: Option<String>,
    pub duration: Option<i32>,
    pub album_id: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub file_path: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub source_id: Option<String>,
    pub provider: Option<String>,
    pub provider_url: Option<String>,
    pub provider_id: Option<String>,
}

// ================================================================================================
// Track Source
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = track_source)]
pub struct TrackSourceEntity {
    pub id: i32,
    pub track_id: i32,
    pub external_id: String,
    pub platform: String,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = track_source)]
pub struct NewTrackSourceEntity {
    pub track_id: i32,
    pub external_id: String,
    pub platform: String,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = track_source)]
pub struct UpdateTrackSourceEntity {
    pub track_id: Option<i32>,
    pub external_id: Option<String>,
    pub platform: Option<String>,
}
