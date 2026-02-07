---
applyTo: "packages/tagger/src/**/*.rs"
---

# Tagger — metadata & fichiers audio

## Rôle

- Providers metadata (MusicBrainz) et tagging des fichiers audio via `tagger::file`.

## Conventions

- Le tagging doit être idempotent autant que possible.
- En cas de match partiel / douteux, remonter une structure exploitable (pour une future validation web).
