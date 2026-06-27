# Specifications

This document defines the functional intent of Soundome. It is deliberately narrower than a full roadmap: it describes the expected product shape, the main constraints, and the boundaries that should guide implementation decisions.

## Product goal

Soundome exists to build and maintain a coherent personal music library from heterogeneous inputs such as Spotify, SoundCloud, YouTube, YouTube Music, and local files.

The target experience is:

1. Accept a track, playlist, artist source, or local file.
2. Resolve enough metadata to identify the music reliably.
3. Download or ingest the best available audio.
4. Tag the file consistently.
5. Organize it in a predictable filesystem layout.
6. Persist enough references to avoid duplicate work and keep source provenance.

## Core functional requirements

### Ingestion

- Import from source URLs and local files.
- Support both single-track and playlist-oriented workflows.
- Keep the original source references even when the audio is downloaded from a different provider.

### Metadata enrichment

- Normalize noisy source metadata before matching.
- Enrich tracks, albums, and artists through metadata providers such as MusicBrainz.
- Keep partial or uncertain matches reviewable instead of silently finalizing bad metadata.

### Download and file handling

- Download audio to a temporary staging directory first.
- Finalize the file only after metadata is good enough or manually approved.
- Keep the best-quality version when duplicates are detected.

### Library management

- Organize the library with a stable `Artist/Album/Track` layout.
- Preserve playlist provenance in the database.
- Allow direct filesystem inspection and manual intervention without making the system unusable.

### Persistence

- Store library entities and references in a relational database.
- Preserve enough external identifiers and URLs to deduplicate future imports.

### Manual review

- Expose a web workflow for tracks that need manual validation.
- Let the user adjust metadata before the staged file is tagged and moved.

## Constraints

- Prefer safe automation over aggressive guessing.
- Avoid duplicates at multiple levels: exact source match, metadata match, and quality comparison.
- Keep the distinction between source metadata and actual downloaded audio.
- Stay compatible with a user-managed filesystem rather than hiding files behind a proprietary storage layer.
- Work incrementally: partial support for a platform is acceptable if failure modes remain explicit.

## Non-goals for the current codebase

- Full-featured CLI workflows.
- Audio fingerprinting-based deduplication.
- Fully automated genre inference.
- Complete proxy support across every third-party dependency.
- A finished ingest pipeline for arbitrary local files.

## Success criteria

The current project direction is successful when a user can submit a source URL, obtain a correctly tagged file in the library, avoid duplicate imports, and manually resolve ambiguous matches from the web interface without losing the staged audio file.
