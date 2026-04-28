---
description: "Use when: editing Rust code in Soundome and you need the repository-wide Rust conventions, error handling rules, logging expectations, and shared architectural patterns."
applyTo: "**/*.rs"
---

# Soundome Rust (global)

## Principles

- Prefer minimal changes that fit the existing crates.
- Use `shared::types::SoundomeResult<T>` and `shared::errors::Error` for errors.
- Use `tracing::{info,warn,error,debug}` for logging.
- Avoid panics such as `unwrap()` and `expect()` outside initialization or boot code.

## Repository patterns

- Global config: `config::Config::get()` after `shared::init_globals()` has run.
- Networking: prefer `shared::libs::http::HttpClientBuilder` for proxy-aware HTTP.
- Domain layer: services talk to repositories through traits in `packages/domain/src/ports/repositories`.

## Style

- Keep public APIs stable when practical.
- Add Cargo dependencies only when necessary and in the right crate.
- Prefer pure functions and isolate side effects such as I/O, DB, and HTTP.
