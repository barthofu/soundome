# Soundome - global Copilot instructions

This repository is a Rust monorepo for an application that centralizes, downloads, enriches, tags, and organizes a personal music library from sources such as Spotify, SoundCloud, YouTube, and YouTube Music.

Your goal is to make small, safe, architecture-aligned changes while keeping in mind that the project is still work in progress and some surfaces are intentionally partial.

## Project goals

- Build a unified music library from heterogeneous sources.
- Minimize manual library management.
- Keep organization and audio quality consistent over time.
- Enrich track metadata automatically when confidence is high enough.
- Expose a manual validation UI for ambiguous cases.

## What currently works in practice

- Inputs: track URLs, playlist URLs, and some groundwork for future local ingest.
- Typical flow:
  - fetch source metadata such as title, artists, album, and references
  - clean noisy metadata, especially for SoundCloud-like sources
  - enrich metadata through MusicBrainz
  - download audio from a provider, often YouTube or YouTube Music even when the source is Spotify
  - deduplicate by URL, then similarity, then quality
  - tag the audio file
  - move the file into the `Artist/Album/Track` library layout and persist the result

Useful workflow references:
- `docs/README.md`
- `docs/workflows/download.md`
- `docs/architecture/design.md`

## Technical architecture

Cargo workspace overview:
- `apps/server`: Rocket API, OpenAPI, static file serving
- `apps/cli`: CLI entry point, still minimal
- `packages/domain`: services and orchestration, especially `DownloadService`
- `packages/fetcher`: source adapters
- `packages/downloader`: audio providers
- `packages/tagger`: metadata providers and file tagging
- `packages/organizer`: filesystem placement
- `packages/database`: Diesel repositories backed by SQLite
- `packages/config`: TOML config with environment overrides and singleton access
- `packages/shared`: models, typed errors, HTTP helpers, logging, utilities
- `packages/ai`: OpenRouter client and prompt helpers for metadata cleanup

Useful code entry points:
- `apps/server/src/main.rs`: bootstraps Rocket, globals, repositories, and services
- `packages/domain/src/services/download_service.rs`: main download and playlist workflow
- `packages/shared/src/models/*`: core models and reference merge logic
- `packages/shared/src/libs/http.rs`: `ProxyRotator` and `HttpClientBuilder`
- `packages/database/src/repositories/*`: repository semantics such as `set_references`

## References and data model

- `shared::models::Reference` uses `ReferenceType`: `Source`, `Provider`, `Metadata`, and `Reference`.
- Important rule:
  - `Source` and `Provider` describe the actual audio path in use and are often replaced
  - `Metadata` keeps durable identifiers and URLs such as MusicBrainz or Spotify and is often merged
- Repository code already encodes this behavior, especially around `set_references`.

## Config and runtime

- `packages/config` loads `config.toml` by default and supports `SOUNDOME_CONFIG_PATH` and `SOUNDOME__...` overrides.
- Global initialization should happen through `shared::init_globals()`.
- Server and CLI boot paths use `dotenvy` with `required = true`, so a local `.env` file is expected.
- Rocket database configuration lives in `Rocket.toml`.
- Diesel uses `diesel.toml`, `packages/database/migrations`, and `packages/database/src/schema.rs`.

## Networking and proxy rules

- Proxy support is configured through `Config.proxy` and the global `ProxyRotator`.
- Prefer `shared::libs::http::HttpClientBuilder` over ad hoc `reqwest::Client` construction.
- See `docs/operations/proxy-configuration.md` and `docs/operations/proxy-usage-example.md`.
- Some third-party libraries still cannot honor the shared proxy configuration directly.

## WIP expectations

- Several features remain incomplete, including parts of the CLI and some provider integrations.
- Do not invent behavior that is not already implied by the codebase.
- When a surface is missing, prefer a minimal scaffold that fits the current architecture and makes the limitation explicit.
- Favor small, testable changes over broad rewrites.

## Code conventions

- Use English for code, comments, and documentation.
- Use `shared::types::SoundomeResult<T>` and `shared::errors::Error` for errors.
- Use `tracing` for logs.
- Avoid `unwrap()` and `expect()` outside boot or initialization code.
- Reuse the `ports/repositories` and `database/repositories` split.

## Useful development commands

- Workspace: `cargo test -q`, `cargo clippy --workspace --all-targets`, `cargo fmt`
- SQLite migrations: `cargo install diesel_cli --no-default-features --features sqlite`, then `diesel migration run`
