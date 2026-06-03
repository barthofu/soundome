---
name: debug-rocket-route
description: Use when debugging a Rocket route, API endpoint, request/response mismatch, State injection issue, Db usage issue, OpenAPI registration problem, or route-level error mapping in apps/server.
---

# Debug Rocket Route

Use this skill to debug an API route in `apps/server` without drifting into unrelated parts of the repository.

## When to use

- an endpoint returns the wrong status or payload
- a route is mounted but not reachable
- `State<Arc<ServiceLayer>>` wiring is wrong
- `Db` usage or blocking async boundaries fail
- Swagger or OpenAPI registration is missing

## Procedure

1. Locate the handler in `apps/server/src/routes/`.
2. Confirm it is exported in `routes/mod.rs` and mounted in `apps/server/src/main.rs`.
3. Verify request DTOs, response DTOs, and error mapping.
4. Check service injection through `rocket::State<Arc<ServiceLayer>>`.
5. Check database execution boundaries such as `db.run`, blocking behavior, and any async bridging.
6. Step one hop into the owning domain service if the route only forwards the work.
7. Apply the smallest patch that explains or fixes the failure.
8. Validate with the narrowest executable check available for that route or handler path.

## Soundome-specific checks

- Prefer thin handlers that defer to `domain::services`.
- Avoid leaking Diesel types into the HTTP layer.
- Keep HTTP error messages safe and free of secrets.
- Update OpenAPI annotations when request or response shapes change.

## Useful files

- `apps/server/src/main.rs`
- `apps/server/src/routes/`
- `apps/server/src/utils/database.rs`
- `packages/domain/src/services/`
