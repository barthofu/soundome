<script lang="ts">
  import { getPendingValidations, approveValidation, rejectValidation } from '../lib/api';
  import TrackCard from '../lib/TrackCard.svelte';
  import type { PendingValidationDto, PatchValidationBody } from '../lib/types';

  interface Props {
    onDownloaded?: () => void;
  }
  let { onDownloaded }: Props = $props();

  type Tab = 'partial' | 'no_match' | 'drm';

  let tracks: PendingValidationDto[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let search = $state('');
  let activeTab: Tab = $state('partial');

  // ── Grouped by reason ──────────────────────────────────────────────────────
  let partialTracks = $derived(
    tracks.filter((t) => t.validation_reason === 'metadata_partial_match'),
  );
  let noMatchTracks = $derived(
    tracks.filter((t) => t.validation_reason === 'metadata_no_match'),
  );
  let drmTracks = $derived(
    tracks.filter((t) => t.validation_reason === 'soundcloud_drm_protected'),
  );

  // Active tab tracks
  let activeTracks = $derived(
    activeTab === 'partial' ? partialTracks : activeTab === 'no_match' ? noMatchTracks : drmTracks,
  );

  // Filtered within active tab
  let filteredTracks = $derived(
    search.trim() === ''
      ? activeTracks
      : activeTracks.filter((t) => {
          const q = search.toLowerCase();
          return (
            t.title.toLowerCase().includes(q) ||
            t.artists.some((a) => a.name.toLowerCase().includes(q)) ||
            (t.album?.title.toLowerCase().includes(q) ?? false)
          );
        }),
  );

  // ── Load ───────────────────────────────────────────────────────────────────
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

  // ── Handlers ──────────────────────────────────────────────────────────────
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

<div class="validations-page">
  <header class="page-header">
    <h1>Validations</h1>
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
    <!-- Tabs -->
    <div class="tabs" role="tablist">
      <button
        role="tab"
        class="tab"
        class:active={activeTab === 'partial'}
        onclick={() => { activeTab = 'partial'; search = ''; }}
        aria-selected={activeTab === 'partial'}
      >
        Partial Match
        {#if partialTracks.length > 0}
          <span class="tab-badge">{partialTracks.length}</span>
        {/if}
      </button>
      <button
        role="tab"
        class="tab"
        class:active={activeTab === 'no_match'}
        onclick={() => { activeTab = 'no_match'; search = ''; }}
        aria-selected={activeTab === 'no_match'}
      >
        No Match
        {#if noMatchTracks.length > 0}
          <span class="tab-badge">{noMatchTracks.length}</span>
        {/if}
      </button>
      <button
        role="tab"
        class="tab"
        class:active={activeTab === 'drm'}
        onclick={() => { activeTab = 'drm'; search = ''; }}
        aria-selected={activeTab === 'drm'}
      >
        Errors
        {#if drmTracks.length > 0}
          <span class="tab-badge tab-badge--red">{drmTracks.length}</span>
        {/if}
      </button>
    </div>

    <!-- Tab description -->
    <p class="tab-desc">
      {#if activeTab === 'partial'}
        A metadata provider found a likely match, but confidence was not high enough for automatic approval. Review the candidates and confirm or correct.
      {:else if activeTab === 'no_match'}
        No metadata match was found automatically. Edit the metadata manually before approving.
      {:else}
        SoundCloud track is DRM-protected and could not be downloaded. Find the matching YouTube video and select it as the audio source.
      {/if}
    </p>

    <!-- Search -->
    {#if activeTracks.length > 0}
      <div class="search-bar">
        <input
          type="text"
          placeholder="Filter by title, artist, album…"
          bind:value={search}
        />
      </div>
    {/if}

    <!-- Track list -->
    {#if activeTracks.length === 0}
      <p class="status empty">Nothing here 🎉</p>
    {:else}
      <p class="count">
        {filteredTracks.length} / {activeTracks.length} track{activeTracks.length > 1 ? 's' : ''}
      </p>
      <ul class="track-list" role="list">
        {#each filteredTracks as track (track.id)}
          <li>
            <TrackCard {track} onApprove={handleApprove} onReject={handleReject} />
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .validations-page {
    max-width: 820px;
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
    color: inherit;
  }

  button:hover:not(:disabled) {
    background: var(--surface-2);
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  /* ── Tabs ── */

  .tabs {
    display: flex;
    gap: 0.25rem;
    border-bottom: 1px solid var(--border);
    margin-bottom: 1rem;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1rem;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    background: none;
    color: var(--muted);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    margin-bottom: -1px;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover:not(:disabled) {
    background: none;
    color: var(--text);
  }

  .tab.active {
    color: var(--text);
    border-bottom-color: var(--accent);
    background: none;
  }

  .tab-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: var(--surface-2);
    color: var(--muted);
    font-size: 0.68rem;
    font-weight: 700;
    border-radius: 9px;
  }

  .tab.active .tab-badge {
    background: var(--accent-muted, rgba(99, 102, 241, 0.18));
    color: var(--accent);
  }

  .tab-badge--red {
    background: rgba(220, 38, 38, 0.12);
    color: #dc2626;
  }

  /* ── Tab description ── */

  .tab-desc {
    font-size: 0.8rem;
    color: var(--muted);
    margin: 0 0 1rem 0;
    line-height: 1.5;
  }

  /* ── Search ── */

  .search-bar {
    margin-bottom: 1rem;
  }

  .search-bar input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    font-size: 0.875rem;
    color: inherit;
    box-sizing: border-box;
  }

  .search-bar input::placeholder {
    color: var(--muted);
  }

  /* ── Status ── */

  .status {
    text-align: center;
    color: var(--muted);
    padding: 3rem 0;
    margin: 0;
  }

  .status.error {
    color: var(--error, #dc2626);
  }

  .count {
    font-size: 0.875rem;
    color: var(--muted);
    margin-bottom: 1rem;
  }

  /* ── List ── */

  .track-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
</style>
