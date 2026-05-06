# Spec: Playlist M3U8 Export

## Problem

Soundome manages source playlists (Spotify, SoundCloud, YouTube) in its database, but those playlists are not visible to other applications that read the library directory — for example Navidrome, Jellyfin, or any client using a music server.

The goal is to export Soundome-managed playlists as `.m3u8` files on disk so that any compliant music application can discover and consume them without depending on Soundome at runtime.

## Non-goals

- Soundome does not become a streaming server.
- Soundome does not parse or import M3U8 files written by other applications.
- Playlist files written by the user in other tools are out of scope.

## Proposed behavior

After each playlist sync, Soundome regenerates one `.m3u8` file per playlist in a configurable output directory (default: `{library_root}/Playlists/`).

Each file:
- is named after the playlist slug (e.g. `My Playlist.m3u8`)
- is fully regenerated from the current playlist state in the database
- uses absolute paths pointing to the finalized library files
- contains `#EXTINF` lines with duration and a `{artist} - {title}` display name

Example output:

```m3u
#EXTM3U
#EXTENC:UTF-8
#EXTINF:213,Aphex Twin - Windowlicker
/home/user/library/Aphex Twin/...I Care Because You Do/Windowlicker.flac
#EXTINF:187,Burial - Archangel
/home/user/library/Burial/Untrue/Archangel.flac
```

Tracks that are still in staging or flagged `needs_validation` are skipped until finalized.

## Config

A new optional section in `config.toml`:

```toml
[playlists]
m3u8_dir = "library/Playlists"  # relative to the working directory, or absolute
```

If absent, defaults to `{library_root}/Playlists/`.

## Implementation plan

### 1. M3U8 writer — `packages/organizer`

Add a `playlist_writer` module in `packages/organizer`. It exposes a single function:

```rust
pub fn write_m3u8(playlist: &Playlist, tracks: &[Track], output_dir: &Path) -> SoundomeResult<PathBuf>
```

Responsibility:
- create `output_dir` if it does not exist
- derive the file name from the playlist name (sanitize to a safe filesystem name)
- write `#EXTM3U` header, then one `#EXTINF` + path pair per track
- skip tracks with no `file_path` (not yet finalized)
- overwrite any existing file with the same name

### 2. Embed duration and display string

`Track` already carries `duration_ms` (or equivalent) and `title` + `artists`. The `#EXTINF` line should use integer seconds:

```
#EXTINF:{duration_seconds},{display_artist} - {title}
```

If duration is unavailable, write `-1` (valid per spec).

### 3. Trigger after playlist sync — `packages/domain`

In `DownloadService` (or a dedicated `PlaylistService`), after the sync loop finalizes tracks, call `organizer::playlist_writer::write_m3u8`. The call is best-effort: a write failure logs a warning but does not abort the sync.

### 4. Expose on demand — `apps/server`

Add a route `POST /api/playlists/:id/export` that triggers an immediate M3U8 regeneration for a single playlist. This is useful after manual validation approvals.

### 5. Config wiring — `packages/config`

Add `PlaylistsConfig` to the existing config struct:

```rust
pub struct PlaylistsConfig {
    pub m3u8_dir: Option<String>,
}
```

Resolve the final path at call time using the library root as the base when the value is relative.

## Risks and open questions

- **Path portability**: absolute paths are convenient but break if the library moves. A future option could write relative paths from the playlist file's own location. For now, absolute paths are acceptable.
- **Stale files**: if a playlist is deleted from Soundome, its `.m3u8` is not cleaned up automatically. A future cleanup pass could remove orphaned files.
- **Ordering**: playlist track order should be respected. The DB query must preserve the playlist's track ordering.
- **Encoding**: file names with special characters need sanitization; path values in the M3U8 body should be percent-encoded or URI-formatted if paths contain spaces (Navidrome tolerates plain paths).
