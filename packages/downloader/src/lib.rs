pub mod soundcloud;
mod utils;
pub mod youtube;
pub mod youtube_music;

use async_trait::async_trait;
use config::model::AppConfig;
use shared::{
    errors::Error,
    models::{Platform, Reference, Track}, types::SoundomeResult,
};
use std::path::PathBuf;

// this is the trait that all downloaders must implement
#[async_trait]
pub trait Provider {
    /// Search the best matching download url for the given track
    async fn search(&self, track: &Track) -> SoundomeResult<Reference>;

    /// Download the track from the given url at the given base directory
    async fn download(&mut self, url: &str, file_name: &str, base_dir: PathBuf) -> SoundomeResult<PathBuf>;

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

pub async fn search(track: &Track, config: &AppConfig) -> SoundomeResult<Reference> {
    // providers
    let youtube = youtube::Youtube::new(
        config
            .providers
            .youtube
            .as_ref()
            .map(|youtube| youtube.invidious_instance.clone())
            .flatten(),
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
        Platform::SoundCloud => Ok(
            Reference {
                id: None,
                ref_type: shared::models::ReferenceType::Provider,
                platform: Platform::SoundCloud,
                external_id: source.external_id.clone(),
                external_url: source.external_url.clone(),
            },
        ),
        _ => Err(Error::Unknown),
    }
}

pub async fn download(
    track: &Track,
    config: &AppConfig,
) -> SoundomeResult<PathBuf> {
    let source = track.get_source();
    let source = source
        .as_ref()
        .ok_or(Error::Custom("track source not defined".to_string()))?;
    let url = source.external_url.clone().ok_or(Error::Custom(
        "track source url not defined".to_string(),
    ))?;
    let track_title = track.title.clone();

    match source.platform {
        Platform::Spotify => {
            let mut youtube = youtube::Youtube::new(
                config
                    .providers
                    .youtube
                    .as_ref()
                    .map(|youtube| youtube.invidious_instance.clone())
                    .flatten(),
            );
            youtube
                .download(&url, &track_title, PathBuf::from(&config.general.base_dir))
                .await
        }
        Platform::Youtube => {
            let mut youtube = youtube::Youtube::new(
                config
                    .providers
                    .youtube
                    .as_ref()
                    .map(|youtube| youtube.invidious_instance.clone())
                    .flatten(),
            );
            youtube
                .download(&url, &track_title, PathBuf::from(&config.general.base_dir))
                .await
        }
        Platform::YoutubeMusic => {
            let mut youtube_music = youtube_music::YoutubeMusic::new();
            youtube_music
                .download(&url, &track_title, PathBuf::from(&config.general.base_dir))
                .await
        }
        Platform::SoundCloud => {
            let mut soundcloud = soundcloud::SoundCloud::new(config.ai.clone()).await?;
            soundcloud
                .download(&url, &track_title, PathBuf::from(&config.general.base_dir))
                .await
        }
        _ => Err(Error::Unknown),
    }
}
