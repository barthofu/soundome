pub mod soundcloud;
pub mod spotify;
pub mod youtube_music;

use async_trait::async_trait;
use config::Config;
use shared::errors::Error;
use shared::models::{Album, Artist, Platform, PlaylistTrack, Track};
use shared::types::SoundomeResult;
use soundcloud::Soundcloud;
use spotify::Spotify;
use youtube_music::YoutubeMusic;

#[async_trait]
pub trait Source {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track>;
    async fn get_tracks_from_query(&self, search: &str) -> SoundomeResult<Vec<Track>>;
    async fn get_playlist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<PlaylistTrack>>;
    async fn get_artist_from_url(&self, url: &str) -> SoundomeResult<Artist>;
    async fn get_artists_from_query(&self, search: &str) -> SoundomeResult<Vec<Artist>>;
    async fn get_album_from_url(&self, url: &str) -> SoundomeResult<Album>;
    async fn get_albums_from_query(&self, search: &str) -> SoundomeResult<Vec<Album>>;
    async fn get_album_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<Track>>;

    /// Clean metadata of a single track
    async fn clean_track_metadata(&self, track: &mut Track) -> SoundomeResult<()>;
    /// Clean metadata (track title, artists names, etc) of multiple tracks
    async fn clean_tracks_metadata(&self, track: &mut Vec<&mut Track>) -> SoundomeResult<()>;
    
    fn is_valid_track_url(url: &str) -> bool;
    fn is_valid_playlist_url(url: &str) -> bool;
}


pub struct Fetcher {
    spotify: Option<Spotify>,
    youtube_music: Option<YoutubeMusic>,
    soundcloud: Option<Soundcloud>,
}

impl Fetcher {
    pub async fn new() -> Self {
        Self {
            spotify: Spotify::new(
                &Config::get().providers.spotify.client_id,
                &Config::get().providers.spotify.client_secret,
            )
                .map_err(|e| {
                    tracing::error!("Failed to initialize Spotify source: {:?}", e);
                    e
                })
                .ok(),
            youtube_music: YoutubeMusic::new()
                .map_err(|e| {
                    tracing::error!("Failed to initialize YouTube Music source: {:?}", e);
                    e
                })
                .ok(),
            soundcloud: Soundcloud::new().await
                .map_err(|e| {
                    tracing::error!("Failed to initialize SoundCloud source: {:?}", e);
                    e
                })
                .ok(),
        }
    }
}

#[async_trait]
impl Source for Fetcher {

    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        match url {
            _ if Spotify::is_valid_track_url(url) => {
                match &self.spotify {
                    Some(spotify) => spotify.get_track_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
                }
            }
            _ if YoutubeMusic::is_valid_track_url(url) => {
                match &self.youtube_music {
                    Some(youtube_music) => youtube_music.get_track_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::YoutubeMusic.to_string())),
                }
            }
            _ if Soundcloud::is_valid_track_url(url) => {
                match &self.soundcloud {
                    Some(soundcloud) => soundcloud.get_track_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
                }
            }
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_playlist_tracks_from_url(&self, url: &str) -> SoundomeResult<Vec<PlaylistTrack>> {
        match url {
            _ if Spotify::is_valid_playlist_url(url) => {
                match &self.spotify {
                    Some(spotify) => spotify.get_playlist_tracks_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::Spotify.to_string())),
                }
            }
            _ if YoutubeMusic::is_valid_playlist_url(url) => {
                match &self.youtube_music {
                    Some(youtube_music) => youtube_music.get_playlist_tracks_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::YoutubeMusic.to_string())),
                }
            }
            _ if Soundcloud::is_valid_playlist_url(url) => {
                match &self.soundcloud {
                    Some(soundcloud) => soundcloud.get_playlist_tracks_from_url(url).await,
                    None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
                }
            }
            _ => Err(Error::InvalidUrl(format!(
                "{} is not compatible with any 'source' available",
                url
            ))),
        }
    }

    async fn get_tracks_from_query(&self, _search: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented("get_tracks_from_query is not implemented yet".to_string()))
    }

    async fn get_artist_from_url(&self, _url: &str) -> SoundomeResult<Artist> {
        Err(Error::NotImplemented("get_artist_from_url is not implemented yet".to_string()))
    }

    async fn get_artists_from_query(&self, _search: &str) -> SoundomeResult<Vec<Artist>> {
        Err(Error::NotImplemented("get_artists_from_query is not implemented yet".to_string()))
    }

    async fn get_album_from_url(&self, _url: &str) -> SoundomeResult<Album> {
        Err(Error::NotImplemented("get_album_from_url is not implemented yet".to_string()))
    }

    async fn get_albums_from_query(&self, _search: &str) -> SoundomeResult<Vec<Album>> {
        Err(Error::NotImplemented("get_albums_from_query is not implemented yet".to_string()))
    }

    async fn get_album_tracks_from_url(&self, _url: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented("get_album_tracks_from_url is not implemented yet".to_string()))
    }

    async fn clean_track_metadata(&self, track: &mut Track) -> SoundomeResult<()> {
        match track.get_source_platform() {
            Platform::SoundCloud => {
                match &self.soundcloud {
                    Some(soundcloud) => soundcloud.clean_track_metadata(track).await,
                    None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
                }
            }
            _ => Ok(()),
        }
    }

    async fn clean_tracks_metadata(&self, tracks: &mut Vec<&mut Track>) -> SoundomeResult<()> {
        match tracks.first() {
            Some(track) => match track.get_source_platform() {
                Platform::SoundCloud => {
                    match &self.soundcloud {
                        Some(soundcloud) => soundcloud.clean_tracks_metadata(tracks).await,
                        None => Err(Error::ProviderUnavailable(Platform::SoundCloud.to_string())),
                    }
                }
                _ => Ok(())
            },
            None => Ok(()),
        }
    }

    fn is_valid_track_url(url: &str) -> bool {
        Spotify::is_valid_track_url(url)
            || YoutubeMusic::is_valid_track_url(url)
            || Soundcloud::is_valid_track_url(url)
        // || Youtube::is_valid_track_url(url)
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        Spotify::is_valid_playlist_url(url)
            || YoutubeMusic::is_valid_playlist_url(url)
            || Soundcloud::is_valid_playlist_url(url)
        // || Youtube::is_valid_playlist_url(url)
    }
}
