mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;
use config::Config;
use rustypipe::{
    client::RustyPipe,
    model::{MusicSearchResult, TrackItem},
};
use shared::{
    errors::Error,
    models::{Album, Artist, Reference, Track},
    types::SoundomeResult,
};

use crate::{utils::ytdlp::download_with_ytdlp, Matcher, Provider};

pub struct YoutubeMusic {
    client: RustyPipe,
    similarity_treshold: f64,
}

impl Default for YoutubeMusic {
    fn default() -> Self {
        Self::new()
    }
}

impl YoutubeMusic {
    pub fn new() -> Self {
        // TODO: rustypipe probably uses reqwest internally.
        // For now, we document this limitation.
        if let Some(proxy) = Config::get().proxy.as_ref() {
            if proxy.enabled {
                tracing::warn!("Proxy configuration for YouTube Music downloader is not yet supported by the rustypipe library. Consider setting HTTP_PROXY environment variable.");
            }
        }

        Self {
            client: RustyPipe::new(),
            similarity_treshold: 0.80,
        }
    }

    fn create_search_query(&self, track: Track) -> String {
        let artist = track
            .artists
            .into_iter()
            .map(|a| a.name.clone())
            .collect::<Vec<String>>()
            .join(" ");
        format!("{} {}", artist, track.title)
    }

    async fn get_results(&self, query: &str) -> SoundomeResult<MusicSearchResult<TrackItem>> {
        self.client
            .query()
            .music_search_tracks(query)
            .await
            .map_err(|err| Error::Custom(err.to_string()))
    }

    fn convert_search_item_to_track(&self, search_item: TrackItem) -> Track {
        Track {
            id: None,
            needs_validation: false,
            validation_reason: None,
            title: search_item.name,
            artists: search_item
                .artists
                .iter()
                .map(|artist| Artist {
                    id: None,
                    name: artist.name.clone(),
                    icon: None,
                    references: vec![],
                })
                .collect(),
            album: search_item.album.map(|album| Album {
                id: None,
                title: album.name,
                artists: vec![],
                album_type: shared::models::AlbumType::Unknown,
                cover: None,
                date: None,
                references: vec![],
            }),
            duration: search_item.duration.map(|duration| duration as i32),
            track_number: search_item.track_nr.map(|track_nr| track_nr as i32),
            date: None,
            cover: None,
            disc_number: None,
            file_path: None,
            genre: None,
            label: None,
            references: vec![shared::models::Reference {
                id: None,
                ref_type: shared::models::ReferenceType::Provider,
                platform: shared::models::Platform::YoutubeMusic,
                external_id: Some(search_item.id.to_string()),
                external_url: Some(format!(
                    "https://music.youtube.com/watch?v={}",
                    search_item.id
                )),
            }],
        }
    }
}

#[async_trait]
impl Provider for YoutubeMusic {
    async fn search(&self, track: &Track) -> SoundomeResult<Reference> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());

        // 2. Search on YouTube Music
        let search_results = self.get_results(&search_query).await?;

        // 3. Process each pattern to find the best match
        let best_match = self
            .match_results(
                search_results
                    .items
                    .items
                    .iter()
                    .map(|search_item| self.convert_search_item_to_track(search_item.clone()))
                    .collect(),
                track.clone(),
            )
            .ok_or(Error::NoMatch("youtube music".to_string(), track.display()))?
            .get_provider()
            .ok_or(Error::NoMatch("youtube music".to_string(), track.display()))?;

        Ok(best_match)
    }

    async fn download(
        &mut self,
        url: &str,
        file_name: &str,
        base_library_dir: PathBuf,
    ) -> SoundomeResult<PathBuf> {
        download_with_ytdlp(url, file_name, base_library_dir).await
    }

    fn is_valid_url(url: &str) -> bool {
        url.contains("music.youtube.com/watch?v=")
    }
}
