# Soundome — Copilot instructions (global)

Ce dépôt est un monorepo Rust pour un programme qui a pour but de **centraliser, télécharger, enrichir (metadata), tagger et organiser** une bibliothèque musicale personnelle à partir de sources comme **Spotify / SoundCloud / YouTube / YouTube Music**.

Objectif d’assistance : produire des changements **petits, sûrs et cohérents** avec l’architecture existante, en gardant en tête que le projet est **WIP** (certaines docs sont partielles/dépréciées).

## Objectifs du projet

- Créer une bibliothèque musicale unifiée à partir de sources diverses
- Minimiser l'intervention manuelle dans la gestion de la bibliothèque
- Maintenir une organisation cohérente et une qualité optimale
- Enrichir automatiquement les métadonnées des pistes
- Interface web de validation manuelle pour les cas ambigus

## Ce que fait Soundome plus ou moins actuellement (fonctionnel)

- Entrée : URL de track / playlist, ou fichiers locaux (ingest à venir)
- Étapes typiques :
  - **Fetch** des métadonnées source (titre, artistes, album, refs)
  - **Nettoyage** (notamment SoundCloud)
  - **Enrichissement metadata** via MusicBrainz (matching + score)
  - **Download** via un provider (souvent YouTube/YT Music en “provider” même si la source est Spotify)
  - **Déduplication** (URL, puis similarité, puis qualité)
  - **Tagging** du fichier audio
  - **Organisation filesystem** `Artist/Album/Track` + persistance DB

Réfs de workflow :
- docs/workflows/download.scenarios.md
- docs/workflow.d2 (diagramme)
- docs/specs.md (objectifs/contraintes)

## Architecture (technique)

Workspace Rust (Cargo workspace) :
- apps/server : API Rocket + OpenAPI/Swagger (WIP)
- apps/cli : CLI (actuellement minimal/WIP)
- packages/domain : services + orchestrateur (DownloadService)
- packages/fetcher : “sources” (Spotify/YT Music/SoundCloud)
- packages/downloader : “providers” (YouTube/YT Music/SoundCloud)
- packages/tagger : tagging / metadata providers (MusicBrainz)
- packages/organizer : organisation filesystem
- packages/database : Diesel + SQLite, repositories
- packages/config : config TOML + env overlay + singleton global
- packages/shared : types, erreurs, modèles, utils (HTTP proxy rotator, logging…)
- packages/ai : client IA (OpenRouter) + prompts (principalement nettoyage metadata SoundCloud)

Points d’entrée code utiles :
- apps/server/src/main.rs : bootstrap Rocket, init globals, injection services/repositories
- packages/domain/src/services/download_service.rs : workflow principal (download/sync playlist)
- packages/shared/src/models/* : Track/Album/Artist/Reference + logique compare/transpose
- packages/shared/src/libs/http.rs : ProxyRotator + HttpClientBuilder
- packages/database/src/repositories/* : sémantique “set_references” et transactions

## Données & références

- Le modèle `Reference` (shared) utilise `ReferenceType` : `Source`, `Provider`, `Metadata`, `Reference`.
- Pattern important :
  - `Source/Provider` représentent l’audio réellement utilisé (souvent **remplacés**)
  - `Metadata` conserve des identifiants/URLs utiles (MusicBrainz, Spotify…) (souvent **merge**)
- La logique DB correspondante existe côté repos Diesel (ex: track `set_references`).

## Config / runtime

- Config : `packages/config` charge `config.toml` (par défaut `./config.toml`) et peut être surchargé via env `SOUNDOME_CONFIG_PATH` + `SOUNDOME__...`.
- Initialisation globale attendue au démarrage : `shared::init_globals()` (Config + ProxyRotator).
- Server/CLI utilisent `dotenvy` avec `required = true` → prévoir un `.env` local si nécessaire.
- Rocket DB : voir `Rocket.toml` (SQLite `data/soundome.db`).
- Diesel : `diesel.toml` pointe vers `packages/database/migrations` et `packages/database/src/schema.rs`.

## Proxy réseau

- Support proxy : `Config.proxy` + `ProxyRotator` global.
- Utiliser préférentiellement `shared::libs::http::HttpClientBuilder` (plutôt que créer un `reqwest::Client` ad hoc).
- Docs : docs/proxy-configuration.md et docs/proxy-usage-example.md.
- Limitation : certaines libs tierces n’acceptent pas nativement la config proxy (voir doc proxy).

## WIP / attentes

- Beaucoup de features sont incomplètes (routes server commentées, CLI vide, fetchers/downloaders partiels).
- Ne pas “inventer” des comportements : si une partie manque, proposer un scaffold minimal cohérent avec l’existant.
- Préférer des PRs petites et testables (migrations Diesel, routes Rocket, services domain).

## Conventions de code (résumé)

- Langue utilisée : anglais pour le code, les commentaires et la documentation.
- Erreurs : utiliser `shared::types::SoundomeResult<T>` et `shared::errors::Error`.
- Logs : `tracing` (pas `println!` sauf boot fatal).
- Éviter les `unwrap()`/`expect()` hors init/boot ; préférer propagation d’erreur.
- Réutiliser la structure “ports/repositories” (traits) + “database/repositories” (Diesel impl).

## Commandes utiles (dev)

- Workspace : `cargo test -q` ; `cargo clippy --workspace --all-targets` ; `cargo fmt`.
- Diesel (SQLite) : `cargo install diesel_cli --no-default-features --features sqlite` puis `diesel migration run`.
