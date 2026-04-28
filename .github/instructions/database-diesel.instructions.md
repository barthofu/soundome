---
description: "Use when: editing Diesel repositories, SQLite persistence, schema.rs, database transactions, or migration-adjacent Rust code in packages/database."
applyTo: "packages/database/src/**/*.rs"
---

# Database (Diesel + SQLite)

## Sources of truth

- Migrations: `packages/database/migrations/`
- Generated schema: `packages/database/src/schema.rs` as configured by `diesel.toml`

## Repositories

- Repositories implement the traits defined in `packages/domain/src/ports/repositories`.
- Preserve the existing semantics: for example, `set_references` replaces `source/provider` and merges the rest.

## Transactions

- For composed operations such as track, album, artist, and reference updates, use `conn.transaction(|tx| { ... })`.

## Tips

- Do not bubble up raw Diesel errors. Map them to `shared::errors::Error::Database`.
