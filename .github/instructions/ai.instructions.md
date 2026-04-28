---
description: "Use when: editing OpenRouter integration, AI prompt generation, metadata-cleanup prompts, or structured AI output handling in packages/ai."
applyTo: "packages/ai/src/**/*.rs"
---

# AI — OpenRouter / prompts

## Role

- `AIClient::new()` reads `Config.ai` and instantiates the configured backend, currently OpenRouter.
- Prompts live in `packages/ai/src/prompts.rs` and mainly support metadata cleanup and standardization.

## Conventions

- Never log secrets such as API keys.
- Keep prompts deterministic: strict input and output shape, with JSON that can be validated.
- Prefer `generate_with_data` when structured JSON output is expected.
