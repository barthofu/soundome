use musicbrainz_rs::entity::recording::RecordingSearchQuery;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::{artist_credit::ArtistCredit, recording::Recording, release::Release};
use shared::utils::enums::Match;
use shared::{models::{album::Album, artist::Artist, track::Track}, utils::date::{format_date, Format}};

use crate::TagProvider;

pub struct MusicBrainz;

impl MusicBrainz {
    const EXACT_MATCH_THRESHOLD: f64 = 0.8;
    const PARTIAL_MATCH_THRESHOLD: f64 = 0.5;

    pub fn new() -> Self {
        Self
    }
}

impl TagProvider for MusicBrainz {

    async fn get(&self, track: &Track) -> Match<Track> {
        // Build the query based on the track information
        let mut query_builder = RecordingSearchQuery::query_builder();
        query_builder
            .recording(track.title.clone().as_str())
            .and().artist(track.artists[0].name.clone().as_str());
        track.album.clone().map(|album| query_builder
            .and().release(album.title.clone().as_str())
        );

        // Execute the query
        let query_result = Recording::search(query_builder.build()).execute().await;

        // Process the results to find the best match
        query_result.map_or(Match::None, |query_result| {
            query_result.entities.iter()
                // Map each recording to a tuple of the match score and the track
                .map(|recording| {
                    let comparing_track = convert_to_track(recording);
                    let match_score = track.compare(&comparing_track);
                    (match_score, comparing_track)
                })
                // Find the track with the highest match score
                .max_by(|(score_a, _), (score_b, _)| score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal))
                // Determine the best match type based on the score threshold
                .map_or(Match::None, |(best_score, best_track)| {
                    if best_score > Self::EXACT_MATCH_THRESHOLD {
                        Match::Exact(best_track)
                    } else if best_score > Self::PARTIAL_MATCH_THRESHOLD {
                        Match::Partial(best_track)
                    } else {
                        Match::None
                    }
                })
        })
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

fn convert_to_artist(artist: &ArtistCredit) -> Artist {
    Artist {
        name: artist.name.clone(),
        url: None,
        icon: None,
    }
}

fn convert_to_album(release: &Release) -> Album {
    Album {
        title: release.title.clone(),
        artists: release.artist_credit.clone()
            .map(|a| a.iter().map(|artist| convert_to_artist(artist)).collect())
            .unwrap_or_default(),
        date: release.date.clone().map(|date| format_date(&date, Format::DATE)),
        url: None,
        // cover: release.get_coverart().res_1200().execute().await.unwrap().
        cover: None
    }
}

fn convert_to_track(recording: &Recording) -> Track {
    let album = recording.releases.clone().map(|release| release.first().map(|first_release| convert_to_album(first_release)).or_else(|| None).unwrap()).or_else(|| None);
    let artists = recording.artist_credit.clone().map(|a| a.iter().map(|artist| convert_to_artist(artist)).collect()).or_else(|| Some(Vec::new())).unwrap();

    Track {
        title: recording.title.clone(),
        artists,
        album,
        genre: None,
        date: recording.first_release_date.clone().map(|date| format_date(&date, Format::DATE)),
        track_number: None,
        disc_number: None,
        url: None,
        cover: None,
        duration: recording.length.map(|length| length as i32),
        label: None,
    }
}
