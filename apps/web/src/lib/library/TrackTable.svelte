<script lang="ts">
  import type { LibraryTrackDto } from '../types';
  import { lib } from './store.svelte';

  let { tracks, showAlbumCol = true }: {
    tracks: LibraryTrackDto[];
    showAlbumCol?: boolean;
  } = $props();
</script>

<div class="table-wrap">
  <table>
    <thead>
      <tr>
        <th>#</th><th>Title</th><th>Artists</th>
        {#if showAlbumCol}<th>Album</th>{/if}
        <th>Genre</th><th>Duration</th><th class="col-actions">Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each tracks as t (t.id)}
        <tr
          class:needs-validation={t.needs_validation}
          onmouseenter={() => (lib.hoveredItem = { type: 'track', id: t.id })}
          onmouseleave={() => (lib.hoveredItem = null)}
        >
          <td class="muted">{t.id}</td>
          <td class="title-cell">
            {t.title}
            {#if t.needs_validation}<span class="badge-warn" title="Awaiting validation">!</span>{/if}
          </td>
          <td class="muted">{t.artists.map(a => a.name).join(', ') || '\u2014'}</td>
          {#if showAlbumCol}<td class="muted">{t.album?.title ?? '\u2014'}</td>{/if}
          <td class="muted">{t.genre ?? '\u2014'}</td>
          <td class="muted mono">{lib.fmtDuration(t.duration)}</td>
          <td class="actions">
            <button class="btn-edit" onclick={() => lib.startEditTrack(t)}>Edit</button>
            <button class="btn-delete" onclick={() => lib.handleDeleteTrack(t.id)}>Delete</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
