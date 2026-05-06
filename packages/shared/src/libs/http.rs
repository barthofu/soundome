use crate::{errors::Error, types::SoundomeResult};
use config::{
    models::{ProxyConfig, ProxyStrategy},
    Config,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;

// =======================================================================
// Proxy Rotator
// =======================================================================

pub static GLOBAL_PROXY_ROTATOR: OnceLock<ProxyRotator> = OnceLock::new();

pub struct ProxyRotator {
    counter: AtomicUsize,
    config: Option<ProxyConfig>,
}

impl ProxyRotator {
    // Static methods

    pub fn init() -> SoundomeResult<()> {
        let rotator = ProxyRotator::new(Config::get().proxy.clone());
        GLOBAL_PROXY_ROTATOR
            .set(rotator)
            .map_err(|_| Error::Config("Failed to set global proxy rotator".into()))
    }

    pub fn new(config: Option<ProxyConfig>) -> Self {
        Self {
            counter: AtomicUsize::new(0),
            config,
        }
    }

    pub fn get() -> &'static Self {
        GLOBAL_PROXY_ROTATOR
            .get()
            .expect("ProxyRotator is not initialized")
    }

    // Instance methods

    /// Selects the next proxy according to the configured strategy
    pub fn get_next_proxy(&self) -> Option<String> {
        if let Some(config) = &self.config {
            if !config.enabled || config.urls.is_empty() {
                return None;
            }

            let url = match config.strategy {
                Some(ProxyStrategy::RoundRobin) | None => {
                    let index = self.counter.fetch_add(1, Ordering::Relaxed) % config.urls.len();
                    config.urls[index].clone()
                }
                Some(ProxyStrategy::Random) => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    // Use timestamp as seed for randomization
                    let mut hasher = DefaultHasher::new();
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                        .hash(&mut hasher);
                    let hash = hasher.finish();
                    let index = (hash as usize) % config.urls.len();
                    config.urls[index].clone()
                }
                Some(ProxyStrategy::StickyPerHour) => {
                    // Change proxy every hour
                    let hours_since_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        / 3600;
                    let index = (hours_since_epoch as usize) % config.urls.len();
                    config.urls[index].clone()
                }
                Some(ProxyStrategy::FirstAvailable) => {
                    // Always use the first proxy (for tests or simplicity)
                    config.urls[0].clone()
                }
            };

            tracing::debug!(
                "Selected proxy URL: {} (strategy: {:?})",
                url,
                config.strategy
            );
            Some(url)
        } else {
            None
        }
    }

    /// Checks if a domain should be excluded from proxy
    pub fn should_use_proxy(&self, domain: &str) -> bool {
        if let Some(config) = &self.config {
            if !config.enabled {
                return false;
            }

            if let Some(no_proxy_list) = &config.no_proxy {
                for no_proxy_domain in no_proxy_list {
                    if domain.contains(no_proxy_domain) {
                        tracing::debug!(
                            "Domain {} excluded from proxy due to no_proxy rule: {}",
                            domain,
                            no_proxy_domain
                        );
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

// =======================================================================
// HTTP Client Builder with Proxy Support
// =======================================================================

pub struct HttpClientBuilder;

impl HttpClientBuilder {
    /// Creates a reqwest client with optional proxy configuration and rotation
    pub fn get_reqwest_client_builder() -> SoundomeResult<reqwest::ClientBuilder> {
        let proxy_config = Config::get().proxy.as_ref();

        let proxy_url = proxy_config
            .filter(|cfg| cfg.enabled && !cfg.urls.is_empty())
            .and_then(|_| ProxyRotator::get().get_next_proxy());

        let mut client_builder = reqwest::Client::builder();

        if let Some(url) = proxy_url {
            let parsed_url = Self::parse_proxy_url(&url);
            let reqwest_proxy = Self::create_reqwest_proxy(&parsed_url)?;
            client_builder = client_builder.proxy(reqwest_proxy);

            if let Some(cfg) = proxy_config {
                if cfg.no_proxy.is_some() {
                    tracing::warn!("no_proxy configuration is not fully supported by reqwest. Use NO_PROXY environment variable or check domains manually.");
                }
            }
        }

        Ok(client_builder)
    }

    pub fn get_reqwest_client() -> SoundomeResult<reqwest::Client> {
        Self::get_reqwest_client_builder()?
            .build()
            .map_err(|e| Error::Network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Creates a reqwest client with a specific proxy
    pub fn get_reqwest_client_with_specific_proxy(
        proxy_url: Option<&str>,
    ) -> SoundomeResult<reqwest::Client> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(url) = proxy_url {
            let parsed_url = Self::parse_proxy_url(url);
            let reqwest_proxy = Self::create_reqwest_proxy(&parsed_url)?;

            client_builder = client_builder.proxy(reqwest_proxy);
        }

        client_builder
            .build()
            .map_err(|e| Error::Network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Creates a reqwest proxy from a URL string, supporting HTTP/HTTPS and SOCKS5
    fn create_reqwest_proxy(proxy_url: &str) -> SoundomeResult<reqwest::Proxy> {
        if proxy_url.starts_with("socks5://") {
            // For SOCKS5, we need to use a different approach
            // reqwest supports SOCKS5 through the "socks" feature
            reqwest::Proxy::all(proxy_url).map_err(|e| {
                Error::Config(format!("Invalid SOCKS5 proxy URL '{}': {}", proxy_url, e))
            })
        } else {
            // For HTTP/HTTPS proxies
            reqwest::Proxy::all(proxy_url).map_err(|e| {
                Error::Config(format!("Invalid HTTP proxy URL '{}': {}", proxy_url, e))
            })
        }
    }

    /// Checks if a domain should be excluded from proxy (static function)
    pub fn should_use_proxy(domain: &str) -> bool {
        ProxyRotator::get().should_use_proxy(domain)
    }

    /// Parses a proxy URL and converts colon-separated format to standard URL
    /// Supports multiple formats:
    /// - Standard URL: "http://user:pass@proxy.com:8080", "socks5://user:pass@proxy.com:1080"
    /// - Colon-separated: "proxy.com:8080:user:pass" or "127.0.0.1:8080:user:pass"
    /// - Protocol-prefixed colon format: "socks5:proxy.com:1080:user:pass"
    fn parse_proxy_url(input: &str) -> String {
        // Check if it's already a valid URL format
        if input.starts_with("http://")
            || input.starts_with("https://")
            || input.starts_with("socks5://")
        {
            return input.to_string();
        }

        // Check for protocol-prefixed colon format: "socks5:host:port:user:pass"
        if let Some(without_protocol) = input.strip_prefix("socks5:") {
            let parts: Vec<&str> = without_protocol.split(':').collect();

            if parts.len() == 4 {
                // Format: socks5:IP:PORT:USERNAME:PASSWORD
                let host = parts[0];
                let port = parts[1];
                let username = parts[2];
                let password = parts[3];

                return format!("socks5://{}:{}@{}:{}", username, password, host, port);
            } else if parts.len() == 2 {
                // Format: socks5:IP:PORT (no auth)
                let host = parts[0];
                let port = parts[1];

                return format!("socks5://{}:{}", host, port);
            }
        }

        // Split by colons to check if it's the colon-separated format
        let parts: Vec<&str> = input.split(':').collect();

        if parts.len() == 4 {
            // Format: IP:PORT:USERNAME:PASSWORD (default to HTTP)
            let host = parts[0];
            let port = parts[1];
            let username = parts[2];
            let password = parts[3];

            format!("http://{}:{}@{}:{}", username, password, host, port)
        } else if parts.len() == 2 {
            // Format: IP:PORT (no authentication, default to HTTP)
            let host = parts[0];
            let port = parts[1];

            format!("http://{}:{}", host, port)
        } else {
            // If format is not recognized, return as-is and let reqwest handle the error
            input.to_string()
        }
    }
}
