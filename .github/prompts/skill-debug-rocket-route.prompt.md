---
name: "Skill: Debug Rocket route"
description: "Diagnostic structuré pour une route Rocket (DB, State, OpenAPI, erreurs)."
---

Contexte : je debug une route Rocket dans `apps/server`.

À faire :
1) Localiser la route + son mount (routes/mod.rs + main.rs).
2) Vérifier injection `State<Arc<ServiceLayer>>` et DB `Db`.
3) Vérifier les types de retour (Json, Result, erreurs).
4) Proposer un patch minimal avec logs `tracing`.

Entrées utilisateur attendues :
- URL + méthode HTTP
- stacktrace/logs Rocket
- extrait de la route
