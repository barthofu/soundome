pub mod youtube;
// pub mod youtube_music;

use std::path::PathBuf;
use async_trait::async_trait;
use config::model::AppConfig;
use shared::models::track::{Track, TrackProvider, TrackSource};
use shared::errors::Error;

// this is the trait that all downloaders must implement
#[async_trait]
pub trait Provider {
    async fn search(&self, track: &Track) -> Result<String, Error>;
    async fn download(&mut self, url: &str, base_dir: PathBuf) -> Result<PathBuf, Error>;
    fn is_valid_url(url: &str) -> bool;
}

pub trait Matcher {
    fn match_results(&self, search_results: Vec<Track>, source_track: Track) -> Option<String>;
}

// ==============================
// Exposed functions
// ==============================

pub async fn search(track: &Track, config: &AppConfig) -> Result<(TrackProvider, String), Error> {
    // providers
    let youtube = youtube::Youtube::new(config.youtube.as_ref().map(|youtube| youtube.invidious_instance.clone()).flatten());

    let source = track.source.as_ref().ok_or(Error::Custom("track source not defined".to_string()))?;

    match source {
        TrackSource::Spotify => youtube.search(track).await.map(|url| (TrackProvider::Youtube, url)),
        TrackSource::Youtube => Ok((TrackProvider::Youtube, track.source_url.clone().unwrap())),
        _ => Err(Error::Unknown),
    }
}

pub async fn download(url: &str, source: &TrackSource, config: &AppConfig) -> Result<PathBuf, Error> {
    // providers
    let mut youtube = youtube::Youtube::new(config.youtube.as_ref().map(|youtube| youtube.invidious_instance.clone()).flatten());

    match source {
        TrackSource::Spotify => youtube.download(&url, PathBuf::from(&config.general.base_dir)).await,
        TrackSource::Youtube => youtube.download(&url, PathBuf::from(&config.general.base_dir)).await,
        _ => Err(Error::Unknown),
    }
}
