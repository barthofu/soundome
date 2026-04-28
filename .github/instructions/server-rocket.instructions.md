---
description: "Use when: editing Rocket routes, request or response DTOs, OpenAPI wiring, HTTP error mapping, or server-layer code in apps/server."
applyTo: "apps/server/src/**/*.rs"
---

# Server (Rocket) — conventions

## Architecture

- The server mounts Rocket routes and OpenAPI through `rocket_okapi`.
- Inject services through `rocket::State<Arc<ServiceLayer>>`.
- Use the `Db` pool from `apps/server/src/utils/database.rs` with `rocket_sync_db_pools`.

## Recommendations

- Prefer thin handlers: call `domain::services` instead of putting business logic in the routes.
- Map HTTP errors cleanly and do not expose secrets.
- Add new routes in `apps/server/src/routes/mod.rs` and in the `mount` list in `main.rs`.

## OpenAPI

- Annotate routes with `#[openapi]`.
- Use serializable types, preferably shared models or dedicated DTOs, and avoid Diesel types in the HTTP layer.
