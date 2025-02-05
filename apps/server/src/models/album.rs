use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::schema::album;

#[derive(JsonSchema, Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Album {
    pub id: i32,
    pub title: String,
    pub cover: Option<String>,
    pub year: Option<i32>,
}

#[derive(JsonSchema, Insertable, Deserialize)]
#[diesel(table_name = album)]
pub struct NewAlbum {
    pub title: String,
    pub cover: Option<String>,
    pub year: Option<i32>,
}

#[derive(JsonSchema, AsChangeset, Deserialize)]
#[diesel(table_name = album)]
pub struct UpdateAlbum {
    pub title: Option<String>,
    pub cover: Option<String>,
    pub year: Option<i32>,
}
