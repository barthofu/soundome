# Soundome instructions

This directory contains workspace instructions that guide Copilot when tasks or edited files match a specific concern.

## How to use this directory

- `project.instructions.md`: high-level repository context used before narrowing to a specific concern.
- language or layer instructions such as `rust.instructions.md`: broad coding conventions.
- crate or framework instructions such as `domain.instructions.md` or `server-rocket.instructions.md`: narrower implementation rules.

## Boundary rules

- Instructions define conventions and constraints.
- Skills define repeatable workflows.
- Agents define role-based ownership with tool restrictions.

## Discovery guidance

- Use keyword-rich `description` frontmatter for on-demand discovery.
- Use `applyTo` only when the instruction should auto-attach to matching files.
- Keep one concern per instruction file.
