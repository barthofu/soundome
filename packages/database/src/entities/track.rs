use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::{entities::AlbumEntity, schema::{track, track_ref}};

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
}

// ================================================================================================
// Track Source
// ================================================================================================

#[derive(Debug, Clone, Associations, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = track_ref)]
#[diesel(belongs_to(TrackEntity, foreign_key = track_id))]
pub struct TrackRefEntity {
    pub id: i32,
    pub track_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = track_ref)]
pub struct NewTrackRefEntity {
    pub track_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = track_ref)]
pub struct UpdateTrackRefEntity {
    pub track_id: Option<i32>,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}
