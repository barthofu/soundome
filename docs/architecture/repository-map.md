# Repository map

This document is a code-oriented map of the repository. Use it when you need to locate the owning crate or the narrowest entry point for a change.

## Applications

### `apps/server`

- Rocket API entry point
- OpenAPI and Swagger wiring
- static file serving for `data/web/`
- route handlers that delegate to the domain layer

Primary entry point: `apps/server/src/main.rs`

### `apps/cli`

- CLI entry point
- currently minimal and not yet the main workflow driver

### `apps/web`

- Svelte admin interface
- Vite development server and production build pipeline

## Packages

### `packages/domain`

- service layer and orchestration
- owns `DownloadService`
- depends on repository traits rather than concrete Diesel implementations

### `packages/database`

- Diesel repositories and migrations
- persistence semantics such as reference replacement and merge behavior

### `packages/fetcher`

- source adapters for platforms such as Spotify, SoundCloud, and YouTube Music
- source metadata cleaning helpers

### `packages/downloader`

- audio providers and download logic
- returns file paths and provider references for the domain layer

### `packages/tagger`

- metadata enrichment providers
- audio tagging logic

### `packages/organizer`

- filesystem placement and final library layout

### `packages/config`

- typed config models
- layered config loading and environment overrides

### `packages/shared`

- shared models
- typed errors and result aliases
- logging utilities
- proxy-aware HTTP helpers

### `packages/ai`

- OpenRouter client
- prompt helpers used mainly for metadata cleanup

## Important code paths

- `apps/server/src/main.rs`: repository and service wiring
- `apps/server/src/routes/`: HTTP layer
- `packages/domain/src/services/download_service.rs`: main business workflow
- `packages/shared/src/models/`: track, album, artist, playlist, and reference models
- `packages/shared/src/libs/http.rs`: proxy rotation and proxy-aware HTTP client builder
- `packages/database/src/repositories/`: persistence implementations

## Change-routing heuristics

- If the behavior is business logic, start in `packages/domain`.
- If the change is about source parsing or source metadata, start in `packages/fetcher`.
- If the change is about downloaded audio or provider lookup, start in `packages/downloader`.
- If the change is about tagging or metadata providers, start in `packages/tagger`.
- If the change is about DB semantics, start in `packages/database`.
- If the change is about HTTP exposure, start in `apps/server`.
