---
description: "Use when: editing domain services, orchestration logic, DownloadService flows, transaction boundaries, or ReferenceType-aware business logic in packages/domain."
applyTo: "packages/domain/src/**/*.rs"
---

# Domain — services & orchestration

## Role

- The `domain` crate owns business workflows such as `DownloadService`.
- Services should stay independent from Rocket.

## DB

- Public service methods usually accept `&mut diesel::SqliteConnection`.
- Wrap multi-step writes in a transaction, for example as done in `TrackService::create_or_update`.

## References

- Respect the `ReferenceType` semantics:
  - `Source` and `Provider` represent the effective audio path and are often replaced
  - `Metadata` keeps durable IDs and URLs such as MusicBrainz or Spotify and is often merged
- When deduplicating by quality, merge references carefully instead of overwriting them.

## WIP

- When a step is still TODO, such as fuller manual validation or broader playlist sync, preserve the current behavior and add new code behind an explicit flag or endpoint when needed.
