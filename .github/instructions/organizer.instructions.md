---
applyTo: "packages/organizer/src/**/*.rs"
---

# Organizer — organisation filesystem

## Rôle

- Déplacer/renommer les fichiers vers `Artist/Album/Track`.

## Conventions

- Toujours retourner des erreurs typées (`shared::errors::Error`) plutôt que `unwrap()`.
- Normaliser les noms de dossiers/fichiers (éviter caractères interdits Windows si c’est un objectif futur).
