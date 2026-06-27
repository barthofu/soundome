use std::path::Path;

use reqwest::Client;
use tokio::io::AsyncWriteExt;

use super::models::{AlbumDto, ArtistDto, IngestResult, PlaylistDto, PlaylistTrackDto, ScanReport, TrackDto};

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

    pub async fn get_artists(&self) -> anyhow::Result<Vec<ArtistDto>> {
        let url = format!("{}/api/artists", self.base_url);
        let artists = self.client.get(&url).send().await?.json().await?;
        Ok(artists)
    }

    pub async fn get_albums(&self) -> anyhow::Result<Vec<AlbumDto>> {
        let url = format!("{}/api/albums", self.base_url);
        let albums = self.client.get(&url).send().await?.json().await?;
        Ok(albums)
    }

    pub async fn get_tracks(&self) -> anyhow::Result<Vec<TrackDto>> {
        let url = format!("{}/api/tracks", self.base_url);
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

    /// Call `POST /api/library/scan` and return the `ScanReport`.
    pub async fn scan(
        &self,
        library_root: Option<&str>,
        dry_run: bool,
    ) -> anyhow::Result<ScanReport> {
        use serde_json::json;

        let url = format!("{}/api/library/scan", self.base_url);
        let body = json!({
            "library_root": library_root,
            "dry_run": dry_run,
        });
        let report = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(report)
    }

    /// Call `POST /api/library/ingest` for a single local audio file.
    pub async fn ingest(&self, file_path: &str) -> anyhow::Result<IngestResult> {
        use serde_json::json;

        let url = format!("{}/api/library/ingest", self.base_url);
        let body = json!({ "file_path": file_path });
        let result = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(result)
    }
}
