use async_trait::async_trait;
use config::Config;
use rustypipe::client::RustyPipe;
use serde::Deserialize;
use shared::{
    errors::Error,
    http::HttpClientBuilder,
    models::{Album, Artist, Platform, Playlist, PlaylistTrack, Reference, ReferenceType, Track},
    types::SoundomeResult,
};
use std::process::Command;

use crate::Source;

pub struct Youtube {
    _client: RustyPipe,
}

/// Minimal shape of yt-dlp JSON output for a single video
#[derive(Debug, Deserialize)]
struct YtDlpVideoInfo {
    #[serde(default)]
    _id: String,
    title: String,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    uploader: Option<String>,
    /// Duration in seconds
    #[serde(default)]
    duration: Option<i32>,
}

impl Youtube {
    pub fn new() -> SoundomeResult<Self> {
        let client = match Config::get().proxy.as_ref() {
            Some(proxy_config) if proxy_config.enabled => {
                let reqwest_client = HttpClientBuilder::get_reqwest_client_builder()?;
                RustyPipe::builder()
                    .build_with_client(reqwest_client)
                    .expect("Failed to create RustyPipe client with proxy")
            }
            _ => RustyPipe::builder()
                .build()
                .expect("Failed to create RustyPipe client"),
        };

        Ok(Self { _client: client })
    }

    /// Extracts the video ID from a YouTube URL
    fn get_id_from_url(url: &str) -> Option<String> {
        // Extract from youtube.com/watch?v=VIDEOID
        if let Some(pos) = url.find("v=") {
            let rest = &url[pos + 2..];
            let id: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if id.len() == 11 {
                return Some(id);
            }
        }

        // Extract from youtu.be/VIDEOID
        if let Some(pos) = url.find("youtu.be/") {
            let rest = &url[pos + 9..];
            let id: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if id.len() == 11 {
                return Some(id);
            }
        }

        None
    }

    /// Extracts the playlist ID from a YouTube playlist URL
    fn get_playlist_id(url: &str) -> Option<String> {
        // Extract from youtube.com/playlist?list=PLAYLISTID or youtube.com/watch?v=...&list=PLAYLISTID
        if let Some(pos) = url.find("list=") {
            let rest = &url[pos + 5..];
            let id: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if !id.is_empty() {
                return Some(id);
            }
        }
        None
    }

    /// Uses yt-dlp to extract video metadata (title, channel/uploader) without downloading
    /// This is a synchronous wrapper that blocks briefly to get metadata.
    fn get_video_metadata(url: &str) -> Option<YtDlpVideoInfo> {
        let output = Command::new("yt-dlp")
            .arg(url)
            .arg("--dump-json")
            .arg("--skip-download")
            .output()
            .ok()?;

        if !output.status.success() {
            tracing::warn!(
                "yt-dlp exited with code {:?} for URL: {}",
                output.status.code(),
                url
            );
            return None;
        }

        let info: YtDlpVideoInfo = serde_json::from_slice(&output.stdout).ok()?;
        Some(info)
    }
}

#[async_trait]
impl Source for Youtube {
    async fn get_track_from_url(&self, url: &str) -> SoundomeResult<Track> {
        let video_id = Self::get_id_from_url(url).ok_or(Error::InvalidUrl(url.to_string()))?;

        // Try to extract metadata using yt-dlp
        let (title, artist_name, duration, needs_validation) =
            if let Some(info) = Self::get_video_metadata(url) {
                let artist = info
                    .channel
                    .or(info.uploader)
                    .unwrap_or_else(|| "Unknown Artist".to_string());
                (info.title, artist, info.duration, false)
            } else {
                // Fallback: create a track with minimal metadata that requires validation
                tracing::warn!(
                    "Failed to extract YouTube metadata for {}, will require validation",
                    url
                );
                (
                    "Unknown Title (YouTube)".to_string(),
                    "Unknown Artist".to_string(),
                    None,
                    true,
                )
            };

        Ok(Track {
            id: None,
            needs_validation,
            validation_reason: if needs_validation {
                Some("youtube_metadata_extraction_failed".to_string())
            } else {
                None
            },
            soundome_id: None,
            title,
            artists: vec![Artist {
                id: None,
                name: artist_name,
                icon: None,
                references: vec![],
            }],
            album: None,
            genre: None,
            duration,
            file_path: None,
            track_number: None,
            disc_number: None,
            label: None,
            date: None,
            cover: None,
            references: vec![Reference {
                id: None,
                ref_type: ReferenceType::Source,
                platform: Platform::Youtube,
                external_id: Some(video_id),
                external_url: Some(url.to_string()),
            }],
        })
    }

