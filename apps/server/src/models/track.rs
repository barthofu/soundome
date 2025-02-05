use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::JsonSchema;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};

use crate::schema::track;

#[derive(JsonSchema, Queryable, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub comments: Option<String>,
    pub label: Option<String>,
    pub file_type: String,
    pub file_size: i32,
    pub file_path: String,
}

#[derive(JsonSchema, Insertable, Deserialize)]
#[diesel(table_name = track)]
pub struct NewTrack {
    pub title: String,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub comments: Option<String>,
    pub label: Option<String>,
    pub file_type: String,
    pub file_size: i32,
    pub file_path: String,
}

#[derive(JsonSchema, AsChangeset, Deserialize)]
#[diesel(table_name = track)]
pub struct UpdateTrack {
    pub title: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub comments: Option<String>,
    pub label: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i32>,
    pub file_path: Option<String>,
}
