# Web admin interface

Soundome ships a lightweight web admin panel served directly by the Rocket server. It currently focuses on two user-facing flows: submitting downloads and reviewing pending validations.

## Architecture

| Layer | Technology | Location |
|---|---|---|
| Backend API | Rocket 0.5 + `rocket_okapi` | `apps/server/src/routes/` |
| Frontend | Svelte 5 + Vite 8 + TypeScript | `apps/web/` |
| Build output | Static files served by Rocket | `data/web/` |

Rocket mounts the compiled SPA at `/` via `FileServer::from("data/web")` and exposes all API endpoints under `/api`.

## Running

```bash
# Development
pnpm dev

# Build only the frontend
pnpm web:build
```

## Pages

### Home (`/`)

- URL input form accepting Spotify, SoundCloud, YouTube, and YouTube Music links
- track-versus-playlist detection based on URL patterns
- download result feedback
- recent-download list refreshed after submissions

### Validations (`/validations`)

Lists all tracks flagged `needs_validation = true`.

Each card shows:

- cover, title, artists, album, genre, date, duration, and track numbers
- `validation_reason`
- staged file path

Available actions:

| Button | Behavior |
|---|---|
| **Edit** | Open inline metadata editing |
| **Approve** | Apply edits, tag the staged file, move it to the library, and clear the validation flag |
| **Reject** | Delete the track row from the database |

## API routes

All routes are documented through Swagger at `/swagger`.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/validations` | List tracks pending validation |
| `PATCH` | `/api/validations/:id` | Approve and finalize a pending track |
| `DELETE` | `/api/validations/:id` | Reject and delete a pending track |
| `GET` | `/api/tracks/recent?limit=N` | List the most recent tracks |
| `POST` | `/api/download` | Submit a track or playlist URL |
| `GET` | `/metrics` | Prometheus metrics (tracks, albums, artists, playlists, tasks by status) |

## Validation workflow

When MusicBrainz returns a partial or no match during the download pipeline, the track is saved for manual review after the audio has already been downloaded into staging.

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

This ensures that a track waiting for validation is not lost if the provider later becomes unavailable.

## Frontend file structure

```
apps/web/
├── src/
│   ├── App.svelte
│   ├── lib/
│   │   ├── api.ts
│   │   ├── types.ts
│   │   └── TrackCard.svelte
│   └── pages/
│       ├── Home.svelte
│       └── Validations.svelte
├── vite.config.ts
└── package.json
```
