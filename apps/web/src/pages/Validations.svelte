<script lang="ts">
  import { getPendingValidations, approveValidation, rejectValidation } from '../lib/api';
  import TrackCard from '../lib/TrackCard.svelte';
  import type { PendingValidationDto } from '../lib/types';
  import type { PatchValidationBody } from '../lib/types';

  interface Props {
    onDownloaded?: () => void;
  }
  let { onDownloaded }: Props = $props();

  let tracks: PendingValidationDto[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  async function load() {
    loading = true;
    error = null;
    try {
      tracks = await getPendingValidations();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  async function handleApprove(id: number, patch: PatchValidationBody) {
    await approveValidation(id, patch);
    tracks = tracks.filter((t) => t.id !== id);
    onDownloaded?.();
  }

  async function handleReject(id: number) {
    await rejectValidation(id);
    tracks = tracks.filter((t) => t.id !== id);
    onDownloaded?.();
  }
</script>

<div class="page">
  <header class="page-header">
    <h1>Pending validations</h1>
    <button onclick={load} disabled={loading}>
      {loading ? 'Loading…' : 'Refresh'}
    </button>
  </header>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if tracks.length === 0}
    <p class="status empty">No pending validations 🎉</p>
  {:else}
    <p class="count">{tracks.length} track{tracks.length > 1 ? 's' : ''} awaiting review</p>
    <ul class="track-list">
      {#each tracks as track (track.id)}
        <li>
          <TrackCard {track} onApprove={handleApprove} onReject={handleReject} />
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .page {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
  }

  button {
    padding: 0.4rem 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button:hover:not(:disabled) {
    background: var(--surface-2);
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .status {
    text-align: center;
    color: var(--muted);
    padding: 3rem 0;
  }

  .status.error {
    color: var(--error);
  }

  .count {
    font-size: 0.875rem;
    color: var(--muted);
    margin-bottom: 1rem;
  }

  .track-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
</style>
