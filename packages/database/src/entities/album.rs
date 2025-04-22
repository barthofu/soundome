use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::schema::{album, album_source};

// ================================================================================================
// Album
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = album)]
pub struct AlbumEntity {
    pub id: i32,
    pub title: String,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = album)]
pub struct NewAlbumEntity {
    pub title: String,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = album)]
pub struct UpdateAlbumEntity {
    pub title: Option<String>,
    pub album_type: Option<String>,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub url: Option<String>,
}

// ================================================================================================
// Album Source
// ================================================================================================

#[derive(Debug, Clone, Queryable, Identifiable, Insertable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = album_source)]
pub struct AlbumSourceEntity {
    pub id: i32,
    pub album_id: i32,
    pub external_id: String,
    pub platform: String,
}

#[derive(Debug, Clone, Insertable, Deserialize, JsonSchema)]
#[diesel(table_name = album_source)]
pub struct NewAlbumSourceEntity {
    pub album_id: i32,
    pub external_id: String,
    pub platform: String,
}

#[derive(Debug, Clone, AsChangeset, Deserialize, JsonSchema)]
#[diesel(table_name = album_source)]
pub struct UpdateAlbumSourceEntity {
    pub album_id: Option<i32>,
    pub external_id: Option<String>,
    pub platform: Option<String>,
}
