# YouTube Search (yt-dlp)

Soundome uses **yt-dlp** for both searching and downloading YouTube audio (Spotify tracks routed to YouTube, direct YouTube links, and the YouTube fallback for SoundCloud DRM tracks). There is no more third-party search backend (Invidious) to configure or select an instance for.

## How it works

- Search runs `yt-dlp "ytsearchN:<query>" --dump-json --skip-download --flat-playlist`, which asks YouTube directly for the first `N` results without downloading anything.
- Soundome parses the newline-delimited JSON output (one JSON object per result) into candidate tracks, then applies the same title/duration similarity matching as before (see `packages/downloader/src/youtube/matcher.rs`).
- Download uses the same `yt-dlp` binary, just without `--flat-playlist`/`--skip-download`.
- Code entry point: `packages/downloader/src/utils/ytdlp.rs` (`search_with_ytdlp`, `download_with_ytdlp`).

## Prerequisites

- `yt-dlp` must be installed and available on `PATH`. Soundome shells out to it as a subprocess; there is no bundled binary.
  - `pip install -U yt-dlp` (or `pipx install yt-dlp`)
  - `brew install yt-dlp` on macOS
  - Standalone binary releases: <https://github.com/yt-dlp/yt-dlp/releases>
- Keep `yt-dlp` up to date. YouTube changes its internal APIs frequently, and outdated yt-dlp releases are the most common cause of sudden search or download failures.

## Overriding the yt-dlp binary at runtime (no image rebuild)

The Docker image bundles a specific `yt-dlp` release at build time (see the `libs` stage in `Dockerfile`). Because YouTube ships countermeasures faster than most release cadences allow for, waiting on an image rebuild to pick up a fix can be too slow — a nightly yt-dlp build often fixes a breakage within hours, while cutting and deploying a new Soundome image can take much longer.

`[downloader.ytdlp]` lets you point Soundome at any static `yt-dlp` binary URL and have it downloaded and used at the next boot, without touching the image:

```toml
[downloader.ytdlp]
binary_url = "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.15/yt-dlp_musllinux"
sha256 = "..."  # optional but recommended
```

Or via environment variables (e.g. in `.env` or your container orchestrator):

```
SOUNDOME__DOWNLOADER__YTDLP__BINARY_URL=https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.15/yt-dlp_musllinux
SOUNDOME__DOWNLOADER__YTDLP__SHA256=...
```

### How it works

- At boot, Soundome resolves a cache path derived from a hash of `binary_url`, under `{directory of database.url}/bin/` (e.g. `./data/bin/yt-dlp-<hash>`). This directory sits next to the SQLite database file, so it is already covered by whatever volume/mount you use to persist `database.url` — no extra volume configuration needed.
- If a file already exists at that cache path, it is reused as-is and **no network request is made**. This means restarting the container with the same `binary_url` never re-downloads anything.
- If the cache path doesn't exist yet, Soundome downloads the binary, optionally verifies it against `sha256` (logging a warning if `sha256` is omitted), marks it executable, and installs it atomically (write to a temp file, then rename).
- Either way (cache hit or fresh download), Soundome runs `<binary> --version` as a smoke test before using it. If that fails, the (broken) cached file is deleted so it isn't reused on the next boot, and provisioning is treated as failed for this boot.
- The resolved binary path is used for both search (`ytsearchN:` queries) and download subprocess calls (`packages/downloader/src/utils/ytdlp.rs`).
- If anything above fails (network error, checksum mismatch, filesystem error, smoke test failure), Soundome logs an `error!` and **falls back to the plain `yt-dlp` command resolved via `PATH`** — i.e. the version baked into the image. Boot never aborts because of this.

### Choosing a `binary_url`

**Match the libc of the host actually running Soundome — not a fixed rule.** yt-dlp publishes several Linux asset variants per release, and the right one depends on where the process runs, not on the Soundome deployment target in general:

- Running in the bundled Docker image (Alpine, musl libc): use a **musl-linked** asset — `yt-dlp_musllinux` (amd64) or `yt-dlp_musllinux_aarch64` (arm64).
- Running directly on a glibc-based host (Debian, Ubuntu, Fedora, etc. — e.g. local development outside the container): use the plain **glibc-linked** asset — `yt-dlp_linux` (or `yt-dlp_linux_aarch64` for arm64). Do **not** use the musllinux build here.

Using the wrong one for the current host produces a file that downloads and installs "successfully" but can't actually run: the kernel returns `ENOENT` ("No such file or directory") because it can't find the dynamic loader path (`/lib/ld-musl-x86_64.so.1` for a musl binary on a glibc host, or the reverse) embedded in the binary — even though the file itself is present on disk. See the smoke-test note above, which exists specifically to catch this at boot instead of at the first real download attempt.


