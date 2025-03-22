mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;
use config::model::AiConfig;
use fetcher::Source;
use shared::{errors::Error, models::track::Track};

use crate::{utils::ytdlp::download_with_ytdlp, Matcher, Provider};

pub struct SoundCloud {
    fetcher: fetcher::soundcloud::Soundcloud,
    similarity_treshold: f64,
}

impl SoundCloud {
    pub async fn new(ai_config: AiConfig) -> Result<Self, Error> {
        fetcher::soundcloud::Soundcloud::new(ai_config)
            .await
            .map(|fetcher| Self {
                fetcher,
                similarity_treshold: 0.80,
            })
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
}

#[async_trait]
impl Provider for SoundCloud {
    async fn search(&self, track: &Track) -> Result<String, Error> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());

        // 2. Search on SoundCloud
        let search_results = self.fetcher.get_tracks_from_query(&search_query).await?;

        // 3. Process each pattern to find the best match
        let best_match = self
            .match_results(search_results, track.clone())
            .ok_or(Error::NoMatch("youtube music".to_string(), track.display()))?;
        Ok(best_match)
    }

    async fn download(&mut self, url: &str, file_name: &str, base_dir: PathBuf) -> Result<PathBuf, Error> {
        download_with_ytdlp(url, file_name, base_dir).await
    }

    fn is_valid_url(url: &str) -> bool {
        fetcher::soundcloud::Soundcloud::is_valid_track_url(url)
    }
}
