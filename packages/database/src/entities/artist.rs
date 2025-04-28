use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::{entities::{album::AlbumEntity, track::TrackEntity}, schema::{artist, artist_albums, artist_ref, artist_tracks}};

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = artist)]
pub struct ArtistEntity {
    pub id: i32,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = artist)]
pub struct NewArtistEntity {
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = artist)]
pub struct UpdateArtistEntity {
    pub name: Option<String>,
    pub icon: Option<String>,
}

// ================================================================================================
// Associations
// ================================================================================================

#[derive(Debug, Clone, Associations, Queryable, Selectable, Identifiable, Insertable, Serialize, PartialEq, PartialOrd)]
#[diesel(belongs_to(ArtistEntity, foreign_key = artist_id))]
#[diesel(belongs_to(TrackEntity, foreign_key = track_id))]
#[diesel(table_name = artist_tracks)]
#[diesel(primary_key(track_id, artist_id))]
pub struct ArtistTrackEntity {
    pub track_id: i32,
    pub artist_id: i32,
}

#[derive(Debug, Clone, Associations, Queryable, Selectable, Identifiable, Insertable, Serialize, PartialEq, PartialOrd)]
#[diesel(belongs_to(ArtistEntity, foreign_key = artist_id))]
#[diesel(belongs_to(AlbumEntity, foreign_key = album_id))]
#[diesel(table_name = artist_albums)]
#[diesel(primary_key(album_id, artist_id))]
pub struct ArtistAlbumEntity {
    pub album_id: i32,
    pub artist_id: i32,
}

// ================================================================================================
// Artist Source
// ================================================================================================

#[derive(Debug, Clone, Associations, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = artist_ref)]
#[diesel(belongs_to(ArtistEntity, foreign_key = artist_id))]
pub struct ArtistRefEntity {
    pub id: i32,
    pub artist_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = artist_ref)]
pub struct NewArtistRefEntity {
    pub artist_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = artist_ref)]
pub struct UpdateArtistRefEntity {
    pub artist_id: Option<i32>,
    #[diesel(column_name = "type_")]
    pub ref_type: Option<String>,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}
