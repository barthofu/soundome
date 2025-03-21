use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub providers: ProvidersConfig,
    pub ai: AiConfig,
}

// ===============================================================================
// General
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct GeneralConfig {
    pub base_dir: String,
}

// ===============================================================================
// Database
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfig {
    pub url: String,
}

// ===============================================================================
// Providers
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct ProvidersConfig {
    pub spotify: SpotifyConfig,
    pub youtube: Option<YoutubeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct YoutubeConfig {
    pub invidious_instance: Option<String>,
}

// ===============================================================================
// AI
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct AiConfig {
    pub enabled: bool,
    pub openrouter: Option<OpenRouterConfig>,
}


#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub timeout: Option<u64>,
}
