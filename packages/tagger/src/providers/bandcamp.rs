use bandcamp::{search, SearchResultItem};
use shared::models::{Album, Artist, Track};
use shared::models::{AlbumType, Platform, Reference, ReferenceType};
use shared::utils::enums::Match;
use shared::utils::string::{string_similarity, SimilarityAlgorithm};

use crate::TagProvider;

#[derive(Default)]
pub struct Bandcamp;

impl Bandcamp {
    const EXACT_MATCH_THRESHOLD: f64 = 0.8;
    const PARTIAL_MATCH_THRESHOLD: f64 = 0.5;

    pub fn new() -> Self {
        // Note: the bandcamp crate uses its own internal reqwest client and does not
        // support proxy configuration from shared::libs::http::HttpClientBuilder.
        Self
    }
}

impl TagProvider for Bandcamp {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        let query = format!(
            "{} {}",
            track.artists.first().map(|a| a.name.as_str()).unwrap_or(""),
            track.title
        );

        let tracks = self.get_matches_from_query(&query).await;

        tracks
            .into_iter()
            .map(|candidate| {
                let score = track.compare(&candidate);
                (score, candidate)
            })
            .filter(|(score, _)| *score > 0.0)
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(Match::None, |(best_score, best_track)| {
                if best_score > Self::EXACT_MATCH_THRESHOLD {
                    Match::Exact(best_track)
                } else if best_score > Self::PARTIAL_MATCH_THRESHOLD {
                    Match::Partial(best_track)
                } else {
                    Match::None
                }
            })
    }

    async fn get_match_from_query(&self, query: &str) -> Match<Track> {
        let normalized_query = query.replace("- ", "");
        let tracks = self.get_matches_from_query(query).await;

        tracks
            .iter()
            .map(|track| {
                let match_score = string_similarity(
                    &normalized_query,
                    &format!("{} {}", track.artists[0].name, track.title),
                    SimilarityAlgorithm::SorensenDice,
                );
                (match_score, track)
            })
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(Match::None, |(best_score, best_track)| {
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
        match search(query).await {
            Ok(results) => results
                .into_iter()
                .filter_map(|item| match item {
                    SearchResultItem::Track(t) => Some(search_result_to_track(t)),
                    _ => None,
                })
                .collect(),
            Err(err) => {
                tracing::warn!("Bandcamp search failed for query {:?}: {}", query, err);
                Vec::new()
            }
        }
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

fn search_result_to_track(item: bandcamp::SearchResultItemTrack) -> Track {
    let album = item.album_name.as_ref().map(|album_title| Album {
        id: None,
        title: album_title.clone(),
        artists: vec![Artist {
            id: None,
            name: item.band_name.clone(),
            icon: None,
            references: vec![],
        }],
        date: None,
        album_type: AlbumType::Unknown,
        cover: None,
        references: item
            .album_id
            .map(|album_id| {
                vec![Reference {
                    id: None,
                    ref_type: ReferenceType::Metadata,
                    platform: Platform::Bandcamp,
                    external_id: Some(album_id.to_string()),
                    external_url: Some(item.url.artist_url.clone()),
                }]
            })
            .unwrap_or_default(),
    });

    Track {
        id: None,
        needs_validation: false,
        validation_reason: None,
        title: item.name,
        artists: vec![Artist {
            id: None,
            name: item.band_name,
            icon: None,
            references: vec![Reference {
                id: None,
                ref_type: ReferenceType::Metadata,
                platform: Platform::Bandcamp,
                external_id: Some(item.band_id.to_string()),
                external_url: Some(item.url.artist_url.clone()),
            }],
        }],
        album,
        date: None,
        genre: None,
        cover: None,
        duration: None,
        track_number: None,
        disc_number: None,
        label: None,
        file_path: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::Bandcamp,
            external_id: Some(item.track_id.to_string()),
            external_url: Some(item.url.item_url),
        }],
    }
}
