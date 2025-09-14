# Proxy Configuration in Soundome

## Overview

Soundome now supports advanced proxy configuration with automatic rotation for external API requests. This feature is useful in enterprise environments or when you need to bypass geographical restrictions.

## Configuration

Soundome supports multiple proxy URL formats to accommodate different proxy providers and use cases.

### Supported proxy URL formats

**1. Standard URL format:**
- `http://proxy.example.com:8080` (no authentication)
- `http://user:password@proxy.example.com:8080` (with authentication)
- `https://user:password@secure-proxy.example.com:443` (HTTPS proxy)

**2. Colon-separated format:**
- `IP:PORT` (no authentication): `192.168.1.100:8080`
- `IP:PORT:USERNAME:PASSWORD` (with authentication): `64.137.96.74:6641:eipncmhd:qoawnl661cmj`
- `HOSTNAME:PORT:USERNAME:PASSWORD`: `proxy.example.com:8080:user:pass`

The colon-separated format is commonly used by proxy providers and is automatically converted to the standard URL format internally.

### Basic configuration with a single proxy

```toml
[proxy]
enabled = true
urls = ["http://proxy.example.com:8080"]
strategy = "first_available"
```

### Configuration with multiple proxies and rotation

```toml
[proxy]
enabled = true
urls = [
    "http://proxy1.example.com:8080",
    "http://proxy2.example.com:8080", 
    "http://proxy3.example.com:8080"
]
strategy = "round_robin"
```

### Configuration with embedded authentication in URL

Soundome supports multiple proxy URL formats:

**Standard URL format:**
```toml
[proxy]
enabled = true
urls = [
    "http://user1:pass1@proxy1.example.com:8080",
    "http://user2:pass2@proxy2.example.com:8080"
]
strategy = "random"
```

**Colon-separated format (IP:PORT:USERNAME:PASSWORD):**
```toml
[proxy]
enabled = true
urls = [
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "192.168.1.100:8080:user2:password2"
]
strategy = "random"
```

**Mixed formats:**
```toml
[proxy]
enabled = true
urls = [
    "http://user1:pass1@proxy1.example.com:8080",
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "proxy.example.com:3128"  # No authentication
]
strategy = "round_robin"
```

### Domain exclusion

You can exclude certain domains from the proxy:

```toml
[proxy]
enabled = true
urls = ["http://proxy.example.com:8080"]
strategy = "first_available"
no_proxy = ["localhost", "127.0.0.1", "*.local", "internal.company.com"]
```

## Rotation strategies

### `round_robin`
Uses proxies in list order, then starts again from the beginning.

```toml
strategy = "round_robin"
```

### `random`
Randomly selects a proxy for each request.

```toml
strategy = "random"
```

### `sticky_per_hour`
Changes proxy every hour, based on system timestamp.

```toml
strategy = "sticky_per_hour"
```

### `first_available`
Always uses the first proxy in the list (simple behavior).

```toml
strategy = "first_available"
```

## Supported components

### Current support

- ✅ **Centralized configuration**: Unified configuration structure with multi-proxy support
- ✅ **Automatic rotation**: Multiple proxy rotation strategies
- ✅ **Embedded authentication**: Support for credentials in proxy URL
- ✅ **HTTP client utility**: Shared module for creating reqwest clients
- ✅ **Documentation and logging**: Appropriate warnings for limitations
- ✅ **Unit tests**: Coverage of rotation and configuration features

### Current limitations

Third-party libraries used by Soundome do not yet natively support proxy configuration:

- ⚠️ **Spotify (rspotify)**: No native proxy support
- ⚠️ **SoundCloud (rsoundcloud)**: No native proxy support  
- ⚠️ **YouTube Music (rustypipe)**: No native proxy support
- ⚠️ **MusicBrainz (musicbrainz_rs)**: No native proxy support
- ⚠️ **YouTube (invidious)**: No native proxy support

### Workarounds

While waiting for libraries to natively support proxies, you can use:

1. **System environment variables**:
   ```bash
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   export NO_PROXY=localhost,127.0.0.1
   ```

2. **System/network level proxy configuration**

3. **Transparent proxy** at infrastructure level

## Usage in code

### Create an HTTP client with proxy

```rust
use shared::utils::http_client::HttpClientBuilder;
use config::model::{ProxyConfig, ProxyStrategy};

// Configuration with multiple proxies and rotation
let proxy_config = ProxyConfig {
    enabled: true,
    urls: vec![
        "http://user:pass@proxy1:8080".to_string(),
        "http://proxy2:8080".to_string(),
    ],
    strategy: ProxyStrategy::RoundRobin,
    no_proxy: Some(vec!["localhost".to_string()]),
};

let client = HttpClientBuilder::get_reqwest_client(Some(&proxy_config))?;

// Without proxy
let client = HttpClientBuilder::get_reqwest_client(None)?;
```

### Advanced proxy rotation

