---
description: "Use when: editing metadata enrichment providers, audio tagging behavior, or file-tag writing logic in packages/tagger."
applyTo: "packages/tagger/src/**/*.rs"
---

# Tagger — metadata and audio files

## Role

- Provide metadata enrichment, mainly through MusicBrainz, and tag audio files through `tagger::file`.

## Conventions

- Keep tagging as idempotent as possible.
- When a match is partial or uncertain, return a structure that can feed the web validation flow.
