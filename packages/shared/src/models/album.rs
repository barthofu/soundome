use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::utils::string::{string_similarity, SimilarityAlgorithm};

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

struct Weights;
impl Weights {
    const TITLE: f64 = 1.0;
    const ARTISTS: f64 = 0.7;
    const RELEASE_DATE: f64 = 0.3;
}

impl Album {

    pub fn get_source(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Source)
            .cloned()
    }

    pub fn get_sources(&self) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|r| r.ref_type == ReferenceType::Source)
            .cloned()
            .collect()
    }

    pub fn get_provider(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
    }

    pub fn get_providers(&self) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Metadata)
            .cloned()
    }

    pub fn display(&self) -> String {
        self.title.clone()
    }

    pub fn compare(&self, other: &Album) -> f64 {
        let title_similarity = string_similarity(
            &self.title,
            &other.title,
            SimilarityAlgorithm::Smart,
        );

        let artists_similarity = if !self.artists.is_empty() && !other.artists.is_empty() {
            let mut total = 0.0;
            for artist in &self.artists {
                let best_match = other.artists.iter()
                    .map(|a| artist.compare(a))
                    .fold(0./0., f64::max); // max or NaN
                total += best_match;
            }
            total / self.artists.len() as f64
        } else {
            0.0
        };

        let release_date_similarity = if let (Some(date1), Some(date2)) = (&self.date, &other.date) {
            if date1 == date2 {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Weighted average
        let total_weight = Weights::TITLE + Weights::ARTISTS + Weights::RELEASE_DATE;
        (title_similarity * Weights::TITLE +
         artists_similarity * Weights::ARTISTS +
         release_date_similarity * Weights::RELEASE_DATE) / total_weight
    }

    pub fn transpose_metadata(&mut self, other: &Album) {
        self.title = other.title.clone();
        self.album_type = other.album_type.clone();
        if let Some(cover) = &other.cover { self.cover = Some(cover.clone()); };
        if let Some(date) = &other.date { self.date = Some(date.clone()); };
        
        // only add new references, do not overwrite existing ones
        for ref_item in &other.references {
            let reference_already_exists = self.references
                .iter()
                .any(|r| 
                    r.platform == ref_item.platform && 
                    r.external_id == ref_item.external_id && 
                    r.ref_type == ref_item.ref_type
                );

            if !reference_already_exists {
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
        match s.to_lowercase().as_str() {
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
