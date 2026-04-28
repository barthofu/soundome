---
name: update-soundome-docs
description: 'Use when: updating repository documentation, reorganizing docs, adding onboarding material, fixing stale architecture docs, or aligning Soundome docs with code and the categorized docs structure.'
argument-hint: 'Describe the doc change, target audience, and code area the docs should reflect.'
---

# Update Soundome Docs

Use this skill when documentation needs to evolve with the codebase.

## Documentation structure

- `docs/getting-started/`: setup and configuration
- `docs/product/`: goals, scope, and constraints
- `docs/architecture/`: design and repository map
- `docs/workflows/`: end-to-end flows
- `docs/operations/`: operational topics such as proxy behavior

## Procedure

1. Start from the code path or behavior that changed.
2. Find the owning category in `docs/` instead of adding another flat page.
3. Update the nearest existing page before creating a new page.
4. If a new page is needed, place it in the appropriate category and link it from the category `README.md`.
5. Keep README-level docs short and hub-oriented.
6. Update any repository-level links that point to moved or replaced docs.
7. Remove or consolidate stale duplicates after links are updated.

## Documentation standards

- Prefer English.
- Distinguish current behavior from planned behavior.
- Keep architecture-sensitive claims aligned with code, not assumptions.
- Use specific file and workflow names when documenting ownership.

## Useful files

- `docs/README.md`
- `docs/getting-started/`
- `docs/product/`
- `docs/architecture/`
- `docs/workflows/`
- `docs/operations/`
