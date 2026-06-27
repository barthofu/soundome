# Local file ingest

If you already have audio files on disk, Soundome can import them into your library: it reads the existing tags, enriches the metadata, deduplicates against what you already have, re-tags consistently, and moves the file into the `Artist/Album/Track` layout.

## How it works

1. **Read embedded tags** — title, artists, album, genre, date, track number (or inferred from filename pattern like `08 - Title.flac`)
2. **Enrich** — queries metadata providers in `ingest_metadata_providers` order (Spotify → MusicBrainz → Bandcamp by default)
3. **Deduplicate** — if a matching track already exists in the library, the better-quality file is kept
4. **Tag** — writes clean, consistent tags including the `SOUNDOME_ID` marker
5. **Organise** — moves the file to `<base_library_dir>/<Artist>/<Album>/<filename>`
6. **Persist** — saves the track to the database

If enrichment fails or returns a partial match, the file is staged and flagged `needs_validation = true`. No data is lost — the file waits in the staging area until you approve it from the Validations tab.

## Supported formats

MP3, FLAC, M4A, MP4, AAC, OGG, OPUS, WAV.

## Setup

Set the ingest directory in your config (default: `./ingest`):

```toml
[general]
ingest_dir = "./ingest"
```

Drop files or directories into this folder before running an ingest. Subdirectory structure is not required; flat directories work fine.

## Ingesting files

### From the web UI (single file or all at once)

1. Open the **Library** page.
2. Use **Ingest files** to trigger an ingest of everything currently in `ingest_dir`.

The background task runs the same enrichment and organisation pipeline as URL downloads.

### From the API

**Ingest a single file:**

```
POST /api/library/ingest
Content-Type: application/json
{ "file_path": "relative/to/ingest_dir/track.flac" }
```

Or with an absolute path:

```
{ "file_path": "/home/user/music/track.flac" }
```

**Ingest all files in `ingest_dir`:**

```
POST /api/library/ingest/all
```

This starts a background task. Monitor progress in the Tasks tab.

**Preview what is in `ingest_dir`:**

```
GET /api/library/ingest/files
```

Returns a list of audio files with their currently embedded tags — useful for checking what Soundome will read before committing.

### From the CLI

```bash
soundome ingest /path/to/track.flac
```

The server must be running. The file is sent via the API.

## Enrichment provider order for ingest

By default, Spotify is queried first for ingest because it tends to provide better cover art, release dates, track numbers, and disc numbers than MusicBrainz alone:

```toml
[tagger]
ingest_metadata_providers = ["spotify", "musicbrainz", "bandcamp"]
```

Spotify credentials are required for the Spotify provider to run. Without them, it is silently skipped and MusicBrainz runs first. See [guides/spotify.md](spotify.md).

You can adjust the order to your preference:

```toml
[tagger]
# Prefer MusicBrainz durable IDs over Spotify cover art
ingest_metadata_providers = ["musicbrainz", "spotify", "bandcamp"]
```

## Deduplication during ingest

If a track already in your library matches the incoming file by title and artist similarity, Soundome compares the audio quality of both files. The better file is kept; the other is discarded. References (source and metadata URLs) are merged rather than overwritten.

This means you can safely ingest a folder of tracks without worrying about creating duplicates — if you already have a higher-quality version, the ingest will keep yours.

## After ingest

Tracks successfully ingested appear in the **Library** tab immediately. Tracks that could not be fully enriched appear in the **Validations** tab with reason `metadata_partial_match` or `metadata_no_match`.

## Library scan (reconcile files vs database)

If you have moved, renamed, or deleted files outside of Soundome, the scan command reconciles the filesystem with the database:

```bash
soundome scan
```

Or with a dry run to see what would change without modifying anything:

```bash
soundome scan --dry-run
```

The scan uses the `SOUNDOME_ID` custom tag (written into every file at finalization) to recognise files even if they have been moved. It reports each file as one of:

| Category | Meaning |
|---|---|
| `ok` | File at expected path, tags consistent |
| `path_changed` | File moved — database path updated automatically |
| `tag_conflict` | Tags in file differ from database — track flagged for review |
| `missing` | Database row exists but file not found anywhere |
| `orphan` | File has a `SOUNDOME_ID` but no matching database row |
| `legacy_match` | Matched via MusicBrainz ID only (no `SOUNDOME_ID` tag yet) |
| `unmanaged` | File in the library directory, unknown to Soundome |

Unmanaged files can be imported using `soundome ingest` or `POST /api/library/ingest`.
