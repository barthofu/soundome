mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;
use rustypipe::{
    client::RustyPipe,
    model::{MusicSearchResult, TrackItem},
};
use shared::{
    errors::Error,
    models::{
        album::Album,
        artist::Artist,
        track::{Track, TrackProvider},
    },
};

use crate::{utils::ytdlp::download_with_ytdlp, Matcher, Provider};

pub struct YoutubeMusic {
    client: RustyPipe,
    similarity_treshold: f64,
}

impl YoutubeMusic {
    pub fn new() -> Self {
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

    async fn get_results(&self, query: &str) -> Result<MusicSearchResult<TrackItem>, Error> {
        self.client
            .query()
            .music_search_tracks(query)
            .await
            .map_err(|err| Error::Custom(err.to_string()))
    }

    fn convert_search_item_to_track(&self, search_item: TrackItem) -> Track {
        Track {
            title: search_item.name,
            artists: search_item
                .artists
                .iter()
                .map(|artist| Artist {
                    name: artist.name.clone(),
                    url: None,
                    icon: None,
                })
                .collect(),
            album: search_item.album.map(|album| Album {
                title: album.name,
                artists: vec![],
                album_type: shared::models::album::AlbumType::Unknown,
                cover: None,
                date: None,
                url: None,
            }),
            duration: search_item.duration.map(|duration| duration as i32),
            track_number: search_item.track_nr.map(|track_nr| track_nr as i32),
            provider: Some(TrackProvider::YoutubeMusic),
            provider_url: Some(format!(
                "https://music.youtube.com/watch?v={}",
                search_item.id
            )),
            source: None,
            source_url: None,
            date: None,
            cover: None,
            disc_number: None,
            file_path: None,
            genre: None,
            label: None,
        }
    }
}

#[async_trait]
impl Provider for YoutubeMusic {
    async fn search(&self, track: &Track) -> Result<String, Error> {
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
            .ok_or(Error::NoMatch("youtube music".to_string(), track.display()))?;
        Ok(best_match)
    }

    async fn download(&mut self, url: &str, base_dir: PathBuf) -> Result<PathBuf, Error> {
        download_with_ytdlp(url, base_dir).await
    }

    fn is_valid_url(url: &str) -> bool {
        url.contains("music.youtube.com/watch?v=")
    }
}
