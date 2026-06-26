mod matcher;

use std::path::PathBuf;

use async_trait::async_trait;
use fetcher::Source;
use shared::{
    errors::Error,
    models::{Reference, Track},
    types::SoundomeResult,
};

use crate::{utils::ytdlp::download_with_ytdlp, Matcher, Provider};

pub struct SoundCloud {
    fetcher: fetcher::soundcloud::Soundcloud,
    similarity_treshold: f64,
}

impl SoundCloud {
    pub async fn new() -> Result<Self, Error> {
        fetcher::soundcloud::Soundcloud::new()
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

/// Returns true when yt-dlp stderr indicates the track is DRM/subscription-protected.
/// SoundCloud DRM manifests in several ways — including a 404 on the JSON metadata endpoint
/// when yt-dlp tries to fetch the stream manifest for a go+ / DRM-gated track.
fn is_drm_error(stderr: &str) -> bool {
    let s = stderr.to_lowercase();
    s.contains("drm")
        || s.contains("subscription")
        || s.contains("premium")
        || s.contains("go+")
        || s.contains("requires purchase")
        || s.contains("not available in your country")
        || s.contains("unable to download json metadata")
}

#[async_trait]
impl Provider for SoundCloud {
    async fn search(&self, track: &Track) -> SoundomeResult<Reference> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());

        // 2. Search on SoundCloud
        let search_results = self.fetcher.get_tracks_from_query(&search_query).await?;

        // 3. Process each pattern to find the best match
        let best_match = self
            .match_results(search_results, track.clone())
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
    ) -> Result<PathBuf, Error> {
        match download_with_ytdlp(url, file_name, base_library_dir).await {
            Err(Error::ExitCode { code, ref stderr }) if is_drm_error(stderr) => {
                tracing::warn!(
                    "SoundCloud DRM protection detected (exit {}) for {}: {}",
                    code,
                    url,
                    stderr
                );
                Err(Error::SoundCloudDrmProtected(url.to_string()))
            }
            other => other,
        }
    }

    fn is_valid_url(url: &str) -> bool {
        fetcher::soundcloud::Soundcloud::is_valid_track_url(url)
    }
}
