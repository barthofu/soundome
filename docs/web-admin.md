# Web Admin Interface

Soundome ships a lightweight web admin panel served directly by the Rocket server.
It covers two main areas: **submitting downloads** and **reviewing pending validations**.

---

## Architecture

| Layer | Technology | Location |
|---|---|---|
| Backend API | Rocket 0.5 + `rocket_okapi` | `apps/server/src/routes/` |
| Frontend | Svelte 5 + Vite 8 + TypeScript | `apps/web/` |
| Build output | Static files served by Rocket | `data/web/` |

Rocket mounts the compiled SPA at `/` via `FileServer::from("data/web")` and exposes all API endpoints under `/api`.

---

## Running

```bash
# Development — hot-reload frontend + live Cargo watch
pnpm dev          # runs both concurrently

# Build only the frontend
pnpm web:build    # outputs to data/web/
```

---

## Pages

### Home (`/`)

- URL input form — accepts any Spotify, SoundCloud, YouTube or YouTube Music link (track or playlist)
- Auto-detects track vs. playlist via URL pattern matching
- Shows download feedback (success / error)
- "Recent downloads" section — last 20 tracks, polled once on load and refreshed after each download (cover, title, artists, album, duration, "review" badge if pending)

### Validations (`/validations`)

Lists all tracks flagged `needs_validation = true`.
The navbar badge shows the pending count and refreshes every 30 seconds.

Each track card displays:
- Cover, title, artists, album, genre, date, duration, track/disc number
- `validation_reason` (e.g. `musicbrainz_partial_match`)
- Staging file path

**Actions per track:**

| Button | Behaviour |
|---|---|
| **Edit** | Opens inline metadata editor (all editable fields) |
| **Approve** | Applies edits (if any), tags the staged file, moves it to the library, clears `needs_validation` |
| **Reject** | Deletes the track from the DB (staged file is not cleaned up automatically) |

The track disappears from the list immediately after either action, and the navbar count updates.

---

## API Routes

All routes are documented via Swagger at `/swagger`.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/validations` | List tracks pending validation |
| `PATCH` | `/api/validations/:id` | Approve — apply patch + finalize (download→tag→organize) |
| `DELETE` | `/api/validations/:id` | Reject — delete track from DB |
| `GET` | `/api/tracks/recent?limit=N` | Last N tracks (default 20, max 100) |
| `POST` | `/api/download` | Submit a URL for download |

---

## Validation Workflow

When MusicBrainz returns a partial or no match during the download pipeline, the track is flagged for manual review. The file is **always downloaded to the staging folder** (`temp_download_dir` in `config.toml`) immediately, so the audio is available regardless of how long the track waits in the queue.

```
fetch metadata
    ↓
MusicBrainz enrichment
    ↓
Always download → staging (temp_download_dir/)
    ↓
Exact match?
 ├── yes → tag + move to library → done
 └── no  → save to DB with needs_validation=true
                    ↓
            User approves via web UI
                    ↓
            Apply metadata patch
                    ↓
            Tag staged file + move to library
                    ↓
            needs_validation = false → done
```

This ensures that a track waiting for validation is never at risk of becoming unavailable on the provider.

---

## Frontend File Structure

```
apps/web/
├── src/
│   ├── App.svelte          # Router + navbar (pending badge)
│   ├── lib/
│   │   ├── api.ts          # Typed API helpers
│   │   ├── types.ts        # Shared TypeScript interfaces
│   │   └── TrackCard.svelte  # Track display + inline edit + approve/reject
│   └── pages/
│       ├── Home.svelte       # Download form + recent tracks
│       └── Validations.svelte # Pending validation list
├── vite.config.ts            # outDir → data/web/, /api proxy → :8000
└── package.json
```
