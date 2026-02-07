---
name: "Skill: Add Diesel migration"
description: "Ajoute une migration Diesel + update entities/repositories de façon cohérente."
---

Objectif : ajouter/adapter le schéma SQLite.

Checklist :
- Créer migration dans `packages/database/migrations/` (up/down).
- Regénérer/mettre à jour `packages/database/src/schema.rs` si nécessaire.
- Mettre à jour entities + mappers + repositories concernés.
- Ajouter une vérif rapide (au minimum compilation + tests pertinents).

Contraintes :
- Ne pas casser la sémantique `set_references` (source/provider replace, autres merge).