```rust
use shared::utils::http_client::{HttpClientBuilder, ProxyRotator};

let rotator = HttpClientBuilder::create_proxy_rotator(&proxy_config);

// Get the next proxy according to strategy
if let Some(proxy_url) = rotator.get_next_proxy() {
    let client = HttpClientBuilder::get_reqwest_client_with_specific_proxy(Some(&proxy_url))?;
    // Use the client...
}
```

### Check if a proxy should be used

```rust
let should_proxy = HttpClientBuilder::should_use_proxy(
    "api.spotify.com", 
    Some(&proxy_config)
);

if should_proxy {
    let rotator = HttpClientBuilder::create_proxy_rotator(&proxy_config);
    if let Some(proxy_url) = rotator.get_next_proxy() {
        // Use the proxy...
    }
}
```

## Configuration examples

### Enterprise configuration with high availability

```toml
[proxy]
enabled = true
urls = [
    "http://proxy-primary.corp.com:8080",
    "http://proxy-backup.corp.com:8080",
    "http://proxy-dr.corp.com:8080"
]
strategy = "round_robin"
no_proxy = ["localhost", "*.corp.com", "10.*", "192.168.*"]
```

### Configuration with per-proxy authentication

```toml
[proxy]
enabled = true
urls = [
    "http://user1:secret@proxy1.example.com:8080",
    "http://user2:secret@proxy2.example.com:8080"
]
strategy = "sticky_per_hour"
```

### Configuration with colon-separated proxy format

```toml
[proxy]
enabled = true
urls = [
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "192.168.1.100:8080:user2:password2",
    "10.0.0.50:3128:admin:secret123"
]
strategy = "random"
```

### Mixed proxy formats configuration

```toml
[proxy]
enabled = true
urls = [
    "http://user1:pass1@proxy1.example.com:8080",    # Standard URL format
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",       # Colon-separated format
    "proxy.example.com:3128",                        # No authentication
    "https://secure-proxy.example.com:443"          # HTTPS proxy
]
strategy = "round_robin"
```

### Development configuration

```toml
[proxy]
enabled = true
urls = ["127.0.0.1:8888:dev:password"]
strategy = "first_available"
no_proxy = ["localhost", "127.0.0.1"]
```

## Roadmap

### Future improvements

1. **Native library support**: Contribute to third-party libraries to add proxy support
2. **Unified HTTP client**: Replace embedded clients with custom reqwest clients
3. **Automated testing**: Integration tests with mock proxy servers
4. **Proxy metrics**: Logging and metrics for proxy requests

### Migration to full support

```rust
// Future vision: unified API with full proxy support
pub trait APIClient {
    async fn with_proxy(config: &ProxyConfig) -> Self;
    async fn get(&self, url: &str) -> Result<Response, Error>;
    async fn post(&self, url: &str, body: Body) -> Result<Response, Error>;
}
```

## Troubleshooting

### Common issues

1. **Proxy not used**: Check that `enabled = true` and the URL is correct
2. **Authentication failed**: Check username/password credentials
3. **Excluded domains**: Check the `no_proxy` list

### Debug logging

Enable `DEBUG` level logs to see proxy connection attempts:

```toml
[general]
log_level = "debug"
```

### Fallback environment variables

If the application's proxy configuration doesn't work, use:

```bash
export HTTP_PROXY=http://proxy:8080
export HTTPS_PROXY=http://proxy:8080
export NO_PROXY=localhost,127.0.0.1,.local
```

## Security

### Best practices

1. **Multiple URL formats**: Use either standard URL format (`http://user:pass@proxy:port`) or colon-separated format (`IP:PORT:USER:PASS`) based on your proxy provider's requirements
2. **Environment variables**: Use environment variables for sensitive secrets in both formats
3. **HTTPS**: Prefer HTTPS proxies when possible
4. **Credential rotation**: Regularly change proxy passwords
5. **Proxy rotation**: Use multiple proxies to distribute load and improve availability

### Proxy URL format examples

**Standard URL format:**
```toml
urls = [
    "http://username:password@proxy.example.com:8080",
    "https://user:pass@secure-proxy.com:443"
]
```

**Colon-separated format:**
```toml
urls = [
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "192.168.1.100:8080:user:password"
]
```

**Mixed formats:**
```toml
urls = [
    "http://user:pass@proxy1.example.com:8080",
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "proxy.example.com:3128"  # No authentication
]
```

### Example with environment variables

```toml
[proxy]
enabled = true
urls = [
    "http://${PROXY_USER1}:${PROXY_PASS1}@proxy1.example.com:8080",
    "http://${PROXY_USER2}:${PROXY_PASS2}@proxy2.example.com:8080"
]
strategy = "round_robin"
```

### Secure configuration for production

```toml
[proxy]
enabled = true
urls = [
    "https://primary-proxy.secure.com:8443",
    "https://backup-proxy.secure.com:8443"
]
strategy = "sticky_per_hour"
no_proxy = ["localhost", "*.internal", "10.*", "192.168.*"]
```
