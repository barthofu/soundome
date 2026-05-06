# Spec: Library Filesystem Sync (bidirectional binding)

## Problem statement

Soundome finalizes a track by writing audio tags, moving the file to `Artist/Album/Track`, and persisting the path in the database. After that point, the binding between the database row and the file is fragile: it relies solely on the stored file path.

External tools (beets, MusicBrainz Picard, manual edits) can:
- rename or move a file without Soundome's knowledge
- update tags so they no longer match the database
- delete a file

Currently there is no mechanism to detect or recover from any of these. The database can silently drift from reality.

The goal of this spec is to define the two-direction binding contract and a viable reconciliation strategy.

---

## Current state

| Direction | Status |
|---|---|
| Soundome → file | Implemented: tags written, file placed, path persisted |
| File → Soundome | Not implemented: no anchor written to files, no scan, no drift detection |

---

## Design: the anchor strategy

The root problem is **identity across renames**. Without an identifier embedded in the file itself, a renamed file looks like a new unknown file and the old DB row looks like a dangling reference.

### Primary anchor: `SOUNDOME_ID` custom tag

Soundome should write a UUID into every finalized file using a custom tag frame:

| Format | Field |
|---|---|
| MP3 / ID3 | `TXXX:SOUNDOME_ID` |
| FLAC / Vorbis | `SOUNDOME_ID` |
| MP4 / M4A | `----:com.soundome:ID` |

This UUID is generated once at finalization and stored in the `tracks` table as a new column (e.g. `soundome_id UUID NOT NULL UNIQUE`). It is never changed even if metadata is updated.

### Secondary anchor: MusicBrainz Recording ID

For files where `SOUNDOME_ID` is absent (pre-existing library files, files touched by Picard), the scanner can fall back to the MusicBrainz Recording ID tag (`MUSICBRAINZ_TRACKID` / `UFID`), which Soundome already enriches. This allows partial reconciliation without requiring a re-tag pass.

### Implication for `audiotags`

The current `audiotags` crate does not expose arbitrary custom frames. Writing `TXXX` / Vorbis comments / freeform MP4 atoms requires direct use of `id3`, `metaflac` / `claxon`, or `mp4ameta`. This is a known limitation that affects implementation scope (see risks section).

---

## Two-direction binding contract

### Direction 1 — Soundome → file (finalization, already implemented, to extend)

At finalization (`tag_file_with_track`):
1. Generate a new UUID if the track does not yet have a `soundome_id`.
2. Write `SOUNDOME_ID` as a custom tag (see format table above).
3. Persist the UUID in the `tracks` table.
4. Write all other standard tags as today.

This is a non-breaking extension. Files finalized before this feature are missing the tag but remain usable with secondary anchoring.

### Direction 2 — file → Soundome (scan, new)

A new scan command walks the library directory and reconciles the filesystem against the database. It does not run automatically; it is triggered explicitly (CLI command or API call).

#### Scan algorithm

```
for each audio file in the library directory:
    read SOUNDOME_ID from file tags
    if SOUNDOME_ID is present:
        look up the DB row by soundome_id
        if found:
            compare stored file_path with actual path
            → if different: candidate for path update (file was moved/renamed)
            compare stored tags (title, artists, album) with file tags
            → if different: candidate for tag conflict
        if not found:
            mark as "orphan with known origin" (was managed by Soundome, row deleted or DB wiped)
    else:
        attempt lookup by MusicBrainz Recording ID if present
        if matched:
            mark as "legacy match" (managed but pre-dates SOUNDOME_ID feature)
        else:
            mark as "unmanaged" (unknown to Soundome — may be from external tool)

for each DB row with status = finalized and a stored file_path:
    check if the file exists at file_path
    if not found:
        mark as "missing" (could not be located by path)
        attempt to match against orphans or legacy matches found in the walk above
```

#### Scan result categories

