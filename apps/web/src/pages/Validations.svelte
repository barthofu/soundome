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
  let focusedIndex = $state(-1);

  async function load() {
    loading = true;
    error = null;
    try {
      tracks = await getPendingValidations();
      focusedIndex = tracks.length > 0 ? 0 : -1;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  // Keyboard shortcuts (same style as Library)
  $effect(() => {
    function onKeydown(e: KeyboardEvent) {
      const tgt = e.target as HTMLElement;
      const inInput = tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.tagName === 'SELECT';
      if (inInput) return;

      if (e.key === 'j' || e.key === 'ArrowDown') {
        e.preventDefault();
        if (tracks.length > 0) focusedIndex = Math.min(focusedIndex + 1, tracks.length - 1);
      } else if (e.key === 'k' || e.key === 'ArrowUp') {
        e.preventDefault();
        if (tracks.length > 0) focusedIndex = Math.max(focusedIndex - 1, 0);
      } else if (e.key === 'e' && focusedIndex >= 0) {
        e.preventDefault();
        // Dispatch edit event to the focused TrackCard
        const card = document.querySelector(`.track-list li:nth-child(${focusedIndex + 1}) .track-card`);
        card?.dispatchEvent(new CustomEvent('shortcut-edit', { bubbles: true }));
      } else if (e.key === 'a' && focusedIndex >= 0) {
        e.preventDefault();
        const card = document.querySelector(`.track-list li:nth-child(${focusedIndex + 1}) .track-card`);
        card?.dispatchEvent(new CustomEvent('shortcut-approve', { bubbles: true }));
      } else if (e.key === 'r' && focusedIndex >= 0) {
        e.preventDefault();
        const card = document.querySelector(`.track-list li:nth-child(${focusedIndex + 1}) .track-card`);
        card?.dispatchEvent(new CustomEvent('shortcut-reject', { bubbles: true }));
      } else if (e.key === 'm' && focusedIndex >= 0) {
        e.preventDefault();
        const card = document.querySelector(`.track-list li:nth-child(${focusedIndex + 1}) .track-card`);
        card?.dispatchEvent(new CustomEvent('shortcut-matches', { bubbles: true }));
      } else if (e.key === 's') {
        e.preventDefault();
        document.querySelector<HTMLInputElement>('.validations-page .search')?.focus();
      }
    }
    document.addEventListener('keydown', onKeydown);
    return () => document.removeEventListener('keydown', onKeydown);
  });

  async function handleApprove(id: number, patch: PatchValidationBody) {
    await approveValidation(id, patch);
    tracks = tracks.filter((t) => t.id !== id);
    if (focusedIndex >= tracks.length) focusedIndex = tracks.length - 1;
    onDownloaded?.();
  }

  async function handleReject(id: number) {
    await rejectValidation(id);
    tracks = tracks.filter((t) => t.id !== id);
    if (focusedIndex >= tracks.length) focusedIndex = tracks.length - 1;
    onDownloaded?.();
  }
</script>

<div class="validations-page">
  <header class="page-header">
    <h1>Pending validations</h1>
    <div class="header-actions">
      <span class="shortcuts-hint">j/k navigate · e edit · a approve · r reject · m matches</span>
      <button onclick={load} disabled={loading}>
        {loading ? 'Loading…' : 'Refresh'}
      </button>
    </div>
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
      {#each tracks as track, i (track.id)}
        <li>
          <TrackCard {track} focused={i === focusedIndex} onApprove={handleApprove} onReject={handleReject} />
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .validations-page {
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .shortcuts-hint {
    font-size: 0.7rem;
    color: var(--muted);
    opacity: 0.7;
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
