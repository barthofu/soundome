<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    value: string;
    direction: 'asc' | 'desc';
    options: Option[];
    onChange?: (value: string) => void;
    onDirectionChange?: (dir: 'asc' | 'desc') => void;
  }

  let { value, direction, options, onChange, onDirectionChange }: Props = $props();

  function toggleDirection() {
    const newDir = direction === 'asc' ? 'desc' : 'asc';
    onDirectionChange?.(newDir);
  }

  function handleSelectChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    onChange?.(target.value);
  }
</script>

<div class="sort-controls">
  <select class="sort-select" value={value} onchange={handleSelectChange} title="Sort by">
    {#each options as opt (opt.value)}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
  
  <button 
    class="sort-direction-btn" 
    class:desc={direction === 'desc'}
    title={direction === 'asc' ? 'Ascending' : 'Descending'}
    onclick={toggleDirection}
    aria-label={direction === 'asc' ? 'Sort ascending' : 'Sort descending'}
  >
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      {#if direction === 'asc'}
        <!-- Ascending arrow (up) -->
        <polyline points="5 12 12 5 19 12"></polyline>
        <line x1="12" y1="5" x2="12" y2="19"></line>
      {:else}
        <!-- Descending arrow (down) -->
        <polyline points="5 12 12 19 19 12"></polyline>
        <line x1="12" y1="5" x2="12" y2="19"></line>
      {/if}
    </svg>
  </button>
</div>

<style>
  .sort-controls {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    white-space: nowrap;
  }

  .sort-select {
    padding: 0.28rem 0.65rem;
    border-radius: 5px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    font-size: 0.8rem;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s;
  }

  .sort-select:hover {
    background: var(--surface);
  }

  .sort-select:focus {
    outline: none;
    border-color: var(--accent);
    background: var(--surface);
  }

  .sort-direction-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface-2);
    color: var(--muted);
    cursor: pointer;
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .sort-direction-btn:hover {
    background: var(--surface);
    color: var(--text);
    border-color: var(--accent);
  }

  .sort-direction-btn:focus {
    outline: none;
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
