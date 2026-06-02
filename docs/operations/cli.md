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

### `library playlist list`

List all playlists in the library.

```bash
soundome library playlist list
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
soundome library playlist download <playlist> [--output <dir>] [--flat]
```

| Argument / flag | Description |
|---|---|
| `<playlist>` | Numeric playlist ID or a partial name (case-insensitive). If several playlists match the name, an interactive picker is shown. |
| `--output <dir>` | Destination directory. Created automatically if it does not exist. Defaults to the current directory. |
| `--flat` | Write files directly into the output directory, without creating a playlist sub-directory. |

#### Default layout (without `--flat`)

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

# Point at a remote server
soundome --api-url http://192.168.1.10:8000 library playlist download 3
```

## How track download works

The CLI calls `GET /api/tracks/:id/download` for each track in the playlist. The server reads the track's `file_path` from the database and serves the file directly. The CLI streams the response to disk chunk by chunk and updates a byte-level progress bar in real time.

After a track is saved, the CLI rewrites the track-number metadata so that `track_number` matches the playlist position (and sets the total track count to the playlist length).

Tracks that are not yet finalized (no `file_path` in the database) or whose audio file is missing on the server are reported as skipped.

## Current limitations

- Only playlist download is implemented. Track-level and album-level commands are not yet available.
- Authentication is not implemented — the server is assumed to be trusted.
