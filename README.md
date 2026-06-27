<p align="center">
    <img height="450" src="./assets/images/soundome_logo.png"></img>
</p>

# What is Soundome?

Soundome is a self-hosted tool that centralizes, downloads, tags, and automatically organizes your music library from multiple sources (Spotify, SoundCloud, YouTube…). It handles metadata, prevents duplicates, keeps track of original sources, and provides a web interface for manual corrections. 

The project is still work in progress. Some surfaces are production-shaped, while others are scaffolds or partial implementations. The documentation in this repository is therefore organized around what is already wired in code versus what is still planned.

## Quick start

You can use the prebuilt Docker image to get started quickly. The following steps assume you have Docker and Docker Compose installed.

### 1. Create the `docker-compose.yml`

```yaml
services:

  soundome:
    image: ghcr.io/barthofu/soundome:latest
    ports:
      - 8000:8000
    environment:
      - SOUNDOME__SERVER__HOST=0.0.0.0
      - SOUNDOME__SERVER__PORT=8000
      - SOUNDOME__DATABASE__URL=/data/soundome.db
      - SOUNDOME__GENERAL__BASE_LIBRARY_DIR=/library
      - SOUNDOME__GENERAL__TEMP_DOWNLOAD_DIR=/temp
      - SOUNDOME__GENERAL__INGEST_DIR=/ingest
      - SOUNDOME__LOGS__LEVEL=info
    volumes:
      - ./data:/data
      - ./library:/library
      - ./temp:/temp
      - ./config.toml:/app/config.toml:ro
    restart: unless-stopped
```

### 2. Run

```bash
docker compose up -d
```

UI available at <http://localhost:8000>. Paste a YouTube, SoundCloud, or YouTube Music URL in the **Download** tab.

---

> Full configuration reference and operational docs: [docs/README.md](docs/README.md)

## Current scope

- Import tracks and playlists from supported source URLs.
- Enrich metadata before the file is finalized.
- Download audio to a staging area, then tag and move it into the library.
- Deduplicate by source URL first, then by content and quality.
- Persist tracks, albums, artists, playlists, and references in SQLite.
- Review partial metadata matches from the web admin interface.
- Route outbound HTTP traffic through the shared proxy layer when the integration supports it.

## Architecture at a glance

```
soundome/
├── apps/
│   ├── cli/          # CLI entry point (minimal / WIP)
│   ├── server/       # Rocket API, Swagger, static file serving
│   └── web/          # Svelte admin application built into data/web/
├── packages/
│   ├── ai/           # OpenRouter client and prompt helpers
│   ├── config/       # config.toml + env overlay
│   ├── database/     # Diesel repositories and migrations
│   ├── domain/       # Application services and orchestration
│   ├── downloader/   # Audio providers
│   ├── fetcher/      # Source metadata adapters
│   ├── organizer/    # Filesystem placement
│   ├── shared/       # Models, errors, HTTP helpers, logging
│   └── tagger/       # MusicBrainz enrichment and file tagging
└── docs/             # Product, architecture, workflow, and operational docs
```

## Documentation map

Start with [docs/README.md](docs/README.md).

**Getting started**
- [docs/getting-started/quickstart.md](docs/getting-started/quickstart.md): first run, paste a URL, understand the result.
- [docs/getting-started/development-setup.md](docs/getting-started/development-setup.md): local setup and daily development commands.
- [docs/getting-started/configuration.md](docs/getting-started/configuration.md): config sections, runtime expectations, and environment overrides.

**Guides**
- [docs/guides/spotify.md](docs/guides/spotify.md): activate Spotify, obtain credentials, what Spotify unlocks.
- [docs/guides/soundcloud.md](docs/guides/soundcloud.md): SoundCloud specifics — noisy metadata, DRM tracks, AI cleanup.
- [docs/guides/ai-metadata.md](docs/guides/ai-metadata.md): configure Ollama or OpenRouter to clean SoundCloud metadata automatically.
- [docs/guides/playlists.md](docs/guides/playlists.md): sync playlists, schedule automatic updates, export M3U8 files.
- [docs/guides/local-ingest.md](docs/guides/local-ingest.md): import audio files you already own.
- [docs/guides/validation.md](docs/guides/validation.md): understand and resolve tracks flagged for manual review.

**Architecture & workflows**
- [docs/product/specs.md](docs/product/specs.md): product scope, goals, constraints, and non-goals.
- [docs/architecture/design.md](docs/architecture/design.md): architecture, workflow ownership, and runtime design.
- [docs/architecture/repository-map.md](docs/architecture/repository-map.md): crate map and important code entry points.
- [docs/workflows/download.md](docs/workflows/download.md): step-by-step import scenarios.
- [docs/workflows/web-admin.md](docs/workflows/web-admin.md): web interface and validation workflow.

**Operations**
- [docs/operations/cli.md](docs/operations/cli.md): CLI reference, commands, and configuration.
- [docs/operations/proxy-configuration.md](docs/operations/proxy-configuration.md): proxy support and limitations.
- [docs/operations/playlist-m3u8-export.md](docs/operations/playlist-m3u8-export.md): M3U8 export behavior and configuration.

## Development

The full development and configuration guides live in:

- [docs/getting-started/development-setup.md](docs/getting-started/development-setup.md)
- [docs/getting-started/configuration.md](docs/getting-started/configuration.md)

Useful workspace commands:

```bash
pnpm cargo:test
pnpm cargo:clippy
pnpm cargo:fmt
pnpm web:check
```

## AI Notice

**Project development:** While the final stages of this project were completed with AI assistance (without which it might never have reached completion), the entire architecture and core codebase were hand-crafted and informed by intensive design work prior to using AI tools (well, it wasn't such a thing back then).

**Runtime AI usage:** Soundome can optionally use AI to curate and standardize base metadata extracted from SoundCloud tracks, where source metadata is often noisy. This feature is optional and can be disabled entirely. When enabled, you can use OpenRouter's API or connect a local [Ollama](https://ollama.ai/) instance for on-device processing.

## License

MIT License

Copyright (c) barthofu