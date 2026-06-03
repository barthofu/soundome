---
name: proxy-aware-http
description: Use when adding or fixing direct HTTP calls, reqwest clients, proxy support, no_proxy behavior, rotating proxy behavior, or network code that should respect Soundome proxy configuration.
---

# Proxy-aware HTTP

Use this skill when an HTTP path in Soundome should respect the shared proxy layer.

## When to use

- add a new `reqwest` client
- migrate ad hoc HTTP construction to the shared builder
- debug proxy rotation or bypass behavior
- document proxy limitations in a new integration

## Procedure

1. Confirm the boot path calls `shared::init_globals()`.
2. Prefer `shared::libs::http::HttpClientBuilder` over raw `reqwest::Client::builder()`.
3. If needed, check `HttpClientBuilder::should_use_proxy(domain)` before selecting a proxy.
4. Use `ProxyRotator::get().get_next_proxy()` only after initialization is guaranteed.
5. Avoid logging full proxy URLs when credentials may be embedded.
6. If a third-party client cannot use the shared builder, document the limitation explicitly.

## Validation checklist

- proxy behavior is driven by `Config.proxy`
- code still works when proxy is disabled
- bypass logic is correct for `no_proxy`
- logs do not expose credentials

## Useful files

- `packages/shared/src/libs/http.rs`
- `packages/shared/src/lib.rs`
- `packages/config/src/models.rs`
- `docs/operations/proxy-configuration.md`
- `docs/operations/proxy-usage-example.md`
