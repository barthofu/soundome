<script lang="ts">
  import { lib } from './store.svelte';

  interface Props {
    /** Currently selected artist names, in order. */
    value: string[];
    onChange: (names: string[]) => void;
  }

  let { value, onChange }: Props = $props();

  let query = $state('');
  let inputEl: HTMLInputElement | undefined = $state(undefined);
  let highlighted = $state(0);
  let focused = $state(false);

  // Case-insensitive index over the already-loaded artist list (`lib.artists`),
  // keyed by lowercased name for O(1) exact-match lookups. Rebuilt only when the
  // underlying artist list changes, not on every keystroke.
  let nameIndex = $derived.by(() => {
    const map = new Map<string, { id: number; name: string }>();
    for (const a of lib.artists) map.set(a.name.toLowerCase(), { id: a.id, name: a.name });
    return map;
  });

  const selectedLower = $derived(new Set(value.map((v) => v.toLowerCase())));

  let suggestions = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    const out: { id: number; name: string }[] = [];
    for (const a of lib.artists) {
      if (selectedLower.has(a.name.toLowerCase())) continue;
      if (a.name.toLowerCase().includes(q)) {
        out.push({ id: a.id, name: a.name });
        if (out.length >= 8) break;
      }
    }
    return out;
  });

  // Whether the current query exactly matches an existing artist (so "Enter"
  // should select it instead of creating a duplicate).
  let exactMatch = $derived(nameIndex.get(query.trim().toLowerCase()) ?? null);

  function addArtist(name: string) {
    const trimmed = name.trim();
    if (!trimmed) return;
    if (selectedLower.has(trimmed.toLowerCase())) { query = ''; return; }
    onChange([...value, trimmed]);
    query = '';
    highlighted = 0;
  }

  function removeArtist(name: string) {
    onChange(value.filter((v) => v !== name));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      if (suggestions.length) { e.preventDefault(); highlighted = (highlighted + 1) % suggestions.length; }
      return;
    }
    if (e.key === 'ArrowUp') {
      if (suggestions.length) { e.preventDefault(); highlighted = (highlighted - 1 + suggestions.length) % suggestions.length; }
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      if (suggestions.length) {
        addArtist(suggestions[Math.min(highlighted, suggestions.length - 1)].name);
      } else if (exactMatch) {
        addArtist(exactMatch.name);
      } else if (query.trim()) {
        // No existing artist matches — create a new one on save (the backend
        // find-or-creates artists by name via `create_or_ignore`).
        addArtist(query);
      }
      return;
    }
    if (e.key === 'Backspace' && !query && value.length > 0) {
      removeArtist(value[value.length - 1]);
      return;
    }
    if (e.key === 'Escape') {
      query = '';
      inputEl?.blur();
    }
  }
</script>

<div class="artist-multiselect" class:focused>
  <div class="chips-row" onclick={() => inputEl?.focus()} role="presentation">
    {#each value as name (name)}
      <span class="artist-chip">
        {name}
        <button
          type="button"
          class="chip-remove"
          onclick={(e) => { e.stopPropagation(); removeArtist(name); }}
          aria-label={`Remove ${name}`}
        >&times;</button>
      </span>
    {/each}
    <input
      bind:this={inputEl}
      bind:value={query}
      onkeydown={handleKeydown}
      onfocus={() => (focused = true)}
      onblur={() => { focused = false; }}
      placeholder={value.length === 0 ? 'Artist 1, Artist 2…' : ''}
      class="chip-input"
    />
  </div>

  {#if focused && (suggestions.length > 0 || query.trim())}
    <ul class="suggestions">
      {#each suggestions as s, i (s.id)}
        <li>
          <button
            type="button"
            class="suggestion-item"
            class:active={i === highlighted}
            onmousedown={(e) => { e.preventDefault(); addArtist(s.name); }}
            onmouseenter={() => (highlighted = i)}
          >{s.name}</button>
        </li>
      {/each}
      {#if query.trim() && !exactMatch}
        <li>
          <button
            type="button"
            class="suggestion-item suggestion-create"
            onmousedown={(e) => { e.preventDefault(); addArtist(query); }}
          >+ Create "{query.trim()}"</button>
        </li>
      {/if}
    </ul>
  {/if}
</div>

<style>
  .artist-multiselect { position: relative; }

  .chips-row {
    display: flex; flex-wrap: wrap; align-items: center; gap: 0.35rem;
    padding: 0.35rem 0.5rem; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: 6px; cursor: text;
  }
  .artist-multiselect.focused .chips-row { border-color: var(--accent); }

  .artist-chip {
    display: inline-flex; align-items: center; gap: 0.3rem;
    background: var(--surface); border: 1px solid var(--border); border-radius: 4px;
    padding: 0.1rem 0.35rem; font-size: 0.8rem; color: var(--text); white-space: nowrap;
  }
  .chip-remove {
    background: none; border: none; color: var(--muted); cursor: pointer;
    font-size: 0.85rem; line-height: 1; padding: 0; display: flex; align-items: center;
  }
  .chip-remove:hover { color: var(--danger, #e05a5a); }

  .chip-input {
    flex: 1; min-width: 6rem; background: none; border: none; outline: none;
    color: var(--text); font-size: 0.875rem; font-family: inherit; padding: 0.15rem 0;
  }

  .suggestions {
    position: absolute; top: calc(100% + 4px); left: 0; right: 0; z-index: 20;
    list-style: none; margin: 0; padding: 0.25rem;
    background: var(--surface); border: 1px solid var(--border); border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.35);
    max-height: 220px; overflow-y: auto;
  }
  .suggestion-item {
    display: block; width: 100%; text-align: left; background: none; border: none;
    color: var(--text); font-size: 0.82rem; font-family: inherit; padding: 0.4rem 0.55rem;
    border-radius: 4px; cursor: pointer;
  }
  .suggestion-item.active, .suggestion-item:hover { background: var(--surface-2); }
  .suggestion-create { color: var(--accent); }
</style>
