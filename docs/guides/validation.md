# Manual validation

When Soundome cannot confidently determine the correct metadata for a track, it saves the track to the database but flags it as `needs_validation = true`. The audio file is staged (downloaded but not yet moved to the library). Nothing is lost — the track just waits for your decision.

## Why tracks end up in the validation queue

| Reason | What it means |
|---|---|
| `metadata_partial_match` | Enrichment found a likely match but the similarity score was below the exact-match threshold. The metadata may be mostly correct but uncertain. |
| `metadata_no_match` | No enrichment provider returned any usable match. The track uses whatever metadata the source provided. |
| `soundcloud_drm_protected` | SoundCloud's API refused to provide a download URL. Soundome already tried an automatic fallback (downloading via a known Spotify metadata match through YouTube/YouTube Music) — this reason means that fallback was unavailable or also failed. No audio file has been staged yet. |

## The Validations page

Open the **Validations** tab in the web UI. Each row shows:

- Cover art (if available), title, artists, album
- The enrichment data Soundome found: genre, date, track number, disc number
- The staged file path
- The validation reason

### Editing metadata directly

Click any field to edit it inline. You can correct the title, artists, album, genre, date, or track number before approving.

### Finding a better metadata match

Click **Show matches** to re-query all enabled metadata providers (MusicBrainz, Bandcamp, Spotify) and see scored candidates. Each candidate shows:

- Title, artists, album, release date
- A similarity score (0–1)
- Links to the source record

Click a candidate to populate the form with its values, then review and approve.

### Resolving DRM-protected SoundCloud tracks

For tracks with reason `soundcloud_drm_protected`, there is no staged audio file yet. Click **Show YouTube candidates** to search YouTube Music and YouTube for matching audio. A list of results appears; click one to select it as the download source.

Alternatively, paste any YouTube or YouTube Music URL into the provider URL field manually.

Once a source is selected, clicking **Approve** downloads the audio from that URL, tags it with the metadata in the form, moves it to the library, and clears the validation flag.

### Approving a track

After confirming the metadata looks correct (editing fields if needed), click **Approve**. Soundome will:

1. Apply the metadata values from the form
2. Tag the staged audio file
3. Move the file to `<base_library_dir>/<Artist>/<Album>/<filename>`
4. Save the final track to the database
5. Remove the `needs_validation` flag

The track disappears from the Validations tab and appears in your library.

### Rejecting a track

Click **Reject** to permanently delete the database row and the staged audio file. Use this when a track was downloaded by mistake or is a duplicate you do not want.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `E` | Edit selected track |
| `Enter` | Save edits |
| `Esc` | Cancel edits |
| `Backspace` | Go back |

## Bulk workflow tips

When coming back to a large validation queue (e.g., after syncing a messy SoundCloud playlist), a practical order is:

1. **Filter by reason** — handle all `soundcloud_drm_protected` tracks first using the YouTube candidate flow, as they require a different action than metadata mismatches.
2. **Use "Show matches"** for `metadata_partial_match` tracks — most will have an obvious correct candidate.
3. **Edit manually** for `metadata_no_match` tracks — especially for very new or niche releases not yet in MusicBrainz.

Enabling AI metadata cleanup ([guides/ai-metadata.md](ai-metadata.md)) before syncing SoundCloud content significantly reduces the number of `metadata_no_match` tracks in the queue.

## API reference

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/validations` | List all pending tracks |
| `PATCH` | `/api/validations/:id` | Approve: apply metadata patch and finalise |
| `DELETE` | `/api/validations/:id` | Reject: delete row and staged file |
| `GET` | `/api/validations/:id/matches` | Re-query metadata providers for candidates |
| `GET` | `/api/validations/:id/youtube-candidates` | Search YouTube and YouTube Music for DRM fallback |
