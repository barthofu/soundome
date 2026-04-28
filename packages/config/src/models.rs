use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Config {
    pub general: GeneralConfig,
    pub logs: LogsConfig,
    pub database: DatabaseConfig,
    pub providers: ProvidersConfig,
    pub ai: AiConfig,
    pub proxy: Option<ProxyConfig>,
    #[serde(default)]
    pub tagger: TaggerConfig,
}

// ===============================================================================
// General
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct GeneralConfig {
    pub base_library_dir: String,
    pub temp_download_dir: String,
}

// ===============================================================================
// Logs
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct LogsConfig {
    pub level: String,
    pub enable_reqwest_logging: bool,
}

// ===============================================================================
// Database
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: Option<u32>,
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

// ===============================================================================
// Tagger
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct TaggerConfig {
    /// List of enabled metadata provider names, in priority order.
    /// Supported values: "musicbrainz", "bandcamp", "spotify"
    #[serde(default = "TaggerConfig::default_providers")]
    pub metadata_providers: Vec<String>,
}

impl Default for TaggerConfig {
    fn default() -> Self {
        Self {
            metadata_providers: Self::default_providers(),
        }
    }
}

impl TaggerConfig {
    fn default_providers() -> Vec<String> {
        vec!["musicbrainz".to_string()]
    }
}

// ===============================================================================
// Proxy
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub urls: Vec<String>, // List of proxy URLs with embedded credentials if needed
    pub strategy: Option<ProxyStrategy>, // Proxy rotation strategy
    pub no_proxy: Option<Vec<String>>, // List of domains to exclude from proxy
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub enum ProxyStrategy {
    #[serde(rename = "round_robin")]
    RoundRobin,
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "sticky_per_hour")]
    StickyPerHour,
    #[serde(rename = "first_available")]
    FirstAvailable,
}
