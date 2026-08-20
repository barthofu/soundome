<script lang="ts">
  import { lib } from './store.svelte';
  import ReferencesPanel from './ReferencesPanel.svelte';
  import ArtistMultiSelect from './ArtistMultiSelect.svelte';

  let dialogEl: HTMLDialogElement | undefined = $state(undefined);

  function handleFileInput(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) lib.uploadImage(file);
    input.value = '';
  }

  function onDropZoneDrop(e: DragEvent) {
    e.preventDefault();
    const file = e.dataTransfer?.files?.[0];
    if (file) lib.uploadImage(file);
  }

  function onDropZoneDragOver(e: DragEvent) {
    e.preventDefault();
  }

  $effect(() => {
    if (!dialogEl) return;
    if (lib.editState) { if (!dialogEl.open) dialogEl.showModal(); }
    else { if (dialogEl.open) dialogEl.close(); }
  });
</script>

<dialog
  bind:this={dialogEl}
  onclose={() => (lib.editState = null)}
  onkeydown={(e) => {
    if (e.key === 'Enter' && !lib.editSaving) {
      const t = e.target as HTMLElement;
      if (t.tagName !== 'BUTTON') { e.preventDefault(); lib.saveEdit(); }
    }
  }}
  class="edit-dialog"
>
  {#if lib.editState}
    <div class="dialog-header">
      <h3>
        {lib.editState.type === 'track' ? 'Edit Track'
          : lib.editState.type === 'album' ? 'Edit Album' : 'Edit Artist'}
      </h3>
      <button class="dialog-close" onclick={() => (lib.editState = null)} aria-label="Close">&times;</button>
    </div>

    {#if lib.editState.type === 'track'}
      <div class="dialog-body">
        <div class="image-section">
          <span class="field-label-text">Cover</span>
          <div
            class="image-drop-zone"
            class:uploading={lib.imageUploading}
            ondrop={onDropZoneDrop}
            ondragover={onDropZoneDragOver}
            role="button"
            tabindex="0"
            aria-label="Upload cover image"
          >
            {#if lib.imageUploading}
              <span class="upload-spinner" aria-hidden="true"></span>
            {:else if lib.editState.item.cover}
              <img src={lib.editState.item.cover} alt="cover" class="image-preview" />
              <span class="image-overlay">Change</span>
            {:else}
              <span class="upload-placeholder">
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                <span>Upload image</span>
                <span class="drop-hint">or drop here</span>
              </span>
            {/if}
            <input
              type="file"
              accept="image/jpeg,image/png,image/webp,image/gif"
              class="file-input"
              onchange={handleFileInput}
              disabled={lib.imageUploading}
            />
          </div>
        </div>
        <label class="field-label">Title
          <input value={lib.trackDraft.title ?? ''}
            oninput={(e) => { lib.trackDraft.title = (e.currentTarget as HTMLInputElement).value; }} />
        </label>
        <label class="field-label">Artists
          <ArtistMultiSelect
            value={lib.trackDraft.artists ?? []}
            onChange={(names) => { lib.trackDraft.artists = names; }}
          />
        </label>
        <label class="field-label">Album
          <input value={lib.trackDraft.album_title ?? ''}
            oninput={(e) => { lib.trackDraft.album_title = (e.currentTarget as HTMLInputElement).value || undefined; }}
            placeholder="Album" />
        </label>
        <div class="field-row">
          <label class="field-label half">Genre
            <input value={lib.trackDraft.genre ?? ''}
              oninput={(e) => { lib.trackDraft.genre = (e.currentTarget as HTMLInputElement).value || undefined; }}
              placeholder="Genre" />
          </label>
          <label class="field-label half">Date
            <input value={lib.trackDraft.date ?? ''}
              oninput={(e) => { lib.trackDraft.date = (e.currentTarget as HTMLInputElement).value || undefined; }}
              placeholder="YYYY-MM-DD" />
          </label>
        </div>
        <div class="field-row">
          <label class="field-label third">Track #
            <input type="number" value={lib.trackDraft.track_number ?? ''}
              oninput={(e) => { const v = (e.currentTarget as HTMLInputElement).valueAsNumber; lib.trackDraft.track_number = isNaN(v) ? undefined : v; }} />
          </label>
          <label class="field-label third">Disc #
            <input type="number" value={lib.trackDraft.disc_number ?? ''}
              oninput={(e) => { const v = (e.currentTarget as HTMLInputElement).valueAsNumber; lib.trackDraft.disc_number = isNaN(v) ? undefined : v; }} />
          </label>
          <label class="field-label third">Label
            <input value={lib.trackDraft.label ?? ''}
              oninput={(e) => { lib.trackDraft.label = (e.currentTarget as HTMLInputElement).value || undefined; }}
              placeholder="Label" />
          </label>
        </div>
        <ReferencesPanel
          references={lib.editState.item.references}
          onAdd={(body) => lib.addReference('tracks', lib.editState!.item.id, body)}
          onDelete={(ref) => lib.deleteReference('tracks', lib.editState!.item.id, ref)}
        />
      </div>

    {:else if lib.editState.type === 'album'}
      <div class="dialog-body">
        <div class="image-section">
          <span class="field-label-text">Cover</span>
          <div class="image-row">
            <div
              class="image-drop-zone"
              class:uploading={lib.imageUploading}
              ondrop={onDropZoneDrop}
              ondragover={onDropZoneDragOver}
              role="button"
              tabindex="0"
              aria-label="Upload cover image"
            >
              {#if lib.imageUploading}
                <span class="upload-spinner" aria-hidden="true"></span>
              {:else if lib.albumDraft.cover}
                <img src={lib.albumDraft.cover} alt="cover" class="image-preview" />
                <span class="image-overlay">Change</span>
              {:else}
                <span class="upload-placeholder">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                  <span>Upload</span>
                  <span class="drop-hint">or drop</span>
                </span>
              {/if}
              <input
                type="file"
                accept="image/jpeg,image/png,image/webp,image/gif"
                class="file-input"
                onchange={handleFileInput}
                disabled={lib.imageUploading || lib.thumbnailFetching}
              />
            </div>
            <label class="field-label url-field">
              Image URL
              <div class="url-input-row">
                <input
                  type="url"
                  value={lib.albumDraft.cover ?? ''}
                  oninput={(e) => { lib.albumDraft.cover = (e.currentTarget as HTMLInputElement).value || undefined; }}
                  placeholder="https://…"
                />
                <button
                  type="button"
                  class="btn-fetch-thumbnail"
                  onclick={() => lib.fetchThumbnailFromReferences()}
                  disabled={lib.imageUploading || lib.thumbnailFetching || lib.editState.item.references.length === 0}
                  title="Fetch cover from this album's references (Spotify, SoundCloud, YouTube Music)"
                >
                  {lib.thumbnailFetching ? '…' : '⤓ From references'}
                </button>
                <button
                  type="button"
                  class="btn-clear-thumbnail"
                  onclick={() => { lib.albumDraft.cover = undefined; }}
                  disabled={lib.imageUploading || lib.thumbnailFetching || !lib.albumDraft.cover}
                  title="Remove cover image"
                >
                  ✕
                </button>
              </div>
            </label>
          </div>
        </div>
        <label class="field-label">Title
          <input value={lib.albumDraft.title ?? ''}
            oninput={(e) => { lib.albumDraft.title = (e.currentTarget as HTMLInputElement).value; }} />
        </label>
        <label class="field-label">Date
          <input value={lib.albumDraft.date ?? ''}
            oninput={(e) => { lib.albumDraft.date = (e.currentTarget as HTMLInputElement).value || undefined; }}
            placeholder="YYYY-MM-DD" />
        </label>
        <ReferencesPanel
          references={lib.editState.item.references}
          onAdd={(body) => lib.addReference('albums', lib.editState!.item.id, body)}
          onDelete={(ref) => lib.deleteReference('albums', lib.editState!.item.id, ref)}
        />
      </div>

    {:else}
      <div class="dialog-body">
        <div class="image-section">
          <span class="field-label-text">Photo</span>
          <div class="image-row">
            <div
              class="image-drop-zone image-drop-zone--round"
              class:uploading={lib.imageUploading}
              ondrop={onDropZoneDrop}
              ondragover={onDropZoneDragOver}
              role="button"
              tabindex="0"
              aria-label="Upload artist photo"
            >
              {#if lib.imageUploading}
                <span class="upload-spinner" aria-hidden="true"></span>
              {:else if lib.artistDraft.icon}
                <img src={lib.artistDraft.icon} alt="artist" class="image-preview" />
                <span class="image-overlay">Change</span>
              {:else}
                <span class="upload-placeholder">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                  <span>Upload</span>
                  <span class="drop-hint">or drop</span>
                </span>
              {/if}
              <input
                type="file"
                accept="image/jpeg,image/png,image/webp,image/gif"
                class="file-input"
                onchange={handleFileInput}
                disabled={lib.imageUploading || lib.thumbnailFetching}
              />
            </div>
            <label class="field-label url-field">
              Photo URL
              <div class="url-input-row">
                <input
                  type="url"
                  value={lib.artistDraft.icon ?? ''}
                  oninput={(e) => { lib.artistDraft.icon = (e.currentTarget as HTMLInputElement).value || undefined; }}
                  placeholder="https://…"
                />
                <button
                  type="button"
                  class="btn-fetch-thumbnail"
                  onclick={() => lib.fetchThumbnailFromReferences()}
                  disabled={lib.imageUploading || lib.thumbnailFetching || lib.editState.item.references.length === 0}
                  title="Fetch photo from this artist's references (Spotify, SoundCloud, YouTube Music)"
                >
                  {lib.thumbnailFetching ? '…' : '⤓ From references'}
                </button>
                <button
                  type="button"
                  class="btn-clear-thumbnail"
                  onclick={() => { lib.artistDraft.icon = undefined; }}
                  disabled={lib.imageUploading || lib.thumbnailFetching || !lib.artistDraft.icon}
                  title="Remove photo"
                >
                   ✕
                </button>
              </div>
            </label>
          </div>
        </div>
        <label class="field-label">Name
          <input value={lib.artistDraft.name ?? ''}
            oninput={(e) => { lib.artistDraft.name = (e.currentTarget as HTMLInputElement).value; }} />
        </label>
        <ReferencesPanel
          references={lib.editState.item.references}
          onAdd={(body) => lib.addReference('artists', lib.editState!.item.id, body)}
          onDelete={(ref) => lib.deleteReference('artists', lib.editState!.item.id, ref)}
        />
      </div>
    {/if}

    <div class="dialog-footer">
      <span class="kbd-hint">Enter to save &nbsp;·&nbsp; Esc to cancel</span>
      <button class="btn-cancel" onclick={() => (lib.editState = null)}>Cancel</button>
      <button class="btn-save" onclick={lib.saveEdit} disabled={lib.editSaving}>
        {lib.editSaving ? '\u2026' : 'Save'}
      </button>
    </div>
  {/if}
</dialog>

<style>
  .edit-dialog {
    background: var(--surface); border: 1px solid var(--border); border-radius: 12px;
    padding: 0; width: min(540px, 92vw); color: var(--text); font-family: inherit;
    box-shadow: 0 24px 60px rgba(0,0,0,0.5);
  }
  .edit-dialog::backdrop { background: rgba(0,0,0,0.55); backdrop-filter: blur(3px); }
  .dialog-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 1.1rem 1.4rem 0.8rem; border-bottom: 1px solid var(--border);
  }
  .dialog-header h3 { margin: 0; font-size: 1rem; font-weight: 600; }
  .dialog-close { background: none; border: none; color: var(--muted); font-size: 1.4rem; line-height: 1; cursor: pointer; padding: 0 0.2rem; }
  .dialog-close:hover { color: var(--text); }
  .dialog-body { padding: 1.2rem 1.4rem; display: flex; flex-direction: column; gap: 0.75rem; }
  .field-label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.78rem; color: var(--muted); font-weight: 500; }
  .field-label input {
    padding: 0.45rem 0.65rem; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: 6px; color: var(--text); font-size: 0.875rem; font-family: inherit;
  }
  .field-label input:focus { outline: none; border-color: var(--accent); }
  .field-row { display: flex; gap: 0.65rem; }
  .field-label.half { flex: 1; }
  .field-label.third { flex: 1; min-width: 0; }
  .dialog-footer {
    display: flex; align-items: center; gap: 0.65rem;
    padding: 0.9rem 1.4rem 1.1rem; border-top: 1px solid var(--border);
  }
  .kbd-hint { font-size: 0.72rem; color: var(--muted); margin-right: auto; }

  /* ── Image upload ─────────────────────────────────────────────────────── */
  .image-section { display: flex; flex-direction: column; gap: 0.3rem; }
  .field-label-text { font-size: 0.78rem; color: var(--muted); font-weight: 500; }

  .image-row { display: flex; align-items: flex-start; gap: 0.75rem; }
  .url-field { flex: 1; min-width: 0; }

  .url-input-row { display: flex; gap: 0.4rem; }
  .url-input-row input { flex: 1; min-width: 0; }

  .btn-fetch-thumbnail {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--accent);
    font-size: 0.75rem;
    padding: 0 0.6rem;
    cursor: pointer;
    font-family: inherit;
    white-space: nowrap;
  }
  .btn-fetch-thumbnail:hover:not(:disabled) { background: var(--surface-2); }
  .btn-fetch-thumbnail:disabled { opacity: 0.45; cursor: default; }

  .btn-clear-thumbnail {
    flex-shrink: 0;
    background: none;
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--muted);
    font-size: 0.85rem;
    padding: 0 0.4rem;
    cursor: pointer;
    font-family: inherit;
    white-space: nowrap;
  }
  .btn-clear-thumbnail:hover:not(:disabled) { background: var(--surface-2); color: var(--text); }
  .btn-clear-thumbnail:disabled { opacity: 0.3; cursor: default; }

  .image-drop-zone {
    position: relative; width: 100px; height: 100px;
    border: 1.5px dashed var(--border); border-radius: 8px;
    background: var(--surface-2); cursor: pointer; overflow: hidden;
    display: flex; align-items: center; justify-content: center;
    transition: border-color 0.15s;
  }
  .image-drop-zone:hover { border-color: var(--accent); }
  .image-drop-zone.uploading { opacity: 0.6; cursor: wait; }
  .image-drop-zone--round { border-radius: 50%; }

  .image-preview { width: 100%; height: 100%; object-fit: cover; display: block; }

  .image-overlay {
    position: absolute; inset: 0;
    background: rgba(0,0,0,0.5); color: #fff;
    font-size: 0.75rem; font-weight: 600; letter-spacing: 0.03em;
    display: flex; align-items: center; justify-content: center;
    opacity: 0; transition: opacity 0.15s;
  }
  .image-drop-zone:hover .image-overlay { opacity: 1; }

  .upload-placeholder {
    display: flex; flex-direction: column; align-items: center; gap: 0.3rem;
    color: var(--muted); font-size: 0.72rem; text-align: center; padding: 0.5rem;
    pointer-events: none;
  }
  .drop-hint { font-size: 0.65rem; opacity: 0.7; }

  .file-input {
    position: absolute; inset: 0; opacity: 0; cursor: pointer; width: 100%; height: 100%;
  }
  .file-input:disabled { cursor: wait; }

  @keyframes spin { to { transform: rotate(360deg); } }
  .upload-spinner {
    width: 20px; height: 20px; border: 2px solid var(--border);
    border-top-color: var(--accent); border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
</style>
