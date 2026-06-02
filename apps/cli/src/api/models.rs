#![allow(dead_code)]

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaylistDto {
    pub id: i32,
    pub name: String,
    pub source: String,
    pub source_url: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistTrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistTrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ArtistDto {
    pub id: i32,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlbumArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlbumDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<AlbumArtistDto>,
    pub album_type: String,
    pub cover: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackArtistDto {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackAlbumDto {
    pub id: Option<i32>,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackDto {
    pub id: i32,
    pub title: String,
    pub artists: Vec<TrackArtistDto>,
    pub album: Option<TrackAlbumDto>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,
    pub file_path: Option<String>,
    pub needs_validation: bool,
}
