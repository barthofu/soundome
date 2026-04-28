---
name: add-diesel-migration
description: 'Use when: adding or modifying SQLite schema, Diesel migrations, repository persistence, schema.rs, entities, mappers, or repository implementations. Safely update migrations and keep Soundome reference semantics intact.'
argument-hint: 'Describe the schema change, affected tables, and expected behavior.'
---

# Add Diesel Migration

Use this skill when you need to add, modify, or debug a Diesel migration in Soundome.

## When to use

- add a new table or column
- rename or reshape persisted fields
- update Diesel repositories after a schema change
- debug mismatches between migrations, `schema.rs`, and repository code

## Repository rules

- Migrations live under `packages/database/migrations/`.
- The generated schema lives in `packages/database/src/schema.rs`.
- Repository traits are defined in `packages/domain/src/ports/repositories`.
- Preserve repository semantics such as `set_references`: replace `source/provider`, merge durable metadata references.

## Procedure

1. Start from the narrowest schema need: table, column, index, or constraint.
2. Identify the affected repository traits and Diesel implementations.
3. Add or update the migration with both `up.sql` and `down.sql`.
4. Regenerate or update `packages/database/src/schema.rs` if the change requires it.
5. Update entities, mappers, repository structs, and transaction boundaries together.
6. Check whether the change touches track, album, artist, playlist, or reference persistence semantics.
7. Validate with the narrowest useful command first, then with a slightly wider DB-focused check if needed.

## Validation checklist

- migration shape matches the intended model
- `schema.rs` matches the database schema
- repositories compile against the updated schema
- transaction boundaries still cover multi-entity writes
- reference merge and replacement behavior is preserved

## Useful files

- `packages/database/migrations/`
- `packages/database/src/schema.rs`
- `packages/database/src/repositories/`
- `packages/domain/src/ports/repositories/`
