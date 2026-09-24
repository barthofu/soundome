# Data Quality area — duplicate review, reference audit, cleanup tools

## Status

In progress. Phases 1 and 2 implemented (see [Implementation status](#implementation-status)).
Tracked in GitHub issue [#28](https://github.com/barthofu/soundome/issues/28).
Phase 1+2 shipped via PR [#29](https://github.com/barthofu/soundome/pull/29).

## Context

A metadata-cleanup mapping bug in the SoundCloud AI-cleanup step
(`packages/fetcher/src/soundcloud/mod.rs`) could silently attach one artist's
SoundCloud reference to a *different* artist name — e.g. the uploader
`SaTu`'s SoundCloud reference ending up permanently attached to an unrelated
artist row named `Unkomun`. The root cause (positional mapping between the
AI-cleaned artist list and the original uploader list) was fixed in
`fix(soundcloud): preserve artist references safely` (commit `8fbe73d`), and a
guard was added to `DieselArtistRepository::create_or_ignore` to stop a
reference from being silently reused when the name similarity is too low.

That fix prevents the bug going forward but does not detect or repair
similar corruption that may already exist in the database, and there was no
dedicated place in the web admin UI to review or fix data-quality issues in
general (existing tools are limited to an ad hoc "Similar" highlight +
manual multi-select merge for artists/albums — see
`apps/web/src/lib/library/ArtistTab.svelte` and `AlbumTab.svelte`).

This spec defines a new, separate **Data Quality** top-level menu that
centralizes:

1. A duplicate-review queue for Artists, Albums, and Tracks (replacing the ad
   hoc "Similar" highlight with an Immich-style reviewable queue).
2. A reference audit (structural, free; and remote, opt-in with network
   calls) to catch reference/name inconsistencies like the SaTu/Unkomun case.
3. General cleanup tools (orphans, playlist consistency, AI cleanup change
   log, relocated cover/icon batch-fetch buttons).

## Decisions

These were confirmed with the maintainer before implementation:

| # | Topic | Decision |
|---|---|---|
| 1 | Duplicate-review remote audit trigger | Manual only — a button ("Run audit"), never a scheduled/automatic job. |
| 2 | Similarity threshold used to group track duplicates | Reuse `Track::compare(other) >= 0.8`, the exact same call and threshold already used by `TrackService::find_track_by_title_and_artist`, `TrackService::SIMILARITY_THRESHOLD`, and `Track::transpose_metadata_impl`. This keeps "is a duplicate" consistent between the download-time dedup pipeline and the manual review queue — no new arbitrary threshold. |
| 3 | Track merge behavior | Reuses `TrackService::is_better_quality` (bitrate-based), exactly like the automatic dedup pipeline in `DownloadService`. No manual "pick which file to keep" UI. |
| 4 | Track merge and playlists | Merging tracks also re-points `playlist_tracks` rows from the source track(s) to the target track (deduplicating position if the target is already in the same playlist). |
| 5 | Ignore-list persistence | A dedicated DB table (`dedup_ignore`), not `localStorage` — the app is single-user but the ignore list should survive across devices/browsers. |
| 6 | `genre`/`track_genres` tables | Left inert. They exist in the schema but are unused (`track.genre` is a free-text column); out of scope for this feature. |

## Goals

- Give the user one place to find and fix library-wide data-quality issues,
  instead of stumbling onto them individually (as happened with the
  SaTu/Unkomun case).
- Make the *existing* artist/album duplicate-merge flow safer and less manual
  (grouped suggestions + persistent "not a duplicate" instead of a bare
  similarity highlight).
- Extend duplicate detection to tracks, which currently has no manual review
  path at all (only automatic dedup at download time).
- Detect reference/name inconsistencies before they cause a wrong track name
  to reach the library, both for free (structural, DB-only) and, on demand,
  via a live check against the source providers.
- Surface what the AI SoundCloud cleanup step actually changed, so future
  hallucinations are visible without waiting for a user to notice a wrong
  track.

## Non-goals

- No automatic/scheduled remote reference audit (see decision #1).
- No UI to manually pick which file to keep when merging tracks (see
  decision #3) — quality comparison is fully automatic.
- No changes to `genre`/`track_genres` tables (see decision #6).
- No changes to the existing per-entity "Fetch from references" button
  behavior (`apps/server/src/routes/images.rs`); phase 6 only *relocates* the
  existing bulk-fetch buttons into the new Cleanup tab, it does not change
  their logic.

## Architecture overview

New top-level menu `Data Quality`, separate from `Library`:

```
Data Quality
├── Duplicates          (Artists / Albums / Tracks — review queue)
├── Reference Audit      (Structural / Remote)
└── Cleanup               (Orphans / Playlist consistency / AI cleanup log / Covers)
```

### Data model

Three new tables, added across three migrations
(`packages/database/migrations/2026-09-17-000000..000002_*`):

```sql
-- dedup_ignore: manual "not a duplicate" list for the review queue.
-- id_a is always stored < id_b so a pair can only be recorded once.
CREATE TABLE dedup_ignore (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,       -- 'artist' | 'album' | 'track'
    id_a INTEGER NOT NULL,
    id_b INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_dedup_ignore_pair ON dedup_ignore (entity_type, id_a, id_b);

-- reference_audit_cache: cached results of the manually-triggered remote audit.
CREATE TABLE reference_audit_cache (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    reference_id INTEGER NOT NULL,
    remote_name TEXT,
    similarity_score REAL,
    status TEXT NOT NULL,             -- 'ok' | 'mismatch' | 'unreachable' | 'unsupported'
    checked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_reference_audit_cache_reference ON reference_audit_cache (reference_id);
CREATE INDEX idx_reference_audit_cache_entity ON reference_audit_cache (entity_type, entity_id);

-- ai_cleanup_log: change log for the SoundCloud AI metadata cleanup step.
CREATE TABLE ai_cleanup_log (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER,                  -- nullable: written before first DB insert
    platform TEXT NOT NULL,
    source_external_id TEXT,
    before_title TEXT NOT NULL,
    before_artists TEXT NOT NULL,      -- JSON array
    after_title TEXT NOT NULL,
    after_artists TEXT NOT NULL,       -- JSON array
    rejected_artists TEXT,             -- JSON array: AI-proposed names the guard rejected
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_ai_cleanup_log_track ON ai_cleanup_log (track_id);
```

`genre`/`track_genres` are untouched (decision #6).

### Backend layers

- `packages/shared/src/models/data_quality.rs` — domain types:
  `DataQualityEntityType`, `StructuralFinding(Kind/Entity)`,
  `DedupIgnoreEntry`, `ReferenceAuditResult(Status)`, `AiCleanupLogEntry`.
- `packages/database/src/entities/data_quality.rs` +
  `packages/database/src/repositories/data_quality.rs` — Diesel entities and
  `DieselDataQualityRepository`.
- `packages/domain/src/ports/repositories.rs` — `DataQualityRepository` trait,
  added as a new field on `RepositoryLayer`.
- `packages/domain/src/services/resources/data_quality_service.rs` —
  `DataQualityService`, a thin pass-through so routes never talk to the
  repository directly (consistent with every other resource service).
- `apps/server/src/routes/data_quality.rs` — HTTP routes.

Planned route surface (see [Implementation status](#implementation-status)
for what actually exists today):

```
GET    /api/data-quality/audit/structural
POST   /api/data-quality/audit/remote/run
GET    /api/data-quality/audit/remote
POST   /api/data-quality/audit/remote/:cache_id/apply
POST   /api/data-quality/audit/remote/:cache_id/dismiss

GET    /api/data-quality/duplicates/artists
GET    /api/data-quality/duplicates/albums
GET    /api/data-quality/duplicates/tracks
POST   /api/data-quality/duplicates/ignore        { entity_type, id_a, id_b }
POST   /api/tracks/merge                            (symmetric to /api/artists/merge)

GET    /api/data-quality/orphans
POST   /api/data-quality/orphans/cleanup
GET    /api/data-quality/playlists/issues
POST   /api/data-quality/playlists/:id/renumber
GET    /api/data-quality/ai-cleanup-log             ?limit=50
```

### Frontend

- `apps/web/src/pages/DataQuality.svelte` — top-level page, mounted as a new
  nav entry in `App.svelte` (`Page = ... | 'data-quality'`).
- Planned sub-components under `apps/web/src/lib/data-quality/`:
  `store.svelte.ts`, `DuplicateQueue.svelte`, `DuplicateReviewCard.svelte`,
  `ReferenceAuditTab.svelte`, `AuditResultRow.svelte`, `CleanupTab.svelte`,
  `AiCleanupLogTable.svelte`.
- The existing client-side similarity helpers (`areSimilarArtistNames`,
  `areSimilarAlbumNames` in `apps/web/src/lib/library/store.svelte.ts`) are
  superseded by server-side grouping for the new queue; they may be kept as
  an instant-display fallback while data loads, but the source of truth
  becomes the API.

## Structural reference audit — the four checks

Pure reads, no network calls, safe to run on every page load
(`DataQualityService::structural_findings` →
`DieselDataQualityRepository::find_structural_findings`):

| Kind | What it detects | Why it matters |
|---|---|---|
| `ConflictingReference` (A) | The same `(platform, external_id/external_url)` is attached to two different entities. | Exact fingerprint of the SoundCloud artist-mapping bug (SaTu's reference pointing at the `Unkomun` row). |
| `MultiplePlatformReferences` (B) | The same entity has two references of the same `(platform, ref_type)` with different `external_id`s. | Usually a botched merge or a reference attached to the wrong entity. |
| `PlatformUrlMismatch` (C) | A reference's `external_url` resolves (via the existing `Platform::from_url`) to a different platform than the one declared on the reference. | Catches manually-entered or malformed references. |
| `TrackMissingSourceReference` (D) | A track has no `Source` reference at all. | Should never happen through the normal download pipeline; a strong signal of a prior bug or manual DB edit. |

## Duplicate review queue (Immich-style)

Unlike the current "Similar" highlight (pairwise, no persistence, no
suggestion), the new queue:

- Groups artists/albums/tracks into **connected components** of the
  similarity graph (if A~B and B~C but not A~C, all three still form one
  group), instead of only surfacing isolated pairs.
- Computes similarity server-side using the existing model methods
  (`Artist::compare`, `Album::compare`, `Track::compare`) instead of the
  separate JS Levenshtein implementation, so the queue and the automatic
  dedup pipeline agree on what counts as a duplicate.
- Suggests a merge target per group: most linked tracks, then most
  references, then the cleanest name (no special characters).
- Persists "not a duplicate" decisions in `dedup_ignore` so a rejected pair
  never resurfaces.
- Reuses the existing `ArtistRepository::merge_into` /
  `AlbumRepository::merge_into` for artists/albums, and adds a new
  `TrackRepository::merge_into` for tracks (quality-aware via
  `TrackService::is_better_quality`, merges `playlist_tracks` — decisions #2–4).

## Cleanup tools

- **Orphan cleanup**: artists/albums with zero linked tracks (can happen
  after a validation reject or a partial merge).
- **Playlist consistency**: duplicate or missing `position` values in
  `playlist_tracks`, with an automatic renumbering action per playlist.
- **AI cleanup log**: a hook in
  `DownloadService::clean_tracks_metadata_with_progress` captures
  `(title, artists)` before/after each `Fetcher::clean_tracks_metadata` call
  and writes a diff row to `ai_cleanup_log` whenever something changed
  (purely additive, no behavior change to the cleanup step itself).
- **Covers/icons**: relocate the existing
  `POST /api/batch/fetch-artist-icons` / `POST /api/batch/fetch-album-covers`
  buttons (already implemented, currently in `apps/web/src/pages/Library.svelte`)
  into the Cleanup tab so all data-quality tooling lives in one place.

## Roadmap

1. **Phase 1** — Migrations + repository/service plumbing. No UI risk.
2. **Phase 2** — Structural reference audit (A-D) + Data Quality page shell.
   Highest priority: directly useful for auditing historical corruption like
   SaTu/Unkomun.
3. **Phase 3** — Duplicate review queue for Artists/Albums (reuses existing
   `merge_into` endpoints) + ignore-list wiring.
4. **Phase 4** — Extend duplicate review to Tracks (new
   `TrackRepository::merge_into`, quality-aware, merges playlist links).
5. **Phase 5** — Remote reference audit (manual trigger only, background task
   via the existing serial `TaskExecutor`, cached results, resolution
   actions: apply remote name / dismiss / delete reference).
6. **Phase 6** — Cleanup tab: orphans, playlist consistency, AI cleanup log
   wiring, relocate cover/icon batch-fetch buttons.

## Implementation status

- ✅ **Phase 1** — `dedup_ignore`, `reference_audit_cache`, `ai_cleanup_log`
  migrations; `DataQualityRepository`/`DataQualityService` wired into
  `RepositoryLayer`/`ServiceLayer`.
- ✅ **Phase 2** — Structural audit implemented end-to-end:
  `DieselDataQualityRepository::find_structural_findings`,
  `GET /api/data-quality/audit/structural`, and the `Data Quality` page/nav
  entry (`apps/web/src/pages/DataQuality.svelte`) with a manual re-scan
  button.
- ⬜ **Phase 3** — Duplicate review queue (Artists/Albums). Not started.
- ⬜ **Phase 4** — Duplicate review queue (Tracks) + `TrackRepository::merge_into`.
  Not started.
- ⬜ **Phase 5** — Remote reference audit. Not started. The
  `reference_audit_cache` table and `ReferenceAuditResult` model already
  exist (phase 1) but no route or provider-querying logic exists yet.
- ⬜ **Phase 6** — Cleanup tab (orphans, playlist consistency, AI cleanup log,
  cover/icon relocation). Not started. The `ai_cleanup_log` table and
  `AiCleanupLogEntry` model already exist (phase 1) but nothing writes to it
  yet — the hook into `clean_tracks_metadata_with_progress` is still to be
  added.

## Known risks / open follow-ups

- `find_structural_findings` loads all `artist`/`album`/`track` rows and
  their `_ref` tables into memory (no pagination). Acceptable for a personal
  library; revisit if the library grows very large.
- Check (C) (`PlatformUrlMismatch`) may produce false positives on
  manually-added references whose URL isn't in the platform's canonical
  domain form.
- No existing data is modified by phases 1-2 — purely additive/read-only.
- Repairing already-corrupted historical data (e.g. an existing artist row
  with a wrong name attached to a valid reference) still requires manual
  action through the structural audit findings; phases 1-2 do not
  auto-repair anything by design.
