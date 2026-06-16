<script lang="ts">
  import { getMatchCandidates } from './api';
  import type { PendingValidationDto, PatchValidationBody, MatchCandidateDto } from './types';

  interface Props {
    track: PendingValidationDto;
    onApprove?: (id: number, patch: PatchValidationBody) => Promise<void>;
    onReject?: (id: number) => Promise<void>;
  }

  let { track, onApprove, onReject }: Props = $props();

  let editing = $state(false);
  let busy = $state(false);

  // Match candidates state
  let showMatches = $state(false);
  let matchesLoading = $state(false);
  let matchCandidates: MatchCandidateDto[] = $state([]);
  let matchesError: string | null = $state(null);

  // editable copies — reset whenever we open the form
  let editTitle = $state(track.title);
  let editArtists = $state(track.artists.map((a) => a.name).join(', '));
  let editAlbum = $state(track.album?.title ?? '');
  let editGenre = $state(track.genre ?? '');
  let editDate = $state(track.date ?? '');
  let editTrackNumber = $state(track.track_number?.toString() ?? '');
  let editDiscNumber = $state(track.disc_number?.toString() ?? '');
  let editLabel = $state(track.label ?? '');

  let cardEl: HTMLElement | undefined = $state();
  let hovered = $state(false);

  // 'e' to open edit when hovered, Escape to close it
  $effect(() => {
    if (!hovered || editing) return;
    function onKeydown(e: KeyboardEvent) {
      const tgt = e.target as HTMLElement;
      if (tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.tagName === 'SELECT') return;
      if (e.key === 'e') { e.preventDefault(); startEdit(); }
    }
    document.addEventListener('keydown', onKeydown);
    return () => document.removeEventListener('keydown', onKeydown);
  });

  $effect(() => {
    if (!editing) return;
    function onKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') { e.preventDefault(); editing = false; }
    }
    document.addEventListener('keydown', onKeydown);
    return () => document.removeEventListener('keydown', onKeydown);
  });

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

  async function toggleMatches() {
    if (showMatches) {
      showMatches = false;
      return;
    }
    showMatches = true;
    if (matchCandidates.length > 0) return; // already loaded
    matchesLoading = true;
    matchesError = null;
    try {
      matchCandidates = await getMatchCandidates(track.id);
    } catch (e: unknown) {
      matchesError = e instanceof Error ? e.message : String(e);
    } finally {
      matchesLoading = false;
    }
  }

  async function selectCandidate(candidate: MatchCandidateDto) {
    if (!onApprove) return;
    busy = true;
    try {
      const patch: PatchValidationBody = {
        title: candidate.title,
        artists: candidate.artists.map((a) => a.name),
        album_title: candidate.album?.title ?? undefined,
        genre: candidate.genre ?? undefined,
        date: candidate.date ?? undefined,
        track_number: candidate.track_number ?? undefined,
        disc_number: candidate.disc_number ?? undefined,
        label: candidate.label ?? undefined,
      };
      await onApprove(track.id, patch);
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

  function formatScore(score: number): string {
    return `${Math.round(score * 100)}%`;
  }

  let sourceUrl = $derived(
    track.references.find((r) => r.ref_type === 'Source' && r.external_url)?.external_url ?? null
  );

  let isPartialMatch = $derived(track.validation_reason === 'metadata_partial_match');
</script>

<article class="track-card" class:editing bind:this={cardEl}
  onmouseenter={() => hovered = true}
  onmouseleave={() => hovered = false}>
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
          {#if sourceUrl}
            <a class="title" href={sourceUrl} target="_blank" rel="noopener noreferrer">{track.title}</a>
          {:else}
            <span class="title">{track.title}</span>
          {/if}
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
            {#if isPartialMatch}
              <button class="btn-ghost btn-matches" onclick={toggleMatches} disabled={busy}>
                {showMatches ? 'Hide matches' : 'Show matches'}
              </button>
            {/if}
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

    {#if showMatches}
      <div class="matches-panel">
        {#if matchesLoading}
          <p class="matches-status">Searching metadata providers…</p>
        {:else if matchesError}
          <p class="matches-status matches-error">{matchesError}</p>
        {:else if matchCandidates.length === 0}
          <p class="matches-status">No candidates found</p>
        {:else}
          <p class="matches-count">{matchCandidates.length} candidate{matchCandidates.length > 1 ? 's' : ''} found</p>
           <ul class="matches-list">
             {#each matchCandidates as candidate, i}
               <li class="match-item">
                 <div class="match-info">
                   <div class="match-main">
                     {#if candidate.references && candidate.references.some(r => r.external_url)}
                       {@const providerUrl = candidate.references.find(r => r.external_url)?.external_url}
                       <a class="match-title" href={providerUrl} target="_blank" rel="noopener noreferrer">{candidate.title}</a>
                     {:else}
                       <span class="match-title">{candidate.title}</span>
                     {/if}
                     <span class="match-artists">{candidate.artists.map(a => a.name).join(', ')}</span>
                   </div>
                   <div class="match-meta">
                     {#if candidate.album}
                       <span class="chip">💿 {candidate.album.title}</span>
                     {/if}
                     {#if candidate.date}
                       <span class="chip">📅 {candidate.date}</span>
                     {/if}
                     {#if candidate.duration}
                       <span class="chip">⏱ {formatDuration(candidate.duration)}</span>
                     {/if}
                     <span class="chip match-score">{formatScore(candidate.score)}</span>
                     <span class="chip match-provider">{candidate.provider}</span>
                   </div>
                 </div>
                 <button class="btn-select" onclick={() => selectCandidate(candidate)} disabled={busy}>
                   Select
                 </button>
               </li>
             {/each}
           </ul>
        {/if}
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

  a.title {
    color: inherit;
    text-decoration: none;
  }

  a.title:hover {
    text-decoration: underline;
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

  .btn-matches {
    border-color: var(--accent);
    color: var(--accent);
  }

  .btn-matches:hover:not(:disabled) {
    background: var(--accent);
    color: #fff;
  }

  /* ---- matches panel ---- */

  .matches-panel {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: var(--surface-2);
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .matches-status {
    font-size: 0.8rem;
    color: var(--muted);
    text-align: center;
    margin: 0;
    padding: 0.5rem 0;
  }

  .matches-error {
    color: var(--error);
  }

  .matches-count {
    font-size: 0.75rem;
    color: var(--muted);
    margin: 0 0 0.5rem 0;
  }

  .matches-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .match-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 5px;
    transition: border-color 0.15s;
  }

  .match-item:hover {
    border-color: var(--accent);
  }

  .match-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .match-main {
    display: flex;
    gap: 0.5rem;
    align-items: baseline;
    flex-wrap: wrap;
  }

  .match-title {
    font-weight: 600;
    font-size: 0.875rem;
  }

  a.match-title {
    color: inherit;
    text-decoration: none;
  }

  a.match-title:hover {
    color: var(--accent);
    text-decoration: underline;
  }

  .match-artists {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .match-meta {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .match-score {
    background: #16a34a22;
    color: #16a34a;
    font-weight: 600;
  }

  .match-provider {
    background: var(--accent-muted, rgba(99, 102, 241, 0.15));
    color: var(--accent);
  }

  .btn-select {
    flex-shrink: 0;
    background: #16a34a;
    color: #fff;
    font-size: 0.75rem;
    padding: 0.25rem 0.6rem;
  }

  .btn-select:hover:not(:disabled) {
    opacity: 0.85;
  }
</style>