    async fn get_tracks_from_query(&self, _search: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented(
            "YouTube track search is not yet implemented".to_string(),
        ))
    }

    async fn get_playlist_from_url(&self, url: &str) -> SoundomeResult<Playlist> {
        let _list_id = Self::get_playlist_id(url).ok_or(Error::InvalidUrl(url.to_string()))?;

        // Create a minimal playlist metadata
        tracing::info!("Creating YouTube playlist from URL");

        Ok(Playlist {
            id: None,
            name: "YouTube Playlist".to_string(),
            source: Platform::Youtube,
            source_url: Some(url.to_string()),
            cover: None,
        })
    }

    async fn get_playlist_tracks_from_url(&self, _url: &str) -> SoundomeResult<Vec<PlaylistTrack>> {
        Err(Error::NotImplemented(
            "YouTube playlist track retrieval is not yet implemented".to_string(),
        ))
    }

    async fn get_artist_from_url(&self, _url: &str) -> SoundomeResult<Artist> {
        Err(Error::NotImplemented(
            "YouTube artist URLs are not yet supported".to_string(),
        ))
    }

    async fn get_artist_tracks_from_url(&self, _url: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented(
            "YouTube artist track retrieval is not yet supported".to_string(),
        ))
    }

    async fn get_artists_from_query(&self, _search: &str) -> SoundomeResult<Vec<Artist>> {
        Err(Error::NotImplemented(
            "YouTube artist search is not yet implemented".to_string(),
        ))
    }

    async fn get_album_from_url(&self, _url: &str) -> SoundomeResult<Album> {
        Err(Error::NotImplemented(
            "YouTube album URLs are not yet supported".to_string(),
        ))
    }

    async fn get_albums_from_query(&self, _search: &str) -> SoundomeResult<Vec<Album>> {
        Err(Error::NotImplemented(
            "YouTube album search is not yet implemented".to_string(),
        ))
    }

    async fn get_album_tracks_from_url(&self, _url: &str) -> SoundomeResult<Vec<Track>> {
        Err(Error::NotImplemented(
            "YouTube album track retrieval is not yet supported".to_string(),
        ))
    }

    async fn clean_track_metadata(&self, _track: &mut Track) -> SoundomeResult<()> {
        Ok(())
    }

    async fn clean_tracks_metadata(
        &self,
        _tracks: &mut Vec<&mut Track>,
        _on_batch: Option<&mut (dyn FnMut(usize, usize) + Send)>,
    ) -> SoundomeResult<()> {
        Ok(())
    }

    fn is_valid_track_url(url: &str) -> bool {
        // Match youtube.com/watch?v=... and youtu.be/...
        (url.contains("youtube.com/watch?v=") || url.contains("youtu.be/")) && url.len() > 20
        // Basic sanity check
    }

    fn is_valid_playlist_url(url: &str) -> bool {
        // YouTube playlists have ?list=... parameter
        // Exclude music.youtube.com (handled by YoutubeMusic source)
        url.contains("youtube.com")
            && !url.contains("music.youtube.com")
            && url.contains("list=")
            && url.len() > 30 // Basic sanity check
    }

    fn is_valid_artist_url(_url: &str) -> bool {
        false // Not yet supported
    }

    fn is_valid_album_url(_url: &str) -> bool {
        false // Not yet supported
    }
}
