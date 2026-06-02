# Soundome documentation

This directory is the main documentation hub for the repository. The content is organized by purpose so it is easier to navigate from onboarding to architecture, workflows, and operations.

## Suggested reading order

1. [getting-started/README.md](getting-started/README.md)
2. [product/specs.md](product/specs.md)
3. [architecture/design.md](architecture/design.md)
4. [workflows/download.md](workflows/download.md)

## Categories

### Getting started

- [getting-started/README.md](getting-started/README.md): quick orientation and reading path.
- [getting-started/development-setup.md](getting-started/development-setup.md): local setup, commands, and development workflow.
- [getting-started/configuration.md](getting-started/configuration.md): config sections, environment expectations, and runtime paths.

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
