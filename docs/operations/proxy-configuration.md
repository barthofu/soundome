# Proxy configuration

## Overview

Soundome exposes a shared proxy layer for HTTP clients built through `shared::libs::http::HttpClientBuilder`. It is useful when running behind corporate proxies, routing traffic through residential proxies, or working around geo-restrictions.

The proxy rotator is initialized globally by `shared::init_globals()`, which is already called by the server boot path.

## Configuration model

Proxy configuration lives under the `proxy` section of `config.toml`.

```toml
[proxy]
enabled = true
urls = ["http://proxy.example.com:8080"]
strategy = "first_available"
no_proxy = ["localhost", "127.0.0.1"]
```

## Supported proxy URL formats

The shared HTTP layer accepts both standard URLs and compact colon-separated formats.

### Standard URL format

- `http://proxy.example.com:8080`
- `http://user:password@proxy.example.com:8080`
- `https://user:password@secure-proxy.example.com:443`
- `socks5://user:password@proxy.example.com:1080`

### Colon-separated format

- `IP:PORT`
- `IP:PORT:USERNAME:PASSWORD`
- `HOSTNAME:PORT:USERNAME:PASSWORD`
- `socks5:HOSTNAME:PORT`
- `socks5:HOSTNAME:PORT:USERNAME:PASSWORD`

These values are normalized internally before being passed to `reqwest`.

## Rotation strategies

- `round_robin`: iterate over the configured URLs in order.
- `random`: pick a proxy based on a timestamp-derived hash.
- `sticky_per_hour`: keep the same proxy for one hour, then rotate.
- `first_available`: always use the first configured proxy.

## Examples

### Single proxy

```toml
[proxy]
enabled = true
urls = ["http://proxy.example.com:8080"]
strategy = "first_available"
```

### Rotating authenticated proxies

```toml
[proxy]
enabled = true
urls = [
    "http://user1:secret@proxy1.example.com:8080",
    "http://user2:secret@proxy2.example.com:8080"
]
strategy = "round_robin"
```

### Mixed proxy formats

```toml
[proxy]
enabled = true
urls = [
    "http://user1:pass1@proxy1.example.com:8080",
    "64.137.96.74:6641:eipncmhd:qoawnl661cmj",
    "socks5:proxy.internal:1080"
]
strategy = "random"
no_proxy = ["localhost", "127.0.0.1", "internal.company.com"]
```

## Current limitations

Proxy support is only guaranteed when code uses the shared HTTP builder. Several third-party libraries used by Soundome still manage their own network stack and therefore may ignore the repository-level proxy configuration.

Known examples include adapters for Spotify, SoundCloud, YouTube Music, MusicBrainz, and some provider-specific clients.

## Operational notes

- `no_proxy` is evaluated by `ProxyRotator::should_use_proxy`, but `reqwest` itself does not fully honor the repository config directly. Use `NO_PROXY` environment variables as a fallback when needed.
- Avoid logging full proxy URLs because they may contain credentials.
- If an integration cannot use the shared HTTP layer yet, document the limitation explicitly instead of assuming proxy support.

## Related documentation

- [proxy-usage-example.md](proxy-usage-example.md)
- [../../.github/skills/proxy-aware-http/SKILL.md](../../.github/skills/proxy-aware-http/SKILL.md)
