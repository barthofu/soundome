mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;

use crate::{
    utils::ytdlp::{download_with_ytdlp, search_with_ytdlp, YtDlpSearchResult},
    Matcher, Provider,
};
use shared::{
    errors::Error,
    models::{Artist, Reference, Track},
    types::SoundomeResult,
};

/// Number of results requested per `ytsearchN:` query. Chosen to match the
/// page size the previous Invidious search endpoint returned, so match
/// quality (handled downstream by `Matcher::match_results`) does not regress.
const SEARCH_RESULT_LIMIT: usize = 20;

pub struct Youtube<'a> {
    patterns: Vec<(&'a str, &'a str)>,
    excluded_words: Vec<&'a str>,
    similarity_treshold: f64,
}

impl Default for Youtube<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Youtube<'_> {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (
                    "{video_author} {video_title}",
                    "{track_artist} {track_title}",
                ),
                (
                    "{video_author} {video_title}",
                    "{track_artists} {track_title}",
                ),
                ("{video_title}", "{track_artist} {track_title}"),
                ("{video_title}", "{track_artists} {track_title}"),
            ],
            excluded_words: vec![
                "lyrics", "- topic", "topic", "official", "audio", "video", "explicit", "music",
                "feat.", "ft", "feat", "(", ")",
            ],
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

    /// Returns all search results without similarity filtering, for manual user selection.
    pub async fn search_all(&self, track: &Track) -> SoundomeResult<Vec<Track>> {
        let query = self.create_search_query(track.clone());
        let results = self.get_results(&query).await?;
        Ok(results
            .into_iter()
            .map(|item| self.convert_search_item_to_track(item))
            .collect())
    }

    async fn get_results(&self, search_query: &str) -> SoundomeResult<Vec<YtDlpSearchResult>> {
        search_with_ytdlp(search_query, SEARCH_RESULT_LIMIT)
            .await
            .map_err(|err| {
                tracing::error!("yt-dlp YouTube search failed: {}", err);
                err
            })
    }

    fn convert_search_item_to_track(&self, search_item: YtDlpSearchResult) -> Track {
        Track {
            id: None,
            needs_validation: false,
            validation_reason: None,
            soundome_id: None,
            title: search_item.title,
            artists: vec![Artist {
                id: None,
                name: search_item.author,
                icon: None,
                references: vec![],
            }],
            album: None,
            duration: search_item.duration,
            cover: None,
            date: None,
            disc_number: None,
            track_number: None,
            file_path: None,
            genre: None,
            label: None,
            references: vec![shared::models::Reference {
                id: None,
                ref_type: shared::models::ReferenceType::Provider,
                platform: shared::models::Platform::Youtube,
                external_id: Some(search_item.id.clone()),
                external_url: Some(format!(
                    "https://www.youtube.com/watch?v={}",
                    search_item.id
                )),
            }],
        }
    }
}

#[async_trait]
impl Provider for Youtube<'_> {
    async fn search(&self, track: &Track) -> SoundomeResult<Reference> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());
        tracing::info!("Youtube search query: {}", search_query);

        // 2. Search on YouTube
        let search_results: Vec<Track> = self
            .get_results(&search_query)
            .await?
            .into_iter()
            .map(|item| self.convert_search_item_to_track(item))
            .collect();

        // 3. Find the best match
        let best_match = self
            .match_results(search_results, track.clone())
            .ok_or(Error::NoMatch("youtube".to_string(), track.display()))?
            .get_provider()
            .ok_or(Error::NoMatch("youtube".to_string(), track.display()))?;

        Ok(best_match)
    }

    async fn download(
        &mut self,
        url: &str,
        file_name: &str,
        base_library_dir: PathBuf,
    ) -> SoundomeResult<PathBuf> {
        // if the url is a youtube music one, convert it to a youtube one
        // let url = url.replace("music.youtube.com", "www.youtube.com");

        download_with_ytdlp(url, file_name, base_library_dir).await
    }

    fn is_valid_url(url: &str) -> bool {
        url.contains("youtube.com/watch?v=")
    }
}
