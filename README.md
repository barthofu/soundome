# Soundome

Soundome is a Rust monorepo for building and maintaining a personal music library from streaming URLs and local files. The current implementation focuses on fetching metadata, enriching tracks with MusicBrainz, downloading audio from a provider, tagging files, organizing them on disk, and persisting the result in SQLite.

The project is still work in progress. Some surfaces are production-shaped, while others are scaffolds or partial implementations. The documentation in this repository is therefore organized around what is already wired in code versus what is still planned.

## Quick start

### 1 — Create the config file

```bash
curl -o config.toml https://raw.githubusercontent.com/barthofu/soundome/main/config.example.toml
```

Open `config.toml` and fill in your Spotify credentials — the only required secret:

```toml
[providers.spotify]
client_id     = "your_spotify_client_id"
client_secret = "your_spotify_client_secret"
```

Everything else has safe defaults. AI enrichment and proxy are disabled by default.

### 2 — Create the `.env` file

```dotenv
DATABASE_URL=data/soundome.db
SOUNDOME__DATABASE__URL=data/soundome.db
```

### 3 — Create the `docker-compose.yml`

```yaml
services:
  soundome:
    image: ghcr.io/barthofu/soundome:latest
    ports:
      - "8000:8000"
    env_file: .env
    environment:
      - SOUNDOME__GENERAL__BASE_LIBRARY_DIR=/library
      - SOUNDOME__GENERAL__TEMP_DOWNLOAD_DIR=/temp
      - SOUNDOME__DATABASE__URL=/data/soundome.db
    volumes:
      - ./data:/data
      - ./library:/library
      - ./temp:/temp
      - ./config.toml:/app/config.toml:ro
    restart: unless-stopped
```

### 4 — Run

```bash
docker compose up -d
```

UI available at <http://localhost:8000>. Paste a Spotify, YouTube, SoundCloud, or YouTube Music URL in the **Download** tab.

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

- [docs/getting-started/development-setup.md](docs/getting-started/development-setup.md): local setup and daily development commands.
- [docs/getting-started/configuration.md](docs/getting-started/configuration.md): config sections, runtime expectations, and environment overrides.
- [docs/product/specs.md](docs/product/specs.md): product scope, goals, constraints, and non-goals.
- [docs/architecture/design.md](docs/architecture/design.md): architecture, workflow ownership, and runtime design.
- [docs/architecture/repository-map.md](docs/architecture/repository-map.md): crate map and important code entry points.
- [docs/workflows/download.md](docs/workflows/download.md): step-by-step import scenarios.
- [docs/workflows/web-admin.md](docs/workflows/web-admin.md): web interface and validation workflow.
- [docs/operations/proxy-configuration.md](docs/operations/proxy-configuration.md): proxy support and limitations.
- [docs/operations/proxy-usage-example.md](docs/operations/proxy-usage-example.md): code-level proxy usage patterns.

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

