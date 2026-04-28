# Soundome Copilot skills

This directory contains reusable Copilot skills for repository-specific workflows.

## How these differ from other customizations

- Agents in `.github/agents/` are for personas and broader task ownership.
- Instructions in `.github/instructions/` are always-on or file-scoped guidance.
- Skills in `.github/skills/` are on-demand workflows for repeatable, narrower tasks.

## Available skills

- `add-diesel-migration`: add or adjust Diesel migrations safely.
- `debug-rocket-route`: debug a Rocket route end to end.
- `extend-download-workflow`: change `DownloadService` without breaking workflow invariants.
- `proxy-aware-http`: make direct HTTP code respect Soundome proxy behavior.
- `soundome-change-routing`: identify the correct crate and validation path for a change.
- `manual-validation-flow`: work on the pending-validation flow across domain, server, and web.
- `reference-semantics`: change track, album, artist, or repository logic without breaking `ReferenceType` semantics.
- `update-soundome-docs`: update docs so they stay aligned with the codebase and the categorized docs structure.

## Notes

- Use `.github/skills/` for reusable multi-step workflows.
- Keep `.github/prompts/` for one-off prompt templates if new prompt files are added later.
- Prefer a skill over the maintainer agent when the task clearly matches one workflow such as migrations, Rocket-route debugging, proxy-aware HTTP, validation flow, or docs maintenance.
