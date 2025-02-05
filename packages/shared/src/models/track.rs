use super::{artist::Artist, album::Album};

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artists: Vec<Artist>,
    pub url: Option<String>,
    pub album: Option<Album>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
}