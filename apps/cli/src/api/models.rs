#![allow(dead_code)]

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PlaylistDto {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistTrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistTrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistTrackDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<PlaylistTrackArtistDto>,
    pub album: Option<PlaylistTrackAlbumDto>,
    pub duration: Option<i32>,
    pub cover: Option<String>,
    pub genre: Option<String>,
    pub file_path: Option<String>,
}