| Category | Meaning | Suggested action |
|---|---|---|
| `ok` | File found at expected path, tags consistent | No action needed |
| `path_changed` | File found via `SOUNDOME_ID` but at a different path | Update `file_path` in DB |
| `tag_conflict` | File found but tags diverge from DB | Flag `needs_validation` with reason `file_tag_divergence`; do not auto-resolve |
| `missing` | DB row exists but file not found anywhere | Mark as `file_missing`; leave row intact for recovery |
| `orphan` | File has `SOUNDOME_ID` but no matching DB row | Log; optionally offer re-import |
| `legacy_match` | File matched via MusicBrainz ID, no `SOUNDOME_ID` | Offer a re-tag pass to write `SOUNDOME_ID` |
| `unmanaged` | File exists in library dir but unknown to Soundome | Ignore or offer external-ingest (out of scope for now) |

---

## Conflict resolution policy

**Soundome does not auto-resolve tag conflicts.** When file tags diverge from the database, the track is flagged `needs_validation = true` with `validation_reason = "file_tag_divergence"`. The manual validation UI already handles this path: the user can approve (accepting the file's tags as ground truth) or reject.

This is safe because:
- the file is not moved or re-tagged automatically
- the existing validation flow is re-used without modification

The only automatic mutations permitted by the scanner are:
- updating `file_path` when a `path_changed` match is unambiguous (single match, same `SOUNDOME_ID`)
- writing `SOUNDOME_ID` to a `legacy_match` file during an explicit re-tag pass

---

## Implementation plan

### Phase 1 — anchor at finalization (smallest viable change)

1. **Migration**: add `soundome_id TEXT UNIQUE` to `tracks` table (nullable initially to ease migration).
2. **Model**: add `soundome_id: Option<String>` to `shared::models::Track`.
3. **Tagger**: in `tag_file_with_track` (and its wrapper in `packages/tagger/src/file.rs`), write `SOUNDOME_ID` as a custom tag. Requires direct `id3` / Vorbis / MP4 calls alongside `audiotags`.
4. **Domain**: generate UUID at finalization in `DownloadService` before calling the tagger; persist it.

This phase alone closes the identity gap for all future finalizations.

### Phase 2 — scan command

1. **Scanner service**: new module `packages/domain/src/services/scan_service.rs` implementing the algorithm above. Outputs a `ScanReport` struct listing results by category.
2. **CLI**: `soundome scan [--library-root <path>] [--dry-run]` via `apps/cli`.
3. **API**: `POST /api/library/scan` returning a JSON `ScanReport`. Marked as best-effort (long-running; consider async job pattern already used for playlist sync).
4. **Auto path update**: after scan, for unambiguous `path_changed` entries, apply the path update automatically (or surface it in the report for the user to confirm if `--dry-run`).
5. **Validation bridge**: for `tag_conflict` entries, insert a `needs_validation` record pointing to the real file path and carry the file's tag values as the candidate metadata.

### Phase 3 — re-tag pass for legacy files (optional, deferred)

A separate `soundome retag --legacy` command walks `legacy_match` files and writes `SOUNDOME_ID` without changing any other tag. This is a convenience pass and is explicitly not part of the core reconciliation spec.

---

## Risks and open questions

- **`audiotags` limitation**: the crate wraps only standard frames. Writing `TXXX` requires direct `id3` crate use. The tagger already depends on `id3` transitively; the question is whether to expose it or switch partially to a lower-level approach. Recommendation: keep `audiotags` for standard tags; add a thin wrapper for custom frame writes.
- **Files outside the library root**: the scanner is scoped to the configured `library_root`. Files placed outside it are not scanned and are invisible to Soundome.
- **Performance**: walking a large library is I/O intensive. The scan should stream results and avoid loading the full library into memory. For large libraries, index the DB result by `soundome_id` into a `HashMap` before the walk.
- **Concurrent modification**: the scan is read-only by default (dry-run semantics unless mutations are explicitly requested). This prevents a scan from conflicting with an ongoing download.
- **SQLite locking**: path updates and validation inserts during scan should be batched in a single transaction per scan run.
