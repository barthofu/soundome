# Proxy usage examples

This document shows the code paths that are actually available in the current repository.

## Boot-time initialization

The preferred entry point is not `ProxyRotator` directly. Soundome initializes the config singleton and the global proxy rotator together through `shared::init_globals()`.

```rust
use shared::init_globals;

fn bootstrap() -> shared::types::SoundomeResult<()> {
    init_globals()?;
    Ok(())
}
```

## Building a proxy-aware `reqwest` client

Use `shared::libs::http::HttpClientBuilder` instead of constructing a raw `reqwest::Client` when you want the repository proxy rules to apply.

```rust
use shared::libs::http::HttpClientBuilder;

async fn fetch_text(url: &str) -> shared::types::SoundomeResult<String> {
    let client = HttpClientBuilder::get_reqwest_client()?;
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}
```

## Picking a proxy explicitly

If you need to inspect or override proxy selection, use `ProxyRotator` and then build a client with that specific proxy.

```rust
use shared::libs::http::{HttpClientBuilder, ProxyRotator};

fn build_client_for_domain(domain: &str) -> shared::types::SoundomeResult<reqwest::Client> {
    let proxy = if HttpClientBuilder::should_use_proxy(domain) {
        ProxyRotator::get().get_next_proxy()
    } else {
        None
    };

    HttpClientBuilder::get_reqwest_client_with_specific_proxy(proxy.as_deref())
}
```

## Important caveats

- `ProxyRotator::get()` assumes initialization already happened. Call it only after `shared::init_globals()`.
- `HttpClientBuilder::should_use_proxy(domain)` is the repository-level check for the `no_proxy` list.
- The shared builder only helps for integrations that use `reqwest` through this layer.
- Do not log full proxy URLs because they may contain credentials.
