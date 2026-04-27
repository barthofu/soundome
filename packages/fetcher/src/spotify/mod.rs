pub mod mappers;

use std::env;

use async_trait::async_trait;
use config::Config;
use rspotify::{
    model::{AlbumId, ArtistId, Country, Market, PlaylistId, SearchResult, SearchType, TrackId},
    prelude::BaseClient,
    ClientCredsSpotify, Credentials,
};
use shared::{
    errors::Error, http::ProxyRotator, models::{Album, Artist, PlaylistTrack, Track}, types::SoundomeResult
};
use tracing::error;

use crate::Source;

pub struct Spotify {
    client: ClientCredsSpotify,
}

impl Spotify {
    pub fn new(client_id: &str, client_secret: &str) -> SoundomeResult<Self> {
        let credentials = Credentials::new(client_id, client_secret);
        
        // If proxy is enabled and ALL_PROXY is not set, set it using the proxy rotator
        if let Some(proxy_config) = Config::get().proxy.as_ref() {
            if proxy_config.enabled && env::var("ALL_PROXY").is_err() {
                let proxy_url = ProxyRotator::get().get_next_proxy();
                env::set_var("ALL_PROXY", proxy_url.unwrap_or_default()); 
            }
        }
        
        let client = ClientCredsSpotify::new(credentials);

        client
            .request_token()
            .map_err(|e| Error::Config(e.to_string()))?;

        Ok(Self { client })
    }

    // =================
    // Utils
    // =================

    /// Extracts the id from a spotify url (e.g: https://open.spotify.com/track/xxxxxxx?si=yyyyyyy -> xxxxxxx)
    fn url_to_id(&self, url: &str) -> String {
        let id = url
            .split('/')
            .last()
            // we can safely unwrap here because it won't panic even with an empty string as input
            .unwrap()
            .split('?')
            .next();

        match id {
            Some(id) => id.to_string(),
            None => String::new(),
        }
    }
}

#[async_trait]
impl Source for Spotify {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        let id = TrackId::from_id(self.url_to_id(url))
            .map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let track = self
            .client
            .track(id, Some(Market::Country(Country::France)))
            .map_err(|e| {
                error!("Spotify API error for track {}: {}", url, e);
                Error::NotFound(format!("Spotify track from {}", url).to_string())
            })?;
        Ok(mappers::convert_track(&track))
    }

    async fn get_tracks_from_query(&self, query: &str) -> SoundomeResult<Vec<Track>> {
        let res = self
            .client
            .search(query, SearchType::Track, None, None, Some(20), Some(0))
            .map_err(mappers::convert_error)?;

        if let SearchResult::Tracks(tracks) = res {
            Ok(tracks
                .items
                .into_iter()
                .map(|track| mappers::convert_track(&track))
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_playlist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<PlaylistTrack>> {
        let id = PlaylistId::from_id(self.url_to_id(url))
            .map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let playlist = self
            .client
            .playlist(id, None, None)
            .map_err(|_| Error::NotFound(format!("Spotify playlist from {}", url).to_string()))?;

        Ok(playlist
            .tracks
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, track)| {
                mappers::convert_playlist_item(track, i.try_into().unwrap_or(0))
            })
            .collect())
    }

    async fn get_artist_from_url(&self, url: &str) -> SoundomeResult<Artist> {
        let id = ArtistId::from_id(self.url_to_id(url))
            .map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let full_artist = self
            .client
            .artist(id)
            .map_err(|_| Error::NotFound(format!("Spotify artist from {}", url).to_string()))?;

        Ok(mappers::convert_full_artist(&full_artist))
    }

    async fn get_artists_from_query(&self, search: &str) -> SoundomeResult<Vec<Artist>> {
        let res = self
            .client
            .search(search, SearchType::Artist, None, None, Some(20), Some(0))
            .map_err(mappers::convert_error)?;

        if let SearchResult::Artists(artists) = res {
            Ok(artists
                .items
                .into_iter()
                .map(|artist| mappers::convert_full_artist(&artist))
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_album_from_url(&self, url: &str) -> SoundomeResult<Album> {
        let id = AlbumId::from_id(self.url_to_id(url))
            .map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let full_album = self
            .client
            .album(id, None)
            .map_err(|_| Error::NotFound(format!("Spotify album from {}", url).to_string()))?;

        Ok(mappers::convert_full_album(&full_album))
    }

    async fn get_albums_from_query(&self, search: &str) -> SoundomeResult<Vec<Album>> {
        let res = self
            .client
            .search(search, SearchType::Album, None, None, Some(20), Some(0))
            .map_err(mappers::convert_error)?;

        if let SearchResult::Albums(albums) = res {
            Ok(albums
                .items
                .iter()
                .map(|album| mappers::convert_simplified_album(&album))
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_album_tracks_from_url(&self, _: &str) -> SoundomeResult<Vec<Track>> {
        todo!()
    }

    async fn clean_track_metadata(&self, _track: &mut Track) -> SoundomeResult<()> {
        Ok(())
    }

    async fn clean_tracks_metadata(&self, _tracks: &mut Vec<&mut Track>) -> SoundomeResult<()> {
        Ok(())
    } 

    fn is_valid_track_url(url: &str) -> bool {
        url.contains("open.spotify.com/track/")
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        url.contains("open.spotify.com/playlist/")
    }
}
