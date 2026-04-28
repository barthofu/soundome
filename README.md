# Soundome

Soundome is a Rust monorepo for building and maintaining a personal music library from streaming URLs and local files. The current implementation focuses on fetching metadata, enriching tracks with MusicBrainz, downloading audio from a provider, tagging files, organizing them on disk, and persisting the result in SQLite.

The project is still work in progress. Some surfaces are production-shaped, while others are scaffolds or partial implementations. The documentation in this repository is therefore organized around what is already wired in code versus what is still planned.

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

