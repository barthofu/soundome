<script lang="ts">
  import { onMount } from 'svelte';
  import { downloadUrl, getRecentTracks } from '../lib/api';
  import type { DownloadResult, RecentTrack } from '../lib/api';

  let { onNavigateTasks = undefined }: { onNavigateTasks?: () => void } = $props();

  let url = $state('');
  let loading = $state(false);
  let result: DownloadResult | null = $state(null);
  let error: string | null = $state(null);

  let recentTracks: RecentTrack[] = $state([]);
  let recentLoading = $state(true);

  async function loadRecent() {
    try {
      recentTracks = await getRecentTracks(20);
    } catch {
      // silently fail if API not up
    } finally {
      recentLoading = false;
    }
  }

  onMount(loadRecent);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!url.trim()) return;
    loading = true;
    result = null;
    error = null;
    try {
      result = await downloadUrl(url.trim());
      url = '';
      await loadRecent();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function formatDuration(s: number | null): string {
    if (!s) return '';
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  }
</script>

<div class="page">
  <div class="hero">
    <h1>Soundome</h1>
    <p class="subtitle">Paste a track or playlist URL to add it to your library.</p>
  </div>

  <form class="download-form" onsubmit={handleSubmit}>
    <div class="input-row">
      <input
        type="url"
        placeholder="https://open.spotify.com/track/… or SoundCloud, YouTube…"
        bind:value={url}
        disabled={loading}
        autocomplete="off"
        spellcheck="false"
      />
      <button type="submit" disabled={loading || !url.trim()}>
        {#if loading}<span class="spinner"></span> Downloading…{:else}Download{/if}
      </button>
    </div>
  </form>

  {#if error}
    <div class="feedback error">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if result && !loading}
    {#if result.type === 'track'}
      <div class="feedback success">
        ✓ <strong>{result.title}</strong>
        {#if result.artists.length}
          — {result.artists.join(', ')}
        {/if}
        {#if result.needs_validation}
          <span class="badge-warning">needs review</span>
        {/if}
      </div>
    {:else}
      <div class="feedback success">
        ✓ Playlist syncing —
        {#if onNavigateTasks}
          <button class="tasks-link" onclick={onNavigateTasks}>view tasks</button>
        {/if}
        (task #{result.task_id})
      </div>
    {/if}
  {/if}

  <div class="supported">
    <span>Supported:</span>
    <span class="platform">Spotify</span>
    <span class="platform">SoundCloud</span>
    <span class="platform">YouTube</span>
    <span class="platform">YouTube Music</span>
  </div>

  <section class="recent">
    <h2>Recent downloads</h2>
    {#if recentLoading}
      <p class="status-msg">Loading…</p>
    {:else if recentTracks.length === 0}
      <p class="status-msg">No tracks yet.</p>
    {:else}
      <ul class="track-list">
        {#each recentTracks.filter((t) => !t.needs_validation) as track (track.id)}
          <li class="track-row">
            <div class="cover">
              {#if track.cover}
                <img src={track.cover} alt="" />
              {:else}
                <span class="cover-placeholder">♪</span>
              {/if}
            </div>
            <div class="track-info">
              <span class="track-title">{track.title}</span>
              <span class="track-artists">
                {track.artists.map((a) => a.name).join(', ')}{track.album ? ` · ${track.album.title}` : ''}
              </span>
            </div>
            <div class="track-meta">
              {#if track.duration}
                <span class="duration">{formatDuration(track.duration)}</span>
              {/if}
              {#if track.needs_validation}
                <span class="badge-warning">review</span>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .page {
    max-width: 640px;
    margin: 0 auto;
    padding: 4rem 1rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .hero {
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 800;
    margin: 0 0 0.5rem;
    letter-spacing: -0.02em;
  }

  .subtitle {
    color: var(--muted);
    margin: 0;
    font-size: 1rem;
  }

  .download-form {
    width: 100%;
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  input[type='url'] {
    flex: 1;
    padding: 0.65rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.15s;
  }

  input[type='url']:focus {
    border-color: var(--accent);
  }

  input[type='url']:disabled {
    opacity: 0.5;
  }

  button[type='submit'] {
    padding: 0.65rem 1.2rem;
    border: none;
    border-radius: 8px;
    background: var(--accent);
    color: #fff;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity 0.15s;
  }

  button[type='submit']:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .feedback {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-size: 0.9rem;
  }

  .feedback.error {
    background: color-mix(in srgb, var(--error) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--error) 40%, transparent);
    color: var(--error);
  }

  .feedback.success {
    background: color-mix(in srgb, #4caf50 12%, transparent);
    border: 1px solid color-mix(in srgb, #4caf50 35%, transparent);
    color: #81c784;
  }

  .badge-warning {
    font-size: 0.7rem;
    padding: 0.15rem 0.45rem;
    background: color-mix(in srgb, var(--warning) 20%, transparent);
    border: 1px solid color-mix(in srgb, var(--warning) 40%, transparent);
    color: var(--warning);
    border-radius: 4px;
    margin-left: 0.25rem;
  }

  button[type='submit'] :global(.spinner) {
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255,255,255,0.35);
    border-top-color: #fff;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .supported {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: center;
    font-size: 0.75rem;
    color: var(--muted);
  }

  .platform {
    padding: 0.2rem 0.5rem;
    background: var(--surface-2);
    border-radius: 4px;
  }

  .recent {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  h2 {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted);
    margin: 0;
  }

  .status-msg {
    font-size: 0.875rem;
    color: var(--muted);
    margin: 0;
  }

  .track-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .track-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.6rem;
    border-radius: 6px;
    transition: background 0.1s;
  }
  .track-row:hover { background: var(--surface); }

  .cover {
    flex-shrink: 0;
    width: 40px;
    height: 40px;
    border-radius: 4px;
    overflow: hidden;
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cover img { width: 100%; height: 100%; object-fit: cover; }
  .cover-placeholder { font-size: 1.1rem; color: var(--muted); }

  .track-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .track-title {
    font-size: 0.875rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-artists {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-meta {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .duration {
    font-size: 0.75rem;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .tasks-link {
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
  }
</style>
