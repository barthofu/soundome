<script lang="ts">
  import { lib } from './store.svelte';
  import TrackTable from './TrackTable.svelte';

  function musicIcon() { return '\u266B'; } // unused, inline SVGs below
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

{#snippet artistCover(src: string | null | undefined, alt: string)}
  <div class="cover-wrap artist-avatar">
    {#if src && (src.startsWith('http://') || src.startsWith('https://'))}
      <img {src} {alt} class="cover-img" loading="lazy" />
    {:else}
      <div class="cover-ph">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
      </div>
    {/if}
  </div>
{/snippet}

<!-- ── ALBUM DETAIL (within artist context) ──────────────────────────────── -->
{#if lib.drillArtist && lib.drillAlbum}
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

<!-- ── ARTIST DETAIL ─────────────────────────────────────────────────────── -->
{:else if lib.drillArtist}
  <div class="detail-hero">
    {@render artistCover(lib.drillArtist.icon, lib.drillArtist.name)}
    <div class="detail-info">
      <div class="detail-type">Artist</div>
      <h2>{lib.drillArtist.name}</h2>
      <div class="detail-sub">
        {#if lib.albumsLoading}…{:else}{lib.artistAlbums.length} album{lib.artistAlbums.length !== 1 ? 's' : ''}{/if}
        &nbsp;·&nbsp;
        {#if lib.tracksLoading}…{:else}{lib.artistTracks.length} track{lib.artistTracks.length !== 1 ? 's' : ''}{/if}
      </div>
      <div class="detail-actions">
        <button class="btn-edit" onclick={() => lib.startEditArtist(lib.drillArtist!)}>Edit</button>
        <button class="btn-delete" onclick={() => lib.handleDeleteArtist(lib.drillArtist!.id)}>Delete</button>
      </div>
    </div>
  </div>

  <!-- Albums compact grid -->
  {#if lib.albumsLoading}
    <p class="status" style="padding:1rem 0">Loading albums…</p>
  {:else if lib.artistAlbums.length > 0}
    <div class="section-title">Albums</div>
    <div class="card-grid compact" style="margin-bottom:1.5rem">
      {#each lib.artistAlbums as a (a.id)}
        <div class="card clickable"
          onmouseenter={() => (lib.hoveredItem = { type: 'album', id: a.id })}
          onmouseleave={() => (lib.hoveredItem = null)}
          onclick={() => lib.drillIntoAlbum(a)} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && lib.drillIntoAlbum(a)}>
          {@render coverWrap(a.cover, a.title)}
          <div class="card-body">
            <div class="card-title" title={a.title}>{a.title}</div>
            {#if a.date}<div class="card-meta">{a.date.slice(0, 4)}</div>{/if}
          </div>
          <div class="card-hover-actions">
            <button class="btn-edit" onclick={(e) => { e.stopPropagation(); lib.startEditAlbum(a); }}>Edit</button>
            <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeleteAlbum(a.id); }}>Del</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Tracks grouped by album -->
  {#if lib.tracksLoading}
    <p class="status" style="padding:1rem 0">Loading tracks…</p>
  {:else if lib.artistTracksByAlbum.length > 0}
    <div class="section-title">Tracks</div>
    {#each lib.artistTracksByAlbum as group (group.albumId ?? '__none__')}
      <div class="album-section">
        <div class="album-section-header">
          {#if group.albumId != null}
            <div class="album-section-thumb">{@render coverWrap(group.albumCover, group.albumTitle ?? '')}</div>
            {@const fullAlbum = lib.albums.find(x => x.id === group.albumId)}
            {#if fullAlbum}
              <span class="album-section-name title-link"
                onclick={() => lib.drillIntoAlbum(fullAlbum)}
                role="button" tabindex="0"
                onkeydown={(e) => e.key === 'Enter' && lib.drillIntoAlbum(fullAlbum)}>
                {group.albumTitle}
              </span>
            {:else}
              <span class="album-section-name">{group.albumTitle}</span>
            {/if}
          {:else}
            <span class="album-section-name muted">No album</span>
          {/if}
          <span class="album-section-count">{group.tracks.length} track{group.tracks.length !== 1 ? 's' : ''}</span>
        </div>
        <TrackTable tracks={group.tracks} showAlbumCol={false} />
      </div>
    {/each}
  {:else if !lib.albumsLoading && lib.artistAlbums.length === 0}
    <p class="status">No content found for this artist.</p>
  {/if}

<!-- ── LOADING / ERROR ────────────────────────────────────────────────────── -->
{:else if lib.drillArtistId != null && !lib.artistsLoaded}
  <p class="status">Loading…</p>
{:else if lib.artistsLoading}
  <p class="status">Loading…</p>
{:else if lib.artistsError}
  <p class="status error">{lib.artistsError}</p>

<!-- ── ARTISTS LIST / GRID ───────────────────────────────────────────────── -->
{:else}
  <div class="toolbar">
    <input class="search" placeholder="Search artists… (S)" bind:value={lib.artistSearch} />
    <button
      class="btn-similar"
      class:active={lib.similarFilterActive}
      onclick={() => { lib.similarFilterActive = !lib.similarFilterActive; }}
      title="Dim artists that have no similar-named peer — helps spot duplicates"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      Similar
      {#if lib.similarFilterActive && lib.similarArtistIds.size > 0}
        <span class="similar-badge">{lib.similarArtistIds.size}</span>
      {/if}
    </button>
    <div class="view-toggle">
      <button class:active={lib.artistsView === 'list'} onclick={() => (lib.artistsView = 'list')} title="List">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
          <circle cx="3" cy="6" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="12" r="1" fill="currentColor" stroke="none"/>
          <circle cx="3" cy="18" r="1" fill="currentColor" stroke="none"/>
        </svg>
      </button>
      <button class:active={lib.artistsView === 'grid'} onclick={() => (lib.artistsView = 'grid')} title="Grid">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
          <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
        </svg>
      </button>
    </div>
    <span class="count">{lib.filteredArtists.length} artist{lib.filteredArtists.length !== 1 ? 's' : ''}</span>
  </div>

  {#if lib.artistsView === 'list'}
    <div class="table-wrap">
      <table>
        <thead><tr><th>#</th><th>Name</th><th class="col-actions">Actions</th></tr></thead>
        <tbody>
          {#each lib.filteredArtists as a (a.id)}
            {@const sel = lib.selectedArtistIds.has(a.id)}
            {@const dimmed = lib.similarFilterActive && !lib.similarArtistIds.has(a.id)}
            {@const pickable = lib.mergePicking && sel}
            <tr
              class:row-selected={sel}
              class:row-dimmed={dimmed}
              class:row-pickable={pickable}
              style={lib.mergePicking && !sel ? 'opacity:0.3;pointer-events:none' : ''}
              onmouseenter={() => (lib.hoveredItem = { type: 'artist', id: a.id })}
              onmouseleave={() => (lib.hoveredItem = null)}
              onclick={(e) => {
                if (lib.mergePicking) { if (sel) lib.pickMergeTarget(a.id); }
                else if (e.shiftKey) { e.preventDefault(); lib.toggleArtistSelection(a.id); }
                else { lib.drillIntoArtist(a); }
              }}
            >
              <td class="muted">{a.id}</td>
              <td>
                <span class:muted={dimmed}>{a.name}</span>
                {#if sel}<span class="badge-sel">✓</span>{/if}
              </td>
              <td class="actions">
                {#if !lib.mergePicking}
                  <button class="btn-edit" onclick={(e) => { e.stopPropagation(); lib.startEditArtist(a); }}>Edit</button>
                  <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeleteArtist(a.id); }}>Delete</button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="card-grid">
      {#each lib.filteredArtists as a (a.id)}
        {@const sel = lib.selectedArtistIds.has(a.id)}
        {@const dimmed = lib.similarFilterActive && !lib.similarArtistIds.has(a.id)}
        {@const pickable = lib.mergePicking && sel}
        <div class="card"
          class:card-selected={sel}
          class:card-dimmed={dimmed}
          class:card-pickable={pickable}
          style={lib.mergePicking && !sel ? 'opacity:0.3;pointer-events:none' : ''}
          onmouseenter={() => (lib.hoveredItem = { type: 'artist', id: a.id })}
          onmouseleave={() => (lib.hoveredItem = null)}
          onclick={(e) => {
            if (lib.mergePicking) { if (sel) lib.pickMergeTarget(a.id); }
            else if (e.shiftKey) { e.preventDefault(); lib.toggleArtistSelection(a.id); }
            else { lib.drillIntoArtist(a); }
          }}
          role="button" tabindex="0"
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              if (lib.mergePicking && sel) lib.pickMergeTarget(a.id);
              else if (!lib.mergePicking) lib.drillIntoArtist(a);
            } else if (e.key === ' ') { e.preventDefault(); lib.toggleArtistSelection(a.id); }
          }}
        >
          {@render artistCover(a.icon, a.name)}
          <div class="card-body">
            <div class="card-title" title={a.name}>{a.name}</div>
          </div>
          {#if sel}<span class="card-sel-badge">✓</span>{/if}
          {#if !lib.mergePicking}
            <div class="card-hover-actions">
              <button class="btn-edit" onclick={(e) => { e.stopPropagation(); lib.startEditArtist(a); }}>Edit</button>
              <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeleteArtist(a.id); }}>Delete</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
  {#if lib.filteredArtists.length === 0}<p class="status">No artists found.</p>{/if}

  <!-- Floating merge / pick-target button -->
  {#if lib.selectedArtistIds.size >= 2}
    <div class="merge-fab" class:fab-picking={lib.mergePicking}>
      {#if lib.mergePicking}
        <span class="fab-hint">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          Click the artist to keep
        </span>
        <button class="fab-btn-cancel" onclick={lib.cancelMergePicking} disabled={lib.mergeSaving}>Cancel</button>
      {:else}
        <span class="fab-count">{lib.selectedArtistIds.size}</span>
        <button class="fab-btn-merge" onclick={lib.startMergePicking} disabled={lib.mergeSaving}>
          {lib.mergeSaving ? 'Merging…' : 'Merge'}
        </button>
        <button class="fab-btn-cancel" onclick={lib.clearArtistSelection} title="Cancel selection">✕</button>
      {/if}
    </div>
  {/if}
{/if}

<style>
  h2 { font-size: 1.35rem; font-weight: 700; margin: 0 0 0.35rem; }
  .album-section { margin-bottom: 2rem; }
  .album-section-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.65rem; }
  .album-section-thumb { width: 36px; height: 36px; flex-shrink: 0; border-radius: 4px; overflow: hidden; }
  .album-section-thumb :global(.cover-wrap) { width: 100%; height: 100%; }
  .album-section-name { font-weight: 600; font-size: 0.9rem; }
  .album-section-count { font-size: 0.75rem; color: var(--muted); margin-left: auto; }
  .detail-hero { display: flex; align-items: center; gap: 1.5rem; padding: 1.5rem; background: var(--surface); border: 1px solid var(--border); border-radius: 10px; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .detail-cover { width: 110px; height: 110px; flex-shrink: 0; border-radius: 6px; overflow: hidden; }
  .detail-cover :global(.cover-wrap) { width: 100%; height: 100%; }
  .detail-info { flex: 1; min-width: 180px; }
  .detail-type { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin-bottom: 0.3rem; }
  .detail-sub { font-size: 0.875rem; color: var(--muted); margin-top: 0.2rem; }
  .detail-actions { display: flex; gap: 0.5rem; margin-top: 0.75rem; }
  .detail-actions button { padding: 0.3rem 0.75rem; border-radius: 5px; border: 1px solid var(--border); cursor: pointer; font-size: 0.8rem; font-family: inherit; background: var(--surface-2); color: var(--text); }
  .detail-actions button:hover { background: var(--surface); }

  /* Similar filter */
  .btn-similar {
    display: inline-flex; align-items: center; gap: 0.35rem; white-space: nowrap;
    padding: 0.28rem 0.65rem; border-radius: 5px; border: 1px solid var(--border);
    background: var(--surface-2); color: var(--muted); cursor: pointer; font-size: 0.8rem; font-family: inherit;
  }
  .btn-similar:hover { color: var(--text); }
  .btn-similar.active { background: color-mix(in srgb, #f59e0b 12%, var(--surface)); color: #f59e0b; border-color: color-mix(in srgb, #f59e0b 40%, transparent); }
  .similar-badge { background: #f59e0b; color: #000; border-radius: 999px; padding: 0 0.35rem; font-size: 0.7rem; font-weight: 700; }

  /* Row selection */
  tr.row-selected td { background: color-mix(in srgb, var(--accent) 9%, transparent); }
  tr.row-dimmed { opacity: 0.2; }
  tr.row-pickable { cursor: pointer; }
  tr.row-pickable:hover td { background: color-mix(in srgb, #22c55e 14%, transparent) !important; }
  .badge-sel {
    display: inline-block; margin-left: 0.35rem; vertical-align: middle;
    background: var(--accent); color: #fff; border-radius: 3px;
    padding: 0 0.28rem; font-size: 0.66rem; font-weight: 700;
  }
  tr.row-pickable .badge-sel { background: #22c55e; }

  /* Card selection */
  .card { position: relative; }
  .card-selected { outline: 2px solid var(--accent); outline-offset: -2px; }
  .card-dimmed { opacity: 0.2; }
  .card-pickable { cursor: pointer; }
  .card-pickable:hover { outline-color: #22c55e !important; background: color-mix(in srgb, #22c55e 10%, var(--surface)); }
  .card-sel-badge {
    position: absolute; top: 0.3rem; right: 0.3rem; z-index: 2;
    background: var(--accent); color: #fff; border-radius: 50%;
    width: 1.2rem; height: 1.2rem; font-size: 0.65rem; font-weight: 700;
    display: flex; align-items: center; justify-content: center;
  }
  .card-pickable .card-sel-badge { background: #22c55e; }

  /* Floating merge button */
  .merge-fab {
    position: fixed; bottom: 1.75rem; left: 50%; transform: translateX(-50%);
    display: flex; align-items: center; gap: 0.5rem; white-space: nowrap;
    padding: 0.55rem 0.9rem;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 999px; box-shadow: 0 4px 22px rgba(0,0,0,0.35);
    z-index: 200; font-size: 0.875rem;
  }
  .merge-fab.fab-picking {
    background: color-mix(in srgb, #22c55e 12%, var(--surface));
    border-color: color-mix(in srgb, #22c55e 50%, transparent);
  }
  .fab-count { background: var(--accent); color: #fff; border-radius: 999px; padding: 0.05rem 0.55rem; font-size: 0.75rem; font-weight: 700; }
  .fab-btn-merge {
    padding: 0.35rem 1rem; border-radius: 999px; border: none; cursor: pointer;
    background: var(--accent); color: #fff; font-weight: 600; font-family: inherit; font-size: 0.875rem;
  }
  .fab-btn-merge:hover:not(:disabled) { filter: brightness(1.12); }
  .fab-btn-merge:disabled { opacity: 0.5; cursor: not-allowed; }
  .fab-hint { display: flex; align-items: center; gap: 0.4rem; color: #22c55e; font-weight: 600; }
  .fab-btn-cancel {
    padding: 0.3rem 0.65rem; border-radius: 999px; border: 1px solid var(--border);
    background: none; color: var(--muted); cursor: pointer; font-family: inherit; font-size: 0.8rem;
  }
  .fab-btn-cancel:hover { color: var(--text); background: var(--surface-2); }
  .fab-btn-cancel:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
