# Scheduled sync v2 — global cron + one-click subscribe

## Status

Draft — spec only, implementation not started.

## Context (current state)

Scheduled sync is managed today from a single settings page
(`apps/web/src/pages/Tools.svelte`, "sync" tab). A dead duplicate page
(`apps/web/src/pages/SyncSchedules.svelte`) exists but is not routed anywhere.

The current data model (`packages/shared/src/models/sync_schedule.rs`,
table `sync_schedule`) is hardcoded to playlists:

```rust
pub struct SyncSchedule {
    pub id: Option<i32>,
    pub playlist_url: String,
    pub label: Option<String>,
    pub interval_seconds: Option<i32>,   // mutually exclusive w/ cron
    pub cron_expression: Option<String>, // mutually exclusive w/ interval
    pub enabled: bool,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}
```

Each row carries its own schedule (interval XOR cron). A background thread in
`apps/server/src/main.rs` polls every 60s for due schedules and enqueues a
`PlaylistSync` task via `TaskService::create_playlist_sync` +
`TaskExecutor::enqueue_playlist_sync`.

Artist and album sync are already fully implemented end-to-end
(`TaskType::SyncArtist`/`SyncAlbum`, `TaskService::create_artist_sync`,
`TaskExecutor::enqueue_artist_sync`, `DownloadService::sync_artist_from_url`)
but are only reachable today through the manual task-retry path — there is no
scheduling glue for them.

`ArtistTab.svelte` already receives `references: ReferenceDto[]` for the
drilled-in artist (via `Artist::get_sources()`/`get_providers()`/etc. on the
backend) but currently does not render them at all.
`PlaylistsTab.svelte` only has a single `source_url`, no references array
(consistent with the `Playlist` domain model).

## Goals

1. Replace per-item interval/cron with a single **global cron** that
   synchronizes everything in one pass. Drop interval-based scheduling
   entirely.
2. Keep the ability to add a link directly from the settings page, even
   though it stops being the primary way to subscribe.
3. Add a one-click "subscribe to scheduled sync" button on the artist and
   playlist detail pages — the main ergonomics improvement.
4. For artists specifically, allow selecting one or more sources (based on
   `ReferenceType::Source` reference metadata) to sync independently.

Out of scope for this iteration: albums (backend already supports
`SyncAlbum`, but no UI button is added yet; the data model stays generic
enough to add it later).

## Data model

### `sync_settings` (new, singleton table, single row with `id = 1`)

| column | type | notes |
|---|---|---|
| `id` | integer PK | always `1` |
| `cron_expression` | text | e.g. `0 3 * * *` |
| `enabled` | integer (bool) | global pause switch |
| `last_run` | nullable timestamp | |
| `next_run` | nullable timestamp | computed via `calculate_next_run` |

`calculate_next_run` (`packages/domain/src/schedule.rs`) drops its
interval-based branch and only supports the cron expression branch going
forward.

### `sync_schedule` (existing table, repurposed as "subscriptions")

| column | type | notes |
|---|---|---|
| `id` | integer PK | |
| `entity_type` | text | `'playlist'` \| `'artist'` |
| `artist_id` | nullable integer FK → `artist.id` | set for artist subscriptions |
| `reference_id` | nullable integer FK → `reference.id`, `ON DELETE SET NULL` | the specific `Source` reference chosen for an artist subscription; disable the row if it becomes null |
| `url` | text | denormalized target URL: `source_url` for a playlist, `reference.external_url` for an artist — lets the scheduler enqueue without extra joins |
| `label` | nullable text | display label |
| `enabled` | integer (bool) | per-item pause/resume, unchanged behavior |
| `last_run` | nullable timestamp | per item |
| `created_at` | timestamp | |

Removed columns: `interval_seconds`, `cron_expression`, `next_run` (no more
per-item "next run" — everything is driven by the global cron).

Uniqueness constraints:
- `(entity_type, artist_id, reference_id)` for artist subscriptions.
- `(entity_type, url)` for playlist subscriptions.

### Migration

- New migration adds `sync_settings` (singleton row seeded with a sane
  default cron, e.g. daily at 03:00, `enabled = true`).
- New migration alters `sync_schedule`: rename `playlist_url` → `url`
  (existing rows become `entity_type = 'playlist'`), add `entity_type`,
  `artist_id`, `reference_id`; drop `interval_seconds`, `cron_expression`,
  `next_run`; drop the now-unused CHECK triggers
  (`sync_schedule_check_interval_or_cron[_update]`).

## Scheduler behavior

