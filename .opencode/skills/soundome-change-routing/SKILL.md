---
name: soundome-change-routing
description: Use when identifying the right Soundome crate, entry point, owning abstraction, or validation command before changing code. Useful for routing work across server, domain, database, fetcher, downloader, tagger, organizer, config, shared, and web.
---

# Soundome Change Routing

Use this skill when you need to decide where a change belongs before editing code.

## Routing guide

- `apps/server`: HTTP handlers, route exposure, Swagger, static serving
- `apps/web`: admin UI and frontend interactions
- `packages/domain`: business workflow orchestration
- `packages/database`: Diesel persistence and repository semantics
- `packages/fetcher`: source URL parsing and source metadata
- `packages/downloader`: provider resolution and audio downloads
- `packages/tagger`: metadata enrichment and audio tagging
- `packages/organizer`: filesystem placement
- `packages/config`: typed configuration and overrides
- `packages/shared`: models, errors, shared helpers, proxy-aware HTTP

## Procedure

1. Start from the user-visible failing behavior or requested feature.
2. Decide whether the owning surface is UI, HTTP, orchestration, persistence, source integration, provider integration, tagging, organizing, or config.
3. Pick the nearest code path that directly computes or controls the behavior.
4. Identify the narrowest validation command for that slice.
5. Only after that, branch into the targeted crate or module.

## Useful references

- `docs/architecture/repository-map.md`
- `docs/architecture/design.md`
- `docs/workflows/download.md`
