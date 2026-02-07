---
applyTo: "packages/ai/src/**/*.rs"
---

# AI — OpenRouter / prompts

## Rôle

- `AIClient::new()` lit `Config.ai` et instancie un backend (OpenRouter).
- Les prompts vivent dans `packages/ai/src/prompts.rs` et servent surtout au nettoyage/standardisation de metadata.

## Conventions

- Ne jamais logger de secrets (API keys).
- Les prompts doivent être déterministes : format d’entrée/sortie strict, et données JSON validables.
- Préférer `generate_with_data` quand on attend un JSON structuré.
