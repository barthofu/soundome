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
- **Rate limiting / bot detection**: configure a proxy (see above) or retry later.
- **Region-locked or removed video**: expected failure, not a configuration issue.

### No search results / no match found

If `yt-dlp` runs successfully but returns no usable candidates, Soundome logs a warning per unparsable result line and otherwise proceeds with an empty candidate list, which surfaces as `Error::NoMatch` upstream. Try the query manually:

```bash
yt-dlp "ytsearch5:artist title" --dump-json --skip-download --flat-playlist
```

## Related

- [Proxy configuration](proxy-configuration.md) — if using a proxy for Soundome itself
- `packages/downloader/src/utils/ytdlp.rs` — subprocess invocation and JSON parsing
- `packages/downloader/src/youtube/mod.rs` — search query construction and candidate matching
