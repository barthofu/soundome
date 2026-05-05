pub mod mappers;

use ai::AIBackend;
use async_trait::async_trait;
use config::Config;
use fancy_regex::Regex;
use futures::future::join_all;
use mappers::convert_track;
use rsoundcloud::{
    ClientError, CollectionParams, PlaylistsApi, ResourceId, SearchApi, SoundCloudClient,
    TracksApi, UsersApi,
};
use shared::{
    errors::Error, http::HttpClientBuilder, models::{Album, Artist, Platform, Playlist, PlaylistTrack, SimplifiedTrack, Track}, types::SoundomeResult
};

use crate::Source;

pub struct Soundcloud {
    client: SoundCloudClient
}

impl Soundcloud {
    const TRACK_REGEX: &str = r"^(https:\/\/soundcloud\.com\/(?:(?!sets|stats|groups|upload|you|mobile|stream|messages|discover|notifications|terms-of-use|people|pages|jobs|settings|logout|charts|imprint|popular)(?:[a-z0-9\-_]{1,25}))\/(?:(?:(?!sets|playlist|stats|settings|logout|notifications|you|messages)(?:[a-z0-9\-_]{1,100}))(?:\/s\-[a-zA-Z0-9\-_]{1,10})?))(?:[a-z0-9\-\?=\/]*)$";
    const PLAYLIST_REGEX: &str = r"^https:\/\/soundcloud\.com\/(?:(?!sets|stats|groups|upload|you|mobile|stream|messages|discover|notifications|terms-of-use|people|pages|jobs|settings|logout|charts|imprint|popular)[a-z0-9\-_]{1,25})\/sets\/[a-z0-9\-_]{1,100}(?:[a-z0-9\-\?=\/]*)$";
    const ARTIST_REGEX: &str = r"^https:\/\/soundcloud\.com\/(?:(?!sets|stats|groups|upload|you|mobile|stream|messages|discover|notifications|terms-of-use|people|pages|jobs|settings|logout|charts|imprint|popular)[a-z0-9\-_]{1,25})\/?(?:\?.*)?$";

    pub async fn new() -> SoundomeResult<Self> {
        let client = match Config::get().proxy.as_ref() {
            Some(proxy_config) if proxy_config.enabled => {
                let reqwest_client = HttpClientBuilder::get_reqwest_client()?;
                let http_client = rsoundcloud::http::HttpClient::new(reqwest_client);
                SoundCloudClient::with_http_client(http_client, None, None).await
            }
            _ => SoundCloudClient::default().await,
        }.map_err(|e| match e {
            ClientError::ClientIDGenerationFailed => {
                Error::Internal("Failed to generate Soundcloud client id".to_string())
            }
            _ => Error::Internal("Failed to create Soundcloud client".to_string()),
        })?;

        Ok(Self {
            client,
        })
    }

    // =================
    // Utils
    // =================

    async fn get_complete_track_from_music_track(
        &self,
        track: rsoundcloud::models::track::Track,
    ) -> Track {
        let album = self
            .client
            .get_track_albums(ResourceId::Id(track.track.id))
            .await
            .ok()
            .and_then(|albums| albums.into_iter().next());
        convert_track(track, album)
    }

    pub async fn clean_tracks_title_and_artist_name(&self, tracks: &mut Vec<&mut Track>) -> SoundomeResult<()> {
        let prompt = ai::prompts::clean_track_title_and_artist_name(false)?;
        let ai_client = ai::AIClient::new().map_err(|e| {
            Error::Internal(format!("Failed to initialize AI client: {}", e))
        })?;

        // Process in chunks to avoid token limit issues
        let chunk_size = 100;
        let mut i = 0;

        while i < tracks.len() {
            let end = usize::min(i + chunk_size, tracks.len());
            let chunk = &mut tracks[i..end];

            let simplified_tracks: Vec<SimplifiedTrack> = chunk.iter().map(|track| SimplifiedTrack {
                id: track.get_source().and_then(|track_ref| track_ref.external_id).unwrap_or_default(),
                title: track.title.clone(),
                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
            }).collect();

            // Send to AI for processing
            tracing::info!("Sending {} tracks to AI for processing", simplified_tracks.len());
            let processed_tracks = ai_client
                .generate_with_data(&prompt, simplified_tracks)
                .await
                .map_err(|e| {
                    Error::Internal(format!("AI processing failed: {}", e))
                })?;

            tracing::info!("Processed tracks: {:#?}", processed_tracks);

            for (i, processed_track) in processed_tracks.iter().enumerate() {
                chunk[i].title = processed_track.title.clone();
                chunk[i].artists = processed_track.artists.iter().enumerate().map(|(j, name)| Artist {
                    id: None,
                    name: name.clone(),
                    icon: chunk[i].artists.get(j).and_then(|artist| artist.icon.clone()),
                    references: chunk[i].artists.get(j).map(|artist| artist.references.clone()).unwrap_or_default(),
                }).collect();
            }

            i += chunk_size;
        }

        Ok(())
    }

}

