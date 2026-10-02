# Soundome MCP server

`apps/mcp` (binary `soundome-mcp`) exposes Soundome to LLM clients through the
[Model Context Protocol](https://modelcontextprotocol.io). Like the CLI, it is a pure
HTTP client of the Rocket API (`<api-url>/api/...`): it never touches the database or the
domain layer, so the `DownloadService` workflow and staged-file invariants stay owned by the server.

The MCP protocol layer (JSON-RPC 2.0: `initialize`, `ping`, `tools/list`, `tools/call`) is
implemented in-crate (`src/server.rs`) with no MCP SDK dependency.
// TODO: consider migrating to the official `rmcp` SDK once the tool surface stabilizes.

## Build & run

```bash
cargo build -p soundome-mcp --release

# stdio (default) — launched by the MCP client
soundome-mcp --api-url http://localhost:8777

# streamable HTTP (JSON responses) on POST /mcp
SOUNDOME_MCP_TOKEN=change-me soundome-mcp --transport http --bind 127.0.0.1:8778
```

| Option | Env | Default |
|---|---|---|
| `--transport stdio\|http` | — | `stdio` |
| `--api-url` | `SOUNDOME_API_URL` | `http://localhost:8777` |
| `--bind` (HTTP) | `SOUNDOME_MCP_BIND` | `127.0.0.1:8778` |
| `--http-token` (HTTP) | `SOUNDOME_MCP_TOKEN` | none |

The HTTP transport checks `Authorization: Bearer <token>` when a token is set, and refuses to
start on a non-loopback address without one. The Rocket API itself has no authentication, so
keep the MCP endpoint behind that token (and TLS when remote). Logs go to stderr.

## Client configuration (OpenCode)

```jsonc
{
  "mcp": {
    "soundome": {
      "type": "local",
      "command": ["target/release/soundome-mcp", "--api-url", "http://localhost:8777"]
    }
    // or remote:
    // "soundome-remote": { "type": "remote", "url": "http://127.0.0.1:8778/mcp",
    //                      "headers": { "Authorization": "Bearer change-me" } }
  }
}
```

## Tools

- **Status**: `get_status`
- **Tracks**: `list_tracks`, `get_recent_tracks`, `get_track`, `get_track_references`, `add_track_reference`, `remove_track_reference`, `update_track`, `merge_tracks`, `delete_track`
- **Albums**: `list_albums`, `get_album`, `get_album_tracks`, `update_album`, `merge_albums`, `delete_album`
- **Artists**: `list_artists`, `get_artist`, `get_artist_tracks`, `get_artist_albums`, `update_artist`, `merge_artists`, `delete_artist`
- **Playlists**: `list_playlists`, `get_playlist_tracks`, `export_playlist`, `delete_playlist`
- **Validations**: `count_pending_validations`, `list_pending_validations`, `get_validation_matches`, `get_validation_youtube_candidates`, `approve_validation`, `reject_validation`
- **Data quality** (`data_quality_*`): `get_structural_findings`, `get_duplicate_groups`, `get_ignored_duplicates`, `ignore_duplicate`, `restore_ignored_duplicate`, `start_remote_audit`, `get_remote_audit`, `apply_remote_audit_name`, `dismiss_remote_audit`, `delete_audited_reference`, `get_orphans`, `cleanup_orphans`, `get_playlist_issues`, `renumber_playlist`, `get_ai_cleanup_log`
- **Import & tasks**: `import_url`, `list_tasks`, `get_task`, `cancel_task`, `retry_task`

Tools are declared in `apps/mcp/src/tools.rs` as a data table mapping each tool to one API route.
Read-only tools carry `readOnlyHint`; deletes, merges, `reject_validation` and `cancel_task`
(plus `data_quality_delete_audited_reference` and `data_quality_cleanup_orphans`)
carry `destructiveHint` so clients can ask for confirmation. Responses over ~60k characters are truncated.

## Adding a tool

Add one `http(...)` entry to `catalog()` in `tools.rs` (path placeholders are `{id}`-style and
must be integers or short slugs such as `artists`), then update the list above.
