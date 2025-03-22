pub mod mappers;

use ai::AIBackend;
use async_trait::async_trait;
use config::model::AiConfig;
use fancy_regex::Regex;
use futures::future::join_all;
use mappers::convert_track;
use rsoundcloud::{
    ClientError, CollectionParams, PlaylistsApi, ResourceId, SearchApi, SoundCloudClient,
    TracksApi, UsersApi,
};
use shared::{
    errors::Error,
    models::{album::Album, artist::Artist, playlist::PlaylistTrack, track::{SimplifiedTrack, Track}}, types::SoundomeResult,
};

use crate::Source;

pub struct Soundcloud {
    client: SoundCloudClient,
    ai_config: AiConfig,
}

impl Soundcloud {
    const TRACK_REGEX: &str = r"^(https:\/\/soundcloud\.com\/(?:(?!sets|stats|groups|upload|you|mobile|stream|messages|discover|notifications|terms-of-use|people|pages|jobs|settings|logout|charts|imprint|popular)(?:[a-z0-9\-_]{1,25}))\/(?:(?:(?!sets|playlist|stats|settings|logout|notifications|you|messages)(?:[a-z0-9\-_]{1,100}))(?:\/s\-[a-zA-Z0-9\-_]{1,10})?))(?:[a-z0-9\-\?=\/]*)$";
    const PLAYLIST_REGEX: &str = r"^https:\/\/soundcloud\.com\/(?:(?!sets|stats|groups|upload|you|mobile|stream|messages|discover|notifications|terms-of-use|people|pages|jobs|settings|logout|charts|imprint|popular)[a-z0-9\-_]{1,25})\/sets\/[a-z0-9\-_]{1,100}(?:[a-z0-9\-\?=\/]*)$";

    pub async fn new(ai_config: AiConfig) -> Result<Self, Error> {
        let client = SoundCloudClient::default().await.map_err(|e| match e {
            ClientError::ClientIDGenerationFailed => {
                Error::Internal("Failed to generate Soundcloud client id".to_string())
            }
            _ => Error::Internal("Failed to create Soundcloud client".to_string()),
        })?;

        Ok(Self {
            client,
            ai_config,
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

    async fn clean_tracks_title_and_artist_name(&self, tracks: &mut Vec<&mut Track>) -> SoundomeResult<()> {
        let prompt = ai::prompts::clean_track_title_and_artist_name(false)?;
        let ai_client = ai::AIClient::new(&self.ai_config).map_err(|e| {
            Error::Internal(format!("Failed to initialize AI client: {}", e))
        })?;

        let processed_tracks = ai_client
            .generate_with_data(&prompt, tracks.iter().map(|track| SimplifiedTrack {
                id: track.source_id.clone().unwrap_or_default(),
                title: track.title.clone(),
                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
            }).collect::<Vec<SimplifiedTrack>>())
            .await
            .map_err(|e| {
                Error::Internal(format!("AI processing failed: {}", e))
            })?;

        println!("Processed tracks: {:#?}", processed_tracks);

        // Update the original tracks
        for (i, processed_track) in processed_tracks.iter().enumerate() {
            tracks[i].title = processed_track.title.clone();
            tracks[i].artists = processed_track.artists.iter().enumerate().map(|(j, name)| Artist {
                name: name.clone(),
                // TODO: only working for the original
                url: tracks[i].artists.get(j).and_then(|artist| artist.url.clone()),
                icon: tracks[i].artists.get(j).and_then(|artist| artist.icon.clone()),
            }).collect();
        }

        Ok(())
    }

}

#[async_trait]
impl Source for Soundcloud {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        println!("Getting SoundCloud track from URL: {}", url);
        let track = self
            .client
            .get_track(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud track from {}", url).to_string()))?;

        let mut track = self.get_complete_track_from_music_track(track).await;
        let _ = self.clean_tracks_title_and_artist_name(&mut vec![&mut track]).await
            .map_err(|e| {
                println!("Failed to clean SoundCloud track title and artist name: {}", e);
            });
        Ok(track)
    }

    async fn get_tracks_from_query(&self, query: &str) -> Result<Vec<Track>, Error> {
        let tracks = self
            .client
            .search_tracks(query.to_string(), CollectionParams::new(Some(10), None))
            .await
            .map_err(mappers::convert_error)?;

        let mut tracks = join_all(
            tracks
                .iter()
                .map(|track| self.get_complete_track_from_music_track(track.clone())),
        )
        .await;

        let mut tracks_refs: Vec<&mut Track> = tracks.iter_mut().collect();
        let _ = self.clean_tracks_title_and_artist_name(&mut tracks_refs).await
            .map_err(|e| {
                println!("Failed to clean SoundCloud tracks title and artist name: {}", e);
            });

        Ok(tracks)
    }

    async fn get_playlist_tracks_from_url(&self, url: &str) -> Result<Vec<PlaylistTrack>, Error> {
        let tracks = self
            .client
            .get_playlist_tracks(ResourceId::Url(url.to_string()))
            .await
            .map_err(mappers::convert_error)?;

        let mut tracks = join_all(
            tracks
                .iter()
                .map(|track| self.get_complete_track_from_music_track(track.clone())),
        )
        .await;

        let mut tracks_refs: Vec<&mut Track> = tracks.iter_mut().collect();
        let _ = self.clean_tracks_title_and_artist_name(&mut tracks_refs).await
            .map_err(|e| {
                println!("Failed to clean SoundCloud track title and artist name: {}", e);
            });

        Ok(tracks
            .iter()
            .enumerate()
            .map(|(i, track)| PlaylistTrack {
                track: track.clone(),
                added_at: None,
                position: Some(i as u32),
            })
            .collect())
    }

    async fn get_artist_from_url(&self, url: &str) -> Result<Artist, Error> {
        let artist = self
            .client
            .get_user(ResourceId::Url(url.to_string()))
            .await
            .map_err(|_| Error::NotFound(format!("Soundcloud artist from {}", url).to_string()))?;
        Ok(mappers::convert_artist(&artist))
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

    async fn get_album_tracks_from_url(&self, _: &str) -> Result<Vec<Track>, Error> {
        todo!()
    }

    fn is_valid_track_url(url: &str) -> bool {
        let re = Regex::new(Self::TRACK_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        let re = Regex::new(Self::PLAYLIST_REGEX).unwrap(); // safe unwrap
        re.is_match(url).unwrap_or(false)
    }
}
