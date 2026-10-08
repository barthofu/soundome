# Soundome CLI

The Soundome CLI is a command-line client for the Soundome API. It does not connect to the database or the domain layer directly — all operations go through the HTTP API exposed by the server.

## Requirements

- The Soundome server must be running and reachable.
- Prebuilt releases are available for Linux (x86-64 and ARM64), macOS (Intel and Apple Silicon), and Windows (x86-64). The one-line installer currently supports Linux and macOS only.
- The one-line installer requires `curl` and either `sha256sum` or `shasum`; resolving the latest CLI release also requires Python 3. Selecting an explicit version skips that Python 3 requirement.

## Install a release

The installer downloads the matching binary from GitHub Releases, verifies its SHA-256 checksum, and installs the `soundome` command into `~/.local/bin` (or `/usr/local/bin` when run as root):

```bash
curl -fsSL https://raw.githubusercontent.com/barthofu/soundome/main/helpers/scripts/install.sh | sh
```

To install a specific CLI release, set `SOUNDOME_VERSION` (without the `v` prefix):

```bash
curl -fsSL https://raw.githubusercontent.com/barthofu/soundome/main/helpers/scripts/install.sh | SOUNDOME_VERSION=1.0.0 sh
```

Set `INSTALL_DIR` to choose another destination. The installer can also be downloaded and reviewed before running. On Windows, download the `soundome-cli-x86_64-pc-windows-msvc.exe` asset from the desired CLI release on GitHub and run it directly.

## Build

```bash
cargo build -p soundome-cli
# or for a release binary
cargo build -p soundome-cli --release
```

The source-build binary is named `soundome-cli` and is placed at `target/debug/soundome-cli` or `target/release/soundome-cli`. The release installer renames it to the user-facing `soundome` command.

## Configuration

| Source | Description |
|---|---|
| `--api-url <url>` | Server base URL, passed explicitly per invocation. |
| `SOUNDOME_API_URL` | Environment variable — overrides the default when set. |
| default | `http://localhost:8777` |

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

Download the local tracks of a playlist to a directory via HTTP streaming. Re-running the command on the same playlist **synchronises** the directory instead of downloading everything again.

```bash
soundome library playlist download <playlist> [--output <dir>] [--flat] [--force] [--with-playlist-id] [--with-track-number] [--manifest <path>]
```

| Argument / flag | Description |
|---|---|
| `<playlist>` | Numeric playlist ID or a partial name (case-insensitive). If several playlists match the name, an interactive picker is shown. |
| `--output <dir>` | Destination directory. Created automatically if it does not exist. Defaults to the current directory. |
| `--flat` | Write files directly into the output directory, without creating a playlist sub-directory. |
| `--force` | Re-download every track, even when a local copy exists. Files of removed tracks are still cleaned up. |
| `--with-playlist-id` | Prefix the playlist directory with the zero-padded playlist ID (`0001 - Name`). Previously the default. |
| `--with-track-number` | Prefix file names with the zero-padded playlist position (`01 - Artist - Title`). Previously the default. |
| `--manifest <path>` | Read/write the JSON manifest at a custom path. Default: `<target>/manifest.json`. |
| `--sync` | Deprecated no-op (hidden): synchronisation is now the default behaviour. |

#### Default layout (without `--flat`)

```
<output>/
  <PlaylistName>/
    manifest.json
    <Artist> - <Title>.<ext>
```

With `--with-playlist-id` and `--with-track-number`:

```
<output>/
  <PlaylistId> - <PlaylistName>/
    <Order> - <Artist> - <Title>.<ext>
```

`<Order>` is a zero-padded index based on playlist order (`01`, `02`, ...). When two tracks resolve to the same file name, the later one gets a ` (2)`, ` (3)`… suffix.

#### Flat layout (`--flat`)

```
<output>/
  <Artist> - <Title>.<ext>
```

#### Synchronisation

The command always writes a JSON manifest containing a summary and per-track status (`downloaded`, `skipped`, `failed`) along with each file name. On the next run, the previous manifest (same playlist ID) is used to:

- keep tracks that are already present locally (`skipped`);
- rename kept files when their expected name changed (e.g. new position with `--with-track-number`);
- rewrite the track-number tags of kept files whose playlist position or the playlist length changed;
- download tracks that are new, missing on disk, or failed previously;
- delete files of tracks that were removed from the playlist. Only files recorded in the manifest are ever deleted.

Without a manifest (first run, or legacy export), a file already present at the expected location is adopted instead of being re-downloaded.

Downloads are written to a `<file>.part` temporary file and moved into place once complete, so an interrupted run never leaves a truncated file behind.

Changing `--with-playlist-id` or `--flat` changes the target directory, so the previous manifest is not found and the playlist is downloaded again into the new directory (the old directory is left untouched).

When a track has no local file on the server, or the server returns a non-2xx response, it is reported as `failed` with a warning. The rest of the playlist continues.

#### Examples

```bash
# Download (or synchronise) by numeric ID into ~/music/tekno
soundome library playlist download 1 --output ~/music/tekno

# Download by partial name, flat layout
soundome library playlist download "late night" --output /tmp/export --flat

# Legacy naming: ID-prefixed directory and numbered files
soundome library playlist download 3 --output /tmp/export --with-playlist-id --with-track-number

# Re-download everything
soundome library playlist download 3 --output /tmp/export --force

# Custom manifest path
soundome library playlist download 3 --manifest /tmp/export/report.json

# Point at a remote server
soundome --api-url http://192.168.1.10:8777 library playlist download 3
```

## How track download works

The CLI calls:

- `GET /api/playlists`
- `GET /api/playlists/:id/tracks`
- `GET /api/tracks/:id/download` (for each track)
- `GET /api/tracks`, `GET /api/albums`, `GET /api/artists` (for `library search`)

Track downloads are streamed chunk by chunk to disk, with a byte-level progress bar.

After a track is saved, the CLI rewrites the track-number metadata so that `track_number` matches the playlist position (and sets the total track count to the playlist length). During a sync, kept files are re-tagged only when their position or the playlist length changed.

Tracks that are not yet finalized (no `file_path` in the database) or whose audio file is missing on the server are reported as skipped.

## Current limitations

- Search is client-side filtering after API fetch (no server-side pagination yet).
- Sync relies on the manifest and file existence only (no checksum/version comparison): a track whose audio was replaced on the server is not re-downloaded unless `--force` is used.
- Authentication is not implemented — the server is assumed to be trusted.
