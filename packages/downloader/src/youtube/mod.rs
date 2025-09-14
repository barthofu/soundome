mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;
use config::Config;
use invidious::{
    hidden::SearchItem, universal::Search, ClientAsync, ClientAsyncTrait, InvidiousError,
    MethodAsync,
};

use crate::{utils::ytdlp::download_with_ytdlp, Matcher, Provider};
use shared::{
    errors::Error,
    models::{
        Artist, Reference, Track
    }, types::SoundomeResult,
};

pub struct Youtube<'a> {
    patterns: Vec<(&'a str, &'a str)>,
    excluded_words: Vec<&'a str>,
    similarity_treshold: f64,
    client: ClientAsync,
}

impl Youtube<'_> {
    pub fn new(invidious_instance: Option<String>) -> Self {
        // TODO: the Invidious client probably uses reqwest internally.
        // For now, we document this limitation.
        if let Some(proxy) = Config::get().proxy.as_ref() {
            if proxy.enabled {
                tracing::warn!("Proxy configuration for YouTube downloader is not yet supported by the invidious library. Consider setting HTTP_PROXY environment variable.");
            }
        }

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
            client: ClientAsync::new(
                invidious_instance.unwrap_or(invidious::INSTANCE.to_string()),
                MethodAsync::default(),
            ),
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

    async fn get_results(&self, search_query: &str) -> SoundomeResult<Search> {
        self.client
            .search(Some(&format!("q={}&type=video", search_query)))
            .await
            .map_err(|err| {
                Error::Custom(match err {
                    InvidiousError::ApiError { message, .. } => message,
                    InvidiousError::Fetch { error } => error.to_string(),
                    _ => "Unknown error from Invidious call".to_string(),
                })
            })
    }

    fn convert_search_item_to_track(&self, search_item: SearchItem) -> Option<Track> {
        match search_item {
            SearchItem::Video(video) => Some(Track {
                id: None,
                title: video.title,
                artists: vec![Artist {
                    id: None,
                    name: video.author,
                    icon: None,
                    references: vec![]
                }],
                album: None,
                duration: Some(video.length as i32),
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
                    external_id: Some(video.id.to_string()),
                    external_url: Some(format!("https://www.youtube.com/watch?v={}", video.id)),
                }],
            }),
            _ => None,
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
            .items
            .iter()
            .map(|item| self.convert_search_item_to_track(item.to_owned()))
            .filter(|track| track.is_some())
            .map(|track| track.unwrap())
            .collect();

        // 3. Find the best match
        let best_match = self
            .match_results(search_results, track.clone())
            .ok_or(Error::NoMatch("youtube".to_string(), track.display()))?
            .get_provider()
            .ok_or(Error::NoMatch("youtube".to_string(), track.display()))?;
        
        Ok(best_match)
    }

    async fn download(&mut self, url: &str, file_name: &str, base_library_dir: PathBuf) -> SoundomeResult<PathBuf> {
        // if the url is a youtube music one, convert it to a youtube one
        // let url = url.replace("music.youtube.com", "www.youtube.com");

        download_with_ytdlp(&url, file_name, base_library_dir).await
    }

    fn is_valid_url(url: &str) -> bool {
        url.contains("youtube.com/watch?v=")
    }
}
