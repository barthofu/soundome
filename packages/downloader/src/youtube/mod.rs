pub mod models;
pub mod matcher;

use std::{path::PathBuf, process::Stdio};

use async_trait::async_trait;
use invidious::{ClientAsync, MethodAsync};
use serde_json::Value;
use tokio::{process::Command, io::AsyncReadExt};

use shared::{errors::Error, models::track::Track};
use crate::Provider;

use self::matcher::Matcher;

pub struct Youtube {
    patterns: Vec<String>,
    excluded_words: Vec<String>,
    treshold: f32,
    duration_offset_percentage: i32,
    client: ClientAsync,
}

impl Youtube {

    pub fn new (invidious_instance: Option<String>) -> Self {
        Self {
            patterns: vec![
                String::from("{{channel}} {{title}}"),
                String::from("{{title}}"),
            ],
            excluded_words: vec![
                String::from("lyrics"),
                String::from("- topic"),
                String::from("topic"),
                String::from("official"),
                String::from("audio"),
                String::from("video"),
                String::from("music"),
                String::from("feat."),
                String::from("feat"),
                String::from("("),
                String::from(")"),
            ],
            treshold: 0.75,
            duration_offset_percentage: 50,
            client: ClientAsync::new(
                invidious_instance.unwrap_or(invidious::INSTANCE.to_string()),
                MethodAsync::default()
            ),
        }
    }
}

#[async_trait]
impl Provider for Youtube {

    /// Find a matching YouTube video from a track
    async fn search(&self, track: &Track) -> Result<String, Error> {
        // 1. Create search query
        let search_query = self.create_search_query(track.clone());

        // 2. Search on YouTube
        let search_results = self.get_results(search_query.clone()).await?;

        // 3. Process each pattern to find the best match
        self.patterns.iter()
            .find_map(|pattern| {
                // 3.1 Match results
                let weighted_results = self.match_results(search_results.clone(), search_query.clone(), pattern.clone());

                // 3.2 Order results
                let ordered_results = self.order_results(weighted_results);

                // 3.3 Get the best result if available
                self.get_best_result(ordered_results, track.clone()).map(|r| r.url)
            })
            .ok_or(Error::NoMatch("youtube".to_string(), track.display()))
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
