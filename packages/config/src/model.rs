use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub spotify: SpotifyConfig,
    pub youtube: Option<YoutubeConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GeneralConfig {
    pub base_dir: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct YoutubeConfig {
    pub invidious_instance: Option<String>,
}

pub struct SoundcloudConfig {
    pub client_id: String,
}