# Todo

- [ ] voir pour renommer les bails de "tag" et "tagger" en "metadata"
- [ ] en plus du "transpose" des metadata, faire un "complete" qui vient simplement compléter les metadata manquantes
- [ ] Allow direct filesystem inspection and manual intervention without making the system unusable.
  - full-scan sync that compares the library with the database and identifies missing files, duplicates, and orphaned metadata. This can be a separate CLI command that runs on demand.
    - it should use the most recent and rich metadata (either db, filesystem, or both)
  - spec: docs/specs/library-filesystem-sync.md
- [ ] Export source playlists as M3U8 files so they are visible in Navidrome and other music applications
  - spec: docs/specs/playlist-m3u8-export.md
