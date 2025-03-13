pub mod models;
pub mod matcher;

use std::{path::PathBuf, process::Stdio};

use async_trait::async_trait;
use invidious::{hidden::SearchItem, universal::Search, ClientAsync, ClientAsyncTrait, InvidiousError, MethodAsync};
use serde_json::Value;
use tokio::{process::Command, io::AsyncReadExt};

use shared::{errors::Error, models::{artist::Artist, track::{Track, TrackProvider}}};
use crate::{Matcher, Provider};

pub struct Youtube<'a> {
    patterns: Vec<(&'a str, &'a str)>,
    excluded_words: Vec<&'a str>,
    similarity_treshold: f64,
    client: ClientAsync,
}

impl Youtube<'_> {

    pub fn new (invidious_instance: Option<String>) -> Self {
        Self {
            patterns: vec![
                ("{video_author} {video_title}", "{track_artist} {track_title}"),
                ("{video_author} {video_title}", "{track_artists} {track_title}"),
                ("{video_title}",                "{track_artist} {track_title}"),
                ("{video_title}",                "{track_artists} {track_title}"),
            ],
            excluded_words: vec![
                "lyrics",
                "- topic",
                "topic",
                "official",
                "audio",
                "video",
                "explicit",
                "music",
                "feat.",
                "ft",
                "feat",
                "(",
                ")",
            ],
            similarity_treshold: 0.80,
            client: ClientAsync::new(
                invidious_instance.unwrap_or(invidious::INSTANCE.to_string()),
                MethodAsync::default()
            ),
        }
    }

    fn create_search_query(&self, track: Track) -> String {
        let artist = track.artists.into_iter().map(|a| a.name.clone()).collect::<Vec<String>>().join(" ");
        format!("{} {}", artist, track.title)
    }

    async fn get_results(&self, search_query: String) -> Result<Search, Error> {
        self.client
            .search(Some(&format!("q={}&type=video", search_query)))
            .await
            .map_err(|err| Error::Custom(match err {
                InvidiousError::ApiError { message, .. } => message,
                InvidiousError::Fetch { error } => error.to_string(),
                _ => "Unknown error from Invidious call".to_string(),
            }))
    }

    fn convert_search_item_to_track(search_item: SearchItem) -> Option<Track> {
        match search_item {
            SearchItem::Video(video) => Some(Track {
                title: video.title,
                artists: vec![Artist {
                    name: video.author,
                    icon: None,
                    url: None
                }],
                album: None,
                duration: Some(video.length as i32),
                provider: Some(TrackProvider::Youtube),
                provider_url: Some(format!("https://www.youtube.com/watch?v={}", video.id)),
                source: None,
                source_url: None,
                cover: None,
                date: None,
                disc_number: None,
                track_number: None,
                file_path: None,
                genre: None,
                label: None,
            }),
            _ => None,
        }
    }
}

#[async_trait]
impl Provider for Youtube<'_> {

    /// Find a matching YouTube video from a track
    async fn search(&self, track: &Track) -> Result<String, Error> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());
        println!("SEARCH QUERY: {}", search_query);

        // 2. Search on YouTube
        let search_results: Vec<Track> = self.get_results(search_query.clone()).await?
            .items.iter()
            .map(|item| Self::convert_search_item_to_track(item.to_owned()))
            .filter(|track| track.is_some())
            .map(|track| track.unwrap())
            .collect();

        // 3. Find the best match
        let best_match = self.match_results(search_results, track.clone()).ok_or(Error::NoMatch("youtube".to_string(), track.display()))?;
        Ok(best_match)
    }

    async fn download(&mut self, url: &str, base_dir: PathBuf) -> Result<PathBuf, Error> {
        let output_path = format!("{}/%(title)s.%(ext)s", base_dir.to_str().unwrap());

        let mut child = Command::new("yt-dlp")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&[
                url,
                "--print-json",
                "-f", "ba",
                "-x",
                "--audio-format", "mp3",
                "--audio-quality", "0",
                "--embed-thumbnail",
                "--output", &output_path,
            ])
            .spawn()?;

        // Read stdout asynchronously to prevent buffer overflow
        let mut stdout = Vec::new();
        if let Some(mut child_stdout) = child.stdout.take() {
            tokio::io::copy(&mut child_stdout, &mut stdout).await?;
        }

        // TODO: Implement timeout handling
        let exit_code = child.wait().await?;

        if !exit_code.success() {
            let mut stderr = Vec::new();
            if let Some(mut reader) = child.stderr {
                reader.read_to_end(&mut stderr).await?;
            }
            return Err(Error::ExitCode {
                code: exit_code.code().unwrap_or(1),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        // Parse JSON output
        let value: Value = serde_json::from_slice(&stdout)?;
        let path = value["_filename"].as_str().ok_or(Error::NotFound("downloaded file path".to_string()))?;

        // Replace extension with .mp3
        let final_path = PathBuf::from(match path.rsplit_once('.') {
            Some((base, _)) => format!("{}.mp3", base),
            None => format!("{}.mp3", path),
        });

        Ok(final_path)
    }

    fn is_valid_url(url: &str) -> bool {
        url.starts_with("https://www.youtube.com/watch?v=")
    }

}
