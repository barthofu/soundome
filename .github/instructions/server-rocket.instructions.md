---
applyTo: "apps/server/src/**/*.rs"
---

# Server (Rocket) — conventions

## Architecture

- Le server monte des routes Rocket + OpenAPI via `rocket_okapi`.
- Injecter les services via `rocket::State<Arc<ServiceLayer>>`.
- DB pool via `Db` (`apps/server/src/utils/database.rs`) avec `rocket_sync_db_pools`.

## Recommandations

- Préférer des handlers fins : appeler `domain::services` plutôt que coder de la logique métier dans les routes.
- Les erreurs HTTP doivent être mappées proprement (ne pas exposer de secrets).
- Ajouter les routes dans `apps/server/src/routes/mod.rs` puis dans le `mount` de `main.rs`.

## OpenAPI

- Annoter les routes avec `#[openapi]`.
- Utiliser des types sérialisables (shared models) et éviter les types Diesel dans la couche HTTP.
