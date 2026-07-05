# Invidious Configuration

When Soundome downloads audio from YouTube sources (Spotify tracks, YouTube links, etc.), it uses the **Invidious** service to search for and retrieve videos. Invidious is a privacy-respecting YouTube frontend.

## Default instance

Soundome comes with a **built-in default instance: `https://invidious.tiekoetter.com`** that reliably allows automated requests without bot protection.

**You don't need to configure anything.** If you want to use a different instance, follow the section below.

## Why might you need a custom instance?

The original default Invidious instance in the library (`invidious.f5.si`) has bot protection enabled and will block requests from Soundome with a **403 Forbidden** error.

**Error signature:**
```
Invidious search call failed: Cannot deserialize response: expected value at line 1 column 1
failed to parse Invidious response: ... (raw response: <html>\r\n<head><title>403 Forbidden</title>...
```

If the Soundome default instance becomes blocked, or if you want to use a different instance, follow the instructions below.

## How to configure a custom instance

### Option 1: Edit config.toml

```toml
[providers.youtube]
invidious_instance = "https://your-instance.example.com"
```

### Option 2: Environment variable

```bash
export SOUNDOME__PROVIDERS__YOUTUBE__INVIDIOUS_INSTANCE=https://your-instance.example.com
```

## Finding a suitable instance

Browse available public instances and their health at:
- **https://docs.invidious.io/instances/** — curated list with uptime and API status

When choosing an instance, look for:
- **Uptime ≥ 90%** in the last 7 days
- **API enabled** (column in the instances table)
- **No bot protection** (avoid instances marked with bot detection or rate limiting)

Recent examples of good instances:
- `https://invidious.tiekoetter.com/` (Soundome default)
- `https://invidious.snopyta.org/`
- `https://yt.artemislena.eu/`

**Note:** Public instances are community-run and may disappear or become rate-limited. If the Soundome default or your chosen instance starts returning 403 errors, switch to a different one from the list above.

## Troubleshooting

### Still getting 403 Forbidden?

1. Verify the instance is alive:
   ```bash
   curl -I https://your-instance.example.com/
   ```
   Should return `200 OK`, not `403 Forbidden` or `502 Bad Gateway`.

2. If using a custom instance, check that the configuration is being read:
   ```bash
   # If using environment variable
   echo $SOUNDOME__PROVIDERS__YOUTUBE__INVIDIOUS_INSTANCE
   
   # Or restart the server and check logs
   ```

3. Try a different instance from the public list.

### Searches are timing out (slow responses)

Some instances may be slow or overloaded. This is normal for community-run services.
- Try a different instance from the list above.
- Consider self-hosting an Invidious instance if you have the infrastructure.

## Self-hosting Invidious (advanced)

If public instances are unreliable for your use case, you can self-host Invidious:
- GitHub: https://github.com/iv-org/invidious
- Docker image available: `quay.io/invidious/invidious`

Once self-hosted, configure Soundome to point to your instance:
```toml
[providers.youtube]
invidious_instance = "https://invidious.your-domain.local"
```

## Related

- [Proxy configuration](proxy-configuration.md) — if using a proxy for Soundome itself
- Error in logs: `Invidious search call failed` → check the error message for next steps
