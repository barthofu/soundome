use std::path::PathBuf;

use strum::AsRefStr;

use crate::{utils::string::{string_similarity, SimilarityAlgorithm}, models::{album::Album, artist::Artist}};

#[derive(Debug, Clone, AsRefStr)]
pub enum TrackSource {
    Local,
    Spotify,
    Youtube,
    Unknown
}

#[derive(Debug, Clone, AsRefStr)]
pub enum TrackProvider {
    Youtube,
    Unknown
}

#[derive(Debug, Clone)]
pub struct Track {
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
    pub source: Option<TrackSource>,
    pub source_url: Option<String>,
    pub provider: Option<TrackProvider>,
    pub provider_url: Option<String>,
}

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

    /**
     * Display a track in a user-friendly format
     */
    pub fn display(&self) -> String {
        let artists = self.artists.iter().map(|artist| artist.name.clone()).collect::<Vec<String>>().join(", ");
        let date = self.date.clone().unwrap_or_else(|| "Unknown".to_string());
        format!("{} by {} ({})", self.title, artists, date)
    }

    /**
     * Returns a normalized similarity score (between 0 and 1) of the match between two tracks
     */
    pub fn compare(&self, other_track: &Track) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Title comparison
        score += Weights::TITLE * string_similarity(&self.title, &other_track.title, SimilarityAlgorithm::Smart);
        total_weight += Weights::TITLE;

        // Artists comparison (sort artists to handle different order)
        let self_artists = self.artists.iter().map(|artist| artist.name.clone()).collect::<Vec<String>>();
        let other_artists = other_track.artists.iter().map(|artist| artist.name.clone()).collect::<Vec<String>>();
        score += Weights::ARTISTS * string_similarity(
            &self_artists.join("; "),
            &other_artists.join("; "),
            SimilarityAlgorithm::Smart
        );
        total_weight += Weights::ARTISTS;

        // Album comparison (if both tracks have an album)
        if let (Some(album1), Some(album2)) = (&self.album, &other_track.album) {
            score += Weights::ALBUM * string_similarity(&album1.title, &album2.title, SimilarityAlgorithm::Smart);
            total_weight += Weights::ALBUM;
        }

        // Duration comparison (within tolerance ranges)
        if let (Some(duration1), Some(duration2)) = (&self.duration, &other_track.duration) {
            let diff = (duration1 - duration2 / 1000).abs();
            let duration_score = if diff <= 2000 {  // <= 2 seconds tolerance
                Weights::DURATION
            } else if diff <= 5000 {  // <= 5 seconds tolerance
                Weights::DURATION / 2.0
            } else {
                0.0
            };
            score += duration_score;
            total_weight += if duration_score > 0.0 { Weights::DURATION } else { 0.0 };
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

    /**
     * Transpose metadata from a source track to a destination track
     */
    pub fn transpose_metadata(&mut self, source_track: &Track) {
        self.title = source_track.title.clone();
        self.album = source_track.album.clone();
        self.artists = source_track.artists.clone();
        self.date = source_track.date.clone();
        self.genre = source_track.genre.clone();
        self.cover = source_track.cover.clone();
        self.duration = source_track.duration.clone();
        self.track_number = source_track.track_number.clone();
        self.disc_number = source_track.disc_number.clone();
        self.label = source_track.label.clone();
    }

}
