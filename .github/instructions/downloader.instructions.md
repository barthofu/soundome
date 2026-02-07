---
applyTo: "packages/downloader/src/**/*.rs"
---

# Downloader — providers (YouTube/YT Music/SoundCloud)

## Rôle

- Trouver un `ReferenceType::Provider` (URL/ID téléchargeable) puis télécharger l’audio.

## Conventions

- Ne pas muter les `Track` côté downloader (retourner `PathBuf` + refs à attacher côté domain).
- Respecter `Config.general.base_library_dir` et `Config.general.temp_download_dir`.
- S’appuyer sur les binaires fournis si nécessaire (voir `assets/bin/ffmpeg` et `assets/bin/yt-dlp`) et documenter les prérequis.
