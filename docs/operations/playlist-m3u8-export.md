# Playlist M3U8 export

## Overview

After each playlist sync, Soundome regenerates one `.m3u8` file per playlist in a configurable output directory. Any M3U8-compliant music player — Navidrome, Jellyfin, mpd, VLC, and others — can discover and read these files without depending on Soundome at runtime.

The feature is always active; no explicit opt-in is required. Only finalized tracks with a library path appear in the exported file.

## Output format

Each file is written as `{playlist name}.m3u8` using UTF-8 encoding. The file name is sanitized so that characters invalid on common operating systems (e.g. `/`, `:`, `*`) are replaced with underscores.

```m3u
#EXTM3U
#EXTENC:UTF-8
#EXTINF:213,Aphex Twin - Windowlicker
/home/user/library/Aphex Twin/...I Care Because You Do/Windowlicker.flac
#EXTINF:187,Burial - Archangel
/home/user/library/Burial/Untrue/Archangel.flac
```

- `#EXTINF:{seconds},{artists} - {title}` — duration in integer seconds, or `-1` when unknown.
- The path line is the absolute path to the finalized audio file.
- Tracks that are still in staging (no `file_path`) or flagged `needs_validation` are skipped until finalized.
- Track order matches the order stored in the database for that playlist.

## Configuration

Add a `[playlists]` section to `config.toml` to change the output directory:

```toml
[playlists]
m3u8_dir = "library/Playlists"  # relative to working directory, or absolute
```

When this key is absent the output directory defaults to `{general.base_library_dir}/Playlists/`.

The directory is created automatically on the first export.

## When files are regenerated

An `.m3u8` file is written (or overwritten) in two situations:

1. **Automatically after each playlist sync** — `DownloadService::sync_playlist_from_url` calls `PlaylistService::export_m3u8` at the end of a sync. A write failure is logged as a warning and does not abort the sync.

2. **On demand via the API** — `POST /api/playlists/:id/export` triggers an immediate regeneration for a single playlist. This is useful after manually approving pending validations so the exported file reflects the newly finalized tracks.

## Stale files

When a playlist is deleted from Soundome its `.m3u8` file is **not** removed automatically. Manual cleanup of the output directory is required if a playlist is no longer needed.

## Code entry points

| Location | Role |
|---|---|
| `packages/organizer/src/playlist_writer.rs` | `write_m3u8` — creates the output dir, sanitizes the file name, writes the M3U8 content |
| `packages/domain/src/services/resources/playlist_service.rs` | `export_m3u8` — loads playlist + tracks from DB, resolves the output dir, calls `write_m3u8` |
| `packages/domain/src/services/download_service.rs` | Triggers `playlist_service.export_m3u8` after `sync_playlist_from_url` |
| `apps/server/src/routes/playlists.rs` | `POST /api/playlists/:id/export` route handler |
| `packages/config/src/models.rs` | `PlaylistsConfig` struct with the `m3u8_dir` field |
