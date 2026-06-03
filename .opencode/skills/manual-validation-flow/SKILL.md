---
name: manual-validation-flow
description: Use when changing pending validation behavior, needs_validation flags, approval or rejection flow, validation routes, staged file finalization, or the web admin validation UI in Soundome.
---

# Manual Validation Flow

Use this skill for changes that touch the review flow for partially matched or uncertain tracks.

## Scope

- `needs_validation` and `validation_reason`
- staged file retention
- approve or reject endpoints
- finalization after manual edits
- frontend validation list and actions

## Procedure

1. Determine whether the change starts in domain, server, or web.
2. Trace the flow across these layers:
   - domain: load staged track, apply patch, tag and organize, clear validation state
   - server: expose approve or reject routes and DTOs
   - web: render pending items and send the correct payloads
3. Preserve the invariant that audio is already present in staging before manual approval.
4. Preserve safe behavior when a user rejects a pending track.
5. Confirm that the validation state transitions are explicit and persisted.
6. Validate the narrowest affected slice first.

## Useful files

- `packages/domain/src/services/download_service.rs`
- `apps/server/src/routes/validations.rs`
- `apps/web/src/`
- `docs/workflows/web-admin.md`
