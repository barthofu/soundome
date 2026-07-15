<script lang="ts">
  import { lib } from './store.svelte';
  import TrackTable from './TrackTable.svelte';
  import SortDropdown from './SortDropdown.svelte';

  const trackSortOptions = [
    { value: 'title', label: 'Title' },
    { value: 'artist', label: 'Artist' },
    { value: 'album', label: 'Album' },
    { value: 'date', label: 'Date' },
    { value: 'duration', label: 'Duration' },
  ];
</script>

{#snippet coverWrap(src: string | null | undefined, alt: string)}
  <div class="cover-wrap">
    {#if src && (src.startsWith('http://') || src.startsWith('https://'))}
      <img {src} {alt} class="cover-img" loading="lazy" />
    {:else}
      <div class="cover-ph">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
    {/if}
  </div>
{/snippet}

{#if lib.tracksLoading}
  <p class="status">Loading…</p>
{:else if lib.tracksError}
  <p class="status error">{lib.tracksError}</p>
{:else}
  <div class="toolbar">
    <input class="search" placeholder="Search tracks or artists… (S)" bind:value={lib.trackSearch} />
    <div class="filter-group">
      <button class="filter-btn" class:active={lib.trackFilter === 'all'} onclick={() => (lib.trackFilter = 'all')}>All</button>
      <button class="filter-btn" class:active={lib.trackFilter === 'ok'} onclick={() => (lib.trackFilter = 'ok')}>Validated</button>
      <button class="filter-btn" class:active={lib.trackFilter === 'pending'} onclick={() => (lib.trackFilter = 'pending')}>
        Needs review{#if lib.pendingCount > 0}&nbsp;<span class="mini-badge">{lib.pendingCount}</span>{/if}
      </button>
    </div>
    <SortDropdown
      value={lib.tracksSortBy}
      direction={lib.tracksSortDir}
      options={trackSortOptions}
      onChange={(val) => lib.tracksSortBy = val as any}
      onDirectionChange={(dir) => lib.tracksSortDir = dir}
    />
    <div class="view-toggle">
      <button class:active={lib.tracksView === 'list'} onclick={() => (lib.tracksView = 'list')} title="List">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
          <circle cx="3" cy="6" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="12" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="18" r="1" fill="currentColor" stroke="none"/>
        </svg>
      </button>
      <button class:active={lib.tracksView === 'grid'} onclick={() => (lib.tracksView = 'grid')} title="Grid">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
          <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
        </svg>
      </button>
    </div>
    <span class="count">{lib.filteredTracks.length} track{lib.filteredTracks.length !== 1 ? 's' : ''}</span>
  </div>

  {#if lib.tracksView === 'list'}
    <TrackTable tracks={lib.filteredTracks} showAlbumCol={true} />
    {#if lib.filteredTracks.length === 0}<p class="status">No tracks found.</p>{/if}
  {:else}
    <div class="card-grid">
      {#each lib.filteredTracks as t (t.id)}
         <div class="card"
           class:warn-border={t.needs_validation}
           role="button"
           tabindex="0"
           onmouseenter={() => (lib.hoveredItem = { type: 'track', id: t.id })}
           onmouseleave={() => (lib.hoveredItem = null)}>
          {@render coverWrap(t.cover, t.title)}
          <div class="card-body">
            <div class="card-title" title={t.title}>{t.title}</div>
            <div class="card-sub">{t.artists.map(a => a.name).join(', ') || '\u2014'}</div>
            {#if t.duration != null}<div class="card-meta mono">{lib.fmtDuration(t.duration)}</div>{/if}
          </div>
          {#if t.needs_validation}<span class="card-badge badge-warn" title="Awaiting validation">!</span>{/if}
          <div class="card-hover-actions">
            <button class="btn-edit" onclick={(e) => { e.stopPropagation(); lib.startEditTrack(t); }}>Edit</button>
            <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeleteTrack(t.id); }}>Delete</button>
          </div>
        </div>
      {/each}
    </div>
    {#if lib.filteredTracks.length === 0}<p class="status">No tracks found.</p>{/if}
  {/if}
{/if}
