<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getDuplicateGroups,
    getIgnoredDuplicates,
    getStructuralFindings,
    ignoreDuplicatePair,
    mergeAlbums,
    mergeArtists,
    mergeTracks,
    restoreIgnoredDuplicate,
  } from '../lib/api';
  import type {
    DedupIgnoreDto,
    DuplicateEntityType,
    DuplicateGroupDto,
    StructuralFindingDto,
  } from '../lib/types';

  type Section = 'duplicates' | 'references';
  const entityTabs: { value: DuplicateEntityType; label: string }[] = [
    { value: 'artists', label: 'Artists' },
    { value: 'albums', label: 'Albums' },
    { value: 'tracks', label: 'Tracks' },
  ];

  let section: Section = $state('duplicates');
  let entityType: DuplicateEntityType = $state('artists');
  let groups: DuplicateGroupDto[] = $state([]);
  let ignored: DedupIgnoreDto[] = $state([]);
  let findings: StructuralFindingDto[] = $state([]);
  let loading = $state(false);
  let findingsLoading = $state(false);
  let error: string | null = $state(null);
  let findingError: string | null = $state(null);
  let operationError: string | null = $state(null);
  let activeGroup = $state<string | null>(null);
  let targetByGroup: Record<string, number> = $state({});
  let duplicateRequestId = 0;
  let findingRequestId = 0;

  const kindLabels: Record<StructuralFindingDto['kind'], string> = {
    conflicting_reference: 'Conflicting reference',
    multiple_platform_references: 'Multiple platform references',
    platform_url_mismatch: 'Platform / URL mismatch',
    track_missing_source_reference: 'Missing Source reference',
  };

  function groupKey(group: DuplicateGroupDto): string {
    return `${group.entity_type}:${group.candidates.map(candidate => candidate.id).sort((a, b) => a - b).join(',')}`;
  }

  function chosenTarget(group: DuplicateGroupDto): number {
    return targetByGroup[groupKey(group)] ?? group.suggested_target_id;
  }

  function chooseTarget(group: DuplicateGroupDto, targetId: number) {
    targetByGroup = { ...targetByGroup, [groupKey(group)]: targetId };
  }

  function addIgnoredPairs(type: DuplicateGroupDto['entity_type'], pairs: [number, number][]) {
    const existing = new Set(ignored.map(pair => `${pair.entity_type}:${pair.id_a}:${pair.id_b}`));
    const additions: DedupIgnoreDto[] = [];
    for (const [left, right] of pairs) {
      const idA = Math.min(left, right);
      const idB = Math.max(left, right);
      const key = `${type}:${idA}:${idB}`;
      if (!existing.has(key)) {
        existing.add(key);
        additions.push({ entity_type: type, id_a: idA, id_b: idB });
      }
    }
    ignored = [...ignored, ...additions];
  }

  async function loadDuplicates() {
    const requestId = ++duplicateRequestId;
    const requestedType = entityType;
    loading = true;
    error = null;
    try {
      const [nextGroups, nextIgnored] = await Promise.all([
        getDuplicateGroups(requestedType),
        getIgnoredDuplicates(requestedType),
      ]);
      if (requestId !== duplicateRequestId || requestedType !== entityType) return;
      groups = nextGroups;
      ignored = nextIgnored;
      targetByGroup = {};
    } catch (e) {
      if (requestId !== duplicateRequestId || requestedType !== entityType) return;
      error = e instanceof Error ? e.message : String(e);
    } finally {
      if (requestId === duplicateRequestId) loading = false;
    }
  }

  async function loadFindings() {
    const requestId = ++findingRequestId;
    findingsLoading = true;
    findingError = null;
    try {
      const nextFindings = await getStructuralFindings();
      if (requestId !== findingRequestId) return;
      findings = nextFindings;
    } catch (e) {
      if (requestId !== findingRequestId) return;
      findingError = e instanceof Error ? e.message : String(e);
    } finally {
      if (requestId === findingRequestId) findingsLoading = false;
    }
  }

  async function selectEntityType(next: DuplicateEntityType) {
    entityType = next;
    await loadDuplicates();
  }

  async function mergeGroup(group: DuplicateGroupDto) {
    const targetId = chosenTarget(group);
    const sourceIds = group.candidates.map(candidate => candidate.id).filter(id => id !== targetId);
    const target = group.candidates.find(candidate => candidate.id === targetId);
    const sources = group.candidates.filter(candidate => candidate.id !== targetId);
    if (!target || sourceIds.length === 0) return;
    const qualityNote = group.entity_type === 'track'
      ? '\n\nThe recording selected by Soundome’s existing quality comparator and its metadata will be kept; the selected row ID will survive.'
      : '';
    if (!confirm(`Merge ${sources.map(candidate => `"${candidate.name}"`).join(', ')} into "${target.name}"?${qualityNote}\n\nThis cannot be undone.`)) return;

    const key = groupKey(group);
    activeGroup = key;
    operationError = null;
    try {
      if (group.entity_type === 'artist') await mergeArtists(sourceIds, targetId);
      else if (group.entity_type === 'album') await mergeAlbums(sourceIds, targetId);
      else await mergeTracks(sourceIds, targetId);
      // Keep the current queue stable after a merge. The user can explicitly
      // request a fresh analysis with Re-scan when ready.
      groups = groups.filter(candidateGroup => groupKey(candidateGroup) !== key);
      ignored = ignored.filter(pair => !sourceIds.includes(pair.id_a) && !sourceIds.includes(pair.id_b));
      targetByGroup = {};
    } catch (e) {
      operationError = e instanceof Error ? e.message : String(e);
    } finally {
      activeGroup = null;
    }
  }

  async function ignoreGroup(group: DuplicateGroupDto) {
    const requestedType = entityType;
    const pairs: [number, number][] = [];
    for (let i = 0; i < group.candidates.length; i++) {
      for (let j = i + 1; j < group.candidates.length; j++) {
        pairs.push([group.candidates[i].id, group.candidates[j].id]);
      }
    }
    const names = group.candidates.map(candidate => `"${candidate.name}"`).join(', ');
    if (!confirm(`Mark every pair in this group as not a duplicate?\n\n${names}\n\nYou can undo this later in the ignored-pairs list.`)) return;

    const key = groupKey(group);
    activeGroup = key;
    operationError = null;
    try {
      for (const [idA, idB] of pairs) {
        await ignoreDuplicatePair(requestedType, idA, idB);
      }
      addIgnoredPairs(group.entity_type, pairs);
      groups = groups.filter(candidateGroup => groupKey(candidateGroup) !== key);
    } catch (e) {
      operationError = e instanceof Error ? e.message : String(e);
    } finally {
      activeGroup = null;
    }
  }

  async function excludeCandidate(group: DuplicateGroupDto, excludedId: number) {
    const requestedType = entityType;
    const excluded = group.candidates.find(candidate => candidate.id === excludedId);
    const otherCandidates = group.candidates.filter(candidate => candidate.id !== excludedId);
    if (!excluded || otherCandidates.length < 2) return;
    if (!confirm(`Remove "${excluded.name}" from this group?\n\nSoundome will persistently mark it as unrelated to each of the other ${otherCandidates.length} items. It can still be suggested with different entities later.`)) return;

    const key = groupKey(group);
    activeGroup = key;
    operationError = null;
    try {
      const pairs = otherCandidates.map(candidate => [excludedId, candidate.id] as [number, number]);
      for (const [idA, idB] of pairs) {
        await ignoreDuplicatePair(requestedType, idA, idB);
      }
      addIgnoredPairs(group.entity_type, pairs);
      const remaining = group.candidates.filter(candidate => candidate.id !== excludedId);
      const updatedGroups: DuplicateGroupDto[] = [];
      for (const candidateGroup of groups) {
        if (groupKey(candidateGroup) !== key) {
          updatedGroups.push(candidateGroup);
        } else if (remaining.length > 1) {
          const suggestedTargetId = remaining.some(candidate => candidate.id === group.suggested_target_id)
            ? group.suggested_target_id
            : remaining[0].id;
          updatedGroups.push({ ...group, candidates: remaining, suggested_target_id: suggestedTargetId });
        }
      }
      groups = updatedGroups;
      targetByGroup = {};
    } catch (e) {
      operationError = e instanceof Error ? e.message : String(e);
    } finally {
      activeGroup = null;
    }
  }

  async function undoIgnore(pair: DedupIgnoreDto) {
    const requestedType = entityType;
    operationError = null;
    try {
      await restoreIgnoredDuplicate(requestedType, pair.id_a, pair.id_b);
      ignored = ignored.filter(existing =>
        !(existing.entity_type === pair.entity_type
          && existing.id_a === pair.id_a
          && existing.id_b === pair.id_b)
      );
    } catch (e) {
      operationError = e instanceof Error ? e.message : String(e);
    }
  }

  function formatDuration(seconds: number | null): string | null {
    if (seconds == null) return null;
    return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
  }

  function formatQualityValue(value: number | null): string | null {
    return value == null ? null : `quality metric ${value}`;
  }

  onMount(() => {
    void loadDuplicates();
  });
