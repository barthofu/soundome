<script lang="ts">
  import type { ReferenceDto, AddReferenceBody } from '../types';

  interface Props {
    references: ReferenceDto[];
    onAdd: (body: AddReferenceBody) => Promise<void>;
    onDelete: (ref: ReferenceDto) => Promise<void>;
  }

  let { references, onAdd, onDelete }: Props = $props();

  // Form state
  let formOpen = $state(false);
  let saving = $state(false);
  let refType = $state('Metadata');
  let platform = $state('Unknown');
  let externalId = $state('');
  let externalUrl = $state('');

  const REF_TYPES = ['Source', 'Provider', 'Metadata', 'Reference'] as const;
  const PLATFORMS = ['Spotify', 'SoundCloud', 'MusicBrainz', 'YoutubeMusic', 'Youtube', 'Bandcamp', 'Unknown'] as const;

  function platformLabel(p: string): string {
    const map: Record<string, string> = {
      soundcloud: 'SoundCloud',
      musicbrainz: 'MusicBrainz',
      youtubemusic: 'YT Music',
      youtube: 'YouTube',
      bandcamp: 'Bandcamp',
      spotify: 'Spotify',
      unknown: 'Unknown',
    };
    return map[p.toLowerCase()] ?? p;
  }

  function refTypeLabel(t: string): string {
    const map: Record<string, string> = {
      Source: 'Source',
      Provider: 'Provider',
      Metadata: 'Metadata',
      Reference: 'Reference',
    };
    return map[t] ?? t;
  }

  function platformIcon(p: string): string {
    const icons: Record<string, string> = {
      Spotify: '🎵',
      SoundCloud: '☁️',
      MusicBrainz: '🎼',
      YoutubeMusic: '▶️',
      Youtube: '▶',
      Bandcamp: '🎸',
    };
    return icons[p] ?? '🔗';
  }

  function resetForm() {
    refType = 'Metadata';
    platform = 'Unknown';
    externalId = '';
    externalUrl = '';
    formOpen = false;
  }

  async function handleAdd() {
    if (!externalId.trim() && !externalUrl.trim()) return;
    saving = true;
    try {
      await onAdd({
        ref_type: refType,
        platform,
        external_id: externalId.trim() || null,
        external_url: externalUrl.trim() || null,
      });
      resetForm();
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      saving = false;
    }
  }

  async function handleDelete(ref: ReferenceDto) {
    if (!confirm('Remove this reference?')) return;
    try {
      await onDelete(ref);
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    }
  }
</script>

