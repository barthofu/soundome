---
applyTo: "**/*.rs"
---

# Soundome Rust (global)

## Principes

- Favoriser des changements **minimaux** et cohérents avec les crates existantes.
- Utiliser `shared::types::SoundomeResult<T>` + `shared::errors::Error` pour les erreurs.
- Logging via `tracing::{info,warn,error,debug}`.
- Éviter les panics (`unwrap/expect`) sauf en init/boot.

## Patterns du repo

- Config globale : `config::Config::get()` (assumer `shared::init_globals()` appelé en amont).
- Réseaux : préférer `shared::libs::http::HttpClientBuilder` (proxy/rotation).
- Domain layer : services appellent les repositories via traits (`packages/domain/src/ports/repositories`).

## Style

- Garder les APIs publiques stables quand possible.
- N’ajouter une dépendance Cargo que si nécessaire, et l’ajouter au bon crate.
- Préférer des fonctions pures/side-effects isolés (I/O, DB, HTTP).
