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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IngestResult {
    pub title: String,
    pub artists: Vec<String>,
    pub needs_validation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanCategory {
    Ok,
    PathChanged,
    TagConflict,
    Missing,
    Orphan,
    LegacyMatch,
    Unmanaged,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScanEntry {
    pub category: ScanCategory,
    pub file_path: Option<String>,
    pub track_id: Option<i32>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScanReport {
    pub dry_run: bool,
    pub library_root: String,
    pub entries: Vec<ScanEntry>,
    pub ok: usize,
    pub path_changed: usize,
    pub tag_conflict: usize,
    pub missing: usize,
    pub orphan: usize,
    pub legacy_match: usize,
    pub unmanaged: usize,
    pub paths_updated: usize,
    pub conflicts_flagged: usize,
}
