use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use super::{artist::Artist, Reference, ReferenceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<i32>,
    pub title: String,
    pub artists: Vec<Artist>,
    pub album_type: AlbumType,
    pub cover: Option<String>,
    pub date: Option<String>,
    pub references: Vec<Reference>
}

impl Album {

    pub fn get_source(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Source)
            .cloned()
    }

    pub fn get_provider(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
    }

    pub fn display(&self) -> String {
        self.title.clone()
    }

    pub fn transpose_metadata(&mut self, other: &Album) {
        self.title = other.title.clone();
        self.album_type = other.album_type.clone();
        if let Some(cover) = &other.cover { self.cover = Some(cover.clone()); };
        if let Some(date) = &other.date { self.date = Some(date.clone()); };
        for ref_item in &other.references {
            if !self.references.iter().any(|r| r.platform == ref_item.platform && r.external_id == ref_item.external_id && r.ref_type == ref_item.ref_type) {
                self.references.push(ref_item.clone());
            }
        }
    }
}

// ================================================================================================
// Enums
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr)]
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
