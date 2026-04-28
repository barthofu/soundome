---
name: reference-semantics
description: 'Use when: changing Track, Album, Artist, Playlist, Reference, ReferenceType, deduplication merge logic, transpose logic, or Diesel repository reference handling in Soundome.'
argument-hint: 'Describe the reference-related behavior to add, preserve, or fix.'
---

# Reference Semantics

Use this skill when changes affect how Soundome stores, merges, replaces, or interprets references.

## Core model

- `Source`: user-requested origin
- `Provider`: actual downloadable audio source
- `Metadata`: durable identifiers and URLs that should survive source changes
- `Reference`: supporting reference data

## Rules to preserve

- Replacing audio does not mean losing durable metadata identifiers.
- Deduplication and quality replacement must preserve future matchability.
- Repository code and model code must agree on reference semantics.
- `set_references` and related persistence logic are architecture-sensitive.

## Procedure

1. Identify whether the change lives in models, domain services, or Diesel repositories.
2. Check the current merge and replacement path before editing.
3. Decide explicitly whether each affected reference should be replaced or merged.
4. Update the model logic and repository logic together when required.
5. Validate the behavior with the narrowest affected flow, especially deduplication or finalization.

## Useful files

- `packages/shared/src/models/`
- `packages/domain/src/services/download_service.rs`
- `packages/database/src/repositories/`
- `docs/architecture/design.md`
