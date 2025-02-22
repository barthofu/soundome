pub mod spotify;

use shared::models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack};
use shared::errors::Error;

// This is the trait that all fetchers must implement
pub trait Fetcher {

    fn get_track_from_url(&self, url: &str) -> Result<Track, Error>;
    fn get_tracks_from_query(&self, search: &str) -> Result<Vec<Track>, Error>;
    fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error>;
    fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error>;
    fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error>;
    fn get_album_from_url(&self, url: &str) -> Result<Album, Error>;
    fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error>;
    fn get_album_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error>;
}
