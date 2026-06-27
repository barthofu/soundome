use std::env;

use config::Config;
use rspotify::{
    model::{FullArtist, FullTrack, SearchResult, SearchType, SimplifiedAlbum, SimplifiedArtist},
    prelude::BaseClient,
    ClientCredsSpotify, Credentials,
};
use shared::http::ProxyRotator;
use shared::models::{Album, AlbumType, Artist, Platform, Reference, ReferenceType, Track};
use shared::utils::enums::Match;
use shared::utils::string::{string_similarity, SimilarityAlgorithm};

use crate::TagProvider;

pub struct Spotify {
    client: ClientCredsSpotify,
}

impl Spotify {
    const EXACT_MATCH_THRESHOLD: f64 = 0.8;
    const PARTIAL_MATCH_THRESHOLD: f64 = 0.5;

    pub fn new() -> Option<Self> {
        let config = Config::get();
        let spotify_cfg = match config.providers.spotify.as_ref() {
            Some(c) => c,
            None => {
                tracing::debug!("Spotify metadata provider: no credentials in config, skipping");
                return None;
            }
        };
        let client_id = &spotify_cfg.client_id;
        let client_secret = &spotify_cfg.client_secret;

        if client_id.is_empty() || client_secret.is_empty() {
            tracing::warn!("Spotify metadata provider: missing credentials, skipping");
            return None;
        }

        let credentials = Credentials::new(client_id, client_secret);

        // If proxy is enabled and ALL_PROXY is not set, set it using the proxy rotator
        if let Some(proxy_config) = config.proxy.as_ref() {
            if proxy_config.enabled && env::var("ALL_PROXY").is_err() {
                if let Some(proxy_url) = ProxyRotator::get().get_next_proxy() {
                    env::set_var("ALL_PROXY", proxy_url);
                }
            }
        }

        let client = ClientCredsSpotify::new(credentials);

        if let Err(e) = client.request_token() {
            tracing::error!("Spotify metadata provider: failed to request token: {}", e);
            return None;
        }

        Some(Self { client })
    }

    fn search_tracks(&self, query: &str) -> Vec<Track> {
        match self
            .client
            .search(query, SearchType::Track, None, None, Some(10), Some(0))
        {
            Ok(SearchResult::Tracks(page)) => {
                page.items.into_iter().map(|t| convert_track(&t)).collect()
            }
            Ok(_) => Vec::new(),
            Err(e) => {
                tracing::warn!("Spotify search failed for query {:?}: {}", query, e);
                Vec::new()
            }
        }
    }

    /// Search Spotify for an artist by name and return the first image URL found.
    ///
    /// Uses the artist search endpoint (not the deprecated `/artists/{id}` batch
    /// endpoint) so it works with current Spotify API limits.  Best-effort: returns
    /// `None` on any error or when no image is available.
    fn fetch_artist_icon(&self, artist_name: &str) -> Option<String> {
        let result = self.client.search(
            artist_name,
            SearchType::Artist,
            None,
            None,
            Some(1),
            Some(0),
        );

        match result {
            Ok(SearchResult::Artists(page)) => page
                .items
                .into_iter()
                .next()
                .and_then(|a: FullArtist| a.images.into_iter().next().map(|img| img.url)),
            Ok(_) => None,
            Err(e) => {
                tracing::warn!(
                    "Spotify artist icon search failed for {:?}: {}",
                    artist_name,
                    e
                );
                None
            }
        }
    }

    /// Enrich `icon` fields on a track's artists by searching Spotify for each
    /// artist that currently has no icon.  One search request per artist without
    /// an icon — best-effort, never fails the enrichment.
    fn enrich_artist_icons(&self, track: &mut Track) {
        for artist in &mut track.artists {
            if artist.icon.is_some() {
                continue;
            }
            if let Some(url) = self.fetch_artist_icon(&artist.name) {
                artist.icon = Some(url);
            }
        }
    }
}

impl TagProvider for Spotify {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        let query = format!(
            "{} {}",
            track.artists.first().map(|a| a.name.as_str()).unwrap_or(""),
            track.title
        );

        let candidates = self.search_tracks(&query);

        let result = candidates
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
            });

        // Enrich artist icons on the matched track.
        match result {
            Match::Exact(mut t) => {
                self.enrich_artist_icons(&mut t);
                Match::Exact(t)
            }
            Match::Partial(mut t) => {
                self.enrich_artist_icons(&mut t);
                Match::Partial(t)
            }
            Match::None => Match::None,
        }
    }

    async fn get_match_from_query(&self, query: &str) -> Match<Track> {
        let normalized_query = query.replace("- ", "");
        let tracks = self.search_tracks(query);

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
        self.search_tracks(query)
    }
}

// ================================================================================================
// Mappers
// ================================================================================================

fn convert_artist(artist: &SimplifiedArtist) -> Artist {
    Artist {
        id: None,
        name: artist.name.clone(),
        icon: None,
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::Spotify,
            external_id: artist.id.as_ref().map(|id| id.to_string()),
            external_url: artist.external_urls.get("spotify").cloned(),
        }],
    }
}

fn convert_simplified_album(album: &SimplifiedAlbum) -> Album {
    Album {
        id: None,
        title: album.name.clone(),
        artists: album.artists.iter().map(convert_artist).collect(),
        album_type: album
            .album_type
            .as_ref()
            .map(|t| match t.as_str() {
                "album" => AlbumType::Album,
                "single" => AlbumType::Single,
                "compilation" => AlbumType::Compilation,
                _ => AlbumType::Unknown,
            })
            .unwrap_or(AlbumType::Unknown),
        cover: album.images.first().map(|image| image.url.clone()),
        date: album.release_date.clone(),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::Spotify,
            external_id: album.id.as_ref().map(|id| id.to_string()),
            external_url: album.external_urls.get("spotify").cloned(),
        }],
    }
}

fn convert_track(track: &FullTrack) -> Track {
    Track {
        id: None,
        needs_validation: false,
        validation_reason: None,
        soundome_id: None,
        title: track.name.clone(),
        artists: track.artists.iter().map(convert_artist).collect(),
        album: Some(convert_simplified_album(&track.album)),
        genre: None,
        duration: Some(track.duration.num_seconds() as i32),
        file_path: None,
        track_number: Some(track.track_number as i32),
        disc_number: Some(track.disc_number),
        label: None,
        date: track.album.release_date.clone(),
        cover: track.album.images.first().map(|image| image.url.clone()),
        references: vec![Reference {
            id: None,
            ref_type: ReferenceType::Metadata,
            platform: Platform::Spotify,
            external_id: track.id.as_ref().map(|id| id.to_string()),
            external_url: track.external_urls.get("spotify").cloned(),
        }],
    }
}
