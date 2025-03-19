use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub spotify: SpotifyConfig,
    pub youtube: Option<YoutubeConfig>,
    pub openrouter: Option<OpenRouterConfig>,
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

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub timeout: Option<u64>,
}

