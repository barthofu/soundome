# Soundome custom agents

This directory contains repository-specific custom agents.

## Current agents

- `Soundome maintainer`: broad implementation and architecture-sensitive work across crates.
- `test`: focused testing and validation work.

## Selection guidance

- Use an agent when the task needs a persona with a clear role and a bounded toolset.
- Use a skill when the task matches a repeatable workflow such as migrations, route debugging, or docs updates.
- Use instructions for coding rules and repository conventions, not for task ownership.

## Boundary rules

- The maintainer agent owns broad code changes and can delegate narrow testing work.
- The test agent owns tests and validation, not feature implementation.
