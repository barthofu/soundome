<script lang="ts">
  import { onMount } from 'svelte';
  import { getStorageStats } from '../lib/api';
  import type { StorageStatsDto } from '../lib/api';

  let stats: StorageStatsDto | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);

  onMount(async () => {
    try {
      stats = await getStorageStats();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  });

  function handleRefresh() {
    loading = true;
    error = null;
    loadStats();
  }

  async function loadStats() {
    try {
      stats = await getStorageStats();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIdx = 0;

    while (size >= 1024 && unitIdx < units.length - 1) {
      size /= 1024;
      unitIdx += 1;
    }

    if (unitIdx === 0) {
      return `${bytes} ${units[0]}`;
    }
    return `${size.toFixed(1)} ${units[unitIdx]}`;
  }
</script>

<div class="storage-page">
  <div class="page-header">
    <h1>Storage Usage</h1>
    <button class="btn-header" onclick={handleRefresh} disabled={loading}>
      {loading ? 'Loading…' : 'Refresh'}
    </button>
  </div>

  {#if error}
    <div class="feedback error">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if loading}
    <div class="loading">Loading storage statistics…</div>
  {:else if stats}
    <div class="stats-header">
      <div class="total-size">
        <span class="label">Library Size</span>
        <span class="size">{stats.total_formatted}</span>
        <span class="bytes">({stats.total_bytes.toLocaleString()} bytes)</span>
      </div>
      <div class="artist-count">
        {stats.artists.length} artists
      </div>
    </div>

    {#if stats.artists.length === 0}
      <p class="no-data">No artists or storage data available.</p>
    {:else}
      <ul class="artists-list">
        {#each stats.artists as artist (artist.id)}
          <li class="artist-row">
            <div class="artist-name">{artist.name}</div>
            <div class="artist-size">
              <div class="bar-container">
                <div
                  class="bar"
                  style="width: {Math.max(artist.percent, 2)}%"
                  title="{artist.name}: {artist.percent.toFixed(1)}% ({formatBytes(artist.bytes)})"
                ></div>
              </div>
            </div>
            <div class="artist-meta">
              <span class="percent">{artist.percent.toFixed(1)}%</span>
              <span class="size">{formatBytes(artist.bytes)}</span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .storage-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
  }

  .btn-header {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-header:hover:not(:disabled) {
    background: var(--surface-2);
  }

  .btn-header:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .feedback {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
  }

  .feedback.error {
    background: color-mix(in srgb, var(--error) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--error) 40%, transparent);
    color: var(--error);
  }

  .loading {
    text-align: center;
    color: var(--muted);
    padding: 2rem;
    font-size: 0.9rem;
  }

  .stats-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 1.5rem;
    background: var(--surface);
    border-radius: 8px;
    border: 1px solid var(--border);
    margin-bottom: 2rem;
  }

  .total-size {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted);
    font-weight: 600;
  }

  .size {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
  }

  .bytes {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .artist-count {
    font-size: 0.875rem;
    color: var(--muted);
  }

  .no-data {
    text-align: center;
    color: var(--muted);
    padding: 2rem;
    font-size: 0.9rem;
    margin: 0;
  }

  .artists-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .artist-row {
    display: grid;
    grid-template-columns: 200px 1fr 120px;
    gap: 1rem;
    align-items: center;
    padding: 0.75rem 1rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    transition: background 0.15s;
  }

  .artist-row:hover {
    background: var(--surface);
  }

  .artist-name {
    font-weight: 500;
    font-size: 0.9rem;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artist-size {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .bar-container {
    flex: 1;
    height: 6px;
    background: var(--surface-2);
    border-radius: 3px;
    overflow: hidden;
    min-width: 40px;
  }

  .bar {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-dim, var(--accent)));
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .artist-meta {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    align-items: center;
  }

  .percent {
    font-size: 0.8rem;
    color: var(--muted);
    min-width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .size {
    font-size: 0.85rem;
    color: var(--text);
    font-weight: 500;
    min-width: 50px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 768px) {
    .stats-header {
      flex-direction: column;
      gap: 1rem;
    }

    .artist-row {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }

    .artist-name {
      grid-column: 1;
    }

    .artist-size {
      grid-column: 1;
    }

    .artist-meta {
      grid-column: 1;
      justify-content: space-between;
    }
  }
</style>
