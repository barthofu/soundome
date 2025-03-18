pub mod spotify;
pub mod youtube_music;
pub mod soundcloud;

use config::model::AppConfig;
use shared::models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack};
use shared::errors::Error;
use soundcloud::Soundcloud;
use spotify::Spotify;
use youtube_music::YoutubeMusic;
use async_trait::async_trait;

#[async_trait]
pub trait Source {

    async fn get_track_from_url(&self, url: &str) -> Result<Track, Error>;
    async fn get_tracks_from_query(&self, search: &str) -> Result<Vec<Track>, Error>;
    async fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error>;
    async fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error>;
    async fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error>;
    async fn get_album_from_url(&self, url: &str) -> Result<Album, Error>;
    async fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error>;
    async fn get_album_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error>;

    fn is_valid_track_url(url: &str) -> bool;
    fn is_valid_playlist_url(url: &str) -> bool;
}

// ==============================
// Exposed functions
// ==============================

pub async fn get_track_from_url(url: &str, config: &AppConfig) -> Result<Track, Error> {
    match url {
        _ if Spotify::is_valid_track_url(url) => {
            let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)?;
            spotify.get_track_from_url(url).await
        }
        _ if YoutubeMusic::is_valid_track_url(url) => {
            let youtube_music = YoutubeMusic::new();
            youtube_music.get_track_from_url(url).await
        }
        _ if Soundcloud::is_valid_track_url(url) => {
            let soundcloud = Soundcloud::new().await?;
            soundcloud.get_track_from_url(url).await
        }
        // _ if Youtube::is_valid_track_url(url) => {
        //     let youtube = youtube::Youtube::new();
        //     youtube.get_track_from_url(url)
        // }
        _ => Err(Error::InvalidUrl(format!("{} is not compatible with any 'source' available", url))),
    }
}

pub async fn get_playlist_tracks_from_url(url: &str, config: &AppConfig) -> Result<Vec<PlaylistTrack>, Error> {
    match url {
        _ if Spotify::is_valid_playlist_url(url) => {
            let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)?;
            spotify.get_playlist_tracks_from_url(url).await
        }
        _ if YoutubeMusic::is_valid_playlist_url(url) => {
            let youtube_music = YoutubeMusic::new();
            youtube_music.get_playlist_tracks_from_url(url).await
        }
        _ if Soundcloud::is_valid_playlist_url(url) => {
            let soundcloud = Soundcloud::new().await?;
            soundcloud.get_playlist_tracks_from_url(url).await
        }
        // _ if Youtube::is_valid_playlist_url(url) => {
        //     let youtube = youtube::Youtube::new();
        //     youtube.get_playlist_tracks_from_url(url)
        // }
        _ => Err(Error::InvalidUrl(format!("{} is not compatible with any 'source' available", url))),
    }
}
