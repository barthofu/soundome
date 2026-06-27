# Spotify integration

Spotify serves two distinct roles in Soundome: a **source adapter** (resolving Spotify URLs into metadata) and a **metadata enrichment provider** (looking up track details when enriching any source). Both use the same credentials.

## Why configure Spotify

Without Spotify credentials, Soundome works fine for YouTube Music and SoundCloud URLs. Adding Spotify credentials unlocks:

- Submitting Spotify track, playlist, album, and artist URLs directly
- Richer metadata during enrichment: cover art, track number, disc number, release date, artist photos
- Higher-confidence deduplication against your existing library when ingesting local files (Spotify is placed first in the ingest enrichment order by default)

## Prerequisites

You need a Premium Spotify account.

1. Go to the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).
2. Click **Create app**.
3. Fill in any name and description. Set the redirect URI to `http://localhost` (unused but required by the form).
4. Open the app settings and note the **Client ID** and **Client Secret**.

## Configuration

Add the following to your `config.toml`:

```toml
[providers.spotify]
client_id = "your_client_id_here"
client_secret = "your_client_secret_here"
```

Or via environment variables (useful in containers or CI):

```
SOUNDOME__PROVIDERS__SPOTIFY__CLIENT_ID=your_client_id_here
SOUNDOME__PROVIDERS__SPOTIFY__CLIENT_SECRET=your_client_secret_here
```

Restart the server after changing the config. The `/api/providers` endpoint will list `"Spotify"` once credentials are valid.

## What Spotify provides

### As a source adapter (Spotify URLs as input)

When you paste a Spotify URL, Soundome fetches the metadata from Spotify and then downloads the audio from YouTube Music or YouTube (not from Spotify directly — Spotify does not allow audio download). This means:

- The metadata (title, artists, album, cover art) comes from Spotify
- The audio file comes from the best available YouTube Music match

Supported URL types:

| URL pattern | What is synced |
|---|---|
| `open.spotify.com/track/...` | Single track |
| `open.spotify.com/playlist/...` | Full playlist (async background task) |
| `open.spotify.com/album/...` | Full album (async background task) |
| `open.spotify.com/artist/...` | All artist tracks (async background task) |

### As a metadata enrichment provider (tagger)

Even when the source is SoundCloud or YouTube Music, Soundome can query Spotify during the enrichment step to get a better match. This is controlled by `tagger.metadata_providers`:

```toml
[tagger]
# Default order: MusicBrainz first (durable IDs), then Bandcamp, then Spotify
metadata_providers = ["musicbrainz", "bandcamp", "spotify"]

# For local file ingest, Spotify is tried first (better cover art and track numbers)
ingest_metadata_providers = ["spotify", "musicbrainz", "bandcamp"]
```

Spotify enrichment adds:
- Cover art URL (usually the highest-quality source)
- Release date
- Track number and disc number
- Artist Spotify IDs (used for deduplication and linking)
- Artist photo URLs

## Behaviour without credentials

If `[providers.spotify]` is absent or the credentials are empty:

- Spotify URLs return a `ProviderUnavailable` error — the download page shows an error message
- The Spotify enrichment provider is silently skipped during metadata matching; MusicBrainz and Bandcamp still run
- The server starts and operates normally for all other sources

There is no crash or degraded startup. You will see a `debug`-level log line: `"Spotify metadata provider: no credentials in config, skipping"`.

## Proxy

The Spotify SDK sets the `ALL_PROXY` environment variable internally rather than using the shared `HttpClientBuilder`. If you need Spotify traffic to go through a proxy, set `ALL_PROXY` directly in your environment in addition to the `[proxy]` config section.

## Troubleshooting

**"ProviderUnavailable: Spotify" when pasting a Spotify URL**
→ Credentials are missing or empty. Check that both `client_id` and `client_secret` are set and non-empty.

**Spotify appears in `/api/providers` but enrichment still uses MusicBrainz only**
→ Normal behaviour. The enrichment provider order is `["musicbrainz", "bandcamp", "spotify"]` by default. If MusicBrainz finds an exact match first, Spotify is not queried. Only when MusicBrainz returns a partial or no match will Spotify be tried.

**Artist photos are missing**
→ Spotify enrichment performs a secondary lookup per artist. This lookup can fail silently for artists with non-exact name matches. It is best-effort.
