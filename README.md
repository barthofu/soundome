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

---

## Monorepo Structure

```bash
soundome/
├── apps/
│ ├── cli/ # Command-line interface for automation
│ └── server/ # Web admin interface (API + frontend)
├── packages/
│ ├── orchestrator/ # Coordinates workflows across the system
│ ├── fetcher/ # Fetch metadata and track references from sources
│ ├── downloader/ # Match and download tracks from providers
│ ├── tagger/ # Apply metadata to audio files
│ ├── database/ # Persistent layer (Diesel + SQLite)
│ ├── config/ # Centralized configuration manager
│ └── shared/ # Shared domain types and utilities
```

---

## Tech Stack

- 🦀 **Rust** — fast, safe, and reliable
- 🗃️ **Diesel** + **SQLite** — simple and robust database layer
- 🧱 **Rocket** + `rocket_okapi` — backend API with OpenAPI support
- 🧾 **Symphonia**, **id3**, **lofty** — for audio metadata handling
- 📦 Monorepo structure with domain-centric packages

---

## Roadmap

- [x] Core domain model (track, artist, album)
- [x] Diesel migrations + SQLite persistence
- [x] Config system with layered support
- [x] CLI workflows
- [ ] Web admin interface
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
```
3. Set up the database:
```bash
diesel setup
diesel migration run
```


