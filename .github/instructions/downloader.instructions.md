---
description: "Use when: editing provider download code, provider reference resolution, temp file download behavior, or binary-backed downloader integrations in packages/downloader."
applyTo: "packages/downloader/src/**/*.rs"
---

# Downloader — providers (YouTube/YT Music/SoundCloud)

## Role

- Resolve a `ReferenceType::Provider` and download the audio.

## Conventions

- Do not mutate `Track` values inside downloader code. Return the `PathBuf` and any references to attach from the domain layer.
- Respect `Config.general.base_library_dir` and `Config.general.temp_download_dir`.
- Reuse bundled binaries when needed, such as `assets/bin/ffmpeg` and `assets/bin/yt-dlp`, and document prerequisites clearly.
