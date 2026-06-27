# AI metadata cleanup

Soundome can call an AI model to clean noisy track metadata before enrichment. This page explains what the AI does, when it helps, how to configure it, and which model to pick.

## What the AI does

The AI cleanup step targets **SoundCloud metadata specifically**. SoundCloud titles often contain artist names, collaboration credits, catalog numbers, and platform tags all mixed into a single string. Before Soundome tries to match the track against MusicBrainz or Bandcamp, it can send the raw title and artist fields to an AI model and receive back cleaned, structured values.

**Example — before AI cleanup:**

```
Title:   Bicep b2b Four Tet - Glassworks (FREE DOWNLOAD) [Resident Advisor]
Artists: rarecandymusic
```

**Example — after AI cleanup:**

```
Title:   Glassworks
Artists: Bicep, Four Tet
```

The AI extracts:
- Artist names embedded at the start of the title
- Featured and collaborating artists (`ft.`, `b2b`, `vs`, `&`, `x`)
- The actual track title, stripped of tags and catalog numbers
- Properly capitalised fields

**What the AI does not do:**
- It does not look up information on external databases
- It does not invent or guess genres, albums, or release dates
- It only operates on the raw strings provided by SoundCloud — nothing is fabricated

For Spotify and YouTube Music sources the metadata is already clean and structured; AI cleanup is skipped entirely for those.

## When to enable it

Enable AI cleanup if you:
- Import SoundCloud tracks or playlists regularly
- Have tracks with combined `Artist - Title` fields in the title
- See many tracks landing in the validation queue after SoundCloud imports

AI cleanup significantly reduces the number of tracks requiring manual review for SoundCloud content.

## Backends

Two backends are supported. You can configure one or both; the `provider_order` list controls which is tried first. If the first fails (model unavailable, timeout), Soundome automatically tries the next one.

### Ollama (local, recommended as primary)

[Ollama](https://ollama.com) runs models locally on your machine. It is free, private, and fast on hardware with a GPU.

**Requirements:**
- Ollama ≥ 0.5.0 (earlier versions do not support structured JSON output, which Soundome requires)
- At least one model pulled

**Recommended models** for this task (good JSON output, low resource requirements):

| Model | VRAM / RAM | Quality | Notes |
|---|---|---|---|
| `qwen2.5:7b` | ~5 GB | Excellent | Best overall for structured extraction tasks |
| `llama3.2:3b` | ~2 GB | Good | Lighter option for machines with limited RAM |
| `mistral:7b` | ~5 GB | Good | Reliable JSON output |

```toml
[ai]
enabled = true
provider_order = ["ollama"]

[ai.ollama]
host = "http://localhost"  # default; change if Ollama runs on another machine
port = 11434               # default Ollama port
model = "qwen2.5:7b"
timeout = 60               # seconds; increase for slow hardware
```

Pull the model before enabling:

```bash
ollama pull qwen2.5:7b
```

### OpenRouter (cloud, recommended as fallback)

[OpenRouter](https://openrouter.ai) is a cloud gateway that routes requests to many hosted models (OpenAI, Anthropic, Mistral, etc.). Useful as a reliable fallback if Ollama is unavailable, or as the primary backend if you do not have a local GPU.

**Requirements:**
- An OpenRouter account and API key (free tier available)

**Recommended models:**

| Model | Notes |
|---|---|
| `openai/gpt-4o-mini` | Excellent quality, fast, very cheap |
| `mistralai/mistral-7b-instruct` | Good quality, cheapest option |
| `anthropic/claude-3-haiku` | High quality, low cost |

```toml
[ai]
enabled = true
provider_order = ["openrouter"]

[ai.openrouter]
api_key = "sk-or-..."
model = "openai/gpt-4o-mini"
timeout = 30
```

Or via environment variable (recommended for secrets):

```
SOUNDOME__AI__OPENROUTER__API_KEY=sk-or-...
```

## Using both backends together

Configure Ollama as the primary and OpenRouter as the fallback. If Ollama is down or too slow, OpenRouter takes over automatically:

```toml
[ai]
enabled = true
provider_order = ["ollama", "openrouter"]

[ai.ollama]
model = "qwen2.5:7b"

[ai.openrouter]
api_key = "sk-or-..."
model = "openai/gpt-4o-mini"
```

## How it works in practice

When Soundome downloads a SoundCloud track or syncs a SoundCloud playlist, it sends the raw metadata to the AI in batches of up to 50 tracks at a time. The prompt asks for a JSON response in exactly the same structure as the input — just with cleaned values. If the response is invalid JSON or the model returns garbage, Soundome logs a warning and continues with the original values rather than failing.

This means: **AI failures are non-fatal.** The worst case is that metadata remains uncleaned and more tracks land in the validation queue.

## Checking that AI is active

Look at the server logs when submitting a SoundCloud URL. With `logs.level = "debug"` you will see lines like:

```
DEBUG soundome::fetcher::soundcloud: cleaning 12 tracks with AI
DEBUG soundome::ai: sending batch of 12 tracks to ollama
```

If AI is not configured or disabled, the cleaning step is skipped silently at `warn` level.

## Cost estimate (OpenRouter)

For `openai/gpt-4o-mini`, cleaning a batch of 50 SoundCloud tracks costs roughly $0.001–0.003 depending on title length. A playlist of 500 tracks costs under $0.03.

## Troubleshooting

**AI is enabled but SoundCloud tracks still have noisy titles**
→ Check the server logs for `warn` or `error` lines from the `ai` or `soundcloud` modules. Common causes: Ollama model not pulled, wrong `host`/`port`, OpenRouter API key missing or invalid.

**Ollama returns a timeout error**
→ Increase `ai.ollama.timeout`. On CPU-only machines, 60–120 seconds is typical for the first request (model warm-up). Subsequent requests in the same session are faster.

**"Ollama 0.5.0+ required for structured output"**
→ Update Ollama: `ollama update` or download from [ollama.com](https://ollama.com).

**The AI cleaned the title incorrectly**
→ The track will be saved with the cleaned (but imperfect) values and still go through enrichment. If the enrichment match is partial or missing, it will appear in the validation queue where you can correct the metadata manually.
