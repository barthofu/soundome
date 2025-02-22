use crate::utils::string::string_similarity;

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

struct Weights {}

impl Weights {
    const TITLE: f64 = 1.0;
    const ARTISTS: f64 = 0.7;
    const ALBUM: f64 = 0.4;
    const DURATION: f64 = 1.0;
    const RELEASE_DATE: f64 = 0.3;
}

impl Track {

    /**
     * Returns a normalized similarity score (between 0 and 1) of the match between two tracks
     */
    pub fn compare(&self, track1: &Track, track2: &Track) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // title
        score += Weights::TITLE * string_similarity(&track1.title, &track2.title);
        total_weight += Weights::TITLE;

        // artists
        let mut track1_artists: Vec<String> = track1.artists.iter().map(|artist| artist.name.clone()).collect();
        track1_artists.sort();
        let mut track2_artists: Vec<String> = track2.artists.iter().map(|artist| artist.name.clone()).collect();
        track2_artists.sort();
        score += Weights::ARTISTS * string_similarity(track1_artists.join("; ").as_str(), track2_artists.join("; ").as_str());
        total_weight += Weights::ARTISTS;

        // album
        if let (Some(album1), Some(album2)) = (&track1.album, &track2.album) {
            score += Weights::ALBUM * string_similarity(&album1.title, &album2.title);
            total_weight += Weights::ALBUM;
        }

        // duration
        if let (Some(duration1), Some(duration2)) = (&track1.duration, &track2.duration) {
            let diff = (duration1 - duration2 / 1000).abs();
            if diff <= 2 {
                score += Weights::DURATION;
                total_weight += Weights::DURATION;
            } else if diff <= 5 {
                score += Weights::DURATION / 2.0;
                total_weight += Weights::DURATION / 2.0;
            }
        }

        // release date
        if let (Some(date1), Some(date2)) = (&track1.date, &track2.date) {
            if date1 == date2 {
                score += Weights::RELEASE_DATE;
                total_weight += Weights::RELEASE_DATE;
            }
        }

        if total_weight == 0.0 {
            return 0.0;
        }

        score / total_weight
    }
}
