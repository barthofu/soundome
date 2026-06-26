pub mod soundcloud;
mod utils;
pub mod youtube;
pub mod youtube_music;

use async_trait::async_trait;
use config::Config;
use shared::{
    errors::Error,
    models::{Platform, Reference, ReferenceType, Track},
    types::SoundomeResult,
};
use std::path::PathBuf;

// this is the trait that all downloaders must implement
#[async_trait]
pub trait Provider {
    /// Search the best matching download url for the given track
    async fn search(&self, track: &Track) -> SoundomeResult<Reference>;

    /// Download the track from the given url at the given base directory
    async fn download(
        &mut self,
        url: &str,
        file_name: &str,
        base_library_dir: PathBuf,
    ) -> SoundomeResult<PathBuf>;

    /// Check if the given url is a valid url for the provider
    fn is_valid_url(url: &str) -> bool;
}

pub trait Matcher {
    /// Match the search results with the source track
    fn match_results(&self, search_results: Vec<Track>, source_track: Track) -> Option<Track>;
}

// ==============================
// Exposed functions
// ==============================

pub async fn search(track: &Track) -> SoundomeResult<Reference> {
    // providers
    let youtube = youtube::Youtube::new(
        Config::get()
            .providers
            .youtube
            .as_ref()
            .and_then(|youtube| youtube.invidious_instance.clone()),
    );
    let youtube_music = youtube_music::YoutubeMusic::new();

    let source = track.get_source();
    let source = source
        .as_ref()
        .ok_or(Error::Custom("track source not defined".to_string()))?;

    match source.platform {
        Platform::Spotify => {
            // we first try to search on youtube music
            match youtube_music.search(track).await {
                Ok(reference) => Ok(reference),
                Err(_) => {
                    // if it fails, we fallback to youtube
                    youtube.search(track).await
                }
            }
        }
        Platform::Youtube => Ok(Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Provider,
            platform: Platform::Youtube,
            external_id: source.external_id.clone(),
            external_url: source.external_url.clone(),
        }),
        Platform::YoutubeMusic => Ok(Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Provider,
            platform: Platform::YoutubeMusic,
            external_id: source.external_id.clone(),
            external_url: source.external_url.clone(),
        }),
        Platform::SoundCloud => Ok(Reference {
            id: None,
            ref_type: shared::models::ReferenceType::Provider,
            platform: Platform::SoundCloud,
            external_id: source.external_id.clone(),
            external_url: source.external_url.clone(),
        }),
        _ => Err(Error::Unknown),
    }
}

pub async fn download(
    source: &Reference,
    provider: &Reference,
    track_title: &str,
    output_dir: PathBuf,
) -> SoundomeResult<PathBuf> {
    if source.ref_type != ReferenceType::Source {
        return Err(Error::Custom(
            "source reference type must be Source".to_string(),
        ));
    }

    if provider.ref_type != ReferenceType::Provider {
        return Err(Error::Custom(
            "provider reference type must be Provider".to_string(),
        ));
    }

    let url = provider
        .external_url
        .clone()
        .ok_or(Error::Custom("track source url not defined".to_string()))?;

    match source.platform {
        Platform::Spotify => {
            let mut youtube = youtube::Youtube::new(
                Config::get()
                    .providers
                    .youtube
                    .as_ref()
                    .and_then(|youtube| youtube.invidious_instance.clone()),
            );

            youtube.download(&url, track_title, output_dir).await
        }
        Platform::Youtube => {
            let mut youtube = youtube::Youtube::new(
                Config::get()
                    .providers
                    .youtube
                    .as_ref()
                    .and_then(|youtube| youtube.invidious_instance.clone()),
            );
            youtube.download(&url, track_title, output_dir).await
        }
        Platform::YoutubeMusic => {
            let mut youtube_music = youtube_music::YoutubeMusic::new();
            youtube_music.download(&url, track_title, output_dir).await
        }
        Platform::SoundCloud => {
            // DRM fallback: if the provider resolved to YouTube/YTMusic, use that downloader.
            match provider.platform {
                Platform::Youtube => {
                    let mut youtube = youtube::Youtube::new(
                        Config::get()
                            .providers
                            .youtube
                            .as_ref()
                            .and_then(|youtube| youtube.invidious_instance.clone()),
                    );
                    youtube.download(&url, track_title, output_dir).await
                }
                Platform::YoutubeMusic => {
                    let mut youtube_music = youtube_music::YoutubeMusic::new();
                    youtube_music.download(&url, track_title, output_dir).await
                }
                _ => {
                    let mut soundcloud = soundcloud::SoundCloud::new().await?;
                    soundcloud.download(&url, track_title, output_dir).await
                }
            }
        }
        _ => Err(Error::Unknown),
    }
}

/// Search YouTube Music and YouTube for all candidates matching the track.
/// Returns raw results without similarity filtering so the user can pick manually.
/// Results from YouTube Music are listed first.
pub async fn search_youtube_candidates(track: &Track) -> SoundomeResult<Vec<Track>> {
    let youtube = youtube::Youtube::new(
        Config::get()
            .providers
            .youtube
            .as_ref()
            .and_then(|y| y.invidious_instance.clone()),
    );
    let youtube_music = youtube_music::YoutubeMusic::new();

    let mut all: Vec<Track> = Vec::new();

    match youtube_music.search_all(track).await {
        Ok(results) => all.extend(results),
        Err(e) => tracing::warn!("YouTube Music candidate search failed: {}", e),
    }

    match youtube.search_all(track).await {
        Ok(results) => all.extend(results),
        Err(e) => tracing::warn!("YouTube candidate search failed: {}", e),
    }

    Ok(all)
}
