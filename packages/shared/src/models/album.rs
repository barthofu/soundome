use strum::AsRefStr;

use super::{artist::Artist, Reference};

#[derive(Debug, Clone)]
pub struct Album {
    pub id: Option<i32>,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album_type: AlbumType,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub references: Vec<Reference>
}

// ================================================================================================
// Enums
// ================================================================================================

#[derive(Debug, Clone, AsRefStr)]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
    EP,
    Mixtape,
    Soundtrack,
    Live,
    Remix,
    Bootleg,
    DJMix,
    Unknown,
}

impl AlbumType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "album" => AlbumType::Album,
            "single" => AlbumType::Single,
            "compilation" => AlbumType::Compilation,
            "ep" => AlbumType::EP,
            "mixtape" => AlbumType::Mixtape,
            "soundtrack" => AlbumType::Soundtrack,
            "live" => AlbumType::Live,
            "remix" => AlbumType::Remix,
            "bootleg" => AlbumType::Bootleg,
            "djmix" => AlbumType::DJMix,
            _ => AlbumType::Unknown,
        }
    }
}
