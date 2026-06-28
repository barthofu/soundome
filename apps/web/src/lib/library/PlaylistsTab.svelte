<script lang="ts">
  import { lib } from './store.svelte';

  function fmtDuration(secs: number | null): string {
    if (secs == null) return '—';
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${String(s).padStart(2, '0')}`;
  }
</script>

{#snippet coverWrap(src: string | null | undefined, alt: string)}
  <div class="cover-wrap">
    {#if src && (src.startsWith('http://') || src.startsWith('https://'))}
      <img {src} {alt} class="cover-img" loading="lazy" />
    {:else}
      <div class="cover-ph">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
          <path d="M9 19V6l12-3v13"/><circle cx="6" cy="19" r="3"/><circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
    {/if}
  </div>
{/snippet}

<!-- ── PLAYLIST DETAIL ──────────────────────────────────────────────────────── -->
{#if lib.drillPlaylistId != null}
  {#if lib.drillPlaylistTracksLoading}
    <p class="status">Loading tracks…</p>
  {:else if lib.drillPlaylistTracksError}
    <p class="status error">{lib.drillPlaylistTracksError}</p>
  {:else}
    {@const playlist = lib.drillPlaylist}
    {#if playlist}
      <div class="detail-hero">
        <div class="detail-cover">{@render coverWrap(playlist.cover, playlist.name)}</div>
        <div class="detail-info">
          <div class="detail-type">{playlist.source}</div>
          <h2>{playlist.name}</h2>
          <div class="detail-meta">{lib.drillPlaylistTracks.length} track{lib.drillPlaylistTracks.length !== 1 ? 's' : ''}</div>
          {#if playlist.source_url}
            <a class="source-link" href={playlist.source_url} target="_blank" rel="noopener noreferrer">Open source ↗</a>
          {/if}
          <div class="detail-actions">
            <button class="btn-delete" onclick={() => lib.handleDeletePlaylist(playlist.id)}>Delete playlist</button>
          </div>
        </div>
      </div>
    {:else}
      <p class="status muted">Loading playlist…</p>
    {/if}

    {#if lib.drillPlaylistTracks.length === 0}
      <p class="status">No tracks in this playlist.</p>
    {:else}
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th>#</th><th>Title</th><th>Artists</th><th>Album</th><th>Genre</th><th>Duration</th></tr>
          </thead>
          <tbody>
            {#each lib.drillPlaylistTracks as t, i (t.id)}
              <tr>
                <td class="muted">{i + 1}</td>
                <td class="title-cell">{t.title}</td>
                <td class="muted">{t.artists.map(a => a.name).join(', ') || '—'}</td>
                <td class="muted">{t.album?.title ?? '—'}</td>
                <td class="muted">{t.genre ?? '—'}</td>
                <td class="muted mono">{fmtDuration(t.duration)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}

<!-- ── PLAYLISTS GRID ─────────────────────────────────────────────────────── -->
{:else if lib.playlistsLoading}
  <p class="status">Loading…</p>
{:else if lib.playlistsError}
  <p class="status error">{lib.playlistsError}</p>
{:else}
  <div class="toolbar">
    <input class="search" placeholder="Search playlists…" bind:value={lib.playlistSearch} />
    <span class="count">{lib.filteredPlaylists.length} playlist{lib.filteredPlaylists.length !== 1 ? 's' : ''}</span>
  </div>

  {#if lib.filteredPlaylists.length === 0}
    <p class="status">No playlists found.</p>
  {:else}
    <div class="grid">
       {#each lib.filteredPlaylists as p (p.id)}
         <div class="card"
           role="button"
           tabindex="0"
           onmouseenter={() => lib.drillIntoPlaylist(p)}
           onmouseleave={() => {}}>
           <button class="card-main" onclick={() => lib.drillIntoPlaylist(p)}>
             {@render coverWrap(p.cover, p.name)}
            <div class="card-info">
              <div class="card-title" title={p.name}>{p.name}</div>
              <div class="card-sub">{p.source}</div>
            </div>
          </button>
          <div class="card-hover-actions">
            <button class="btn-delete" onclick={(e) => { e.stopPropagation(); lib.handleDeletePlaylist(p.id); }}>Del</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .search {
    flex: 1;
    min-width: 0;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--border, #333);
    border-radius: 4px;
    background: var(--input-bg, #1a1a1a);
    color: inherit;
    font-size: 0.85rem;
  }
  .count {
    font-size: 0.8rem;
    color: var(--muted, #888);
    white-space: nowrap;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1rem;
  }
  .card {
    background: var(--card-bg, #1c1c1c);
    border: 1px solid var(--border, #333);
    border-radius: 6px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
    transition: border-color 0.15s;
  }
  .card:hover {
    border-color: var(--accent, #7cb7ff);
  }
  .card-main {
    display: flex;
    flex-direction: column;
    cursor: pointer;
    text-align: left;
    padding: 0;
    background: none;
    border: none;
    color: inherit;
    font-family: inherit;
    width: 100%;
  }
  .cover-wrap {
    width: 100%;
    aspect-ratio: 1;
    overflow: hidden;
    background: var(--cover-bg, #111);
    flex-shrink: 0;
  }
  .cover-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .cover-ph {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted, #555);
  }
  .cover-ph svg { width: 40%; height: 40%; }
  .card-info {
    padding: 0.5rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }
  .card-title {
    font-size: 0.85rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card-sub {
    font-size: 0.75rem;
    color: var(--muted, #888);
    text-transform: capitalize;
  }
  .card-hover-actions {
    display: none;
    position: absolute;
    top: 0.35rem;
    right: 0.35rem;
    gap: 0.25rem;
  }
  .card:hover .card-hover-actions {
    display: flex;
  }

  /* Detail hero actions */
  .detail-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
  .detail-actions button {
    padding: 0.3rem 0.75rem;
    border-radius: 5px;
    border: 1px solid var(--border, #333);
    cursor: pointer;
    font-size: 0.8rem;
    font-family: inherit;
    background: var(--surface-2, #1c1c1c);
    color: var(--text, #eee);
  }
  .detail-actions button:hover { background: var(--surface, #141414); }

  /* Delete button */
  .btn-delete {
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    border: 1px solid color-mix(in srgb, #e05 35%, transparent);
    background: color-mix(in srgb, #e05 10%, var(--surface, #111));
    color: #e05;
    cursor: pointer;
    font-size: 0.75rem;
    font-family: inherit;
  }
  .btn-delete:hover { background: color-mix(in srgb, #e05 20%, var(--surface, #111)); }

  /* Detail view */
  .detail-hero {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
    margin-bottom: 1.5rem;
  }
  .detail-cover {
    width: 120px;
    height: 120px;
    border-radius: 6px;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--cover-bg, #111);
  }
  .detail-info {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    min-width: 0;
  }
  .detail-type {
    font-size: 0.75rem;
    color: var(--muted, #888);
    text-transform: capitalize;
    letter-spacing: 0.04em;
  }
  .detail-info h2 { margin: 0; font-size: 1.3rem; font-weight: 700; }
  .detail-meta { font-size: 0.85rem; color: var(--muted, #888); }
  .source-link {
    font-size: 0.8rem;
    color: var(--accent, #7cb7ff);
    text-decoration: none;
    margin-top: 0.2rem;
  }
  .source-link:hover { text-decoration: underline; }

  .table-wrap { overflow-x: auto; }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }
  th {
    text-align: left;
    padding: 0.4rem 0.6rem;
    font-weight: 500;
    color: var(--muted, #888);
    border-bottom: 1px solid var(--border, #333);
    white-space: nowrap;
  }
  td {
    padding: 0.45rem 0.6rem;
    border-bottom: 1px solid var(--border-subtle, #222);
    vertical-align: middle;
  }
  tr:last-child td { border-bottom: none; }
  .title-cell { font-weight: 500; }
  .muted { color: var(--muted, #888); }
  .mono { font-variant-numeric: tabular-nums; }

  .status {
    color: var(--muted, #888);
    font-size: 0.9rem;
    padding: 1rem 0;
  }
  .status.error { color: var(--error, #e05); }
</style>
