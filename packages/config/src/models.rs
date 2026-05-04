use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub logs: LogsConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub ai: AiConfig,
    pub proxy: Option<ProxyConfig>,
    #[serde(default)]
    pub tagger: TaggerConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

// ===============================================================================
// General
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct GeneralConfig {
    #[serde(default = "GeneralConfig::default_base_library_dir")]
    pub base_library_dir: String,
    #[serde(default = "GeneralConfig::default_temp_download_dir")]
    pub temp_download_dir: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            base_library_dir: Self::default_base_library_dir(),
            temp_download_dir: Self::default_temp_download_dir(),
        }
    }
}

impl GeneralConfig {
    fn default_base_library_dir() -> String { "./library".to_string() }
    fn default_temp_download_dir() -> String { "./temp".to_string() }
}

// ===============================================================================
// Logs
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct LogsConfig {
    #[serde(default = "LogsConfig::default_level")]
    pub level: String,
    #[serde(default)]
    pub enable_reqwest_logging: bool,
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            enable_reqwest_logging: false,
        }
    }
}

impl LogsConfig {
    fn default_level() -> String { "info".to_string() }
}

// ===============================================================================
// Database
// ===============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfig {
    #[serde(default = "DatabaseConfig::default_url")]
    pub url: String,
    pub pool_size: Option<u32>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            pool_size: None,
        }
    }
}

impl DatabaseConfig {
    fn default_url() -> String { "./data/soundome.db".to_string() }
}

// ===============================================================================
// Providers
// ===============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(unused)]
pub struct ProvidersConfig {
    pub spotify: Option<SpotifyConfig>,
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
    #[serde(default)]
    pub enabled: bool,
    pub openrouter: Option<OpenRouterConfig>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            openrouter: None,
        }
    }
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
        vec!["musicbrainz".to_string(), "bandcamp".to_string(), "spotify".to_string()]
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

// ===============================================================================
// Server
// ===============================================================================

/// Optional server binding overrides. When omitted, Rocket.toml values apply.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(unused)]
pub struct ServerConfig {
    /// IP address or hostname to bind. E.g. "0.0.0.0" or "127.0.0.1".
    /// ENV: SOUNDOME__SERVER__HOST
    pub host: Option<String>,
    /// TCP port to listen on.
    /// ENV: SOUNDOME__SERVER__PORT
    pub port: Option<u16>,
}
