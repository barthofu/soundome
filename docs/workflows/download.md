# Download and organization scenarios

This document captures the intended end-to-end workflow for the two main ingestion paths that already shape the codebase: importing from a source URL and importing from a local file.

## Concurrency model

Every download-related job — single track, playlist sync, artist sync, album sync, and ingest-dir batch — is dispatched through a **shared serial task executor**. Only **one** job runs at a time; additional jobs sit in a FIFO queue as tasks in status `Pending` until the worker picks them up. The queue holds regardless of whether the job was triggered from an API route, from the retry endpoint, from the scheduler, or from boot-time stale-task recovery.

Rationale:
- Prevents SQLite `database is locked` errors caused by two writers racing for the exclusive DB lock.
- Keeps external API usage (Spotify, SoundCloud, YouTube, MusicBrainz…) well below rate-limit thresholds by never fanning out.
- Prevents `yt-dlp` / `ffmpeg` from stepping on each other on limited-resource hosts.

Implementation: `apps/server/src/utils/task_executor.rs` — see also `TaskStatus::Pending` (in queue) and `TaskStatus::Running` (currently executing).

## Import a track from a source URL

For each track to import:

1. URL-level deduplication.
   1.1. If a track reference with the same `external_url` already exists in the database, skip the track.

2. Fetch source metadata.
   2.1. Extract track metadata from the source platform, such as Spotify, SoundCloud, or YouTube.
   2.2. If the track comes from a playlist, keep the playlist reference for later reconstruction and ordering.

3. Identify related entities.
   3.1. Resolve the album and artists, including featured artists when possible.
   3.2. For each album or artist, check whether it already exists in the database.
   3.3. Reuse the existing entity when it is already known. Otherwise keep enough reference data to create or match it later.

4. Enrich metadata.
   4.1. Query MusicBrainz or another metadata provider.
   4.2. If a reliable match exists, attach the metadata references to the track and reuse that information for album and artist matching.
   4.3. If the match is partial or weak, mark the track for manual validation instead of finalizing it blindly.

5. Download the audio.
   5.1. Find a provider and download the file into the staging directory.
   5.2. If a SoundCloud download fails because the track is DRM-protected, and the track already carries a Spotify metadata reference (attached during step 4), automatically retry through the same Spotify → YouTube/YouTube Music matching flow used for Spotify-sourced tracks, keeping SoundCloud as the recorded source. If no Spotify reference is available or the retry also fails, fall back to manual validation (see `soundcloud_drm_protected` in [../guides/validation.md](../guides/validation.md)).

6. Deduplicate by content and quality.
   6.1. If an existing track matches by name, artists, year, or durable metadata identifiers, compare audio quality.
   6.2. If the existing version is better, keep it and discard the new staged file while preserving useful metadata.
   6.3. Otherwise continue with the new file.

7. Finalize the file.
   7.1. Move the file into the library using the `Artist/Album/Track` layout.
   7.2. Tag the file with the enriched metadata.

8. Persist the result.
   8.1. Save tracks, albums, artists, playlist links, and references to the database.

9. Export playlist.
   9.1. If the import came from a playlist, regenerate the `.m3u8` file for that playlist in the configured output directory. Failures are logged as warnings and do not block the import.
   9.2. The exported file can also be regenerated on demand via `POST /api/playlists/:id/export`.

10. Handle failures.
   10.1. Clean up temporary files when a step fails.
   10.2. Log the failure clearly so the user can retry or validate manually.

## Import a local file from an ingest directory

For each local audio file to import:

1. Read the file metadata.
   1.1. Extract the path and any embedded tags.

2. Evaluate metadata quality.
   2.1. If title, artists, album, and date are already good enough, continue.
   2.2. Otherwise try to enrich the metadata through MusicBrainz using existing tags or the file name.
   2.3. If the match remains weak, mark the track for manual validation.

3. Deduplicate.
   3.1. Look for an existing track with the same identifying metadata or durable references.
   3.2. If an existing version is better quality, skip the new file.
   3.3. If the new file is better, replace the existing audio while preserving useful metadata.
   3.4. If quality is equivalent, leave room for a manual decision instead of silently replacing.

4. Finalize and organize.
   4.1. Move the file into the library.
   4.2. Tag the file.
   4.3. Persist the result in the database.
   4.4. Create playlist-oriented filesystem artifacts later if that ingest path requires them.

5. Handle failures.
   5.1. Clean up or roll back temporary state when needed.
   5.2. Log enough context for manual follow-up.
