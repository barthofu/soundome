# Soundome CLI

The Soundome CLI is a command-line client for the Soundome API. It does not connect to the database or the domain layer directly — all operations go through the HTTP API exposed by the server.

## Requirements

- The Soundome server must be running and reachable.
- No additional local setup is needed beyond building the binary.

## Build

```bash
cargo build -p cli
# or for a release binary
cargo build -p cli --release
```

The binary is placed at `target/debug/cli` or `target/release/cli`.

## Configuration

| Source | Description |
|---|---|
| `--api-url <url>` | Server base URL, passed explicitly per invocation. |
| `SOUNDOME_API_URL` | Environment variable — overrides the default when set. |
| default | `http://localhost:8000` |

The `.env` file at the repository root is loaded automatically when present (via `dotenvy`). You can set `SOUNDOME_API_URL` there for local development.

## Command reference

### Global flag

```
soundome [--api-url <url>] <command>
```

### `library search`

Search library entities with optional filters.

```bash
soundome library search <entity> [options]
```

`<entity>` can be one of:

- `tracks`
- `albums`
- `artists`
- `playlists`

Common options:

| Option | Description |
|---|---|
| `--query <text>` | Free-text query (name/title/artist depending on entity). |
| `--limit <n>` | Limit number of returned rows. |
| `--format <table\|json\|jsonl>` | Output format. Defaults to `table`. |

Entity-specific filters:

| Entity | Option | Description |
|---|---|---|
| `tracks` | `--genre <genre>` | Filter tracks by genre (contains, case-insensitive). |
| `tracks` | `--needs-validation` | Keep only tracks requiring manual validation. |
| `tracks` | `--has-file` | Keep only tracks with a local `file_path`. |
| `playlists` | `--source <source>` | Filter playlists by source (contains, case-insensitive). |

Examples:

```bash
# Find tracks matching a text query
soundome library search tracks --query "acid" --limit 20

# JSON output for scripting
soundome library search playlists --source spotify --format json

# JSONL output
soundome library search artists --query "tek" --format jsonl

# Tracks requiring validation and already downloaded
soundome library search tracks --needs-validation --has-file
```

### `library playlist list`

List all playlists in the library.

```bash
soundome library playlist list [--format <table|json|jsonl>]
```

Output:

```
  ID  Name                                      Source
────────────────────────────────────────────────────────────────
   1  Tekno & friends                           Spotify
   2  Late night                                SoundCloud
```

### `library playlist download`

Download the local tracks of a playlist to a directory via HTTP streaming.

```bash
soundome library playlist download <playlist> [--output <dir>] [--flat] [--sync] [--manifest <path>]
```

| Argument / flag | Description |
|---|---|
| `<playlist>` | Numeric playlist ID or a partial name (case-insensitive). If several playlists match the name, an interactive picker is shown. |
| `--output <dir>` | Destination directory. Created automatically if it does not exist. Defaults to the current directory. |
| `--flat` | Write files directly into the output directory, without creating a playlist sub-directory. |
| `--sync` | Skip tracks whose destination file already exists. |
| `--manifest <path>` | Write a JSON manifest at a custom path. Default: `<target>/manifest.json`. |

#### Default layout (without `--flat`)

The command always writes a JSON manifest containing summary and per-track status (`downloaded`, `skipped`, `failed`).

```
<output>/
  <PlaylistId> - <PlaylistName>/
    <Order> - <Artist> - <Title>.<ext>
```

#### Flat layout (`--flat`)

```
<output>/
  <Order> - <Artist> - <Title>.<ext>
```

`<Order>` is a zero-padded index based on playlist order (`01`, `02`, ...).

When a track has no local file on the server, or the server returns a non-2xx response, it is skipped with a warning. The rest of the playlist continues.

#### Examples

```bash
# Download by numeric ID into ~/music/tekno
soundome library playlist download 1 --output ~/music/tekno

# Download by partial name, flat layout
soundome library playlist download "late night" --output /tmp/export --flat

# Sync mode: only missing files are downloaded
soundome library playlist download 3 --output /tmp/export --sync

# Custom manifest path
soundome library playlist download 3 --manifest /tmp/export/report.json

# Point at a remote server
soundome --api-url http://192.168.1.10:8000 library playlist download 3
```

## How track download works

The CLI calls:

- `GET /api/playlists`
- `GET /api/playlists/:id/tracks`
- `GET /api/tracks/:id/download` (for each track)
- `GET /api/tracks`, `GET /api/albums`, `GET /api/artists` (for `library search`)

Track downloads are streamed chunk by chunk to disk, with a byte-level progress bar.

After a track is saved, the CLI rewrites the track-number metadata so that `track_number` matches the playlist position (and sets the total track count to the playlist length).

Tracks that are not yet finalized (no `file_path` in the database) or whose audio file is missing on the server are reported as skipped.

## Current limitations

- Search is client-side filtering after API fetch (no server-side pagination yet).
- `--sync` currently checks destination file existence only (no checksum/version comparison).
- Authentication is not implemented — the server is assumed to be trusted.
