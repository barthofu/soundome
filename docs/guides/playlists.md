# Playlists

Soundome can sync entire playlists from Spotify, SoundCloud, and YouTube Music, keep them up to date automatically, and export them as standard M3U8 files for use in any music player.

## Syncing a playlist

Paste a playlist URL in the **Download** field and press Enter. Soundome detects the URL type automatically and starts a background sync task.

**Supported playlist URL formats:**

| Source | URL pattern |
|---|---|
| Spotify | `open.spotify.com/playlist/...` |
| SoundCloud | `soundcloud.com/<artist>/sets/<playlist>` |
| YouTube Music | `music.youtube.com/playlist?list=...` |

Soundome curates the submitted link before syncing: tracking/share query parameters such as `?si=...`, `utm_source=...`, `utm_medium=...`, or `utm_campaign=...` are stripped automatically (while parameters a platform actually needs, like YouTube's `v`/`list`, are kept). This means pasting the "same" playlist link with or without those parameters is recognized as the same playlist and re-syncs the existing entry instead of creating a duplicate.

The sync runs in the background. You can monitor progress in the **Tasks** tab: it shows the number of tracks downloaded, skipped (already in library), flagged for validation, and any errors.

## What happens during sync

For each track in the playlist, Soundome runs the full download workflow:

1. Check if the track is already in your library by URL — if so, it is linked to the playlist and skipped
2. Fetch metadata from the source
3. Clean metadata (AI cleanup for SoundCloud, if enabled)
4. Enrich via MusicBrainz / Bandcamp / Spotify
5. Download audio, tag, organise, persist

Tracks that fail enrichment or download are saved to the database with `needs_validation = true` and can be reviewed in the **Validations** tab. They do not abort the rest of the sync.

## Cancelling a sync

Open the **Tasks** tab and click **Cancel** on the running task. The sync stops at the next track boundary — the current track completes first.

## Retrying a failed sync

If a sync task fails or is cancelled mid-way, open the **Tasks** tab and click **Retry**. Soundome restarts the sync from scratch, but tracks already in the library are skipped at the URL-dedup step so they are not re-downloaded.

## Albums and artist discographies

The same download field handles album and artist URLs:

| URL pattern | What is synced |
|---|---|
| `open.spotify.com/album/...` | Full album |
| `music.youtube.com/playlist?list=OLAK5uy_...` | YouTube Music album |
| `open.spotify.com/artist/...` | All tracks by the artist |
| `music.youtube.com/channel/...` | All YouTube Music uploads by the artist |
| `soundcloud.com/<artist>` (profile root) | All SoundCloud uploads by the artist |

These are all async background tasks with the same progress, cancel, and retry behaviour as playlist syncs.

## Scheduled syncs

Soundome synchronizes everything in a single pass driven by one **global cron
schedule** (configured once, not per item). Subscribe artists and playlists
individually — what gets synced — and let the global schedule decide when.

### Subscribing

The quickest path is a one-click button:

- On an **artist page**, pick one or more `Source` references to subscribe
  (an artist can have several, e.g. Spotify and SoundCloud — each is synced
  independently).
- On a **playlist page**, a single "Add to scheduled sync" button subscribes
  its `source_url`.

You can also add a link manually from the **Tools → Sync** page, or via the API:

```
POST /api/sync-schedules
{
  "url": "https://open.spotify.com/playlist/...",
  "label": "My Weekend Mix"
}
```

The entity type (playlist vs. artist) is auto-detected from the URL. To
subscribe a specific artist source directly:

```
POST /api/sync-schedules
{
  "artist_id": 42,
  "reference_id": 7,
  "label": "My Favourite Artist"
}
```

### The global schedule

```
GET /api/sync-settings
PATCH /api/sync-settings
{
  "cron_expression": "0 0 3 * * *",
  "enabled": true
}
```

The cron expression uses 6 fields (seconds, minutes, hours, day of month,
month, day of week) — e.g. `0 0 3 * * *` runs daily at 3am.

Trigger a full pass immediately, without waiting for the cron:

```
POST /api/sync-settings/trigger
```

### Managing subscriptions

From the **Tools → Sync** page (or the `/api/sync-schedules` endpoints) you can:

- **Pause** a subscription without deleting it (toggle `enabled`)
- **Resume** a paused subscription
- **Trigger immediately** — runs this one subscription now without waiting
  for the next global cron pass
- **Edit** the label
- **Remove** the subscription (unsubscribe)

### Manual trigger via API

```
POST /api/sync-schedules/:id/trigger
```

## M3U8 playlist export

After every playlist sync, Soundome writes a `.m3u8` file so that Navidrome, Jellyfin, mpd, and other players can discover your playlists without needing Soundome at runtime.

### File location

By default, M3U8 files are written to:

```
<base_library_dir>/Playlists/<PlaylistName>.m3u8
```

Override the directory:

```toml
[playlists]
m3u8_dir = "/mnt/music/Playlists"
```

Or:

```
SOUNDOME__PLAYLISTS__M3U8_DIR=/mnt/music/Playlists
```

### File format

Standard M3U8 with UTF-8 encoding, one entry per track:

```m3u8
#EXTM3U
#EXTENC:UTF-8
#EXTINF:213,Artist Name - Track Title
/absolute/path/to/library/Artist/Album/track.mp3
```

Tracks with `needs_validation = true` or no file path (not yet downloaded) are excluded from the export.

Track order in the M3U8 matches the order stored in the database (which reflects the source playlist order at the time of the last sync).

### Triggering an export manually

```
POST /api/playlists/:id/export
```

This regenerates the M3U8 from the current database state. Useful after editing metadata, approving validated tracks, or moving files.

### Stale M3U8 files

M3U8 files are not deleted automatically when a playlist is removed from Soundome. If you delete a playlist, remove the corresponding `.m3u8` file from the playlists directory manually.

## Troubleshooting

**Playlist sync task shows many errors**
→ Open the task details. Common causes: SoundCloud DRM-protected tracks (expected), enrichment failures for unusual or very new tracks (will appear in the validation queue), or network issues mid-sync (retry the task).

**New tracks added to the playlist are not appearing**
→ Re-trigger the sync manually or wait for the scheduled run. Soundome does not watch for changes in real time; it syncs when asked.

**The M3U8 file is missing some tracks**
→ Tracks with `needs_validation = true` are excluded from the export. Approve them in the Validations tab and then re-export.

**Player shows incorrect track order**
→ The order in the M3U8 reflects the order at the time of the last sync from the source. If the source playlist has been reordered since the last sync, re-trigger a sync and then re-export.
