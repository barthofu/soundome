use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::schema::{album, album_ref};

// ================================================================================================
// Album
// ================================================================================================

#[derive(
    Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd,
)]
#[diesel(table_name = album)]
pub struct AlbumEntity {
    pub id: i32,
    pub title: String,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = album)]
pub struct NewAlbumEntity {
    pub title: String,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = album)]
pub struct UpdateAlbumEntity {
    pub title: Option<String>,
    pub album_type: Option<String>,
    pub cover: Option<String>,
    pub date: Option<String>,
}

// ================================================================================================
// Album Source
// ================================================================================================

#[derive(
    Debug,
    Clone,
    Associations,
    Queryable,
    Identifiable,
    Insertable,
    Serialize,
    Ord,
    Eq,
    PartialEq,
    PartialOrd,
)]
#[diesel(table_name = album_ref)]
#[diesel(belongs_to(AlbumEntity, foreign_key = album_id))]
pub struct AlbumRefEntity {
    pub id: i32,
    pub album_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = album_ref)]
pub struct NewAlbumRefEntity {
    pub album_id: i32,
    #[diesel(column_name = "type_")]
    pub ref_type: String,
    pub platform: String,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = album_ref)]
pub struct UpdateAlbumRefEntity {
    pub album_id: Option<i32>,
    #[diesel(column_name = "type_")]
    pub ref_type: Option<String>,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
}
