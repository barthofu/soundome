# Development setup

This guide describes the local setup expected by the current repository.

## Requirements

- Rust stable toolchain
- `pnpm`
- `ffmpeg`
- SQLite
- `diesel_cli` with SQLite support
- a local `.env` file for server and CLI boot

## Install dependencies

From the repository root:

```bash
pnpm install
cargo install diesel_cli --no-default-features --features sqlite
```

## Initialize the database

```bash
diesel setup
diesel migration run
```

Rocket uses `Rocket.toml` and points SQLite to `data/soundome.db` by default.

## Configure the runtime

1. Review `config.toml`.
2. Create the local `.env` file expected by `dotenvy` in the server and CLI entry points.
3. Ensure any provider credentials you want to use are available through config or environment overrides.

See [configuration.md](configuration.md) for the config shape.

## Start the development servers

```bash
pnpm dev
```

This starts:

- the Rocket server on `http://localhost:8000`
- the Vite frontend on `http://localhost:5173`

## Useful commands

```bash
pnpm cargo:test
pnpm cargo:clippy
pnpm cargo:fmt
pnpm web:check
pnpm web:build
```

## Working notes

- The CLI crate exists but is still minimal.
- The server is currently the main entry point for the end-to-end workflow.
- The frontend build is emitted into `data/web/` and served by Rocket outside Vite development mode.
