pub mod models;
pub mod matcher;

use std::{path::{Path, PathBuf}, process::Stdio};

use async_trait::async_trait;
use serde_json::Value;
use tokio::{time::timeout, process::Command, io::AsyncReadExt};

use shared::{errors::Error, models::track::Track};
use crate::Provider;

use self::{models::YoutubeSearchResult, matcher::Matcher};

pub struct Youtube {
    patterns: Vec<String>,
    excluded_words: Vec<String>,
    treshold: f32,
    duration_offset_percentage: i32,
}

impl Youtube {

    pub fn new () -> Self {
        Self {
            // fuzzy_matcher: SkimMatcherV2::default(),
            patterns: vec![
                String::from("{{title}}"),
                String::from("{{title}} {{channel}}"),
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
            // yt_dlp: None
        }
    }

}

#[async_trait]
impl Provider for Youtube {

    /// find a matching youtube video from a track
    async fn search(&self, track: Track) -> Option<String> {

        // 1. create search query
        let search_query = self.create_search_query(track.clone());

        // 2. search on youtube
        let search_results = self.get_results(search_query.clone()).await;
        if search_results.is_none() {
            return None;
        }

        let mut result: Option<YoutubeSearchResult> = None;

        // 3. process each result by matching them with the track
        for pattern in &self.patterns {

            // 3.1 match results
            let weighted_results = self.match_results(search_results.clone().unwrap(), search_query.clone(), pattern.clone());
            if weighted_results.is_empty() {
                continue;
            }

            // 3.2 order results
            let ordered_results = self.order_results(weighted_results);

            // 3.3 get the best result
            let best_result = self.get_best_result(ordered_results, track.clone());

            // 3.4 check if the best result is good enough
            if best_result.is_some() {
                result = Some(best_result.unwrap().clone());
                break;
            }
        }

        result.map(|r| r.url)
    }

    async fn download(&mut self, url: &str, base_dir: &Path) -> Result<PathBuf, Error> {

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
                "--embed-metadata",
                "--add-metadata",
                "--metadata-from-title", "%(artist)s - %(title)s",
                "--output", &format!("{}/%(title)s.%(ext)s", base_dir.to_str().unwrap())
            ])
            .spawn()?;

        // Continually read from stdout so that it does not fill up with large output and hang forever.
        // We don't need to do this for stderr since only stdout has potentially giant JSON.
        let mut stdout = Vec::new();
        let child_stdout = child.stdout.take();
        tokio::io::copy(&mut child_stdout.unwrap(), &mut stdout).await?;

        let exit_code = if let Some(duration) = None { // TODO: timeout
            match timeout(duration, child.wait()).await {
                Ok(n) => n?,
                Err(_) => {
                    child.kill().await?;
                    return Err(Error::ProcessTimeout);
                }
            }
        } else {
            child.wait().await?
        };

        println!("exit code: {:?}", exit_code);

        if exit_code.success() {

            // let decoded_stdout = std::str::from_utf8(&stdout).expect("invalid utf-8 output");

            // println!("stdout: {:?}", decoded_stdout);
            let value: Value = serde_json::from_reader(stdout.as_slice())?;

            let path = value["_filename"].as_str();

            return Ok(path.unwrap().into());

        } else {

            let mut stderr = vec![];

            if let Some(mut reader) = child.stderr {
                reader.read_to_end(&mut stderr).await?;
            }

            let stderr = String::from_utf8(stderr).unwrap_or_default();

            return Err(Error::ExitCode {
                code: exit_code.code().unwrap_or(1),
                stderr,
            });
        }

    }

}
