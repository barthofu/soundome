<script lang="ts">
  import { lib } from '../lib/library/store.svelte';
  import ArtistTab from '../lib/library/ArtistTab.svelte';
  import AlbumTab from '../lib/library/AlbumTab.svelte';
  import TracksTab from '../lib/library/TracksTab.svelte';
  import PlaylistsTab from '../lib/library/PlaylistsTab.svelte';
  import EditModal from '../lib/library/EditModal.svelte';

  // Load data for initial tab / drilled state
  $effect(() => {
    if (lib.tab === 'tracks' && !lib.tracksLoaded) lib.loadTracks();
    if (lib.tab === 'albums' && !lib.albumsLoaded) lib.loadAlbums();
    if (lib.tab === 'artists' && !lib.artistsLoaded) lib.loadArtists();
    if (lib.tab === 'playlists' && !lib.playlistsLoaded) lib.loadPlaylists();
  });

  // Load extra data when drilling
  $effect(() => {
    if (lib.drillArtistId != null) {
      if (!lib.albumsLoaded) lib.loadAlbums();
      if (!lib.tracksLoaded) lib.loadTracks();
    } else if (lib.drillAlbumId != null) {
      if (!lib.tracksLoaded) lib.loadTracks();
    }
  });

  // Start auto-poll when the Library page mounts, stop when it unmounts.
  $effect(() => {
    lib.startPoll();
    return () => lib.stopPoll();
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
  .library-page { max-width: 1200px; margin: 0 auto; padding: 2rem 1rem; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }
  h1 { font-size: 1.5rem; font-weight: 700; margin: 0; }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .last-refreshed {
    font-size: 0.78rem;
    color: var(--muted);
  }

  .btn-header {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    cursor: pointer;
    font-size: 0.875rem;
    color: inherit;
    font-family: inherit;
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

  .tabs { display: flex; gap: 0.25rem; border-bottom: 1px solid var(--border); margin-bottom: 1.5rem; }
  .tab { background: none; border: none; color: var(--muted); font-size: 0.875rem; padding: 0.5rem 1rem; cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -1px; border-radius: 4px 4px 0 0; display: flex; align-items: center; gap: 0.4rem; transition: color 0.15s; font-family: inherit; }
  .tab:hover { color: var(--text); }
  .tab.active { color: var(--accent); border-bottom-color: var(--accent); }
  .tab-count { font-size: 0.75rem; color: var(--muted); background: var(--surface-2); border-radius: 10px; padding: 0 0.4rem; }
  .tab-badge { background: var(--warning); color: #000; font-size: 0.68rem; font-weight: 700; border-radius: 10px; padding: 0 0.35rem; }

  .breadcrumb { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1.25rem; font-size: 0.875rem; }
  .crumb-btn { background: none; border: none; color: var(--accent); cursor: pointer; padding: 0; font-size: inherit; font-family: inherit; }
  .crumb-btn:hover { text-decoration: underline; }
  .crumb-sep { color: var(--muted); }
  .crumb-current { color: var(--text); font-weight: 500; }
  .muted { color: var(--muted); }
</style>
