---
name: extend-download-workflow
description: Use when changing DownloadService, playlist sync, source-to-provider flow, staging logic, deduplication, tagging handoff, organize step, or track finalization without breaking Soundome workflow invariants.
---

# Extend Download Workflow

Use this skill for changes that touch the main ingestion and finalization flow owned by `packages/domain/src/services/download_service.rs`.

## Invariants to preserve

- Deduplicate by source URL before doing expensive work.
- `Source` and `Provider` represent the effective audio path.
- `Metadata` references preserve durable IDs and URLs.
- Partial metadata matches should remain reviewable instead of being silently finalized.
- Better audio can replace current audio, but useful metadata from both sides should survive.
- Worse audio should be discarded while retaining useful metadata references when appropriate.

## Procedure

1. Start from the precise entry point: track URL download, playlist sync, or validation finalization.
2. Identify which step changes: fetch, clean, enrich, stage, dedupe, tag, organize, or persist.
3. Read the nearest owning branch in `DownloadService` before widening scope.
4. Trace any effect on `TrackService`, `PlaylistService`, tagging, and organizing.
5. Preserve staged-file behavior for tracks requiring manual validation.
6. Check whether reference replacement versus merge behavior changes.
7. Apply the smallest change that preserves the workflow contract.
8. Validate with the narrowest workflow-scoped check available.

## Ask for clarification when needed

- whether the new case is single-track, playlist, local ingest, or manual validation
- the expected behavior for weak MusicBrainz matches
- the expected behavior when a lower-quality duplicate is found

## Useful files

- `packages/domain/src/services/download_service.rs`
- `packages/domain/src/services/track_service.rs`
- `packages/domain/src/services/playlist_service.rs`
- `packages/shared/src/models/`
- `docs/workflows/download.md`
