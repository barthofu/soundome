use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::{
    models::{album::Album, artist::Artist, Platform},
    utils::string::{string_similarity, SimilarityAlgorithm},
};

use super::{Reference, ReferenceType};

// ================================================================================================
// Enums
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr)]
pub enum TrackSource {
    Local,
    Spotify,
    Youtube,
    YoutubeMusic,
    SoundCloud,
    Unknown,
}

impl TrackSource {
    pub fn from_str(source: &str) -> Self {
        match source.to_lowercase().as_str() {
            "local" => TrackSource::Local,
            "spotify" => TrackSource::Spotify,
            "youtube" => TrackSource::Youtube,
            "youtube_music" => TrackSource::YoutubeMusic,
            "soundcloud" => TrackSource::SoundCloud,
            _ => TrackSource::Unknown,
        }
    }

    pub fn from_string(source: String) -> Self {
        TrackSource::from_str(&source)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr)]
pub enum TrackProvider {
    Youtube,
    YoutubeMusic,
    SoundCloud,
    Unknown,
}

impl TrackProvider {
    pub fn from_str(provider: &str) -> Self {
        match provider.to_lowercase().as_str() {
            "youtube" => TrackProvider::Youtube,
            "youtube_music" => TrackProvider::YoutubeMusic,
            "soundcloud" => TrackProvider::SoundCloud,
            _ => TrackProvider::Unknown,
        }
    }

    pub fn from_string(provider: String) -> Self {
        TrackProvider::from_str(&provider)
    }
}

// ================================================================================================
// Structs
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<i32>,

    // Audio metadata
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub date: Option<String>,
    pub genre: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub label: Option<String>,

    // Utils
    pub file_path: Option<PathBuf>,
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimplifiedTrack {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
}


// ================================================================================================
// Implementations
// ================================================================================================

struct Weights;
impl Weights {
    const TITLE: f64 = 1.0;
    const ARTISTS: f64 = 0.7;
    const ALBUM: f64 = 0.4;
    const DURATION: f64 = 1.0;
    const RELEASE_DATE: f64 = 0.3;
}

impl Track {
    pub fn get_primary_artist(&self) -> Artist {
        self.album
            .as_ref()
            .and_then(|a| a.artists.first())
            .cloned()
            .unwrap_or_else(|| self.artists.first().unwrap().clone())
    }

    pub fn get_source(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Source)
            .cloned()
    }

    pub fn get_source_platform(&self) -> Platform {
        self.get_source()
            .map(|s| s.platform)
            .unwrap_or(Platform::Unknown)
    }

    pub fn get_provider(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
    }

    pub fn get_year(&self) -> Option<String> {
        self.date.as_ref().and_then(|d| d.split('-').next().map(|s| s.to_string()))
    }

    /// Display a track in a user-friendly format
    pub fn display(&self) -> String {
        let artists = self
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        let date = self.date.clone().unwrap_or_else(|| "Unknown".to_string());
        format!("{} by {} ({})", self.title, artists, date)
    }

    /// Returns a normalized similarity score (between 0 and 1) of the match between two tracks
    pub fn compare(&self, other_track: &Track) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Title comparison
        score += Weights::TITLE
            * string_similarity(&self.title, &other_track.title, SimilarityAlgorithm::Smart);
        total_weight += Weights::TITLE;

        // Artists comparison (sort artists to handle different order)
        let self_artists = self
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect::<Vec<String>>();
        let other_artists = other_track
            .artists
            .iter()
            .map(|artist| artist.name.clone())
            .collect::<Vec<String>>();
        score += Weights::ARTISTS
            * string_similarity(
                &self_artists.join("; "),
                &other_artists.join("; "),
                SimilarityAlgorithm::Smart,
            );
        total_weight += Weights::ARTISTS;

        // Album comparison (if both tracks have an album)
        if let (Some(album1), Some(album2)) = (&self.album, &other_track.album) {
            score += Weights::ALBUM
                * string_similarity(&album1.title, &album2.title, SimilarityAlgorithm::Smart);
            total_weight += Weights::ALBUM;
        }

        // Duration comparison (within tolerance ranges)
        if let (Some(duration1), Some(duration2)) = (&self.duration, &other_track.duration) {
            let diff = (duration1 - duration2 / 1000).abs();
            let duration_score = if diff <= 2000 {
                // <= 2 seconds tolerance
                Weights::DURATION
            } else if diff <= 5000 {
                // <= 5 seconds tolerance
                Weights::DURATION / 2.0
            } else {
                0.0
            };
            score += duration_score;
            total_weight += if duration_score > 0.0 {
                Weights::DURATION
            } else {
                0.0
            };
        }

        // Release date comparison (exact match)
        if let (Some(date1), Some(date2)) = (&self.date, &other_track.date) {
            if date1 == date2 {
                score += Weights::RELEASE_DATE;
                total_weight += Weights::RELEASE_DATE;
            }
        }

        // Return normalized score (0.0 if no valid comparison)
        if total_weight == 0.0 {
            0.0
        } else {
            score / total_weight
        }
    }

    /// Transpose metadata from a source track to a destination track
    pub fn transpose_metadata(&mut self, other: &Track) {
        self.title = other.title.clone();
        if let Some(val) = &other.album {
            match &mut self.album {
                Some(album) => album.transpose_metadata(val),
                None => self.album = Some(val.clone()),
            }
        }
        self.artists = other.artists.clone();
        if let Some(val) = &other.date { self.date = Some(val.clone()); };
        if let Some(val) = &other.genre { self.genre = Some(val.clone()); };
        if let Some(val) = &other.cover { self.cover = Some(val.clone()); };
        if let Some(val) = &other.duration { self.duration = Some(val.clone()); };
        if let Some(val) = &other.track_number { self.track_number = Some(val.clone()); };
        if let Some(val) = &other.disc_number { self.disc_number = Some(val.clone()); };
        if let Some(val) = &other.label { self.label = Some(val.clone()); };
        for ref_item in &other.references {
            if !self.references.iter().any(|r| r.platform == ref_item.platform && r.external_id == ref_item.external_id && r.ref_type == ref_item.ref_type) {
                self.references.push(ref_item.clone());
            }
        }
    }
}
