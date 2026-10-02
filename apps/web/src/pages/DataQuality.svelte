<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getDuplicateGroups,
    getIgnoredDuplicates,
    getRemoteReferenceAudit,
    getStructuralFindings,
    startRemoteReferenceAudit,
    ignoreDuplicatePair,
    mergeAlbums,
    mergeArtists,
    mergeTracks,
    applyRemoteAuditName,
    dismissRemoteAuditResult,
    deleteAuditedReference,
    restoreIgnoredDuplicate,
    batchFetchAlbumCovers,
    batchFetchArtistIcons,
    cleanupOrphans,
    getAiCleanupLog,
    getOrphans,
    getPlaylistConsistencyIssues,
    renumberPlaylist,
  } from '../lib/api';
  import type {
    DedupIgnoreDto,
    DuplicateEntityType,
    DuplicateGroupDto,
    ReferenceAuditViewDto,
    StructuralFindingDto,
    AiCleanupLogDto,
    OrphanedEntityDto,
    PlaylistConsistencyIssueDto,
  } from '../lib/types';

  type Section = 'duplicates' | 'references' | 'cleanup';
  type ReferenceTab = 'structural' | 'remote';
  type RemoteStatusFilter = ReferenceAuditViewDto['status'] | 'all';
  const entityTabs: { value: DuplicateEntityType; label: string }[] = [
    { value: 'artists', label: 'Artists' },
    { value: 'albums', label: 'Albums' },
    { value: 'tracks', label: 'Tracks' },
  ];

  let section: Section = $state('duplicates');
  let referenceTab: ReferenceTab = $state('structural');
  let entityType: DuplicateEntityType = $state('artists');
  let groups: DuplicateGroupDto[] = $state([]);
  let ignored: DedupIgnoreDto[] = $state([]);
  let findings: StructuralFindingDto[] = $state([]);
  let remoteResults: ReferenceAuditViewDto[] = $state([]);
  let loading = $state(false);
  let findingsLoading = $state(false);
  let remoteLoading = $state(false);
  let remoteStarting = $state(false);
  let error: string | null = $state(null);
  let findingError: string | null = $state(null);
  let remoteError: string | null = $state(null);
  let remoteTaskId: number | null = $state(null);
  let remoteStatusFilter: RemoteStatusFilter = $state('mismatch');
  let remoteSearch = $state('');
  let remotePlatformFilter = $state('all');
  let remoteEntityFilter: 'all' | 'artist' | 'album' | 'track' = $state('all');
  let operationError: string | null = $state(null);
  let orphans: OrphanedEntityDto[] = $state([]);
  let playlistIssues: PlaylistConsistencyIssueDto[] = $state([]);
  let cleanupLog: AiCleanupLogDto[] = $state([]);
  let cleanupLoading = $state(false);
  let cleanupError: string | null = $state(null);
  let cleanupMessage: string | null = $state(null);
  let cleanupWorking = $state(false);
  let iconsFetching = $state(false);
  let coversFetching = $state(false);
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

  async function loadRemoteResults() {
    remoteLoading = true;
    remoteError = null;
    try {
      remoteResults = await getRemoteReferenceAudit();
    } catch (e) {
      remoteError = e instanceof Error ? e.message : String(e);
    } finally {
      remoteLoading = false;
    }
  }

  async function selectReferenceTab(next: ReferenceTab) {
    referenceTab = next;
    if (next === 'remote') await loadRemoteResults();
    else await loadFindings();
  }

  async function loadCleanup() {
    cleanupLoading = true;
    cleanupError = null;
    try {
      [orphans, playlistIssues, cleanupLog] = await Promise.all([
        getOrphans(), getPlaylistConsistencyIssues(), getAiCleanupLog(),
      ]);
    } catch (e) {
      cleanupError = e instanceof Error ? e.message : String(e);
    } finally {
      cleanupLoading = false;
    }
  }

  async function selectSection(next: Section) {
    section = next;
    if (next === 'cleanup') await loadCleanup();
    else if (next === 'references') await selectReferenceTab(referenceTab);
  }

  async function removeOrphans() {
    if (orphans.length === 0 || !confirm(`Delete ${orphans.length} orphaned artist/album record(s)? This cannot be undone.`)) return;
    cleanupWorking = true;
    cleanupError = null;
    try {
      const result = await cleanupOrphans();
      cleanupMessage = `Deleted ${result.artists_deleted} artist(s) and ${result.albums_deleted} album(s).`;
      await loadCleanup();
    } catch (e) {
      cleanupError = e instanceof Error ? e.message : String(e);
    } finally {
      cleanupWorking = false;
    }
  }

  async function fixPlaylist(issue: PlaylistConsistencyIssueDto) {
    cleanupWorking = true;
    cleanupError = null;
    try {
      await renumberPlaylist(issue.playlist_id);
      cleanupMessage = `Renumbered “${issue.playlist_name}”.`;
      await loadCleanup();
    } catch (e) {
      cleanupError = e instanceof Error ? e.message : String(e);
    } finally {
      cleanupWorking = false;
    }
  }

  async function fetchImages(kind: 'icons' | 'covers') {
    if (kind === 'icons') iconsFetching = true;
    else coversFetching = true;
    cleanupError = null;
    try {
      const result = kind === 'icons' ? await batchFetchArtistIcons() : await batchFetchAlbumCovers();
      cleanupMessage = `${result.count} fetched · ${result.skipped} not found.`;
    } catch (e) {
      cleanupError = e instanceof Error ? e.message : String(e);
    } finally {
      if (kind === 'icons') iconsFetching = false;
      else coversFetching = false;
    }
  }

  async function runRemoteAudit() {
    remoteStarting = true;
    remoteError = null;
    try {
      const task = await startRemoteReferenceAudit();
      remoteTaskId = task.task_id;
      await loadRemoteResults();
    } catch (e) {
      remoteError = e instanceof Error ? e.message : String(e);
    } finally {
      remoteStarting = false;
    }
  }

  async function applyRemoteName(result: ReferenceAuditViewDto) {
    if (!result.remote_name || !confirm(`Apply "${result.remote_name}" to "${result.local_name}"?`)) return;
    try {
      await applyRemoteAuditName(result.id);
      await loadRemoteResults();
    } catch (e) {
      remoteError = e instanceof Error ? e.message : String(e);
    }
  }

  async function dismissRemoteResult(result: ReferenceAuditViewDto) {
    try {
      await dismissRemoteAuditResult(result.id);
      await loadRemoteResults();
    } catch (e) {
      remoteError = e instanceof Error ? e.message : String(e);
    }
  }

  async function removeAuditedReference(result: ReferenceAuditViewDto) {
    if (!confirm(`Delete the ${result.platform ?? 'unknown platform'} reference from "${result.local_name}"? This cannot be undone.`)) return;
    try {
      await deleteAuditedReference(result.id);
      await loadRemoteResults();
    } catch (e) {
      remoteError = e instanceof Error ? e.message : String(e);
    }
  }

  function auditStatusLabel(status: ReferenceAuditViewDto['status']): string {
    return {
      ok: 'Matches',
      mismatch: 'Name mismatch',
      unreachable: 'Unreachable',
      unsupported: 'Unsupported',
      dismissed: 'Dismissed',
      missing: 'Missing metadata',
    }[status] ?? status;
  }

  let remoteIssues = $derived(remoteResults.filter(result => result.status === 'mismatch'));
  let remoteMatchesCount = $derived(remoteResults.filter(result => result.status === 'ok').length);
  let remoteDismissedCount = $derived(remoteResults.filter(result => result.status === 'dismissed').length);
  let remoteUnreachableCount = $derived(remoteResults.filter(result => result.status === 'unreachable').length);
  let remoteUnsupportedCount = $derived(remoteResults.filter(result => result.status === 'unsupported').length);
  let remoteMissingCount = $derived(remoteResults.filter(result => result.status === 'missing').length);
  let remoteVisibleResults = $derived.by(() => {
    const search = remoteSearch.trim().toLowerCase();
    return remoteResults
      .filter(result => remoteStatusFilter === 'all' || result.status === remoteStatusFilter)
      .filter(result => remotePlatformFilter === 'all' || result.platform === remotePlatformFilter)
      .filter(result => remoteEntityFilter === 'all' || result.entity_type === remoteEntityFilter)
      .filter(result => {
        if (!search) return true;
        return [result.local_name, result.remote_name, result.external_id, result.external_url]
          .some(value => value?.toLowerCase().includes(search));
      })
      .sort((a, b) => {
        const scoreA = a.similarity_score ?? Number.POSITIVE_INFINITY;
        const scoreB = b.similarity_score ?? Number.POSITIVE_INFINITY;
        return scoreA - scoreB || a.local_name.localeCompare(b.local_name);
      });
  });

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
    <button class:active={section === 'duplicates'} onclick={() => selectSection('duplicates')}>Duplicates</button>
    <button class:active={section === 'references'} onclick={() => selectSection('references')}>Reference audit</button>
    <button class:active={section === 'cleanup'} onclick={() => selectSection('cleanup')}>Cleanup</button>
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
  {:else if section === 'references'}
    <div class="entity-tabs" aria-label="Reference audit mode">
      <button class:active={referenceTab === 'structural'} onclick={() => selectReferenceTab('structural')}>Structural</button>
      <button class:active={referenceTab === 'remote'} onclick={() => selectReferenceTab('remote')}>Remote</button>
    </div>

    {#if referenceTab === 'structural'}
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
    {:else}
      <div class="toolbar reference-toolbar">
        <button class="btn-merge" onclick={runRemoteAudit} disabled={remoteStarting}>
          {remoteStarting ? 'Starting…' : 'Run remote audit'}
        </button>
        <button class="btn-refresh" onclick={loadRemoteResults} disabled={remoteLoading}>
          {remoteLoading ? 'Refreshing…' : 'Refresh results'}
        </button>
      </div>
      <div class="audit-summary">
        <span class="summary-title">Filter:</span>
        <button class="summary-item mismatch" class:active={remoteStatusFilter === 'mismatch'} onclick={() => { remoteStatusFilter = 'mismatch'; }}>Mismatch ({remoteIssues.length})</button>
        <button class="summary-item unreachable" class:active={remoteStatusFilter === 'unreachable'} onclick={() => { remoteStatusFilter = 'unreachable'; }}>Unreachable ({remoteUnreachableCount})</button>
        <button class="summary-item unsupported" class:active={remoteStatusFilter === 'unsupported'} onclick={() => { remoteStatusFilter = 'unsupported'; }}>Unsupported ({remoteUnsupportedCount})</button>
        <button class="summary-item missing" class:active={remoteStatusFilter === 'missing'} onclick={() => { remoteStatusFilter = 'missing'; }}>Missing metadata ({remoteMissingCount})</button>
        <button class="summary-item verified" class:active={remoteStatusFilter === 'ok'} onclick={() => { remoteStatusFilter = 'ok'; }}>Matches ({remoteMatchesCount})</button>
        <button class="summary-item dismissed" class:active={remoteStatusFilter === 'dismissed'} onclick={() => { remoteStatusFilter = 'dismissed'; }}>Dismissed ({remoteDismissedCount})</button>
        <button class="summary-item all-filter" class:active={remoteStatusFilter === 'all'} onclick={() => { remoteStatusFilter = 'all'; }}>All ({remoteResults.length})</button>
      </div>
      <div class="remote-filters">
        <input class="remote-search" placeholder="Search local name, remote name, URL or ID…" bind:value={remoteSearch} />
        <select bind:value={remotePlatformFilter} aria-label="Filter by platform">
          <option value="all">All platforms</option>
          {#each [...new Set(remoteResults.map(result => result.platform).filter((platform): platform is string => platform != null))].sort() as platform}
            <option value={platform}>{platform}</option>
          {/each}
        </select>
        <select bind:value={remoteEntityFilter} aria-label="Filter by item type">
          <option value="all">All item types</option>
          <option value="artist">Artists</option>
          <option value="album">Albums</option>
          <option value="track">Tracks</option>
        </select>
      </div>
      <p class="helper">This sends manual requests to supported providers. Results are cached; this view never starts an audit automatically.</p>
      {#if remoteTaskId != null}
        <p class="task-notice">Remote audit queued as task #{remoteTaskId}. Track its progress on the Tasks page, then refresh these results.</p>
      {/if}
      {#if remoteError}
        <p class="status error">{remoteError}</p>
      {:else if remoteLoading}
        <p class="status">Loading cached remote audit results…</p>
      {:else if remoteResults.length === 0}
        <p class="status">No remote audit results yet. Use “Run remote audit” to query the providers.</p>
      {:else if remoteVisibleResults.length === 0}
        <p class="status ok">No result matches the current filters.</p>
      {:else}
        <div class="findings">
          {#each remoteVisibleResults as result (result.id)}
            <article class="finding-card remote-result status-{result.status}">
              <div class="finding-header">
                <span class="finding-kind">{result.entity_type} · {result.local_name}</span>
                <span class="audit-status">{auditStatusLabel(result.status)}</span>
                {#if result.platform}<span class="finding-platform">{result.platform}</span>{/if}
                {#if result.ref_type}<span class="finding-platform">{result.ref_type}</span>{/if}
              </div>
              <div class="remote-comparison">
                <span><strong>Local:</strong> {result.local_name}</span>
                <span><strong>Remote:</strong> {result.remote_name ?? '—'}</span>
                {#if result.similarity_score != null}<span><strong>Similarity:</strong> {Math.round(result.similarity_score * 100)}%</span>{/if}
              </div>
              {#if result.external_url}
                <a class="audit-url" href={result.external_url} target="_blank" rel="noreferrer">Open reference ↗</a>
              {/if}
              {#if result.checked_at}<div class="candidate-meta">Checked {result.checked_at}</div>{/if}
              <div class="group-actions audit-actions">
                {#if result.status === 'mismatch' && result.remote_name}
                  <button class="btn-merge" onclick={() => applyRemoteName(result)}>Apply remote name</button>
                {/if}
                {#if result.status !== 'dismissed'}
                  <button class="btn-ignore" onclick={() => dismissRemoteResult(result)}>Dismiss</button>
                {/if}
                {#if result.platform != null && result.ref_type != null}
                  <button class="btn-delete-reference" onclick={() => removeAuditedReference(result)}>Delete reference</button>
                {/if}
              </div>
            </article>
          {/each}
        </div>
      {/if}
    {/if}
  {:else}
    <div class="toolbar cleanup-toolbar">
      <button class="btn-refresh" onclick={loadCleanup} disabled={cleanupLoading}>{cleanupLoading ? 'Refreshing…' : 'Refresh cleanup data'}</button>
    </div>
    {#if cleanupError}<p class="status error">{cleanupError}</p>{/if}
    {#if cleanupMessage}<p class="status ok">{cleanupMessage}</p>{/if}
    {#if cleanupLoading}
      <p class="status">Loading cleanup data…</p>
    {:else}
      <section class="cleanup-card">
        <h2>Orphans <span>{orphans.length}</span></h2>
        <p>Artists and albums with no linked tracks.</p>
        {#if orphans.length > 0}<div class="chip-list">{#each orphans as orphan (orphan.entity_type + orphan.id)}<span class="entity-chip">{orphan.entity_type} #{orphan.id} — {orphan.name}</span>{/each}</div>{/if}
        <button class="btn-delete-reference" onclick={removeOrphans} disabled={cleanupWorking || orphans.length === 0}>Delete all orphans</button>
      </section>
      <section class="cleanup-card">
        <h2>Playlist consistency <span>{playlistIssues.length}</span></h2>
        <p>Missing or duplicate positions are repaired into a deterministic zero-based order.</p>
        {#if playlistIssues.length === 0}<p class="status ok">No playlist position issues found.</p>{/if}
        {#each playlistIssues as issue (issue.playlist_id)}
          <div class="cleanup-row"><span><strong>{issue.playlist_name}</strong> · {issue.track_count} tracks · {issue.missing_positions} missing · duplicates: {issue.duplicate_positions.join(', ') || 'none'}</span><button class="btn-merge" onclick={() => fixPlaylist(issue)} disabled={cleanupWorking}>Renumber</button></div>
        {/each}
      </section>
      <section class="cleanup-card">
        <h2>AI cleanup log <span>{cleanupLog.length}</span></h2>
        <p>Metadata changes captured during SoundCloud AI cleanup.</p>
        {#if cleanupLog.length === 0}<p class="status">No cleanup changes recorded yet.</p>{/if}
        {#each cleanupLog as entry (entry.id)}
          <div class="log-row"><strong>{entry.before_title}</strong> → <strong>{entry.after_title}</strong><br /><span>{entry.before_artists.join(', ') || '—'} → {entry.after_artists.join(', ') || '—'} · {entry.created_at ?? 'pending timestamp'}</span></div>
        {/each}
      </section>
      <section class="cleanup-card">
        <h2>Covers &amp; icons</h2>
        <p>Fetch missing images from existing references. This retains the established best-effort behavior.</p>
        <div class="group-actions"><button class="btn-refresh" onclick={() => fetchImages('icons')} disabled={iconsFetching}>{iconsFetching ? 'Fetching icons…' : 'Fetch artist icons'}</button><button class="btn-refresh" onclick={() => fetchImages('covers')} disabled={coversFetching}>{coversFetching ? 'Fetching covers…' : 'Fetch album covers'}</button></div>
      </section>
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
  .btn-delete-reference { padding: 0.35rem 0.8rem; border: 1px solid color-mix(in srgb, #e05252 55%, var(--border)); border-radius: 6px; background: transparent; color: #e05252; cursor: pointer; font: inherit; font-size: 0.82rem; }
  .btn-delete-reference:hover { background: color-mix(in srgb, #e05252 10%, transparent); }
  .btn-toggle-verified { padding: 0.35rem 0.8rem; border: 1px solid var(--border); border-radius: 6px; background: transparent; color: var(--muted); cursor: pointer; font: inherit; font-size: 0.82rem; }
  .btn-toggle-verified:hover,.btn-toggle-verified.active { background: var(--surface-2); color: var(--text); }
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
  .remote-result.status-mismatch { border-left: 3px solid #e05252; }
  .remote-result.status-ok { border-left: 3px solid #22c55e; }
  .remote-result.status-unreachable,.remote-result.status-unsupported { border-left: 3px solid #f59e0b; }
  .remote-result.status-dismissed { border-left: 3px solid #6b7280; opacity: 0.75; }
  .finding-header { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.35rem; }
  .finding-kind { font-weight: 700; font-size: 0.85rem; }
  .audit-status { border-radius: 999px; background: var(--surface-2); padding: 0.1rem 0.5rem; color: var(--muted); font-size: 0.7rem; }
  .finding-platform { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); background: var(--surface-2); border-radius: 4px; padding: 0.1rem 0.4rem; }
  .finding-message { margin: 0 0 0.5rem; font-size: 0.85rem; }
  .finding-entities { display: flex; flex-wrap: wrap; gap: 0.4rem; }
  .entity-chip { font-size: 0.75rem; background: var(--surface-2); border: 1px solid var(--border); border-radius: 999px; padding: 0.1rem 0.6rem; color: var(--muted); }
  .remote-comparison { display: grid; gap: 0.3rem; font-size: 0.82rem; overflow-wrap: anywhere; }
  .audit-url { display: inline-block; margin-top: 0.45rem; font-size: 0.75rem; color: var(--accent); }
  .task-notice { padding: 0.55rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; color: var(--muted); font-size: 0.8rem; }
  .cleanup-card { border: 1px solid var(--border); border-radius: 8px; background: var(--surface); padding: 0.9rem 1rem; margin-bottom: 0.75rem; }
  .cleanup-card h2 { font-size: 0.95rem; margin: 0 0 0.3rem; }.cleanup-card h2 span { color: var(--muted); font-weight: 500; }.cleanup-card p { color: var(--muted); font-size: 0.8rem; margin: 0 0 0.65rem; }.chip-list { display: flex; flex-wrap: wrap; gap: 0.4rem; margin-bottom: 0.65rem; }.cleanup-row,.log-row { padding: 0.6rem 0; border-top: 1px solid var(--border); font-size: 0.8rem; }.cleanup-row { display: flex; justify-content: space-between; align-items: center; gap: 0.75rem; }.log-row span { color: var(--muted); font-size: 0.75rem; }
  .audit-summary { display: flex; flex-wrap: wrap; align-items: center; gap: 0.55rem; margin: 0.25rem 0 0.25rem; }
  .summary-title { font-weight: 700; font-size: 0.85rem; margin-right: 0.2rem; }
  .summary-item { border: 1px solid transparent; border-radius: 999px; padding: 0.12rem 0.55rem; font-size: 0.72rem; background: var(--surface-2); color: var(--muted); cursor: pointer; font: inherit; }
  .summary-item:hover,.summary-item.active { border-color: var(--border); color: var(--text); }
  .summary-item.mismatch { color: #e05252; }
  .summary-item.unreachable,.summary-item.unsupported { color: #f59e0b; }
  .summary-item.missing { color: #a78bfa; }
  .summary-item.verified { color: #22c55e; }
  .summary-item.dismissed { color: var(--muted); }
  .summary-item.all-filter { color: var(--text); }
  .remote-filters { display: grid; grid-template-columns: minmax(220px, 1fr) 170px 150px; gap: 0.55rem; margin: 0.65rem 0 0.9rem; }
  .remote-search,.remote-filters select { min-width: 0; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-2); color: var(--text); padding: 0.42rem 0.6rem; font: inherit; font-size: 0.8rem; }
  .remote-search:focus,.remote-filters select:focus { outline: 1px solid var(--accent); }
  .audit-actions { justify-content: flex-start; margin-top: 0.55rem; border-top: 0; padding: 0; flex-wrap: wrap; }
  @media (max-width: 640px) { .data-quality { padding: 1rem; } .entity-tabs { flex-wrap: wrap; } .toolbar-actions { margin-left: 0; } .group-header { align-items: start; flex-direction: column; } .remote-filters { grid-template-columns: 1fr; } }
</style>
