# Quick start

This page walks you through the first minutes with Soundome: starting the server, downloading your first track, and understanding what just happened.

## Before you begin

Make sure you have completed the steps in [development-setup.md](development-setup.md):

- Rust toolchain installed
- `pnpm install` run
- Database migrated (`diesel migration run`)
- `config.toml` present (copy from `config.example.toml`)
- `.env` file present

You do not need Spotify credentials to follow this guide — a YouTube Music or SoundCloud URL is enough to get started.

## Start the server

```bash
pnpm dev
```

This starts two processes in parallel:

- **Rocket API server** on `http://localhost:8000`
- **Vite development frontend** on `http://localhost:5173`

Open `http://localhost:5173` in your browser.

## Download your first track

1. Open the **Download** tab (the home page).
2. Paste a supported URL into the input field and press Enter or click **Download**.

### Supported URL types

| What you paste | What Soundome does |
|---|---|
| A YouTube Music track URL (`music.youtube.com/watch?...`) | Downloads the track immediately |
| A SoundCloud track URL (`soundcloud.com/artist/title`) | Downloads the track immediately |
| A Spotify track URL (`open.spotify.com/track/...`) | Requires Spotify credentials — see [guides/spotify.md](../guides/spotify.md) |
| A playlist, album, or artist URL (any of the above) | Starts a background sync task and returns immediately |

> **Note:** plain `youtube.com/watch?v=...` URLs are not accepted as input. Use `music.youtube.com` instead.

## What happens next

After you submit a URL, Soundome runs the following steps automatically:

1. **Fetch metadata** — title, artists, album, cover art from the source.
2. **Enrich** — queries MusicBrainz (and optionally Bandcamp and Spotify) to find a canonical match.
3. **Download** — calls yt-dlp or scdl to download the audio to a staging directory.
4. **Deduplicate** — checks if you already own this track; compares quality if so.
5. **Tag** — writes title, artists, album, cover art, date, track number, and a unique `SOUNDOME_ID` into the audio file.
6. **Organize** — moves the file to `<base_library_dir>/<Artist>/<Album>/<filename>`.
7. **Persist** — saves everything to the database.

This takes a few seconds for a single track. You will see the result inline on the download page.

## Reading the result

Once the download completes, the track appears in the result area with:

- Cover art, title, artists, album
- A **Review** badge if the enrichment match was partial or missing — see [guides/validation.md](../guides/validation.md)

The track is now in your library under `./library/` (or whatever `base_library_dir` is set to).

## Checking the library

Open the **Library** tab to browse your tracks, albums, and artists. You can search, filter, and edit metadata directly from the UI.

## What to read next

- [guides/spotify.md](../guides/spotify.md) — activate Spotify for higher-quality metadata
- [guides/soundcloud.md](../guides/soundcloud.md) — SoundCloud specifics: DRM tracks, AI cleanup
- [guides/playlists.md](../guides/playlists.md) — sync entire playlists and schedule automatic updates
- [guides/ai-metadata.md](../guides/ai-metadata.md) — clean noisy metadata with a local or cloud AI model
- [guides/validation.md](../guides/validation.md) — resolve tracks flagged for manual review
- [guides/local-ingest.md](../guides/local-ingest.md) — add audio files you already own
