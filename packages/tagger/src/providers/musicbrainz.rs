use musicbrainz_rs::entity::recording::RecordingSearchQuery;
use musicbrainz_rs::entity::{artist_credit::ArtistCredit, recording::Recording, release::Release};
use musicbrainz_rs::prelude::*;
use shared::models::{AlbumType, Platform, Reference, ReferenceType};
use shared::utils::enums::Match;
use shared::utils::string::{string_similarity, SimilarityAlgorithm};
use shared::{
    models::{Album, Artist, Track},
    utils::date::{format_date, Format},
};

use crate::TagProvider;

#[derive(Default)]
pub struct MusicBrainz {
    // client: MusicBrainzClient
}

impl MusicBrainz {
    const EXACT_MATCH_THRESHOLD: f64 = 0.8;
    const PARTIAL_MATCH_THRESHOLD: f64 = 0.5;

    pub fn new() -> Self {
        // TODO: will be possible when musicbrainz_rs v0.12.0 is fixed (currently broken when building as dependency in this project)
        // let client = match Config::get().proxy.as_ref() {
        //     Some(proxy_config) if proxy_config.enabled => {
        //         HttpClientBuilder::get_reqwest_client()
        //             .map(|client| MusicBrainzClient::new_with_reqwest_client(client))
        //             .map_err(|_| MusicBrainzClient::new())
        //     }
        //     _ => MusicBrainzClient::new(),
        // };

        Self {
            // client,
        }
    }
}

impl TagProvider for MusicBrainz {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        // Need at least one artist to build a meaningful query
        let first_artist = match track.artists.first() {
            Some(artist) => &artist.name,
            None => return Match::None,
        };

        // Build the query based on the track information
        let mut query_builder = RecordingSearchQuery::query_builder();
        query_builder
            .recording(&track.title)
            .and()
            .artist(first_artist);

        if let Some(album) = &track.album {
            query_builder.and().release(&album.title);
        }

        // Execute the query
        let query_result = Recording::search(query_builder.build()).execute().await;

        // Process the results to find the best match
        query_result.map_or(Match::None, |query_result| {
            query_result
                .entities
                .iter()
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
                .max_by(|(score_a, _), (score_b, _)| {
                    score_a
                        .partial_cmp(score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
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

        let best_match = tracks
            .iter()
            .map(|track| {
                let match_score = string_similarity(
                    &normalized_query,
                    &format!("{} {}", track.artists[0].name, track.title),
                    SimilarityAlgorithm::SorensenDice,
                );
                (match_score, track)
            })
            .max_by(|(score_a, _), (score_b, _)| {
                score_a
                    .partial_cmp(score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

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
        // Parse query to extract artist and title
        // Expected format: "Artist Title" or "Artist - Title"
        let parts: Vec<&str> = if query.contains(" - ") {
            query.splitn(2, " - ").collect()
        } else {
            // Split on first space and treat rest as title
            let mut parts = query.splitn(2, ' ').collect::<Vec<&str>>();
            if parts.len() == 1 {
                parts.push(""); // No artist found
            }
            parts
        };

        let artist = parts[0].trim();
        let title = parts.get(1).map(|s| s.trim()).unwrap_or("");

        // Build structured query for better precision
        let mut query_builder = RecordingSearchQuery::query_builder();

        if !title.is_empty() {
            query_builder.recording(title);
            if !artist.is_empty() {
                query_builder.and().artist(artist);
            }
        } else if !artist.is_empty() {
            // Fallback to artist-only search if no title found
            query_builder.artist(artist);
        } else {
            // Fallback to free-text search as last resort
            let query_result = Recording::search(format!("query={}", query))
                .execute()
                .await;

            return query_result.map_or(Vec::new(), |query_result| {
                query_result.entities.iter().map(convert_to_track).collect()
            });
        }

        // Execute the structured query
        let query_result = Recording::search(query_builder.build()).execute().await;

        // Process the results
        query_result.map_or(Vec::new(), |query_result| {
            query_result.entities.iter().map(convert_to_track).collect()
        })
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

fn convert_to_artist(artist: &ArtistCredit) -> Artist {
    Artist {
        id: None,
        name: artist.artist.name.clone(),
        icon: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::MusicBrainz,
            external_id: Some(artist.artist.id.clone()),
            external_url: Some("https://musicbrainz.org/artist/".to_string() + &artist.artist.id),
        }],
    }
}

fn convert_to_album(release: &Release) -> Album {
    Album {
        id: None,
        title: release.title.clone(),
        artists: release
            .artist_credit
            .clone()
            .map(|a| a.iter().map(convert_to_artist).collect())
            .unwrap_or_default(),
        date: release
            .date
            .as_ref()
            .map(|date| format_date(date, Format::DATE)),
        album_type: AlbumType::Unknown,
        cover: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::MusicBrainz,
            external_id: Some(release.id.clone()),
            external_url: Some("https://musicbrainz.org/release/".to_string() + &release.id),
        }],
    }
}

fn convert_to_track(recording: &Recording) -> Track {
    let release = recording
        .releases
        .as_ref()
        .and_then(|releases| releases.first().cloned());

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
        id: None,
        needs_validation: false,
        validation_reason: None,
        soundome_id: None,
        title: recording.title.clone(),
        artists,
        album,
        genre: None,
        date: recording
            .first_release_date
            .as_ref()
            .map(|date| format_date(date, Format::DATE)),
        track_number: Some(track_number),
        disc_number: None,
        cover: None,
        duration: recording.length.map(|length| length as i32 / 1000),
        label: None,
        file_path: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::MusicBrainz,
            external_id: Some(recording.id.clone()),
            external_url: Some("https://musicbrainz.org/recording/".to_string() + &recording.id),
        }],
    }
}
