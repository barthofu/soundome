pub mod mappers;

use rspotify::{Credentials, ClientCredsSpotify, model::{SearchType, SearchResult, TrackId, PlaylistId, ArtistId, AlbumId}, prelude::BaseClient};

use crate::{models::{track::Track, artist::Artist, album::Album, playlist::PlaylistTrack}, utils::{errors::Error, source::Source}};

pub struct Spotify {

    client: ClientCredsSpotify,    
}

impl Spotify {
    
    pub fn new() -> Result<Self, Error> {

        let credentials = Credentials::from_env().ok_or(Error::Config)?;
        let client = ClientCredsSpotify::new(credentials);

        match client.request_token() {
            Ok(it) => it,
            Err(_) => return Err(Error::Config),
        };

        Ok(Self {
            client
        })
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

impl Source for Spotify {

    fn get_track_from_url(&self, url: &str) -> Result<Track, Error> {

        match TrackId::from_id(&self.url_to_id(url)) {
            Ok(id) => {
                
                let track = self.client.track(id);
                match track {
                    Ok(track) => Ok(track.into()),
                    Err(_) => Err(Error::NotFound),
                }
            }
            Err(_) => Err(Error::BadURL),
        }
    }

    fn get_tracks_from_query(&self, query: &str) -> Result<Vec<Track>, Error> {

        let res = self.client.search(query, SearchType::Track, None, None, Some(20), Some(0));
    
        match res {
            Ok(res) => match res {
                SearchResult::Tracks(tracks) => {
                    Ok(tracks.items.into_iter().map(|track| track.into()).collect())
                }
                _ => Ok(vec![]),
            },
            Err(e) => Err(Error::from(e)),
        }
    }

    fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error> {

        match PlaylistId::from_id(&self.url_to_id(url)) {
            Ok(id) => {
                
                let playlist = self.client.playlist(id, None, None);
                match playlist {
                    Ok(playlist) => {

                        let items = playlist.tracks;
                        let tracks: Vec<PlaylistTrack> = items.items
                            .into_iter()
                            .map(|item| PlaylistTrack::from(item))
                            .collect();
                        
                        Ok(tracks)

                        // Ok(tracks.items.into_iter().map(|item| item.track.unwrap().into()).collect())
                    },
                    Err(_) => Err(Error::NotFound),
                }
            }
            Err(_) => Err(Error::BadURL),
        }
    }

    fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error> {
        
        match ArtistId::from_id(&self.url_to_id(url)) {
            Ok(id) => {
                
                match self.client.artist(id) {
                    Ok(artist) => Ok(artist.into()),
                    Err(_) => Err(Error::NotFound),
                }
            }
            Err(_) => Err(Error::BadURL),
        }
    }

    fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error> {
        
        let res = self.client.search(search, SearchType::Artist, None, None, Some(20), Some(0));
    
        match res {
            Ok(res) => match res {
                SearchResult::Artists(artists) => {
                    Ok(artists.items.into_iter().map(|artist| artist.into()).collect())
                }
                _ => Ok(vec![]),
            },
            Err(e) => Err(Error::from(e)),
        }
    }

    fn get_album_from_url(&self, url: &str) -> Result<Album, Error> {
        
        match AlbumId::from_id(&self.url_to_id(url)) {
            Ok(id) => {
                
                match self.client.album(id) {
                    Ok(album) => Ok(album.into()),
                    Err(_) => Err(Error::NotFound),
                }
            }
            Err(_) => Err(Error::BadURL),
        }
    }

    fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error> {
        
        let res = self.client.search(search, SearchType::Album, None, None, Some(20), Some(0));
    
        match res {
            Ok(res) => match res {
                SearchResult::Albums(albums) => {
                    Ok(albums.items.into_iter().map(|album| album.into()).collect())
                }
                _ => Ok(vec![]),
            },
            Err(e) => Err(Error::from(e)),
        }
    }

    fn get_album_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error> {
        todo!()
    }
}