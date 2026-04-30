# Configuration

Soundome reads its runtime configuration from `config.toml` and supports environment-based overrides through the `config` crate. Global initialization happens through `shared::init_globals()`.

A fully annotated starting point is available at [`config.example.toml`](../../config.example.toml).

## Main configuration files

| File | Purpose |
|---|---|
| `config.toml` | Application settings: library paths, providers, logs, AI, and proxy. |
| `Rocket.toml` | Rocket server address, port, and database pool. |
| `.env` | Required by the server and CLI boot paths via `dotenvy`. |

## Environment variable overrides

Every key in `config.toml` can be overridden at runtime with an environment variable. The convention is:

```
SOUNDOME__<SECTION>__<KEY>
```

- Prefix: `SOUNDOME`
- Separator: `__` (double underscore)
- Case: uppercase

The path to the config file itself is controlled by a separate variable with a single underscore:

```
SOUNDOME_CONFIG_PATH=/path/to/config.toml
```

---

## `[general]`

Core filesystem paths.

| Key | Type | Default | Description |
|---|---|---|---|
| `base_library_dir` | string | `"./library"` | Root directory for the organized music library. Tracks are placed under `<Artist>/<Album>/<Track>`. |
| `temp_download_dir` | string | `"./temp"` | Staging directory for files being downloaded or processed. |

| Key | Environment variable |
|---|---|
| `general.base_library_dir` | `SOUNDOME__GENERAL__BASE_LIBRARY_DIR` |
| `general.temp_download_dir` | `SOUNDOME__GENERAL__TEMP_DOWNLOAD_DIR` |

---

## `[logs]`

Logging and tracing output.

| Key | Type | Default | Description |
|---|---|---|---|
| `level` | string | `"info"` | Minimum tracing level. One of `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"`. |
| `enable_reqwest_logging` | bool | `false` | Enables verbose HTTP request/response logging. Requires `level = "debug"` or lower. |

| Key | Environment variable |
|---|---|
| `logs.level` | `SOUNDOME__LOGS__LEVEL` |
| `logs.enable_reqwest_logging` | `SOUNDOME__LOGS__ENABLE_REQWEST_LOGGING` |

---

## `[database]`

SQLite connection used by `packages/database`.

> Rocket also declares its own database location in `Rocket.toml`. Keep both paths aligned.

| Key | Type | Default | Description |
|---|---|---|---|
| `url` | string | — | SQLite connection URL (e.g. `"./data/soundome.db"`). |
| `pool_size` | integer | — | Optional connection pool size. Omit to use the built-in default. |

| Key | Environment variable |
|---|---|
| `database.url` | `SOUNDOME__DATABASE__URL` |
| `database.pool_size` | `SOUNDOME__DATABASE__POOL_SIZE` |

---

## `[providers]`

Credentials and settings for external provider integrations.

### `[providers.spotify]`

Required. Used for metadata fetching and source resolution. Obtain credentials at <https://developer.spotify.com/dashboard>.

| Key | Type | Description |
|---|---|---|
| `client_id` | string | Spotify OAuth application client ID. |
| `client_secret` | string | Spotify OAuth application client secret. |

| Key | Environment variable |
|---|---|
| `providers.spotify.client_id` | `SOUNDOME__PROVIDERS__SPOTIFY__CLIENT_ID` |
| `providers.spotify.client_secret` | `SOUNDOME__PROVIDERS__SPOTIFY__CLIENT_SECRET` |

### `[providers.youtube]` (optional)

When this section is omitted, the default YouTube / YouTube Music integration is used directly.

| Key | Type | Description |
|---|---|---|
| `invidious_instance` | string | Base URL of an Invidious instance to use instead of direct YouTube access. |

| Key | Environment variable |
|---|---|
| `providers.youtube.invidious_instance` | `SOUNDOME__PROVIDERS__YOUTUBE__INVIDIOUS_INSTANCE` |

---

## `[ai]`

AI-assisted metadata cleanup via OpenRouter (`packages/ai`). Set `enabled = false` to skip all AI enrichment steps without removing the section.

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `false` | Master switch for AI-powered metadata enrichment. |

| Key | Environment variable |
|---|---|
| `ai.enabled` | `SOUNDOME__AI__ENABLED` |

### `[ai.openrouter]` (required when `ai.enabled = true`)

Obtain an API key at <https://openrouter.ai>.

| Key | Type | Default | Description |
|---|---|---|---|
| `api_key` | string | — | OpenRouter API key. |
| `model` | string | app default | Model identifier, e.g. `"openai/gpt-4o-mini"`. |
| `provider` | string | — | Preferred inference provider within OpenRouter, e.g. `"openai"`. |
| `base_url` | string | OpenRouter default | Override the API base URL. Useful for local proxies or testing. |
| `timeout` | integer | — | HTTP request timeout in seconds. |

| Key | Environment variable |
|---|---|
| `ai.openrouter.api_key` | `SOUNDOME__AI__OPENROUTER__API_KEY` |
| `ai.openrouter.model` | `SOUNDOME__AI__OPENROUTER__MODEL` |
| `ai.openrouter.provider` | `SOUNDOME__AI__OPENROUTER__PROVIDER` |
| `ai.openrouter.base_url` | `SOUNDOME__AI__OPENROUTER__BASE_URL` |
| `ai.openrouter.timeout` | `SOUNDOME__AI__OPENROUTER__TIMEOUT` |

---

## `[tagger]`

Controls which metadata providers are used when enriching and tagging audio files.

| Key | Type | Default | Description |
|---|---|---|---|
| `metadata_providers` | string array | `["musicbrainz"]` | Ordered list of metadata providers. Tried in order; first successful result is used. Supported values: `"musicbrainz"`, `"bandcamp"`, `"spotify"`. |

| Key | Environment variable |
|---|---|
| `tagger.metadata_providers` | `SOUNDOME__TAGGER__METADATA_PROVIDERS` |

---

## `[proxy]` (optional)

Outbound HTTP proxy configuration shared across the application. Omit this section entirely to disable proxy support. See [../operations/proxy-configuration.md](../operations/proxy-configuration.md) for full details.

Proxy URLs support HTTP, HTTPS, and SOCKS5. Credentials can be embedded directly or provided in colon-separated form (`host:port:user:pass`).

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | — | Enable or disable the proxy without removing the section. |
| `urls` | string array | — | List of proxy URLs. When multiple are given, the rotation strategy applies. |
| `strategy` | string | `"round_robin"` | Rotation strategy. One of `"round_robin"`, `"random"`, `"sticky_per_hour"`, `"first_available"`. |
| `no_proxy` | string array | — | Hosts and domain patterns that bypass the proxy (e.g. `["localhost", "*.local"]`). |

| Key | Environment variable |
|---|---|
| `proxy.enabled` | `SOUNDOME__PROXY__ENABLED` |
| `proxy.urls` | `SOUNDOME__PROXY__URLS` |
| `proxy.strategy` | `SOUNDOME__PROXY__STRATEGY` |
| `proxy.no_proxy` | `SOUNDOME__PROXY__NO_PROXY` |

---

## Practical guidance

- Copy `config.example.toml` to `config.toml` and fill in your values before first run.
- Keep `base_library_dir`, `temp_download_dir`, and the Rocket database path local to your machine.
- Do not commit secrets such as Spotify credentials or OpenRouter API keys.
- Prefer environment variable overrides (e.g. in `.env`) for secrets in containerized or CI environments.
- If proxy behavior looks inconsistent, verify both `config.toml` and the environment variables visible to the process.