- The background thread in `main.rs` keeps its 60s polling interval, but now
  checks the single `sync_settings` row instead of per-row `next_run`.
- When due: `mark_ran` on `sync_settings` (recomputes `next_run` from the
  global cron), then iterate **all** enabled subscriptions and enqueue the
  matching task type:
  - `entity_type = 'playlist'` → `create_playlist_sync` + `enqueue_playlist_sync`
  - `entity_type = 'artist'` → `create_artist_sync` + `enqueue_artist_sync`
  - update each subscription's `last_run` as it's processed.
- A manual "Run now" action on the settings page triggers the same batch
  on demand (in addition to the existing per-item "sync now" trigger, which
  stays available and does not touch the global `next_run`).

## API changes

- `GET /sync-settings`, `PATCH /sync-settings` — manage the singleton global
  cron (new).
- `POST /sync-settings/trigger` — run the full batch immediately (new).
- `POST /sync-schedules` — generalized body:
  - `{ url, label? }` for a manually pasted link; entity type is
    auto-detected via the existing fetcher URL-type detection.
  - `{ entity_type: "artist", artist_id, reference_id, label? }` for the
    one-click subscribe action from the artist page.
  - `{ entity_type: "playlist", playlist_id?, url, label? }` for the
    one-click subscribe action from the playlist page.
- `PATCH /sync-schedules/{id}` — now only accepts `label`/`enabled` (no more
  interval/cron fields).
- `DELETE /sync-schedules/{id}` — unchanged, also used to "unsubscribe".
- `SyncScheduleDto` — drop `interval_hours`/`cron_expression`/`next_run`, add
  `entity_type` and artist-display fields (artist name, source platform) for
  artist rows.

## Frontend changes

### Settings page (`Tools.svelte`, "sync" tab)

- Global cron section at the top: cron expression input, enabled toggle,
  last/next run display, "Run now" button.
- Generalized "add link" form: single URL input + optional label, type
  auto-detected server-side. Replaces the old playlist-only form with
  interval/cron pickers.
- Subscription list: shows label/name, entity type + source platform badge,
  enabled/pause toggle, per-item "sync now", delete.
- Delete the dead duplicate page `apps/web/src/pages/SyncSchedules.svelte`.

### Artist page (`ArtistTab.svelte`, plus the artist `EditModal`)

- Render the artist's references (currently not shown at all despite the
  data being available), filtered to `Source`/`Metadata` references that
  carry a usable `external_url`. **Correction from the initial draft:**
  filtering to `ReferenceType::Source` only excludes Spotify/SoundCloud
  artists entirely, since their references are created with
  `ReferenceType::Metadata` in `packages/fetcher` (only YouTube Music
  artists get `Source`) — both types must be selectable. A `Metadata`
  reference is only eligible when its platform is itself a valid artist
  sync source (Spotify, SoundCloud, YouTube Music) — e.g. a MusicBrainz
  `Metadata` reference is enrichment-only and must stay excluded. This
  eligibility check is centralized in
  `domain::schedule::is_eligible_artist_sync_reference` and mirrored
  client-side.
- Each eligible reference gets a toggle: "Scheduled sync" on/off.
  - On: creates a subscription (`entity_type: "artist"`, `artist_id`,
    `reference_id`).
  - Off: deletes (or disables) the matching subscription.
  - No immediate sync on click — subscribing only affects the next global
    cron pass.
- If the artist has no eligible reference, show an explicit empty state
  instead of a blank control.
- Implemented as a shared `ArtistSyncSources.svelte` component, rendered
  both on the artist detail page and inside the artist `EditModal` (next to
  `ReferencesPanel`), since reference management already lives in the modal
  and users look for related controls there.

### Playlist page (`PlaylistsTab.svelte`)

- Single toggle button in `.detail-actions`: "Add to scheduled sync" /
  "Remove from scheduled sync", based on `source_url` (no source picker,
  since a playlist has a single URL).

## Risks / assumptions

- Assumes a single global cron is an acceptable trade-off vs. per-item
  timing granularity (explicitly requested).
- Artist/album "sync" today re-lists everything from the source and diffs
  by URL against the DB — no lightweight "what's new" API exists upstream,
  so every scheduled pass pays a full listing cost per subscribed source.
- Existing rows in `sync_schedule` must be migrated to `entity_type =
  'playlist'` without data loss; any row previously relying on
  `interval_seconds` needs a one-time reconciliation against the new global
  cron default.
- `reference_id` deletion (e.g. user removes a reference from an artist)
  must cascade to disabling the corresponding subscription rather than
  leaving a dangling row.
