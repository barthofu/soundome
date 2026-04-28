# Configuration

Soundome reads its runtime configuration from `config.toml` and supports environment-based overrides through the config crate. Global initialization happens through `shared::init_globals()`.

## Main configuration files

- `config.toml`: application settings for library paths, providers, logs, AI, and proxy.
- `Rocket.toml`: Rocket server address, port, and database settings.
- `.env`: required by the server and CLI boot paths through `dotenvy`.

## `config.toml` sections

### `general`

- `base_library_dir`: target library root for finalized files.
- `temp_download_dir`: staging directory for downloaded files before finalization.

### `logs`

- `level`: tracing level such as `info` or `debug`.
- `enable_reqwest_logging`: enables more verbose request logging when needed.

### `database`

- `url`: SQLite database path used by the config crate.
- `pool_size`: optional pool size for code paths that use this config section.

Rocket also declares its own database location in `Rocket.toml`, so keep both aligned when changing paths.

### `providers`

- `spotify.client_id`
- `spotify.client_secret`
- `youtube.invidious_instance`

Some integrations are partial, so provider configuration may exist before the corresponding workflow is fully exposed.

### `ai`

- `enabled`
- `openrouter.api_key`
- `openrouter.model`
- `openrouter.provider`
- `openrouter.base_url`
- `openrouter.timeout`

This is mainly used by the `packages/ai` crate for metadata cleanup helpers.

### `tagger`

- `metadata_providers`: enabled metadata providers in priority order.

The current config model supports values such as `musicbrainz` and `bandcamp`.

### `proxy`

- `enabled`
- `urls`
- `strategy`
- `no_proxy`

See [../operations/proxy-configuration.md](../operations/proxy-configuration.md) for details.

## Environment overrides

The config crate supports `SOUNDOME_CONFIG_PATH` and `SOUNDOME__...` environment overrides. Use them when you need to switch config files or override a specific setting without editing `config.toml`.

## Practical guidance

- Keep `base_library_dir`, `temp_download_dir`, and the Rocket database path local to your machine.
- Do not commit secrets such as Spotify credentials or OpenRouter keys.
- If proxy behavior looks inconsistent, verify both `config.toml` and the environment variables visible to the process.
