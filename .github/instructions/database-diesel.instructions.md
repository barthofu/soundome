---
applyTo: "packages/database/src/**/*.rs"
---

# Database (Diesel + SQLite)

## Sources de vérité

- Migrations : `packages/database/migrations/`
- Schema généré : `packages/database/src/schema.rs` (voir diesel.toml)

## Repositories

- Les repos implémentent les traits dans `packages/domain/src/ports/repositories`.
- Conserver la sémantique existante : ex `set_references` remplace `source/provider` et merge le reste.

## Transactions

- Pour les opérations composées (track + album + artists + refs), utiliser `conn.transaction(|tx| { ... })`.

## Tips

- Ne pas faire remonter des erreurs Diesel brutes : mapper vers `shared::errors::Error::Database`.
