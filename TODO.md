# Todo

- [ ] voir pour renommer les bails de "tag" et "tagger" en "metadata"
- [ ] en plus du "transpose" des metadata, faire un "complete" qui vient simplement compléter les metadata manquantes
- [ ] Allow direct filesystem inspection and manual intervention without making the system unusable.
  - full-scan sync that compares the library with the database and identifies missing files, duplicates, and orphaned metadata. This can be a separate CLI command that runs on demand.
    - it should use the most recent and rich metadata (either db, filesystem, or both)
  - spec: docs/specs/library-filesystem-sync.md
- [ ] Export source playlists as M3U8 files so they are visible in Navidrome and other music applications
  - spec: docs/specs/playlist-m3u8-export.md

## CLI

### MVP

- [ ] Add machine-readable output modes (`--format table|json|jsonl`) for scripting and CI usage.
- [ ] Add search and filter commands for library entities (tracks, albums, artists, playlists).
- [ ] Add playlist sync mode (`library playlist download --sync`) to download only new tracks.
- [ ] Add export manifest output (`manifest.json`) with downloaded/skipped/failed entries.

### Phase 2

- [ ] Add manual validation commands (list pending, approve, reject) from the terminal.
- [ ] Add M3U8 export command from CLI with path strategy options.
- [ ] Add playlist utilities: diff between playlists and dated snapshot export.
- [ ] Add shell completion generation for bash/zsh/fish.

### Phase 3

- [ ] Add resumable downloads for interrupted playlist exports.
- [ ] Add maintenance commands: integrity check between DB and filesystem, and cleanup of temp/cache.
