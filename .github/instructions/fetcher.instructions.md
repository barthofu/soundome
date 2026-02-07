---
applyTo: "packages/fetcher/src/**/*.rs"
---

# Fetcher — sources (Spotify/YT Music/SoundCloud)

## Rôle

- Convertir une URL source en `shared::models::*` + `ReferenceType::Source`.
- Fournir des helpers de “clean” metadata (ex SoundCloud : titre/artistes).

## Conventions

- Si un provider n’est pas initialisable (credentials manquants, etc.), retourner `Error::ProviderUnavailable(Platform)`.
- Garder `is_valid_*_url` strict (ne pas accepter de faux positifs).
- Pour tout HTTP direct, utiliser le client proxy-aware (shared http builder) si possible.
