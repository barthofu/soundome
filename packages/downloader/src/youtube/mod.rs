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
    models::{Artist, Reference, Track},
    types::SoundomeResult,
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

    /// Returns all search results without similarity filtering, for manual user selection.
    pub async fn search_all(&self, track: &Track) -> SoundomeResult<Vec<Track>> {
        let query = self.create_search_query(track.clone());
        let results = self.get_results(&query).await?;
        Ok(results
            .items
            .iter()
            .filter_map(|item| self.convert_search_item_to_track(item.to_owned()))
            .collect())
    }

    /// Extract HTTP status code from HTML error response (e.g., "403 Forbidden").
    /// Looks for patterns like <title>403 Forbidden</title> or HTTP status lines.
    /// Returns the status line if found, otherwise None.
    fn extract_status_from_html(html: &str) -> Option<String> {
        // Try to extract from <title>...</title>
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                let status = &html[start + 7..start + end];
                return Some(status.to_string());
            }
        }
        
        // Try to extract from <h1>...</h1>
        if let Some(start) = html.find("<h1>") {
            if let Some(end) = html[start..].find("</h1>") {
                let status = &html[start + 4..start + end];
                return Some(status.to_string());
            }
        }
        
        None
    }

    async fn get_results(&self, search_query: &str) -> SoundomeResult<Search> {
        // TODO: the search query is interpolated raw into the querystring here;
        // `invidious` 0.7.8 does not URL-encode `params` before building the
        // request (see `ClientAsync::fetch`), so titles/artists containing
        // `&`, `#`, `%`, or `+` can produce a malformed request. Revisit if
        // this turns out to affect specific tracks.
        self.client
            .search(Some(&format!("q={}&type=video", search_query)))
            .await
            .map_err(|err| {
                // Log the full underlying error before collapsing it into a
                // `shared::errors::Error`, since some `InvidiousError` variants
                // (notably `SerdeError`, which fires when the configured
                // Invidious instance returns a non-JSON body, e.g. a rate-limit
                // or maintenance page) carry diagnostic detail that is
                // otherwise lost. This is a common failure mode of the
                // library's default public instance; configuring
                // `providers.youtube.invidious_instance` with a dedicated
                // instance is the usual mitigation.
                tracing::error!("Invidious search call failed: {}", err);

                Error::Custom(match err {
                    InvidiousError::ApiError { message, .. } => message,
                    InvidiousError::Fetch { error } => error.to_string(),
                    InvidiousError::SerdeError { error, original } => {
                        // SerdeError typically indicates the Invidious instance returned
                        // a non-JSON response (e.g., 403 Forbidden, rate-limit page, or
                        // maintenance page). Extract and format cleanly.
                        if let Some(ref raw_response) = original {
                            // Log the raw response for debugging (first 500 chars)
                            tracing::debug!("Invidious raw response (first 500 chars): {}", 
                                &raw_response.chars().take(500).collect::<String>());
                        }
                        
                        let status = original
                            .as_ref()
                            .and_then(|o| Self::extract_status_from_html(o))
                            .unwrap_or_else(|| "unknown error".to_string());

                        let suggestion = if status.contains("403") {
                            "\n\nThe Invidious instance returned 403 Forbidden. This means your request was blocked, \
                            possibly due to bot detection or rate limiting.\n\n\
                            Fix: Configure a different Invidious instance in config.toml:\n  \
                            [providers.youtube]\n  \
                            invidious_instance = \"https://invidious.tiekoetter.com/\"\n\n\
                            Or set via environment variable:\n  \
                            SOUNDOME__PROVIDERS__YOUTUBE__INVIDIOUS_INSTANCE=https://invidious.tiekoetter.com/\n\n\
                            Available instances: https://docs.invidious.io/instances/"
                        } else if status.contains("50") {
                            "\n\nThe Invidious instance returned a server error (5xx). It may be down or overloaded.\n\
                            Try a different instance: https://docs.invidious.io/instances/"
                        } else {
                            "\n\nTry configuring a different Invidious instance: https://docs.invidious.io/instances/"
                        };

                        format!(
                            "failed to parse Invidious response from server (got: {}): {}{}",
                            status,
                            error,
                            suggestion
                        )
                    },
                    InvidiousError::Message { message } => message,
                })
            })
    }

    fn convert_search_item_to_track(&self, search_item: SearchItem) -> Option<Track> {
        match search_item {
            SearchItem::Video(video) => Some(Track {
                id: None,
                needs_validation: false,
                validation_reason: None,
                soundome_id: None,
                title: video.title,
                artists: vec![Artist {
                    id: None,
                    name: video.author,
                    icon: None,
                    references: vec![],
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
            .filter_map(|item| self.convert_search_item_to_track(item.to_owned()))
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
