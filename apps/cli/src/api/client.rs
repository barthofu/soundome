use std::path::Path;

use reqwest::Client;
use tokio::io::AsyncWriteExt;

use super::models::{PlaylistDto, PlaylistTrackDto};

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_playlists(&self) -> anyhow::Result<Vec<PlaylistDto>> {
        let url = format!("{}/api/playlists", self.base_url);
        let playlists = self.client.get(&url).send().await?.json().await?;
        Ok(playlists)
    }

    pub async fn get_playlist_tracks(&self, id: i32) -> anyhow::Result<Vec<PlaylistTrackDto>> {
        let url = format!("{}/api/playlists/{}/tracks", self.base_url, id);
        let tracks = self.client.get(&url).send().await?.json().await?;
        Ok(tracks)
    }

    /// Stream `GET /api/tracks/:id/download` into `dest`, calling `on_chunk(bytes)`
    /// after each received chunk so the caller can update a progress bar.
    pub async fn download_track(
        &self,
        track_id: i32,
        dest: &Path,
        mut on_chunk: impl FnMut(u64),
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/tracks/{}/download", self.base_url, track_id);
        let mut response = self.client.get(&url).send().await?.error_for_status()?;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = tokio::fs::File::create(dest).await?;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            on_chunk(chunk.len() as u64);
        }

        file.flush().await?;
        Ok(())
    }
}
