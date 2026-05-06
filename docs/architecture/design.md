# Design

This document summarizes the current technical design of Soundome based on the repository layout and the code paths that are already wired.

## System overview

Soundome is a Cargo workspace centered on a domain orchestration layer. The server boot path wires repositories into services, exposes HTTP routes through Rocket, and serves the built web application from `data/web/`.

## Main runtime entry points

- `apps/server/src/main.rs`: initializes globals, repositories, services, Rocket routes, Swagger, and static file serving.
- `packages/domain/src/services/download_service.rs`: main download and playlist orchestration workflow.
- `packages/shared/src/lib.rs`: initializes shared global state, including config and proxy rotation.

## Download workflow ownership

The current end-to-end workflow is owned by `DownloadService`.

1. Reject the request early if the exact source URL already exists in the database.
2. Fetch source metadata and normalize it.
3. Enrich metadata and reuse existing album or artist entities when references already exist.
4. Download audio into the staging directory.
5. If enrichment is partial, persist the staged track as `needs_validation = true` and stop before tagging or organizing.
6. Otherwise deduplicate against existing tracks and compare quality.
7. Keep the better audio version while merging useful metadata references.
8. Tag the chosen file, move it into the library, and persist the final entity graph.

## Reference model

The `ReferenceType` split is one of the core design decisions:

- `Source`: where the user asked Soundome to import from.
- `Provider`: where the actual downloadable audio came from.
- `Metadata`: durable identifiers that remain useful even if the audio source changes.
- `Reference`: generic supporting reference data.

This distinction matters during deduplication. When a better audio source replaces an existing track, Soundome should replace `Source` and `Provider` as needed while preserving useful metadata identifiers.

## Server and web application

The Rocket server currently mounts API routes under `/api`, Swagger under `/swagger`, and serves the built SPA at `/`.

A Prometheus-compatible metrics endpoint is available at `GET /metrics`. It is not registered with OpenAPI and is intended for scraping only. It returns library statistics in the standard Prometheus text format (`text/plain; version=0.0.4`) via `apps/server/src/routes/metrics.rs`.

The web app covers:

- track or playlist submission
- recent download browsing
- approval or rejection of pending validations
- background task monitoring for playlist sync

## Playlist M3U8 export

After each playlist sync, `PlaylistService::export_m3u8` writes one `.m3u8` file per playlist to a configurable output directory (default: `{base_library_dir}/Playlists/`). This makes playlists visible to external music players such as Navidrome or Jellyfin without requiring Soundome to be running.

The export can also be triggered on demand via `POST /api/playlists/:id/export`.

See [../operations/playlist-m3u8-export.md](../operations/playlist-m3u8-export.md) for full operational details.

## Configuration and globals

`shared::init_globals()` initializes:

- the layered configuration system from `packages/config`
- the shared `ProxyRotator`

The server and CLI boot paths expect a local `.env` file. Runtime paths and credentials come from `config.toml` and environment overrides.

## Known gaps

- The CLI is still minimal.
- Some integrations remain partial or rely on third-party libraries with limited proxy support.
- Older design ideas such as richer ingest workflows or advanced duplicate detection are not yet fully implemented.
