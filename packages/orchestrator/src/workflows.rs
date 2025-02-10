use config::AppConfig;
use downloader::{youtube::Youtube, Provider};
use fetcher::{spotify::Spotify, Fetcher};

pub async fn download_spotify_track(url: &str, config: &AppConfig) {
    let spotify = Spotify::new(&config.spotify.client_id, &config.spotify.client_secret).unwrap();
    let mut youtube = Youtube::new();

    let track = Spotify::get_track_from_url(&spotify, url);
    let url = youtube.search(track.unwrap()).await;
    println!("{}", url.clone().unwrap_or_else(|| String::from("No results found")));

    let file_path = youtube.download(&url.clone().unwrap(), config.base_dir.as_ref()).await;
    println!("{}", file_path.unwrap().to_str().unwrap());
}
