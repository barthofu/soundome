# Soundome documentation

This directory is the main documentation hub for the repository. The content is organized by purpose so it is easier to navigate from onboarding to architecture, workflows, and operations.

## Suggested reading order

1. [getting-started/README.md](getting-started/README.md)
2. [getting-started/quickstart.md](getting-started/quickstart.md)
3. [product/specs.md](product/specs.md)
4. [architecture/design.md](architecture/design.md)
5. [workflows/download.md](workflows/download.md)

## Categories

### Getting started

- [getting-started/README.md](getting-started/README.md): quick orientation and reading path.
- [getting-started/quickstart.md](getting-started/quickstart.md): first run, paste a URL, understand the result.
- [getting-started/development-setup.md](getting-started/development-setup.md): local setup, commands, and development workflow.
- [getting-started/configuration.md](getting-started/configuration.md): config sections, environment expectations, and runtime paths.

### Guides

- [guides/README.md](guides/README.md): guide index.
- [guides/spotify.md](guides/spotify.md): activate Spotify, obtain credentials, what Spotify unlocks.
- [guides/soundcloud.md](guides/soundcloud.md): SoundCloud specifics — noisy metadata, DRM tracks, AI cleanup.
- [guides/ai-metadata.md](guides/ai-metadata.md): configure Ollama or OpenRouter to clean SoundCloud metadata automatically.
- [guides/playlists.md](guides/playlists.md): sync playlists, schedule automatic updates, export M3U8 files.
- [guides/local-ingest.md](guides/local-ingest.md): import audio files you already own.
- [guides/validation.md](guides/validation.md): understand and resolve tracks flagged for manual review.

### Product

- [product/README.md](product/README.md): product-level overview.
- [product/specs.md](product/specs.md): goals, requirements, constraints, and non-goals.

### Architecture

- [architecture/README.md](architecture/README.md): architecture entry point.
- [architecture/design.md](architecture/design.md): current runtime design and workflow ownership.
- [architecture/repository-map.md](architecture/repository-map.md): crate map and important code entry points.

### Workflows

- [workflows/README.md](workflows/README.md): workflow overview.
- [workflows/download.md](workflows/download.md): import, deduplication, and finalization scenarios.
- [workflows/web-admin.md](workflows/web-admin.md): admin UI, validation flow, and API surface.

### Operations

- [operations/README.md](operations/README.md): operational entry point.
- [operations/cli.md](operations/cli.md): CLI reference, commands, and configuration.
- [operations/proxy-configuration.md](operations/proxy-configuration.md): proxy support, formats, and limitations.
- [operations/proxy-usage-example.md](operations/proxy-usage-example.md): code-level usage of the shared proxy-aware HTTP layer.
- [operations/playlist-m3u8-export.md](operations/playlist-m3u8-export.md): M3U8 export behavior, configuration, and code entry points.

## Diagrams and supporting assets

- `architecture.d2`: architecture diagram source.
- `workflow.d2`: workflow diagram source.
- `workflow.svg` and `workflow.png`: exported workflow diagrams.