</script>

<div class="data-quality">
  <div class="header">
    <div>
      <h1>Data Quality</h1>
      <p class="subtitle">Review duplicate suggestions and inspect structural reference issues.</p>
    </div>
  </div>

  <nav class="section-tabs" aria-label="Data quality sections">
    <button class:active={section === 'duplicates'} onclick={() => { section = 'duplicates'; }}>Duplicates</button>
    <button class:active={section === 'references'} onclick={() => { section = 'references'; loadFindings(); }}>Reference audit</button>
  </nav>

  {#if section === 'duplicates'}
    <div class="entity-tabs" aria-label="Duplicate entity type">
      {#each entityTabs as tab (tab.value)}
        <button class:active={entityType === tab.value} onclick={() => selectEntityType(tab.value)}>{tab.label}</button>
      {/each}
      <div class="toolbar-actions">
        <span class="count">{groups.length} group{groups.length !== 1 ? 's' : ''}</span>
        <button class="btn-refresh" onclick={loadDuplicates} disabled={loading}>{loading ? 'Scanning…' : 'Re-scan'}</button>
      </div>
    </div>

    {#if operationError}<p class="status error">{operationError}</p>{/if}
    {#if error}
      <p class="status error">{error}</p>
    {:else if loading}
      <p class="status">Scanning for similar {entityType}…</p>
    {:else if groups.length === 0}
      <p class="status ok">No duplicate suggestions found for {entityType}.</p>
    {:else}
      <p class="helper">Groups use the existing Soundome similarity models. The suggested keep-target has the most linked tracks, then references, then the cleanest name.</p>
      <div class="group-list">
        {#each groups as group (groupKey(group))}
          {@const key = groupKey(group)}
          {@const targetId = chosenTarget(group)}
          <article class="group-card">
            <header class="group-header">
              <div>
                <strong>{group.candidates.length} similar {group.entity_type}s</strong>
                <span class="group-subtitle">Choose the record to keep, or mark the group as unrelated.</span>
              </div>
              <span class="recommended">Suggested: #{group.suggested_target_id}</span>
            </header>

            <div class="candidate-grid">
              {#each group.candidates as candidate (candidate.id)}
                <section class="candidate" class:target={targetId === candidate.id}>
                  <div class="candidate-heading">
                    <div>
                      <div class="candidate-name">{candidate.name}</div>
                      <div class="candidate-id">ID #{candidate.id}</div>
                    </div>
                    <label class="target-choice">
                      <input type="radio" name={key} checked={targetId === candidate.id} onchange={() => chooseTarget(group, candidate.id)} />
                      Keep
                    </label>
                  </div>

                  {#if group.candidates.length > 2}
                    <button
                      class="btn-exclude"
                      onclick={() => excludeCandidate(group, candidate.id)}
                      disabled={activeGroup === key}
                      title="Persistently exclude this item from the current group"
                    >
                      Exclude from group
                    </button>
                  {/if}

                  {#if candidate.artists.length > 0}<div class="candidate-meta">Artists: {candidate.artists.join(', ')}</div>{/if}
                  {#if candidate.album_title}<div class="candidate-meta">Album: {candidate.album_title}</div>{/if}
                  <div class="candidate-meta">
                    {#if candidate.track_count > 0}{candidate.track_count} track{candidate.track_count !== 1 ? 's' : ''}{/if}
                    {#if candidate.album_count > 0} · {candidate.album_count} album{candidate.album_count !== 1 ? 's' : ''}{/if}
                    {#if candidate.date} · {candidate.date.slice(0, 4)}{/if}
                    {#if formatDuration(candidate.duration)} · {formatDuration(candidate.duration)}{/if}
                    {#if formatQualityValue(candidate.quality_value)} · {formatQualityValue(candidate.quality_value)}{/if}
                    · {candidate.reference_count} reference{candidate.reference_count !== 1 ? 's' : ''}
                  </div>
                  {#if candidate.references.length > 0}
                    <div class="reference-links">
                      {#each candidate.references as reference, index (reference.id ?? `${reference.platform}-${index}`)}
                        {#if reference.external_url}
                          <a href={reference.external_url} target="_blank" rel="noreferrer">{reference.platform} · {reference.ref_type}</a>
                        {:else}
                          <span>{reference.platform} · {reference.ref_type}</span>
                        {/if}
                      {/each}
                    </div>
                  {/if}
                </section>
              {/each}
            </div>

            <footer class="group-actions">
              <button class="btn-ignore" onclick={() => ignoreGroup(group)} disabled={activeGroup === key}>
                Not a duplicate
              </button>
              <button class="btn-merge" onclick={() => mergeGroup(group)} disabled={activeGroup === key}>
                {activeGroup === key ? 'Working…' : `Merge into #${targetId}`}
              </button>
            </footer>
          </article>
        {/each}
      </div>
    {/if}

    <details class="ignored-list">
      <summary>Ignored pairs ({ignored.length})</summary>
      {#if ignored.length === 0}
        <p class="status">No pairs ignored.</p>
      {:else}
        <div class="ignored-rows">
          {#each ignored as pair (pair.id_a + ':' + pair.id_b)}
            <div class="ignored-row">
              <span>{pair.entity_type} #{pair.id_a} and #{pair.id_b}</span>
              <button class="btn-undo" onclick={() => undoIgnore(pair)}>Undo</button>
            </div>
          {/each}
        </div>
      {/if}
    </details>
  {:else}
    <div class="toolbar reference-toolbar">
      <button class="btn-refresh" onclick={loadFindings} disabled={findingsLoading}>
        {findingsLoading ? 'Scanning…' : 'Re-scan'}
      </button>
      <span class="count">{findings.length} finding{findings.length !== 1 ? 's' : ''}</span>
      <span class="helper">Structural checks only — no network calls.</span>
    </div>

    {#if findingError}
      <p class="status error">{findingError}</p>
    {:else if findingsLoading}
      <p class="status">Scanning…</p>
    {:else if findings.length === 0}
      <p class="status ok">No structural anomaly found. Your reference data looks consistent.</p>
    {:else}
      <div class="findings">
        {#each findings as finding, i (i)}
          <div class="finding-card kind-{finding.kind}">
            <div class="finding-header">
              <span class="finding-kind">{kindLabels[finding.kind]}</span>
              {#if finding.platform}<span class="finding-platform">{finding.platform}</span>{/if}
            </div>
            <p class="finding-message">{finding.message}</p>
            {#if finding.entities.length > 0}
              <div class="finding-entities">
                {#each finding.entities as entity (entity.entity_type + entity.id)}
                  <span class="entity-chip">{entity.entity_type} #{entity.id} — {entity.name}</span>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .data-quality { padding: 1.5rem; max-width: 1100px; margin: 0 auto; }
  .header { display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem; }
  h1 { font-size: 1.35rem; font-weight: 700; margin: 0 0 0.25rem; }
  .subtitle,.helper { color: var(--muted); font-size: 0.85rem; margin: 0; }
  .helper { margin: 0.3rem 0 0.85rem; }

  .section-tabs,.entity-tabs { display: flex; align-items: center; gap: 0.4rem; border-bottom: 1px solid var(--border); margin-bottom: 1rem; }
  .section-tabs button,.entity-tabs > button {
    border: 0; border-bottom: 2px solid transparent; padding: 0.65rem 0.8rem;
    background: transparent; color: var(--muted); cursor: pointer; font: inherit;
  }
  .section-tabs button.active,.entity-tabs > button.active { color: var(--text); border-bottom-color: var(--accent); }
  .entity-tabs { border-bottom: 0; }
  .toolbar-actions { margin-left: auto; display: flex; align-items: center; gap: 0.75rem; }
  .toolbar,.reference-toolbar { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem; }
  .btn-refresh,.btn-ignore,.btn-merge,.btn-undo,.btn-exclude {
    padding: 0.35rem 0.8rem; border-radius: 6px; border: 1px solid var(--border);
    background: var(--surface-2); color: var(--text); cursor: pointer; font-family: inherit; font-size: 0.82rem;
  }
  button:disabled { opacity: 0.55; cursor: not-allowed; }
  .btn-refresh:hover:not(:disabled),.btn-ignore:hover:not(:disabled),.btn-undo:hover:not(:disabled),.btn-exclude:hover:not(:disabled) { background: var(--surface); }
  .btn-exclude { margin-top: 0.6rem; color: var(--muted); }
  .btn-merge { background: var(--accent); border-color: var(--accent); color: white; font-weight: 650; }
  .btn-merge:hover:not(:disabled) { filter: brightness(1.1); }
  .count { color: var(--muted); font-size: 0.85rem; white-space: nowrap; }
  .status { padding: 1rem 0; color: var(--muted); }
  .status.error { color: #e05252; }
  .status.ok { color: #22c55e; }

  .group-list { display: flex; flex-direction: column; gap: 1rem; }
  .group-card { border: 1px solid var(--border); background: var(--surface); border-radius: 9px; overflow: hidden; }
  .group-header { padding: 0.8rem 1rem; display: flex; align-items: center; justify-content: space-between; gap: 1rem; border-bottom: 1px solid var(--border); }
  .group-header > div { display: flex; flex-direction: column; gap: 0.2rem; }
  .group-subtitle,.candidate-id { color: var(--muted); font-size: 0.75rem; }
  .recommended { color: #22c55e; font-size: 0.75rem; white-space: nowrap; }
  .candidate-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr)); gap: 0.65rem; padding: 0.8rem; }
  .candidate { min-width: 0; padding: 0.75rem; border: 1px solid var(--border); border-radius: 7px; background: var(--bg); }
  .candidate.target { border-color: color-mix(in srgb, #22c55e 65%, var(--border)); box-shadow: inset 0 0 0 1px color-mix(in srgb, #22c55e 35%, transparent); }
  .candidate-heading { display: flex; align-items: start; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.55rem; }
  .candidate-name { font-weight: 700; overflow-wrap: anywhere; }
  .target-choice { display: flex; align-items: center; gap: 0.25rem; color: #22c55e; font-size: 0.75rem; white-space: nowrap; cursor: pointer; }
  .candidate-meta { color: var(--muted); font-size: 0.75rem; margin-top: 0.25rem; overflow-wrap: anywhere; }
  .reference-links { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-top: 0.5rem; }
  .reference-links a,.reference-links span { color: var(--accent); font-size: 0.7rem; }
  .reference-links span { color: var(--muted); }
  .group-actions { display: flex; justify-content: flex-end; gap: 0.5rem; border-top: 1px solid var(--border); padding: 0.7rem 0.8rem; }
  .ignored-list { margin-top: 1.25rem; border-top: 1px solid var(--border); padding-top: 0.8rem; }
  .ignored-list summary { cursor: pointer; color: var(--muted); font-size: 0.85rem; }
  .ignored-rows { margin-top: 0.5rem; }
  .ignored-row { display: flex; justify-content: space-between; align-items: center; padding: 0.45rem 0.15rem; border-bottom: 1px solid var(--border); font-size: 0.8rem; }

  .findings { display: flex; flex-direction: column; gap: 0.75rem; }
  .finding-card { border: 1px solid var(--border); border-radius: 8px; padding: 0.85rem 1rem; background: var(--surface); }
  .finding-card.kind-conflicting_reference { border-left: 3px solid #e05252; }
  .finding-card.kind-multiple_platform_references,.finding-card.kind-platform_url_mismatch { border-left: 3px solid #f59e0b; }
  .finding-card.kind-track_missing_source_reference { border-left: 3px solid #6b7280; }
  .finding-header { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.35rem; }
  .finding-kind { font-weight: 700; font-size: 0.85rem; }
  .finding-platform { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); background: var(--surface-2); border-radius: 4px; padding: 0.1rem 0.4rem; }
  .finding-message { margin: 0 0 0.5rem; font-size: 0.85rem; }
  .finding-entities { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .entity-chip { font-size: 0.75rem; background: var(--surface-2); border: 1px solid var(--border); border-radius: 999px; padding: 0.1rem 0.6rem; color: var(--muted); }
  @media (max-width: 640px) { .data-quality { padding: 1rem; } .entity-tabs { flex-wrap: wrap; } .toolbar-actions { margin-left: 0; } .group-header { align-items: start; flex-direction: column; } }
</style>
