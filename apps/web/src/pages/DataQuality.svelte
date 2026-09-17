<script lang="ts">
  import { onMount } from 'svelte';
  import { getStructuralFindings } from '../lib/api';
  import type { StructuralFindingDto } from '../lib/types';

  let findings: StructuralFindingDto[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  const kindLabels: Record<StructuralFindingDto['kind'], string> = {
    conflicting_reference: 'Conflicting reference',
    multiple_platform_references: 'Multiple platform references',
    platform_url_mismatch: 'Platform / URL mismatch',
    track_missing_source_reference: 'Missing Source reference',
  };

  async function load() {
    loading = true;
    error = null;
    try {
      findings = await getStructuralFindings();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<div class="data-quality">
  <div class="header">
    <h1>Data Quality</h1>
    <p class="subtitle">Structural reference audit — no network calls, computed from current data.</p>
  </div>

  <div class="toolbar">
    <button class="btn-refresh" onclick={load} disabled={loading}>
      {loading ? 'Scanning…' : 'Re-scan'}
    </button>
    <span class="count">{findings.length} finding{findings.length !== 1 ? 's' : ''}</span>
  </div>

  {#if error}
    <p class="status error">{error}</p>
  {:else if loading}
    <p class="status">Scanning…</p>
  {:else if findings.length === 0}
    <p class="status ok">No structural anomaly found. Your reference data looks consistent.</p>
  {:else}
    <div class="findings">
      {#each findings as f, i (i)}
        <div class="finding-card kind-{f.kind}">
          <div class="finding-header">
            <span class="finding-kind">{kindLabels[f.kind]}</span>
            {#if f.platform}<span class="finding-platform">{f.platform}</span>{/if}
          </div>
          <p class="finding-message">{f.message}</p>
          {#if f.entities.length > 0}
            <div class="finding-entities">
              {#each f.entities as e (e.entity_type + e.id)}
                <span class="entity-chip">{e.entity_type} #{e.id} — {e.name}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .data-quality { padding: 1.5rem; max-width: 900px; margin: 0 auto; }
  .header { margin-bottom: 1rem; }
  h1 { font-size: 1.35rem; font-weight: 700; margin: 0 0 0.25rem; }
  .subtitle { color: var(--muted); font-size: 0.85rem; margin: 0; }

  .toolbar { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
  .btn-refresh {
    padding: 0.35rem 0.9rem; border-radius: 6px; border: 1px solid var(--border);
    background: var(--surface-2); color: var(--text); cursor: pointer; font-family: inherit; font-size: 0.85rem;
  }
  .btn-refresh:hover:not(:disabled) { background: var(--surface); }
  .btn-refresh:disabled { opacity: 0.6; cursor: not-allowed; }
  .count { color: var(--muted); font-size: 0.85rem; }

  .status { padding: 1rem 0; color: var(--muted); }
  .status.error { color: #e05252; }
  .status.ok { color: #22c55e; }

  .findings { display: flex; flex-direction: column; gap: 0.75rem; }
  .finding-card {
    border: 1px solid var(--border); border-radius: 8px; padding: 0.85rem 1rem;
    background: var(--surface);
  }
  .finding-card.kind-conflicting_reference { border-left: 3px solid #e05252; }
  .finding-card.kind-multiple_platform_references { border-left: 3px solid #f59e0b; }
  .finding-card.kind-platform_url_mismatch { border-left: 3px solid #f59e0b; }
  .finding-card.kind-track_missing_source_reference { border-left: 3px solid #6b7280; }

  .finding-header { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.35rem; }
  .finding-kind { font-weight: 700; font-size: 0.85rem; }
  .finding-platform {
    font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--muted); background: var(--surface-2); border-radius: 4px; padding: 0.1rem 0.4rem;
  }
  .finding-message { margin: 0 0 0.5rem; font-size: 0.85rem; }
  .finding-entities { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .entity-chip {
    font-size: 0.75rem; background: var(--surface-2); border: 1px solid var(--border);
    border-radius: 999px; padding: 0.1rem 0.6rem; color: var(--muted);
  }
</style>
