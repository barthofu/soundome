use musicbrainz_rs::entity::recording::RecordingSearchQuery;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::entity::{artist_credit::ArtistCredit, recording::Recording, release::Release};
use shared::utils::enums::Match;
use shared::utils::string::{string_similarity, SimilarityAlgorithm};
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

    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        // Build the query based on the track information
        let mut query_builder = RecordingSearchQuery::query_builder();
        query_builder
            .recording(&track.title)
            .and().artist(&track.artists[0].name);

        if let Some(album) = &track.album {
            query_builder.and().release(&album.title);
        }

        // Execute the query
        let query_result = Recording::search(query_builder.build()).execute().await;

        // Process the results to find the best match
        query_result
            .map_or(Match::None, |query_result| {
                query_result.entities.iter()
                    .filter_map(|recording| {
                        let comparing_track = convert_to_track(recording);
                        let match_score = track.compare(&comparing_track);
                        // Only consider tracks with a valid match score
                        if match_score > 0.0 {
                            Some((match_score, comparing_track))
                        } else {
                            None
                        }
                    })
                    .max_by(|(score_a, _), (score_b, _)| score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal))
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


    async fn get_match_from_query(&self, query: &str) -> Match<Track> {
        let normalized_query = query.replace("- ", "");
        let tracks = self.get_matches_from_query(query).await;

        let best_match = tracks.iter()
            .map(|track| {
                let match_score = string_similarity(&normalized_query, &format!("{} {}", track.artists[0].name, track.title), SimilarityAlgorithm::SorensenDice);
                (match_score, track)
            })
            .max_by(|(score_a, _), (score_b, _)| score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal));

        best_match.map_or(Match::None, |(best_score, best_track)| {
            if best_score > Self::EXACT_MATCH_THRESHOLD {
                Match::Exact(best_track.clone())
            } else if best_score > Self::PARTIAL_MATCH_THRESHOLD {
                Match::Partial(best_track.clone())
            } else {
                Match::None
            }
        })
    }

    async fn get_matches_from_query(&self, query: &str) -> Vec<Track> {
        // Execute the query
        let query_result = Recording::search(format!("query={}", query)).execute().await;

        // Process the results to find the best match
        query_result.map_or(Vec::new(), |query_result| {
            query_result.entities.iter()
                // Map each recording to a track
                .map(|recording| convert_to_track(recording))
                .collect()
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
        date: release.date.as_ref().map(|date| format_date(&date, Format::DATE)),
        url: None,
        // cover: release.get_coverart().res_1200().execute().await.unwrap().
        cover: None
    }
}

fn convert_to_track(recording: &Recording) -> Track {
    let release = recording.releases.as_ref().and_then(|releases| releases.first().cloned());

    let album = release.as_ref().map(convert_to_album);
    let artists = recording
        .artist_credit
        .as_ref()
        .map(|credits| credits.iter().map(convert_to_artist).collect())
        .unwrap_or_default();

    let track_number: i32 = release
        .as_ref()
        .and_then(|r| r.media.as_ref()?.first()?.position.map(|pos| pos as i32))
        .unwrap_or(1);

    Track {
        title: recording.title.clone(),
        artists,
        album,
        genre: None,
        date: recording.first_release_date.as_ref().map(|date| format_date(&date, Format::DATE)),
        track_number: Some(track_number),
        disc_number: None,
        url: None,
        cover: None,
        duration: recording.length.map(|length| length as i32),
        label: None,
    }
}
