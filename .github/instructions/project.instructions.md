---
description: "Use when: starting work in the Soundome repository, needing project context, identifying major architecture boundaries, or understanding global product and repository expectations before choosing a narrower instruction or skill."
---

# Soundome project context

## Overview

Soundome is a personal music-library management project. It pulls data from streaming sources and local files, enriches metadata, downloads or ingests audio, tags files, organizes them on disk, and persists enough context to keep the library clean over time.

The repository is still evolving, so prefer the current code paths over older notes when the documentation and the implementation diverge.

## Product goals

- Build a unified library from multiple music sources.
- Minimize manual intervention.
- Keep the filesystem layout predictable and library quality high.
- Enrich metadata automatically when confidence is acceptable.
- Preserve playlist and source provenance without reintroducing duplicates.

## Main architecture

### Import pipeline

- Accept a track or playlist URL.
- Fetch source metadata.
- Clean noisy metadata.
- Enrich through metadata providers.
- Download to staging.
- Deduplicate and compare quality.
- Tag, organize, and persist.

### Persistence layer

- SQLite for entities and references.
- Filesystem library for the finalized audio files.
- Temporary staging area for downloaded files waiting to be finalized.

### Manual validation layer

- A web admin UI exists for ambiguous metadata matches.
- Tracks can remain staged until a user approves the final metadata.

## Important data-model ideas

- `Track`, `Album`, `Artist`, and `Playlist` live in `packages/shared/src/models`.
- `ReferenceType::Source` and `ReferenceType::Provider` describe the effective audio path.
- `ReferenceType::Metadata` preserves durable identifiers even when the source or provider changes.

## Important code entry points

- `apps/server/src/main.rs`
- `packages/domain/src/services/download_service.rs`
- `packages/shared/src/libs/http.rs`
- `packages/database/src/repositories/*`

## Working expectations

- Prefer small, reversible changes.
- Keep behavior explicit when a feature is still partial.
- Avoid adding parallel abstractions when an existing service or repository already owns the workflow.
- Use the docs under `docs/` as the first stop, but verify behavior in code before changing architecture-sensitive areas.
