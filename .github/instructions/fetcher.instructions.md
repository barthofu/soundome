---
description: "Use when: editing source adapters, source URL validation, source metadata extraction, or metadata cleanup logic in packages/fetcher."
applyTo: "packages/fetcher/src/**/*.rs"
---

# Fetcher — sources (Spotify/YT Music/SoundCloud)

## Role

- Convert a source URL into `shared::models::*` values plus `ReferenceType::Source` references.
- Provide metadata-cleaning helpers, for example for SoundCloud titles and artist names.

## Conventions

- If a provider cannot be initialized, for example because credentials are missing, return `Error::ProviderUnavailable(Platform)`.
- Keep `is_valid_*_url` strict and avoid false positives.
- For direct HTTP calls, use the proxy-aware shared HTTP builder when possible.
