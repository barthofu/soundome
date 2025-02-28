pub mod spotify;

use config::model::AppConfig;
use shared::models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack};
use shared::errors::Error;
use spotify::Spotify;

// This is the trait that all fetchers must implement
pub trait Source {

    fn get_track_from_url(&self, url: &str) -> Result<Track, Error>;
    fn get_tracks_from_query(&self, search: &str) -> Result<Vec<Track>, Error>;
    fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error>;
    fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error>;
    fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error>;
    fn get_album_from_url(&self, url: &str) -> Result<Album, Error>;
    fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error>;
    fn get_album_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error>;

    fn is_valid_track_url(url: &str) -> bool;
    fn is_valid_playlist_url(url: &str) -> bool;
}

// ==============================
// Exposed functions
// ==============================

pub fn get_track_from_url(url: &str, config: &AppConfig) -> Result<Track, Error> {
    match url {
        _ if Spotify::is_valid_track_url(url) => {
            let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)?;
            spotify.get_track_from_url(url)
        }
        // _ if Youtube::is_valid_track_url(url) => {
        //     let youtube = youtube::Youtube::new();
        //     youtube.get_track_from_url(url)
        // }
        _ => Err(Error::InvalidUrl),
    }
}

pub fn get_playlist_tracks_from_url(url: &str, config: &AppConfig) -> Result<Vec<PlaylistTrack>, Error> {
    match url {
        _ if Spotify::is_valid_playlist_url(url) => {
            let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)?;
            spotify.get_playlist_tracks_from_url(url)
        }
        // _ if Youtube::is_valid_playlist_url(url) => {
        //     let youtube = youtube::Youtube::new();
        //     youtube.get_playlist_tracks_from_url(url)
        // }
        _ => Err(Error::InvalidUrl),
    }
}
