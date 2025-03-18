pub mod youtube;
pub mod youtube_music;
pub mod soundcloud;
mod utils;

use std::path::PathBuf;
use async_trait::async_trait;
use config::model::AppConfig;
use shared::{errors::Error, models::track::{Track, TrackProvider, TrackSource}};

// this is the trait that all downloaders must implement
#[async_trait]
pub trait Provider {

    /// Search the best matching download url for the given track
    async fn search(&self, track: &Track) -> Result<String, Error>;

    /// Download the track from the given url at the given base directory
    async fn download(&mut self, url: &str, base_dir: PathBuf) -> Result<PathBuf, Error>;

    /// Check if the given url is a valid url for the provider
    fn is_valid_url(url: &str) -> bool;
}

pub trait Matcher {

    /// Match the search results with the source track
    fn match_results(&self, search_results: Vec<Track>, source_track: Track) -> Option<String>;
}

// ==============================
// Exposed functions
// ==============================

pub async fn search(track: &Track, config: &AppConfig) -> Result<(TrackProvider, String), Error> {
    // providers
    let youtube = youtube::Youtube::new(config.youtube.as_ref().map(|youtube| youtube.invidious_instance.clone()).flatten());
    let youtube_music = youtube_music::YoutubeMusic::new();

    let source = track.source.as_ref().ok_or(Error::Custom("track source not defined".to_string()))?;

    match source {
        TrackSource::Spotify => {
            // We first try to search on youtube music
            let youtube_music_url = youtube_music.search(track).await;
            if let Ok(url) = youtube_music_url {
                return Ok((TrackProvider::YoutubeMusic, url));
            } else {
                // If it fails we fallback to youtube
                let youtube_url = youtube.search(track).await?;
                return Ok((TrackProvider::Youtube, youtube_url));
            }
        },
        TrackSource::Youtube => Ok((TrackProvider::Youtube, track.source_url.clone().unwrap())),
        TrackSource::YoutubeMusic => Ok((TrackProvider::YoutubeMusic, track.source_url.clone().unwrap())),
        TrackSource::SoundCloud => Ok((TrackProvider::SoundCloud, track.source_url.clone().unwrap())),
        _ => Err(Error::Unknown),
    }
}

pub async fn download(url: &str, source: &TrackSource, config: &AppConfig) -> Result<PathBuf, Error> {
    match source {
        TrackSource::Spotify => {
            let mut youtube = youtube::Youtube::new(config.youtube.as_ref().map(|youtube| youtube.invidious_instance.clone()).flatten());
            youtube.download(&url, PathBuf::from(&config.general.base_dir)).await
        },
        TrackSource::Youtube => {
            let mut youtube = youtube::Youtube::new(config.youtube.as_ref().map(|youtube| youtube.invidious_instance.clone()).flatten());
            youtube.download(&url, PathBuf::from(&config.general.base_dir)).await
        },
        TrackSource::YoutubeMusic => {
            let mut youtube_music = youtube_music::YoutubeMusic::new();
            youtube_music.download(&url, PathBuf::from(&config.general.base_dir)).await
        },
        TrackSource::SoundCloud => {
            let mut soundcloud = soundcloud::SoundCloud::new().await?;
            soundcloud.download(&url, PathBuf::from(&config.general.base_dir)).await
        },
        _ => Err(Error::Unknown),
    }
}
