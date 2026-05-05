<script lang="ts">
  import { lib } from './store.svelte';
  import TrackTable from './TrackTable.svelte';
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

<!-- ── ALBUM DETAIL ──────────────────────────────────────────────────────── -->
{#if lib.drillAlbum}
  <div class="detail-hero">
    <div class="detail-cover">{@render coverWrap(lib.drillAlbum.cover, lib.drillAlbum.title)}</div>
    <div class="detail-info">
      <div class="detail-type">{lib.drillAlbum.album_type}</div>
      <h2>{lib.drillAlbum.title}</h2>
      <div class="detail-sub">{lib.drillAlbum.artists.map(a => a.name).join(', ')}</div>
      {#if lib.drillAlbum.date}<div class="detail-sub">{lib.drillAlbum.date}</div>{/if}
      <div class="detail-actions">
        <button class="btn-edit" onclick={() => lib.startEditAlbum(lib.drillAlbum!)}>Edit</button>
        <button class="btn-delete" onclick={() => lib.handleDeleteAlbum(lib.drillAlbum!.id)}>Delete</button>
      </div>
    </div>
  </div>
  <div class="section-title">
    {#if lib.tracksLoading}Loading tracks…{:else}{lib.albumTracks.length} track{lib.albumTracks.length !== 1 ? 's' : ''}{/if}
  </div>
  <TrackTable tracks={lib.albumTracks} showAlbumCol={false} />
  {#if !lib.tracksLoading && lib.albumTracks.length === 0}<p class="status">No tracks in this album.</p>{/if}

<!-- ── LOADING / ERROR ────────────────────────────────────────────────────── -->
{:else if lib.drillAlbumId != null && !lib.albumsLoaded}
  <p class="status">Loading…</p>
{:else if lib.albumsLoading}
  <p class="status">Loading…</p>
{:else if lib.albumsError}
  <p class="status error">{lib.albumsError}</p>

<!-- ── ALBUMS LIST / GRID ────────────────────────────────────────────────── -->
{:else}
  <div class="toolbar">
    <input class="search" placeholder="Search albums or artists… (S)" bind:value={lib.albumSearch} />
    <div class="view-toggle">
      <button class:active={lib.albumsView === 'list'} onclick={() => (lib.albumsView = 'list')} title="List">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
          <circle cx="3" cy="6" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="12" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="18" r="1" fill="currentColor" stroke="none"/>
        </svg>
      </button>
      <button class:active={lib.albumsView === 'grid'} onclick={() => (lib.albumsView = 'grid')} title="Grid">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
          <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
        </svg>
      </button>
    </div>
    <span class="count">{lib.filteredAlbums.length} album{lib.filteredAlbums.length !== 1 ? 's' : ''}</span>
  </div>

  {#if lib.albumsView === 'list'}
    <div class="table-wrap">
      <table>
        <thead><tr><th>#</th><th>Title</th><th>Artists</th><th>Type</th><th>Date</th><th class="col-actions">Actions</th></tr></thead>
        <tbody>
          {#each lib.filteredAlbums as a (a.id)}
            <tr
              onmouseenter={() => (lib.hoveredItem = { type: 'album', id: a.id })}
              onmouseleave={() => (lib.hoveredItem = null)}
            >
              <td class="muted">{a.id}</td>
              <td>
                <span class="title-link" onclick={() => lib.drillIntoAlbum(a)} role="button" tabindex="0"
                  onkeydown={(e) => e.key === 'Enter' && lib.drillIntoAlbum(a)}>{a.title}</span>
              </td>
              <td class="muted">{a.artists.map(x => x.name).join(', ') || '\u2014'}</td>
              <td class="muted">{a.album_type}</td>
              <td class="muted">{a.date ?? '\u2014'}</td>
              <td class="actions">
                <button class="btn-edit" onclick={() => lib.startEditAlbum(a)}>Edit</button>
                <button class="btn-delete" onclick={() => lib.handleDeleteAlbum(a.id)}>Delete</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="card-grid">
      {#each lib.filteredAlbums as a (a.id)}
        <div class="card clickable"
          onmouseenter={() => (lib.hoveredItem = { type: 'album', id: a.id })}
          onmouseleave={() => (lib.hoveredItem = null)}
          onclick={() => lib.drillIntoAlbum(a)} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && lib.drillIntoAlbum(a)}>
          {@render coverWrap(a.cover, a.title)}
          <div class="card-body">
            <div class="card-title" title={a.title}>{a.title}</div>
            <div class="card-sub">{a.artists.map(x => x.name).join(', ') || '\u2014'}</div>
            {#if a.date}<div class="card-meta">{a.date.slice(0, 4)}</div>{/if}
          </div>
          <div class="card-hover-actions">
            <button class="btn-edit" onclick={(e) => { e.stopPropagation(); lib.startEditAlbum(a); }}>Edit</button>
            <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeleteAlbum(a.id); }}>Delete</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
  {#if lib.filteredAlbums.length === 0}<p class="status">No albums found.</p>{/if}
{/if}

<style>
  h2 { font-size: 1.35rem; font-weight: 700; margin: 0 0 0.35rem; }
  .detail-hero { display: flex; align-items: center; gap: 1.5rem; padding: 1.5rem; background: var(--surface); border: 1px solid var(--border); border-radius: 10px; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .detail-cover { width: 110px; height: 110px; flex-shrink: 0; border-radius: 6px; overflow: hidden; }
  .detail-cover :global(.cover-wrap) { width: 100%; height: 100%; }
  .detail-info { flex: 1; min-width: 180px; }
  .detail-type { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin-bottom: 0.3rem; }
  .detail-sub { font-size: 0.875rem; color: var(--muted); margin-top: 0.2rem; }
  .detail-actions { display: flex; gap: 0.5rem; margin-top: 0.75rem; }
  .detail-actions button { padding: 0.3rem 0.75rem; border-radius: 5px; border: 1px solid var(--border); cursor: pointer; font-size: 0.8rem; font-family: inherit; background: var(--surface-2); color: var(--text); }
  .detail-actions button:hover { background: var(--surface); }
</style>
