use config::Config;
use shared::{models::Track, utils::enums::Match};

use crate::{providers, TagProvider};

/// A scored candidate returned by metadata providers.
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    pub track: Track,
    pub score: f64,
    pub provider: String,
}

/// A single metadata provider variant, dispatched dynamically based on config.
enum MetadataProvider {
    MusicBrainz(providers::musicbrainz::MusicBrainz),
    Bandcamp(providers::bandcamp::Bandcamp),
    Spotify(providers::spotify::Spotify),
}

impl MetadataProvider {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        match self {
            Self::MusicBrainz(p) => p.get_best_match_from_track(track).await,
            Self::Bandcamp(p) => p.get_best_match_from_track(track).await,
            Self::Spotify(p) => p.get_best_match_from_track(track).await,
        }
    }

    async fn get_matches_from_query(&self, query: &str) -> Vec<Track> {
        match self {
            Self::MusicBrainz(p) => p.get_matches_from_query(query).await,
            Self::Bandcamp(p) => p.get_matches_from_query(query).await,
            Self::Spotify(p) => p.get_matches_from_query(query).await,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::MusicBrainz(_) => "musicbrainz",
            Self::Bandcamp(_) => "bandcamp",
            Self::Spotify(_) => "spotify",
        }
    }
}

/// Instantiate all metadata providers that are enabled in config, in config order.
fn build_providers() -> Vec<MetadataProvider> {
    Config::get()
        .tagger
        .metadata_providers
        .iter()
        .filter_map(|name| match name.as_str() {
            "musicbrainz" => Some(MetadataProvider::MusicBrainz(
                providers::musicbrainz::MusicBrainz::new(),
            )),
            "spotify" => providers::spotify::Spotify::new().map(MetadataProvider::Spotify),
            "bandcamp" => Some(MetadataProvider::Bandcamp(
                providers::bandcamp::Bandcamp::new(),
            )),
            other => {
                tracing::warn!(
                    "Unknown tagger metadata provider in config: {:?}, skipping",
                    other
                );
                None
            }
        })
        .collect()
}

/// Query all enabled metadata providers in priority order and return the first
/// `Exact` match found, falling back to the best `Partial` match across all providers.
pub async fn get_best_match_from_track(track: &Track) -> Match<Track> {
    let providers = build_providers();

    if providers.is_empty() {
        tracing::warn!("No tagger metadata providers enabled in config");
        return Match::None;
    }

    let mut best_partial: Option<Track> = None;

    for provider in &providers {
        match provider.get_best_match_from_track(track).await {
            Match::Exact(t) => {
                return Match::Exact(t);
            }
            Match::Partial(t) => {
                if best_partial.is_none() {
                    best_partial = Some(t);
                }
            }
            Match::None => {}
        }
    }

    match best_partial {
        Some(t) => Match::Partial(t),
        None => Match::None,
    }
}

/// Query all enabled metadata providers and return all candidates with their scores.
/// Used by the validation UI to let the user pick the correct match.
pub async fn get_candidates_for_track(track: &Track) -> Vec<MatchCandidate> {
    let providers = build_providers();

    if providers.is_empty() {
        tracing::warn!("No tagger metadata providers enabled in config");
        return Vec::new();
    }

    let query = format!(
        "{} {}",
        track.artists.first().map(|a| a.name.as_str()).unwrap_or(""),
        track.title,
    );

    let mut candidates: Vec<MatchCandidate> = Vec::new();

    for provider in &providers {
        let provider_name = provider.name();
        let results = provider.get_matches_from_query(&query).await;

        for candidate in results {
            let score = track.compare(&candidate);
            if score > 0.0 {
                candidates.push(MatchCandidate {
                    track: candidate,
                    score,
                    provider: provider_name.to_string(),
                });
            }
        }
    }

    // Sort by score descending
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates
}