#[async_trait]
impl Source for Soundcloud {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        tracing::info!("Getting SoundCloud track from URL: {}", url);
        let track = self
            .client
            .get_track(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud track from {}", url).to_string()))?;

        Ok(self.get_complete_track_from_music_track(track).await)
    }

    async fn get_tracks_from_query(&self, query: &str) -> Result<Vec<Track>, Error> {
        let tracks = self
            .client
            .search_tracks(query.to_string(), CollectionParams::new(Some(10), None))
            .await
            .map_err(mappers::convert_error)?;

        Ok(
            join_all(
                tracks
                    .iter()
                    .map(|track| self.get_complete_track_from_music_track(track.clone())),
            )
        .await
        )
    }

    async fn get_playlist_from_url(&self, url: &str) -> SoundomeResult<Playlist> {
        let playlist = self
            .client
            .get_playlist(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("SoundCloud playlist from {}", url).to_string()))?;

        let cover = playlist.album_playlist.artwork_url.clone();
        Ok(Playlist {
            id: None,
            name: playlist.album_playlist.title.clone(),
            source: Platform::SoundCloud,
            source_url: Some(url.to_string()),
            cover,
        })
    }

    async fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error> {
        let tracks = self
            .client
            .get_playlist_tracks(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("SoundCloud playlist tracks from {}", url)))?;

        Ok(
            join_all(tracks.into_iter().map(|track| self.get_complete_track_from_music_track(track)))
                .await
                .into_iter()
                .enumerate()
                .map(|(i, track)| PlaylistTrack {
                    id: None,
                    track,
                    added_at: None,
                    position: Some(i as u32),
                })
                .collect()
        )
    }

    async fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error> {
        let artist = self
            .client
            .get_user(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud artist from {}", url).to_string()))?;
        Ok(mappers::convert_artist(&artist))
    }

    async fn get_artist_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error> {
        let tracks = self
            .client
            .get_user_tracks(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud artist tracks from {}", url)))?;

        Ok(tracks
            .into_iter()
            .map(|basic_track| mappers::convert_basic_track(basic_track, None))
            .collect())
    }

    async fn get_artists_from_query(&self, search: &str) -> Result<Vec<Artist>, Error> {
        let users = self
            .client
            .search_users(search.to_string(), CollectionParams::default())
            .await
            .map_err(mappers::convert_error)?;

        Ok(users
            .iter()
            .map(|user| mappers::convert_artist(user))
            .collect())
    }

    async fn get_album_from_url(&self, url: &str) -> Result<Album, Error> {
        let album = self
            .client
            .get_playlist(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud album from {}", url).to_string()))?;
        Ok(mappers::convert_basic_album(&album))
    }

    async fn get_albums_from_query(&self, search: &str) -> Result<Vec<Album>, Error> {
        let albums = self
            .client
            .search_albums(search.to_string(), CollectionParams::default())
            .await
            .map_err(mappers::convert_error)?;

        Ok(albums
            .iter()
            .map(|album| mappers::convert_album(album))
            .collect())
    }

    async fn get_album_tracks_from_url(&self, url: &str) -> Result<Vec<Track>, Error> {
        // SoundCloud albums are technically playlists, reuse playlist track fetching
        let tracks = self
            .client
            .get_playlist_tracks(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("SoundCloud album tracks from {}", url)))?;

        Ok(
            join_all(tracks.into_iter().map(|track| self.get_complete_track_from_music_track(track)))
                .await
        )
    }

    async fn clean_track_metadata(&self, track: &mut Track) -> SoundomeResult<()> {
        let mut tracks = vec![track];
        self.clean_tracks_metadata(&mut tracks).await
    }

    async fn clean_tracks_metadata(&self, tracks: &mut Vec<&mut Track>) -> SoundomeResult<()> {
        self.clean_tracks_title_and_artist_name(tracks).await
    }

    fn is_valid_track_url(url: &str) -> bool {
        let re = Regex::new(Self::TRACK_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        let re = Regex::new(Self::PLAYLIST_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_artist_url(url: &str) -> bool {
        // Artist URL must not match track or playlist patterns
        if Self::is_valid_track_url(url) || Self::is_valid_playlist_url(url) {
            return false;
        }
        let re = Regex::new(Self::ARTIST_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_album_url(_url: &str) -> bool {
        // SoundCloud albums use the same /sets/ URL pattern as playlists,
        // so album URLs are handled through the playlist path.
        false
    }
}
