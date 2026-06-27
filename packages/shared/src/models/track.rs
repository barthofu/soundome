use std::{fs::File, path::PathBuf};

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
    #[allow(clippy::should_implement_trait)]
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
    #[allow(clippy::should_implement_trait)]
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

    // Validation (for later manual review in the web UI)
    pub needs_validation: bool,
    pub validation_reason: Option<String>,

    // Bidirectional filesystem anchor — written as a custom tag at finalization
    pub soundome_id: Option<String>,

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
    /// When both tracks have a track number, a mismatch is a strong negative signal.
    /// Two tracks on the same album by the same artist should differ here.
    const TRACK_NUMBER: f64 = 0.8;
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

    pub fn get_sources(&self) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|r| r.ref_type == ReferenceType::Source)
            .cloned()
            .collect()
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

    pub fn get_year(&self) -> Option<String> {
        self.date
            .as_ref()
            .and_then(|d| d.split('-').next().map(|s| s.to_string()))
    }

    pub fn get_bitrate(&self) -> Option<u32> {
        let path = self.file_path.as_ref()?;

        // Use symphonia for audio probing
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::probe::Hint;

        let file = File::open(path).ok()?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Provide file extension hint so symphonia can select the correct reader
        // without scanning through potentially large ID3 tags / metadata.
        let mut hint = Hint::new();
        if let Some(ext) = PathBuf::from(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = match symphonia::default::get_probe().format(
            &hint,
            mss,
            &Default::default(),
            &Default::default(),
        ) {
            Ok(probed) => probed,
            Err(_) => return None,
        };
        let format = probed.format;

        // Find the first audio track with a defined bits_per_coded_sample
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.bits_per_coded_sample.is_some())?;
        track.codec_params.bits_per_coded_sample
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
        // All duration values are in seconds throughout the codebase.
        if let (Some(duration1), Some(duration2)) = (self.duration, other_track.duration) {
            let diff = (duration1 - duration2).abs();
            let duration_score = if diff <= 2 {
                // within 2 s — effectively the same track
                Weights::DURATION
            } else if diff <= 5 {
                // within 5 s — likely the same (intro / outro differences)
                Weights::DURATION * 0.5
            } else {
                // beyond 5 s tolerance — clearly different recordings; add the
                // weight to the denominator but contribute nothing to the score
                // so it acts as a penalty rather than being ignored.
                0.0
            };
            // Always include DURATION in the denominator so that a large diff
            // lowers the overall score rather than being silently skipped.
            score += duration_score;
            total_weight += Weights::DURATION;
        }

        // Release date comparison (exact match)
        if let (Some(date1), Some(date2)) = (&self.date, &other_track.date) {
            if date1 == date2 {
                score += Weights::RELEASE_DATE;
                total_weight += Weights::RELEASE_DATE;
            }
        }

        // Track number: when both sides provide it, a match is a strong positive signal
        // and a mismatch is a strong negative one — this is the key discriminant between
        // two different tracks on the same album by the same artist.
        if let (Some(tn1), Some(tn2)) = (self.track_number, other_track.track_number) {
            total_weight += Weights::TRACK_NUMBER;
            if tn1 == tn2 {
                score += Weights::TRACK_NUMBER;
            }
            // mismatch contributes 0 — effectively penalises the total score
        }

        // Return normalized score (0.0 if no valid comparison)
        if total_weight == 0.0 {
            0.0
        } else {
            score / total_weight
        }
    }

    /// Transpose metadata from an other track
    pub fn transpose_metadata(&mut self, other: &Track) {
        self.title = other.title.clone();

        if let Some(val) = &other.date {
            self.date = Some(val.clone());
        };
        if let Some(val) = &other.genre {
            self.genre = Some(val.clone());
        };
        if let Some(val) = &other.cover {
            self.cover = Some(val.clone());
        };
        if let Some(val) = other.duration {
            self.duration = Some(val);
        };
        if let Some(val) = other.track_number {
            self.track_number = Some(val);
        };
        if let Some(val) = other.disc_number {
            self.disc_number = Some(val);
        };
        if let Some(val) = &other.label {
            self.label = Some(val.clone());
        };

        // only add new references, do not overwrite existing ones
        self.transpose_refs(other);

        // transpose album
        if let Some(val) = &other.album {
            match &mut self.album {
                Some(album) => album.transpose_metadata(val),
                None => self.album = Some(val.clone()),
            }
        }

        // transpose artists
        // Pour chaque artiste du self, si un artiste similaire existe dans other, on le transpose
        const SIMILARITY_THRESHOLD: f64 = 0.8;
        for artist in &mut self.artists {
            if let Some(matching_artist) = other
                .artists
                .iter()
                .find(|a| a.compare(artist) > SIMILARITY_THRESHOLD)
            {
                artist.transpose_metadata(matching_artist);
            }
        }

        // TODO: really needed ? + ça induit un problème de duplication d'artistes
        // Pour chaque artiste du other, si aucun artiste similaire n'existe dans self, on l'ajoute
        // for other_artist in &other.artists {
        //     let exists_in_self = self.artists.iter().any(|a| a.compare(other_artist) > 0.8);
        //     if !exists_in_self {
        //         self.artists.push(other_artist.clone());
        //     }
        // }
    }

    /// Transpose references from another track (add only new references)
    pub fn transpose_refs(&mut self, other: &Track) {
        for ref_item in &other.references {
            let reference_already_exists = self.references.iter().any(|r| {
                r.platform == ref_item.platform
                    && r.external_id == ref_item.external_id
                    && r.ref_type == ref_item.ref_type
            });
            if !reference_already_exists {
                self.references.push(ref_item.clone());
            }
        }

        // Also merge references on nested album
        if let Some(other_album) = &other.album {
            match &mut self.album {
                Some(self_album) => {
                    for ref_item in &other_album.references {
                        let exists = self_album.references.iter().any(|r| {
                            r.platform == ref_item.platform
                                && r.external_id == ref_item.external_id
                                && r.ref_type == ref_item.ref_type
                        });
                        if !exists {
                            self_album.references.push(ref_item.clone());
                        }
                    }
                }
                None => self.album = Some(other_album.clone()),
            }
        }

        // And merge references on artists (match by similarity)
        const SIMILARITY_THRESHOLD: f64 = 0.8;
        for self_artist in &mut self.artists {
            if let Some(other_artist) = other
                .artists
                .iter()
                .find(|a| a.compare(self_artist) > SIMILARITY_THRESHOLD)
            {
                for ref_item in &other_artist.references {
                    let exists = self_artist.references.iter().any(|r| {
                        r.platform == ref_item.platform
                            && r.external_id == ref_item.external_id
                            && r.ref_type == ref_item.ref_type
                    });
                    if !exists {
                        self_artist.references.push(ref_item.clone());
                    }
                }
            }
        }
    }

    /// Transpose references from another track, but only for track and artists (not album).
    /// Used for partial matches where we don't want to introduce potentially incorrect album data.
    pub fn transpose_refs_without_album(&mut self, other: &Track) {
        // Merge track-level references
        for ref_item in &other.references {
            let reference_already_exists = self.references.iter().any(|r| {
                r.platform == ref_item.platform
                    && r.external_id == ref_item.external_id
                    && r.ref_type == ref_item.ref_type
            });
            if !reference_already_exists {
                self.references.push(ref_item.clone());
            }
        }

        // Merge references on artists (match by similarity)
        const SIMILARITY_THRESHOLD: f64 = 0.8;
        for self_artist in &mut self.artists {
            if let Some(other_artist) = other
                .artists
                .iter()
                .find(|a| a.compare(self_artist) > SIMILARITY_THRESHOLD)
            {
                for ref_item in &other_artist.references {
                    let exists = self_artist.references.iter().any(|r| {
                        r.platform == ref_item.platform
                            && r.external_id == ref_item.external_id
                            && r.ref_type == ref_item.ref_type
                    });
                    if !exists {
                        self_artist.references.push(ref_item.clone());
                    }
                }
            }
        }
    }
}
