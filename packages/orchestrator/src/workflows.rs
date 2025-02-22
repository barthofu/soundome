use config::AppConfig;
use diesel::SqliteConnection;
use downloader::{youtube::Youtube, Provider};
use fetcher::{spotify::Spotify, Fetcher};
use tagger::file::tag_file_with_track;
use std::path::PathBuf;

use shared::errors::Error;

pub async fn download_spotify_track(url: &str, conn: &SqliteConnection, config: &AppConfig) -> Result<PathBuf, Error> {
    let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret)
        .map_err(|_| Error::Config)?;
    let mut youtube = Youtube::new();

    let track = Spotify::get_track_from_url(&spotify, url).map_err(|_| Error::NotFound)?;

    let youtube_url = youtube
        .search(&track)
        .await
        .ok_or(Error::NotFound)?;

    let file_path = youtube
        .download(&youtube_url, config.base_dir.as_ref())
        .await
        .map_err(|_| Error::InternalServer)?;

    println!("Downloaded track to: {}", file_path.to_str().unwrap_or("Invalid path"));

    tag_file_with_track(&file_path, &track)
        .map_err(|_| Error::InternalServer)?;

    Ok(file_path)
}
