<script lang="ts">
  import type { PendingValidationDto, PatchValidationBody } from './types';

  interface Props {
    track: PendingValidationDto;
    onApprove?: (id: number, patch: PatchValidationBody) => Promise<void>;
    onReject?: (id: number) => Promise<void>;
  }

  let { track, onApprove, onReject }: Props = $props();

  let editing = $state(false);
  let busy = $state(false);

  // editable copies — reset whenever we open the form
  let editTitle = $state(track.title);
  let editArtists = $state(track.artists.map((a) => a.name).join(', '));
  let editAlbum = $state(track.album?.title ?? '');
  let editGenre = $state(track.genre ?? '');
  let editDate = $state(track.date ?? '');
  let editTrackNumber = $state(track.track_number?.toString() ?? '');
  let editDiscNumber = $state(track.disc_number?.toString() ?? '');
  let editLabel = $state(track.label ?? '');

  function startEdit() {
    editTitle = track.title;
    editArtists = track.artists.map((a) => a.name).join(', ');
    editAlbum = track.album?.title ?? '';
    editGenre = track.genre ?? '';
    editDate = track.date ?? '';
    editTrackNumber = track.track_number?.toString() ?? '';
    editDiscNumber = track.disc_number?.toString() ?? '';
    editLabel = track.label ?? '';
    editing = true;
  }

  async function handleApprove() {
    if (!onApprove) return;
    busy = true;
    try {
      const patch: PatchValidationBody = {};
      if (editing) {
        const t = editTitle.trim();
        if (t && t !== track.title) patch.title = t;

        const rawArtists = editArtists
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean);
        const origArtists = track.artists.map((a) => a.name);
        if (JSON.stringify(rawArtists) !== JSON.stringify(origArtists) && rawArtists.length > 0)
          patch.artists = rawArtists;

        const al = editAlbum.trim();
        if (al !== (track.album?.title ?? '')) patch.album_title = al || undefined;

        const g = editGenre.trim();
        if (g !== (track.genre ?? '')) patch.genre = g || undefined;

        const d = editDate.trim();
        if (d !== (track.date ?? '')) patch.date = d || undefined;

        const tn = parseInt(editTrackNumber);
        if (!isNaN(tn) && tn !== track.track_number) patch.track_number = tn;

        const dn = parseInt(editDiscNumber);
        if (!isNaN(dn) && dn !== track.disc_number) patch.disc_number = dn;

        const lb = editLabel.trim();
        if (lb !== (track.label ?? '')) patch.label = lb || undefined;
      }
      await onApprove(track.id, patch);
    } finally {
      busy = false;
    }
  }

  async function handleReject() {
    if (!onReject) return;
    busy = true;
    try {
      await onReject(track.id);
    } finally {
      busy = false;
    }
  }

  function formatDuration(seconds: number | null): string {
    if (!seconds) return '—';
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function artistNames(): string {
    return track.artists.map((a) => a.name).join(', ') || '—';
  }
</script>

<article class="track-card" class:editing>
  <div class="cover">
    {#if track.cover}
      <img src={track.cover} alt="cover" />
    {:else}
      <div class="cover-placeholder">♪</div>
    {/if}
  </div>

  <div class="body">
    {#if editing}
      <div class="edit-form">
        <div class="field">
          <label>Title</label>
          <input bind:value={editTitle} placeholder="Title" />
        </div>
        <div class="field">
          <label>Artists <span class="hint">comma-separated</span></label>
          <input bind:value={editArtists} placeholder="Artist 1, Artist 2" />
        </div>
        <div class="field">
          <label>Album</label>
          <input bind:value={editAlbum} placeholder="Album title" />
        </div>
        <div class="field-row">
          <div class="field">
            <label>Genre</label>
            <input bind:value={editGenre} placeholder="Genre" />
          </div>
          <div class="field">
            <label>Date</label>
            <input bind:value={editDate} placeholder="YYYY-MM-DD" />
          </div>
          <div class="field narrow">
            <label>Track #</label>
            <input bind:value={editTrackNumber} placeholder="1" type="number" min="1" />
          </div>
          <div class="field narrow">
            <label>Disc #</label>
            <input bind:value={editDiscNumber} placeholder="1" type="number" min="1" />
          </div>
        </div>
        <div class="field">
          <label>Label</label>
          <input bind:value={editLabel} placeholder="Label" />
        </div>
      </div>
    {:else}
      <div class="info">
        <div class="row main">
          <span class="title">{track.title}</span>
          <span class="artists">{artistNames()}</span>
        </div>

        <div class="row meta">
          {#if track.album}
            <span class="chip">💿 {track.album.title}</span>
          {/if}
          {#if track.date}
            <span class="chip">📅 {track.date}</span>
          {/if}
          {#if track.genre}
            <span class="chip">🎵 {track.genre}</span>
          {/if}
          {#if track.duration}
            <span class="chip">⏱ {formatDuration(track.duration)}</span>
          {/if}
        </div>

        {#if track.validation_reason}
          <div class="reason">
            ⚠️ <code>{track.validation_reason}</code>
          </div>
        {/if}

        {#if track.file_path}
          <div class="filepath">
            <code>{track.file_path}</code>
          </div>
        {/if}
      </div>
    {/if}

    {#if onApprove || onReject}
      <div class="actions">
        <div class="actions-left">
          {#if editing}
            <button class="btn-ghost" onclick={() => (editing = false)} disabled={busy}>
              Cancel
            </button>
          {:else}
            <button class="btn-ghost btn-edit" onclick={startEdit} disabled={busy}>
              Edit
            </button>
          {/if}
        </div>
        <div class="actions-right">
          {#if onReject}
            <button class="btn-reject" onclick={handleReject} disabled={busy}>
              {busy ? '…' : 'Reject'}
            </button>
          {/if}
          {#if onApprove}
            <button class="btn-approve" onclick={handleApprove} disabled={busy}>
              {busy ? '…' : 'Approve'}
            </button>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</article>

<style>
  .track-card {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    transition: border-color 0.15s;
  }

  .track-card.editing {
    border-color: var(--accent);
  }

  .cover {
    flex-shrink: 0;
    width: 72px;
    height: 72px;
    border-radius: 6px;
    overflow: hidden;
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    font-size: 1.8rem;
    color: var(--muted);
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    min-width: 0;
  }

  /* ---- read-only info ---- */

  .info {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: baseline;
  }

  .title {
    font-weight: 600;
    font-size: 1rem;
  }

  .artists {
    font-size: 0.875rem;
    color: var(--muted);
  }

  .chip {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    background: var(--surface-2);
    border-radius: 4px;
    white-space: nowrap;
  }

  .reason {
    font-size: 0.75rem;
    color: var(--warning, #f59e0b);
  }

  .filepath {
    font-size: 0.7rem;
    color: var(--muted);
    word-break: break-all;
  }

  /* ---- edit form ---- */

  .edit-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    flex: 1;
  }

  .field.narrow {
    flex: 0 0 80px;
  }

  .field-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  label {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
  }

  .hint {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    opacity: 0.7;
  }

  input {
    padding: 0.35rem 0.5rem;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: inherit;
    font-size: 0.875rem;
    width: 100%;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: var(--accent);
  }

  /* ---- actions ---- */

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding-top: 0.25rem;
  }

  .actions-left,
  .actions-right {
    display: flex;
    gap: 0.4rem;
  }

  button {
    padding: 0.3rem 0.8rem;
    border-radius: 5px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: opacity 0.1s;
  }

  button:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .btn-ghost {
    background: transparent;
    border-color: var(--border);
    color: var(--muted);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--surface-2);
    color: inherit;
  }

  .btn-approve {
    background: #16a34a;
    color: #fff;
  }

  .btn-approve:hover:not(:disabled) {
    opacity: 0.85;
  }

  .btn-reject {
    background: transparent;
    border-color: #dc2626;
    color: #dc2626;
  }

  .btn-reject:hover:not(:disabled) {
    background: #dc2626;
    color: #fff;
  }
</style>
