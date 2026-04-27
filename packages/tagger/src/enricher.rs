use config::Config;
use shared::{models::Track, utils::enums::Match};

use crate::{TagProvider, providers};

/// A single metadata provider variant, dispatched dynamically based on config.
enum MetadataProvider {
    MusicBrainz(providers::musicbrainz::MusicBrainz),
    Bandcamp(providers::bandcamp::Bandcamp),
}

impl MetadataProvider {
    async fn get_best_match_from_track(&self, track: &Track) -> Match<Track> {
        match self {
            Self::MusicBrainz(p) => p.get_best_match_from_track(track).await,
            Self::Bandcamp(p) => p.get_best_match_from_track(track).await,
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
            "musicbrainz" => Some(MetadataProvider::MusicBrainz(providers::musicbrainz::MusicBrainz::new())),
            "bandcamp" => Some(MetadataProvider::Bandcamp(providers::bandcamp::Bandcamp::new())),
            other => {
                tracing::warn!("Unknown tagger metadata provider in config: {:?}, skipping", other);
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
