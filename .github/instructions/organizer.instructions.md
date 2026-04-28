---
description: "Use when: editing filesystem placement, file renaming, library layout rules, or organizer behavior in packages/organizer."
applyTo: "packages/organizer/src/**/*.rs"
---

# Organizer — organisation filesystem

## Role

- Move and rename files into the `Artist/Album/Track` layout.

## Conventions

- Always return typed errors from `shared::errors::Error` instead of using `unwrap()`.
- Normalize directory and file names, especially if future Windows compatibility matters.