### Picking up a new version

This is a **boot-time** resolution, not a live hot-swap: to switch to a different yt-dlp build (e.g. a newer nightly),

1. Update `binary_url` (and `sha256`, if used) in `config.toml` or the environment.
2. Restart the container (or process).

The new URL hashes to a different cache path, so it downloads fresh rather than reusing the previous binary. No image rebuild, no new Soundome release required.

### Where the logic lives

- `packages/shared/src/libs/ytdlp_binary.rs` — resolution, caching, checksum verification, fallback.
- `apps/server/src/main.rs` — calls `shared::ytdlp_binary::init()` once at boot, before Rocket's own runtime starts (a short-lived tokio runtime is created just for this call).
- `packages/downloader/src/utils/ytdlp.rs` — uses `shared::ytdlp_binary::path()` instead of a hardcoded `"yt-dlp"` when spawning the subprocess.

## Proxy behavior

Search and download both honor the shared proxy configuration (`[proxy]` in `config.toml`, `ProxyRotator`) the same way: when a proxy is configured and enabled, Soundome passes `--proxy <url>` to `yt-dlp`. See [proxy-configuration.md](proxy-configuration.md) for setup details.

## Troubleshooting

### `yt-dlp` not found / process spawn error

Verify the binary is installed and on `PATH` for the user/environment running Soundome:

```bash
yt-dlp --version
```

### Search or download fails with a non-zero exit code

Soundome surfaces `yt-dlp`'s captured stderr in the error message (`Error::ExitCode { code, stderr }`). Common causes:

- **Outdated yt-dlp**: update it (`pip install -U yt-dlp`) — YouTube extraction breakages are usually fixed within days upstream.
- **Rate limiting / bot detection**: this is the most common cause of *intermittent* 403s (the same URL fails on one run and succeeds on the next, or succeeds when run manually). Soundome automatically retries transient-looking failures (stderr containing `403`, `429`, "too many requests", "rate limit", or "sign in to confirm") up to `MAX_ATTEMPTS` times with a short backoff before giving up — see `run_ytdlp_with_retry` in `packages/downloader/src/utils/ytdlp.rs`. Each retry rebuilds the yt-dlp args, so a rotating proxy (`ProxyRotator` with `RoundRobin`/`Random` strategy) will pick a different upstream IP on retry. If failures persist after retries, configure a proxy (see above) or retry later.
- **Region-locked or removed video**: expected failure, not a configuration issue, and is not retried.

### No search results / no match found

If `yt-dlp` runs successfully but returns no usable candidates, Soundome logs a warning per unparsable result line and otherwise proceeds with an empty candidate list, which surfaces as `Error::NoMatch` upstream. Try the query manually:

```bash
yt-dlp "ytsearch5:artist title" --dump-json --skip-download --flat-playlist
```

### Persistent 403s even after retries (YouTube countermeasures)

If failures persist across all retries and don't clear up over time, and updating the image's bundled `yt-dlp` requires a rebuild you'd rather avoid, use `[downloader.ytdlp].binary_url` (see above) to point at a fresher/nightly build without rebuilding the image. Check the logs at boot for a line like:

```
INFO shared::libs::ytdlp_binary: Using yt-dlp binary: ./data/bin/yt-dlp-<hash>
```

If instead you see an `ERROR` log about falling back to `"yt-dlp"` from `PATH`, the custom download failed (bad URL, checksum mismatch, network issue, or the binary couldn't execute) — check the error message for details.

### Custom yt-dlp binary downloads but `spawn` fails with "No such file or directory"

This means the downloaded file exists on disk but the kernel can't execute it — almost always a **libc mismatch** between the `binary_url` asset and the host actually running Soundome (see "Choosing a `binary_url`" above): e.g. a `yt-dlp_musllinux` build on a glibc host (Debian/Ubuntu dev environment), or the reverse (a `yt-dlp_linux` glibc build on the Alpine/musl container). Soundome's boot-time smoke test (`<binary> --version`) should catch this automatically and fall back to `yt-dlp` from `PATH` with a clear error log — if you still hit this at download/search time rather than at boot, you're likely on a Soundome version older than that check; update, or manually fix `binary_url` to match the current host's libc and delete the stale cached file under `{directory of database.url}/bin/`.

## Related

- [Proxy configuration](proxy-configuration.md) — if using a proxy for Soundome itself
- [Configuration reference — `[downloader.ytdlp]`](../getting-started/configuration.md#downloader-optional) — config keys and environment variables
- `packages/downloader/src/utils/ytdlp.rs` — subprocess invocation and JSON parsing
- `packages/downloader/src/youtube/mod.rs` — search query construction and candidate matching
- `packages/shared/src/libs/ytdlp_binary.rs` — runtime binary provisioning, caching, and fallback
