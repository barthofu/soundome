# Soundome

**Soundome** is a self-hosted tool that centralizes, downloads, tags, and organizes your music library from multiple platforms such as **Spotify**, **SoundCloud**, and **YouTube**. It ensures a clean, structured, and metadata-rich collection — all fully under your control.

---

## Features

- 🔁 Sync playlists, artists from multiple platforms
- 📥 Download audio files from Spotify, SoundCloud, YouTube (and more to come)
- 🏷️ Automatically tag and organize tracks by artist, album, etc.
- 🧠 Detect and remove duplicates intelligently
- 🧾 Keep metadata on original source (platform + playlist)
- ❓ Web panel to review and manually tag unknown files
- 📁 Organize files physically (`Artist/Album/Track`) and maintain symbolic links for playlists
- 🌐 **Proxy support** for enterprise environments and geo-restrictions

---

## Monorepo Structure

```
soundome/
├── apps/
│   ├── cli/          # Command-line interface (WIP)
│   ├── server/       # Rocket API + OpenAPI/Swagger
│   └── web/          # Svelte 5 admin SPA (served by the server)
├── packages/
│   ├── domain/       # Core services & orchestrator (DownloadService)
│   ├── fetcher/      # Source adapters (Spotify, SoundCloud, YouTube Music)
│   ├── downloader/   # Provider adapters (YouTube, YT Music, SoundCloud)
│   ├── tagger/       # Audio file tagging + MusicBrainz provider
│   ├── organizer/    # Filesystem organisation (Artist/Album/Track)
│   ├── database/     # Diesel + SQLite repositories
│   ├── config/       # TOML config + env overlay singleton
│   ├── ai/           # OpenRouter client + prompts (metadata cleaning)
│   └── shared/       # Domain types, errors, HTTP proxy rotator, logging
```

---

## Tech Stack

- 🦀 **Rust** — fast, safe, and reliable
- 🗃️ **Diesel** + **SQLite** — simple and robust database layer
- 🚀 **Rocket** + `rocket_okapi` — backend API with OpenAPI/Swagger support
- 🎨 **Svelte 5** + **Vite** + **TypeScript** — reactive admin SPA
- 🧾 **Symphonia**, **id3**, **lofty** — audio metadata handling
- 📦 Cargo workspace + pnpm workspace (monorepo)

---

## Web Admin Panel

Soundome includes a web admin panel (served by the Rocket server) for:

- Submitting download URLs (track or playlist)
- Reviewing and approving/rejecting tracks pending metadata validation
- Browsing recent downloads

See [docs/web-admin.md](docs/web-admin.md) for the full documentation.

---

## Roadmap

- [x] Core domain model (track, artist, album)
- [x] Diesel migrations + SQLite persistence
- [x] Config system with layered support
- [x] Download pipeline: fetch → enrich (MusicBrainz) → stage → tag → organize
- [x] Web admin interface (download, recent tracks, validation review)
- [ ] CLI workflows
- [ ] Audio fingerprinting for duplicate detection
- [ ] Smart tagging using ML
- [ ] Remote sync and backup options

---

## Requirements

- Rust (latest stable)
- SQLite (or another database if extended)
- `ffmpeg` (for audio conversion/muxing)
- API keys for platforms (Spotify, YouTube, etc.)

---

## Development Setup

1. Clone the repository:
```bash
git clone https://github.com/barthofu/soundome.git
cd soundome
```
2. Install dependencies:
```bash
make shell
cargo install diesel_cli --no-default-features --features sqlite
npm install -g pnpm
pnpm install
```
3. Set up the database:
```bash
diesel setup
diesel migration run
```

4. Configure the application:
```bash
cp config.example.toml config.toml
# Edit config.toml with your API keys and proxy settings if needed
```

5. Start the development environment:
```bash
pnpm dev   # starts Cargo watch (server) + Vite dev server (frontend) concurrently
```

The Rocket server runs on `http://localhost:8000` and the Vite dev server (with API proxy) on `http://localhost:5173`.

For proxy configuration in enterprise environments, see [docs/proxy-configuration.md](docs/proxy-configuration.md).

