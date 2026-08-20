<script lang="ts">
  import { getSyncSchedules, createSyncSchedule, deleteSyncSchedule } from '../api';
  import type { SyncScheduleDto } from '../api';
  import type { ReferenceDto } from '../types';

  let { artistId, artistName, references }: {
    artistId: number;
    artistName: string;
    references: ReferenceDto[];
  } = $props();

  // Module-level cache so the artist detail page and the edit modal don't
  // each fetch/hold their own out-of-sync copy of the subscription list.
  let syncSchedules: SyncScheduleDto[] = $state([]);
  let syncSchedulesLoaded = $state(false);
  let syncTogglingRefId: number | null = $state(null);
  let syncError: string | null = $state(null);

  async function loadSyncSchedules() {
    try {
      syncSchedules = await getSyncSchedules();
    } catch (e: unknown) {
      syncError = e instanceof Error ? e.message : String(e);
    } finally {
      syncSchedulesLoaded = true;
    }
  }

  function findSubscription(referenceId: number): SyncScheduleDto | undefined {
    return syncSchedules.find(
      (s) => s.entity_type === 'artist' && s.artist_id === artistId && s.reference_id === referenceId,
    );
  }

  async function toggleSourceSync(referenceId: number) {
    syncError = null;
    syncTogglingRefId = referenceId;
    try {
      const existing = findSubscription(referenceId);
      if (existing) {
        await deleteSyncSchedule(existing.id);
        syncSchedules = syncSchedules.filter((s) => s.id !== existing.id);
      } else {
        const created = await createSyncSchedule({
          artist_id: artistId,
          reference_id: referenceId,
          label: artistName.trim() || undefined,
        });
        syncSchedules = [...syncSchedules, created];
      }
    } catch (e: unknown) {
      syncError = e instanceof Error ? e.message : String(e);
    } finally {
      syncTogglingRefId = null;
    }
  }

  $effect(() => {
    if (!syncSchedulesLoaded) loadSyncSchedules();
  });

  let sources = $derived(references.filter((r) => r.ref_type === 'Source'));
</script>

<div class="sync-panel">
  <div class="sync-panel-title">Scheduled sync</div>
  {#if syncError}
    <p class="sync-error">{syncError}</p>
  {/if}
  {#if !syncSchedulesLoaded}
    <p class="sync-empty">Loading…</p>
  {:else if sources.length === 0}
    <p class="sync-empty">No source reference yet — add one above to enable scheduled sync.</p>
  {:else}
    <div class="sync-sources">
      {#each sources as ref (ref.id)}
        {@const subscribed = ref.id != null && !!findSubscription(ref.id)}
        <label class="sync-source" class:active={subscribed}>
          <input
            type="checkbox"
            checked={subscribed}
            disabled={ref.id == null || syncTogglingRefId === ref.id}
            onchange={() => ref.id != null && toggleSourceSync(ref.id)}
          />
          {ref.platform}
        </label>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sync-panel { margin-top: 0.4rem; }
  .sync-panel-title { font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin-bottom: 0.4rem; font-weight: 500; }
  .sync-sources { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .sync-source {
    display: inline-flex; align-items: center; gap: 0.4rem;
    padding: 0.28rem 0.65rem; border-radius: 999px; border: 1px solid var(--border);
    background: var(--surface-2); color: var(--muted); cursor: pointer; font-size: 0.8rem;
  }
  .sync-source.active { background: color-mix(in srgb, var(--accent) 14%, var(--surface)); color: var(--accent); border-color: color-mix(in srgb, var(--accent) 45%, transparent); }
  .sync-source input[type="checkbox"] { cursor: pointer; }
  .sync-empty { font-size: 0.8rem; color: var(--muted); margin: 0; }
  .sync-error { font-size: 0.8rem; color: var(--error); margin: 0 0 0.4rem; }
</style>
