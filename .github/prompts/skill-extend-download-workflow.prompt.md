---
name: "Skill: Extend download workflow"
description: "Étend `DownloadService` sans casser les invariants (refs, dédup, qualité, tagging)."
---

Tu étends le workflow de `packages/domain/src/services/download_service.rs`.

Invariants :
- Dédup URL en amont (track_ref external_url).
- References : `Source/Provider` = audio effectif, `Metadata` = identifiants conservés.
- Si meilleure qualité : remplacer l’audio et fusionner metadata.
- Si moins bonne qualité : supprimer le nouveau fichier et conserver les nouvelles metadata en `Metadata`.

Demande des infos si nécessaires :
- nouveau cas d’usage (playlist, ingest local, validation)
- comportement attendu en cas de match partiel MusicBrainz