<div class="refs-panel">
  <div class="refs-header">
    <span class="refs-title">References</span>
    <button class="btn-add-ref" onclick={() => (formOpen = !formOpen)}>
      {formOpen ? '✕' : '+ Add'}
    </button>
  </div>

  {#if formOpen}
    <div class="ref-form">
      <div class="ref-form-row">
        <label class="ref-field">
          Type
          <select bind:value={refType}>
            {#each REF_TYPES as t}
              <option value={t}>{refTypeLabel(t)}</option>
            {/each}
          </select>
        </label>
        <label class="ref-field">
          Platform
          <select bind:value={platform}>
            {#each PLATFORMS as p}
              <option value={p}>{platformLabel(p)}</option>
            {/each}
          </select>
        </label>
      </div>
      <label class="ref-field">
        External ID
        <input
          bind:value={externalId}
          placeholder="e.g. spotify track id"
        />
      </label>
      <label class="ref-field">
        External URL
        <input
          bind:value={externalUrl}
          type="url"
          placeholder="https://…"
        />
      </label>
      <div class="ref-form-actions">
        <button class="btn-cancel-ref" onclick={resetForm} disabled={saving}>Cancel</button>
        <button
          class="btn-save-ref"
          onclick={handleAdd}
          disabled={saving || (!externalId.trim() && !externalUrl.trim())}
        >
          {saving ? '…' : 'Save'}
        </button>
      </div>
    </div>
  {/if}

  {#if references.length === 0 && !formOpen}
    <p class="refs-empty">No references.</p>
  {:else}
    <ul class="refs-list">
      {#each references as ref (ref.id ?? `${ref.platform}-${ref.external_id}-${ref.external_url}`)}
        <li class="ref-item">
          <span class="ref-icon" title={platformLabel(ref.platform)}>{platformIcon(ref.platform)}</span>
          <span class="ref-meta">
            <span class="ref-type-badge">{refTypeLabel(ref.ref_type)}</span>
            <span class="ref-platform">{platformLabel(ref.platform)}</span>
            {#if ref.external_url}
              <a href={ref.external_url} target="_blank" rel="noopener noreferrer" class="ref-link">
                {ref.external_id ?? ref.external_url}
              </a>
            {:else if ref.external_id}
              <span class="ref-id">{ref.external_id}</span>
            {/if}
          </span>
          <button
            class="btn-delete-ref"
            onclick={() => handleDelete(ref)}
            title="Remove reference"
          >✕</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .refs-panel {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    border-top: 1px solid var(--border);
    padding-top: 0.75rem;
    margin-top: 0.25rem;
  }

  .refs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .refs-title {
    font-size: 0.78rem;
    color: var(--muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .btn-add-ref {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--accent);
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    cursor: pointer;
    font-family: inherit;
    line-height: 1.4;
  }
  .btn-add-ref:hover { background: var(--surface-2); }

  /* ── Form ─────────────────────────────────────────────────────────────── */
  .ref-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.65rem 0.75rem;
  }

  .ref-form-row { display: flex; gap: 0.5rem; }

  .ref-field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.75rem;
    color: var(--muted);
    flex: 1;
    min-width: 0;
  }

  .ref-field input,
  .ref-field select {
    padding: 0.35rem 0.55rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
    font-size: 0.82rem;
    font-family: inherit;
  }
  .ref-field input:focus,
  .ref-field select:focus { outline: none; border-color: var(--accent); }

  .ref-form-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .btn-cancel-ref,
  .btn-save-ref {
    padding: 0.3rem 0.75rem;
    border-radius: 5px;
    font-size: 0.8rem;
    font-family: inherit;
    cursor: pointer;
    border: 1px solid var(--border);
  }
  .btn-cancel-ref { background: none; color: var(--muted); }
  .btn-cancel-ref:hover { background: var(--surface-2); color: var(--text); }
  .btn-save-ref { background: var(--accent); color: #fff; border-color: var(--accent); }
  .btn-save-ref:disabled { opacity: 0.45; cursor: default; }
  .btn-save-ref:not(:disabled):hover { filter: brightness(1.1); }

  /* ── List ─────────────────────────────────────────────────────────────── */
  .refs-empty {
    font-size: 0.78rem;
    color: var(--muted);
    margin: 0;
  }

  .refs-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .ref-item {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.35rem 0.55rem;
    font-size: 0.8rem;
    min-width: 0;
  }

  .ref-icon { font-size: 0.9rem; flex-shrink: 0; }

  .ref-meta {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex: 1;
    min-width: 0;
    flex-wrap: wrap;
  }

  .ref-type-badge {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-size: 0.68rem;
    padding: 0.05rem 0.35rem;
    color: var(--muted);
    white-space: nowrap;
  }

  .ref-platform {
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    font-size: 0.78rem;
  }

  .ref-link {
    color: var(--accent);
    text-decoration: none;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    font-size: 0.78rem;
  }
  .ref-link:hover { text-decoration: underline; }

  .ref-id {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    font-size: 0.78rem;
    font-family: monospace;
  }

  .btn-delete-ref {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 0.7rem;
    padding: 0.1rem 0.25rem;
    flex-shrink: 0;
    line-height: 1;
    border-radius: 3px;
  }
  .btn-delete-ref:hover { color: var(--danger, #e05a5a); background: var(--surface); }
</style>
