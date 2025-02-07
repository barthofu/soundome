use std::path::Path;

use downloader::{youtube::Youtube, Provider};
use fetcher::{spotify::Spotify, Fetcher};

pub async fn download_spotify_track(url: &str, base_dir: &Path) {
    let spotify = Spotify::new().unwrap();
    let mut youtube = Youtube::new();

    let track = Spotify::get_track_from_url(&spotify, url);
    let url = youtube.search(track.unwrap()).await;
    println!("{}", url.clone().unwrap_or_else(|| String::from("No results found")));

    let file_path = youtube.download(&url.clone().unwrap(), base_dir).await;
    println!("{}", file_path.unwrap().to_str().unwrap());

}
