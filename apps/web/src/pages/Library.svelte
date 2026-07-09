<script lang="ts">
  import { lib } from '../lib/library/store.svelte';
  import ArtistTab from '../lib/library/ArtistTab.svelte';
  import AlbumTab from '../lib/library/AlbumTab.svelte';
  import TracksTab from '../lib/library/TracksTab.svelte';
  import PlaylistsTab from '../lib/library/PlaylistsTab.svelte';
  import EditModal from '../lib/library/EditModal.svelte';

  // Refresh all collections when arriving on this page.
  $effect(() => {
    lib.loadAll();
  });

  // Browser back / forward
  $effect(() => {
    function onPopState() { lib.applyHash(); }
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  });

  // Keyboard shortcuts
  $effect(() => {
    function onKeydown(e: KeyboardEvent) {
      const tgt = e.target as HTMLElement;
      const inInput = tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.tagName === 'SELECT';
      if (lib.editState || inInput) return;
      if (e.key === 's') {
        e.preventDefault();
        document.querySelector<HTMLInputElement>('.library-page .search')?.focus();
      } else if (e.key === 'e' && lib.hoveredItem) {
        e.preventDefault();
        lib.openEditForHovered();
      } else if (e.key === 'Backspace' && !e.metaKey && !e.ctrlKey) {
        if (lib.drillAlbumId != null) { e.preventDefault(); lib.navigate(lib.tab, lib.drillArtistId ?? undefined); }
        else if (lib.drillArtistId != null) { e.preventDefault(); lib.navigate(lib.tab); }
      } else if (e.key === 'Escape') {
        if (lib.mergePicking) { e.preventDefault(); lib.cancelMergePicking(); }
        else if (lib.selectedArtistIds.size > 0) { e.preventDefault(); lib.clearArtistSelection(); }
      } else if (e.key === 'm' && lib.tab === 'artists' && lib.selectedArtistIds.size >= 2) {
        e.preventDefault();
        if (lib.mergePicking) lib.cancelMergePicking();
        else lib.startMergePicking();
      }
    }
    document.addEventListener('keydown', onKeydown);
    return () => document.removeEventListener('keydown', onKeydown);
  });

  function formatLastRefreshed(d: Date | null): string {
    if (!d) return '';
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
</script>

<div class="library-page">
  <div class="page-header">
    <h1>Library</h1>
    <div class="header-right">
      {#if lib.lastRefreshed}
        <span class="last-refreshed">Updated {formatLastRefreshed(lib.lastRefreshed)}</span>
      {/if}
      <button class="btn-header" onclick={lib.handleRefresh} disabled={lib.refreshing}>
        {#if lib.refreshing}
          <span class="spinner"></span> Refreshing…
        {:else}
          Refresh
        {/if}
      </button>
    </div>
  </div>

  <div class="tabs">
    {#each (['artists', 'albums', 'tracks', 'playlists'] as const) as t}
      <button class="tab" class:active={lib.tab === t} onclick={() => lib.switchTab(t)}>
        {t === 'artists' ? 'Artists' : t === 'albums' ? 'Albums' : t === 'tracks' ? 'Tracks' : 'Playlists'}
        {#if t === 'artists' && lib.artistsLoaded}<span class="tab-count">{lib.artists.length}</span>{/if}
        {#if t === 'albums' && lib.albumsLoaded}<span class="tab-count">{lib.albums.length}</span>{/if}
        {#if t === 'tracks' && lib.tracksLoaded}<span class="tab-count">{lib.tracks.length}</span>{/if}
        {#if t === 'tracks' && lib.pendingCount > 0}<span class="tab-badge">{lib.pendingCount}</span>{/if}
        {#if t === 'playlists' && lib.playlistsLoaded}<span class="tab-count">{lib.playlists.length}</span>{/if}
      </button>
    {/each}
  </div>

  {#if (lib.tab === 'artists' || lib.tab === 'albums') && (lib.batchFetchingArtists || lib.batchFetchingAlbums || lib.batchFetchResult)}
    <div class="batch-tools">
      {#if lib.tab === 'artists'}
        <button
          class="btn-batch"
          onclick={() => lib.batchFetchArtistIconsAction()}
          disabled={lib.batchFetchingArtists}
        >
          {#if lib.batchFetchingArtists}
            ⏳ Fetching icons…
          {:else}
            🖼️ Fetch all artist photos from references
          {/if}
        </button>
      {:else if lib.tab === 'albums'}
        <button
          class="btn-batch"
          onclick={() => lib.batchFetchAlbumCoversAction()}
          disabled={lib.batchFetchingAlbums}
        >
          {#if lib.batchFetchingAlbums}
            ⏳ Fetching covers…
          {:else}
            🖼️ Fetch all album covers from references
          {/if}
        </button>
      {/if}
      {#if lib.batchFetchResult}
        <span class="batch-result">
          {lib.batchFetchResult.count} fetched · {lib.batchFetchResult.skipped} not found
        </span>
      {/if}
    </div>
  {/if}

  {#if lib.drillArtist || lib.drillAlbum || lib.drillArtistId || lib.drillAlbumId}
    <nav class="breadcrumb">
      <button class="crumb-btn" onclick={lib.backToRoot}>
        {lib.tab === 'artists' ? 'Artists' : 'Albums'}
      </button>
      {#if lib.drillArtist}
        <span class="crumb-sep">›</span>
        {#if lib.drillAlbum}
          <button class="crumb-btn" onclick={lib.backToArtist}>{lib.drillArtist.name}</button>
          <span class="crumb-sep">›</span>
          <span class="crumb-current">{lib.drillAlbum.title}</span>
        {:else}
          <span class="crumb-current">{lib.drillArtist.name}</span>
        {/if}
      {:else if lib.drillAlbum}
        <span class="crumb-sep">›</span>
        <span class="crumb-current">{lib.drillAlbum.title}</span>
      {:else}
        <span class="crumb-sep">›</span>
        <span class="crumb-current muted">Loading…</span>
      {/if}
    </nav>
  {:else if lib.drillPlaylistId != null}
    <nav class="breadcrumb">
      <button class="crumb-btn" onclick={() => lib.navigate('playlists')}>Playlists</button>
      <span class="crumb-sep">›</span>
      {#if lib.drillPlaylist}
        <span class="crumb-current">{lib.drillPlaylist.name}</span>
      {:else}
        <span class="crumb-current muted">Loading…</span>
      {/if}
    </nav>
  {/if}

  {#if lib.tab === 'artists'}
    <ArtistTab />
  {:else if lib.tab === 'albums'}
    <AlbumTab />
  {:else if lib.tab === 'playlists'}
    <PlaylistsTab />
  {:else}
    <TracksTab />
  {/if}
</div>

<EditModal />

<style>
  .library-page { 
    max-width: 1400px; 
    margin: 0 auto; 
    padding: 1rem 0.75rem;
  }

  @media (min-width: 640px) {
    .library-page {
      padding: 1.5rem 1rem;
    }
  }

  @media (min-width: 1024px) {
    .library-page {
      padding: 2rem 1.5rem;
    }
  }

  .page-header { 
    display: flex; 
    flex-direction: column;
    gap: 1rem;
    align-items: flex-start;
    justify-content: space-between; 
    margin-bottom: 1.5rem;
  }

  @media (min-width: 640px) {
    .page-header {
      flex-direction: row;
      align-items: center;
    }
  }

  h1 { 
    font-size: 1.25rem; 
    font-weight: 700; 
    margin: 0;
  }

  @media (min-width: 768px) {
    h1 {
      font-size: 1.5rem;
    }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    width: 100%;
  }

  @media (min-width: 640px) {
    .header-right {
      width: auto;
      gap: 0.75rem;
    }
  }

  .last-refreshed {
    font-size: 0.7rem;
    color: var(--muted);
    display: none;
  }

  @media (min-width: 640px) {
    .last-refreshed {
      display: block;
      font-size: 0.78rem;
    }
  }

  .btn-header {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.7rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    cursor: pointer;
    font-size: 0.75rem;
    color: inherit;
    font-family: inherit;
    white-space: nowrap;
  }

  @media (min-width: 768px) {
    .btn-header {
      padding: 0.4rem 1rem;
      font-size: 0.875rem;
      gap: 0.4rem;
    }
  }

  .btn-header:hover:not(:disabled) { background: var(--surface-2); }
  .btn-header:disabled { opacity: 0.5; cursor: default; }

  .spinner {
    display: inline-block;
    width: 11px;
    height: 11px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .tabs { 
    display: flex; 
    gap: 0.25rem; 
    border-bottom: 1px solid var(--border); 
    margin-bottom: 1.5rem;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .tab { 
    background: none; 
    border: none; 
    color: var(--muted); 
    font-size: 0.75rem; 
    padding: 0.4rem 0.7rem; 
    cursor: pointer; 
    border-bottom: 2px solid transparent; 
    margin-bottom: -1px; 
    border-radius: 4px 4px 0 0; 
    display: flex; 
    align-items: center; 
    gap: 0.3rem; 
    transition: color 0.15s; 
    font-family: inherit;
    white-space: nowrap;
  }

  @media (min-width: 768px) {
    .tab {
      font-size: 0.875rem;
      padding: 0.5rem 1rem;
      gap: 0.4rem;
    }
  }

  .tab:hover { color: var(--text); }
  .tab.active { color: var(--accent); border-bottom-color: var(--accent); }
  .tab-count { 
    font-size: 0.65rem; 
    color: var(--muted); 
    background: var(--surface-2); 
    border-radius: 10px; 
    padding: 0 0.3rem;
  }

  @media (min-width: 768px) {
    .tab-count {
      font-size: 0.75rem;
      padding: 0 0.4rem;
    }
  }

  .tab-badge { 
    background: var(--warning); 
    color: #000; 
    font-size: 0.6rem; 
    font-weight: 700; 
    border-radius: 10px; 
    padding: 0 0.25rem;
  }

  @media (min-width: 768px) {
    .tab-badge {
      font-size: 0.68rem;
      padding: 0 0.35rem;
    }
  }

  .batch-tools {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    padding: 0.75rem 1rem;
    background: var(--surface-2);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .btn-batch {
    background: var(--accent);
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 0.85rem;
    padding: 0.5rem 0.9rem;
    cursor: pointer;
    font-family: inherit;
    font-weight: 500;
    white-space: nowrap;
    transition: opacity 0.15s;
  }

  .btn-batch:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-batch:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .batch-result {
    font-size: 0.75rem;
    color: var(--muted);
    margin-left: auto;
  }

  .breadcrumb { 
    display: flex; 
    align-items: center; 
    gap: 0.4rem; 
    margin-bottom: 1.25rem; 
    font-size: 0.75rem;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  @media (min-width: 768px) {
    .breadcrumb {
      font-size: 0.875rem;
      gap: 0.5rem;
    }
  }

  .crumb-btn { 
    background: none; 
    border: none; 
    color: var(--accent); 
    cursor: pointer; 
    padding: 0; 
    font-size: inherit; 
    font-family: inherit;
    white-space: nowrap;
  }
  .crumb-btn:hover { text-decoration: underline; }
  .crumb-sep { color: var(--muted); }
  .crumb-current { color: var(--text); font-weight: 500; }
  .muted { color: var(--muted); }
</style>
