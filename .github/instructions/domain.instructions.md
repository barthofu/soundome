---
applyTo: "packages/domain/src/**/*.rs"
---

# Domain — services & orchestration

## Rôle

- La crate `domain` orchestre les workflows (ex: `DownloadService`).
- Les services doivent rester indépendants de Rocket.

## DB

- Les méthodes publiques de service acceptent en général `&mut diesel::SqliteConnection`.
- Encapsuler les écritures multi-étapes dans une transaction (voir `TrackService::create_or_update`).

## Références

- Respecter la sémantique `ReferenceType` :
  - `Source`/`Provider` = audio effectif, souvent “replace”
  - `Metadata` = conserver des IDs/URLs (MusicBrainz/Spotify/etc), souvent “merge”
- Quand on dédupe par qualité, fusionner proprement les refs au lieu d’écraser.

## WIP

- Quand une étape est TODO (validation manuelle, playlist sync complet), garder le comportement actuel et ajouter du code derrière un flag / endpoint explicite si besoin.
