# Soundome web app

This package contains the Svelte admin interface for Soundome.

## Responsibilities

- Submit track and playlist URLs to the server.
- Display recent downloads.
- Review and approve or reject tracks that need manual validation.
- Poll server-side task state when playlist downloads run in the background.

## Runtime model

- Development: Vite serves the SPA on `http://localhost:5173` and proxies API traffic to Rocket.
- Production build: static assets are emitted into `data/web/` and served by the Rocket server.

## Useful commands

From the repository root:

```bash
pnpm web:dev
pnpm web:build
pnpm web:check
```

## Related documentation

- [../../docs/workflows/web-admin.md](../../docs/workflows/web-admin.md)
- [../../README.md](../../README.md)
