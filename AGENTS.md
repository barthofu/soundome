# Soundome

Rust monorepo for a personal music library manager — centralizes, downloads, enriches, tags, and organizes music from Spotify, SoundCloud, YouTube, and YouTube Music.

## Quick commands

- `cargo test -q` — run tests quietly
- `cargo clippy --workspace --all-targets` — lint check
- `cargo fmt --all` — format
- `diesel migration run` — run SQLite migrations (requires `diesel_cli` with sqlite feature)

## Project structure

- `apps/server` — Rocket API, OpenAPI, static file serving
- `apps/cli` — CLI entry point (minimal)
- `packages/domain` — services & orchestration (especially `DownloadService`)
- `packages/fetcher` — source adapters (Spotify, YT Music, SoundCloud)
- `packages/downloader` — audio providers (YouTube, YT Music, SoundCloud)
- `packages/tagger` — metadata enrichment (MusicBrainz) & file tagging
- `packages/organizer` — filesystem placement (`Artist/Album/Track`)
- `packages/database` — Diesel repositories backed by SQLite
- `packages/config` — TOML config with env overrides
- `packages/shared` — models, typed errors, HTTP helpers, logging
- `packages/ai` — OpenRouter client & prompt helpers for metadata cleanup

## Key entry points

- `apps/server/src/main.rs` — Rocket boot, globals, repos, services
- `packages/domain/src/services/download_service.rs` — main workflow
- `packages/shared/src/models/*` — core models & reference merge logic
- `packages/shared/src/libs/http.rs` — `ProxyRotator` & `HttpClientBuilder`
- `packages/database/src/repositories/*` — repo semantics

## Import pipeline

1. Accept a track or playlist URL
2. Fetch source metadata
3. Clean noisy metadata (especially SoundCloud)
4. Enrich via MusicBrainz
5. Download to staging
6. Deduplicate by URL → similarity → quality
7. Tag, organize, persist

## Code conventions

- Use `shared::types::SoundomeResult<T>` and `shared::errors::Error` for errors
- Use `tracing::{info,warn,error,debug}` for logging
- Avoid `unwrap()`/`expect()` outside boot/init code
- Prefer `shared::libs::http::HttpClientBuilder` over ad hoc `reqwest::Client`
- Respect `ReferenceType` semantics: `Source`/`Provider` = effective audio path (often replaced), `Metadata` = durable IDs (often merged)
- Favor small, architecture-aligned changes over broad rewrites
- Preserve staged-file behavior for tracks needing manual validation

## Config & runtime

- Config: `packages/config` loads `config.toml`, supports `SOUNDOME_CONFIG_PATH` and `SOUNDOME__...` overrides
- Global init via `shared::init_globals()`
- `.env` file expected (dotenvy with `required = true`)
- Rocket DB config in `Rocket.toml`
- Diesel: `diesel.toml`, `packages/database/migrations`, `packages/database/src/schema.rs`

## Additional instructions

For crate-specific or workflow-specific guidance, the following files are referenced via `opencode.jsonc` instructions and contain detailed conventions:
- `.github/instructions/rust.instructions.md`
- `.github/instructions/project.instructions.md`
- `.github/instructions/domain.instructions.md`
- `.github/instructions/database-diesel.instructions.md`
- `.github/instructions/downloader.instructions.md`
- `.github/instructions/fetcher.instructions.md`
- `.github/instructions/organizer.instructions.md`
- `.github/instructions/server-rocket.instructions.md`
- `.github/instructions/tagger-organizer.instructions.md`
- `.github/instructions/ai.instructions.md`
- `.github/copilot-instructions.md`
