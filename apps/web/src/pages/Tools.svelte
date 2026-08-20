<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getStorageStats,
    getSyncSchedules,
    createSyncSchedule,
    updateSyncSchedule,
    deleteSyncSchedule,
    triggerSyncSchedule,
    getSyncSettings,
    updateSyncSettings,
    triggerAllSyncSchedules,
  } from '../lib/api';
  import type { StorageStatsDto, SyncScheduleDto, SyncSettingsDto } from '../lib/api';

  // ── Tab ────────────────────────────────────────────────────────────────────
  type Tab = 'storage' | 'sync';
  let activeTab: Tab = $state('sync');

  // ── Storage ─────────────────────────────────────────────────────────────────
  let stats: StorageStatsDto | null = $state(null);
  let storageLoading = $state(true);
  let storageError: string | null = $state(null);

  async function loadStorage() {
    storageLoading = true;
    storageError = null;
    try {
      stats = await getStorageStats();
    } catch (err: unknown) {
      storageError = err instanceof Error ? err.message : String(err);
    } finally {
      storageLoading = false;
    }
  }

  function formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIdx = 0;
    while (size >= 1024 && unitIdx < units.length - 1) {
      size /= 1024;
      unitIdx += 1;
    }
    if (unitIdx === 0) return `${bytes} ${units[0]}`;
    return `${size.toFixed(1)} ${units[unitIdx]}`;
  }

  // ── Sync: global cron settings ───────────────────────────────────────────────
  let settings: SyncSettingsDto | null = $state(null);
  let settingsLoading = $state(true);
  let settingsError: string | null = $state(null);
  let savingSettings = $state(false);
  let runningAll = $state(false);
  let runAllMsg: string | null = $state(null);

  // Editable draft, only pushed to the server on submit
  let cronDraft = $state('0 3 * * *');
  let enabledDraft = $state(true);

  async function loadSettings() {
    settingsLoading = true;
    settingsError = null;
    try {
      settings = await getSyncSettings();
      cronDraft = settings.cron_expression;
      enabledDraft = settings.enabled;
    } catch (e: unknown) {
      settingsError = e instanceof Error ? e.message : String(e);
    } finally {
      settingsLoading = false;
    }
  }

  async function handleSaveSettings(e: SubmitEvent) {
    e.preventDefault();
    savingSettings = true;
    settingsError = null;
    try {
      settings = await updateSyncSettings({ cron_expression: cronDraft.trim(), enabled: enabledDraft });
    } catch (e: unknown) {
      settingsError = e instanceof Error ? e.message : String(e);
    } finally {
      savingSettings = false;
    }
  }

  async function handleRunAll() {
    runningAll = true;
    runAllMsg = null;
    try {
      const res = await triggerAllSyncSchedules();
      runAllMsg = `Started ${res.task_ids.length} sync task(s)`;
      await loadSettings();
      await loadSync();
    } catch (e: unknown) {
      runAllMsg = e instanceof Error ? e.message : String(e);
    } finally {
      runningAll = false;
      setTimeout(() => (runAllMsg = null), 5000);
    }
  }

  // ── Sync: subscriptions ──────────────────────────────────────────────────────
  let schedules: SyncScheduleDto[] = $state([]);
  let syncLoading = $state(true);
  let syncError: string | null = $state(null);

  // Add-link form (manual, secondary path — type is auto-detected server-side)
  let newUrl = $state('');
  let newLabel = $state('');
  let creating = $state(false);
  let createError: string | null = $state(null);

  let triggeringId: number | null = $state(null);
  let triggerMsg: string | null = $state(null);

  async function loadSync() {
    syncLoading = true;
    syncError = null;
    try {
      schedules = await getSyncSchedules();
    } catch (e: unknown) {
      syncError = e instanceof Error ? e.message : String(e);
    } finally {
      syncLoading = false;
    }
  }

  async function handleCreate(e: SubmitEvent) {
    e.preventDefault();
    if (!newUrl.trim()) return;
    creating = true;
    createError = null;
    try {
      await createSyncSchedule({ url: newUrl.trim(), label: newLabel.trim() || null });
      newUrl = '';
      newLabel = '';
      await loadSync();
    } catch (e: unknown) {
      createError = e instanceof Error ? e.message : String(e);
    } finally {
      creating = false;
    }
  }

  async function toggleEnabled(schedule: SyncScheduleDto) {
    try {
      const updated = await updateSyncSchedule(schedule.id, { enabled: !schedule.enabled });
      schedules = schedules.map((s) => (s.id === schedule.id ? updated : s));
    } catch (e: unknown) {
      syncError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleDelete(id: number) {
    if (!confirm('Remove this subscription from scheduled sync?')) return;
    try {
      await deleteSyncSchedule(id);
      schedules = schedules.filter((s) => s.id !== id);
    } catch (e: unknown) {
      syncError = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleTrigger(id: number) {
    triggeringId = id;
    triggerMsg = null;
    try {
      const res = await triggerSyncSchedule(id);
      triggerMsg = `Sync started (task #${res.task_id})`;
      await loadSync();
    } catch (e: unknown) {
      triggerMsg = e instanceof Error ? e.message : String(e);
    } finally {
      triggeringId = null;
      setTimeout(() => (triggerMsg = null), 5000);
    }
  }

  function formatDate(dt: string | null): string {
    if (!dt) return '—';
    const d = new Date(dt.replace(' ', 'T'));
    return d.toLocaleString();
  }

  onMount(() => {
    loadStorage();
    loadSettings();
    loadSync();
  });

  function switchTab(tab: Tab) {
    activeTab = tab;
  }
</script>

<div class="tools-page">
  <div class="page-header">
    <h1>Tools</h1>
  </div>

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'sync'} onclick={() => switchTab('sync')}>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
      </svg>
      Sync
    </button>
    <button class="tab" class:active={activeTab === 'storage'} onclick={() => switchTab('storage')}>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <ellipse cx="12" cy="5" rx="9" ry="3"/>
        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
      </svg>
      Storage
    </button>
  </div>

  <!-- ── Storage tab ─────────────────────────────────────────────────────────── -->
  {#if activeTab === 'storage'}
    <div class="tab-content">
      <div class="section-toolbar">
        <button class="btn-header" onclick={loadStorage} disabled={storageLoading}>
          {storageLoading ? 'Loading…' : 'Refresh'}
        </button>
      </div>

      {#if storageError}
        <div class="feedback error"><strong>Error:</strong> {storageError}</div>
      {/if}

      {#if storageLoading}
        <div class="status">Loading storage statistics…</div>
      {:else if stats}
        <div class="stats-header">
          <div class="total-size">
            <span class="stat-label">Library Size</span>
            <span class="stat-value">{stats.total_formatted}</span>
            <span class="stat-bytes">({stats.total_bytes.toLocaleString()} bytes)</span>
          </div>
          <div class="stat-aside">{stats.artists.length} artists</div>
        </div>

        {#if stats.artists.length === 0}
          <p class="status">No artists or storage data available.</p>
        {:else}
          <ul class="artists-list">
            {#each stats.artists as artist (artist.id)}
              <li class="artist-row">
                <div class="artist-name">{artist.name}</div>
                <div class="artist-bar">
                  <div class="bar-track">
                    <div
                      class="bar-fill"
                      style="width: {Math.max(artist.percent, 2)}%"
                      title="{artist.name}: {artist.percent.toFixed(1)}% ({formatBytes(artist.bytes)})"
                    ></div>
                  </div>
                </div>
                <div class="artist-meta">
                  <span class="meta-pct">{artist.percent.toFixed(1)}%</span>
                  <span class="meta-size">{formatBytes(artist.bytes)}</span>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- ── Sync tab ────────────────────────────────────────────────────────────── -->
  {#if activeTab === 'sync'}
    <div class="tab-content">
      <p class="sync-subtitle">
        Everything below is synchronized in a single pass on the schedule configured here.
        Subscribe artists and playlists from their own pages for the quickest path — this page
        also lets you add a link manually and fine-tune the global schedule.
      </p>

      <section class="create-section">
        <h3>Global schedule</h3>
        {#if settingsError}
          <p class="feedback error">{settingsError}</p>
        {/if}
        {#if settingsLoading}
          <p class="status">Loading…</p>
        {:else if settings}
          <form class="create-form" onsubmit={handleSaveSettings}>
            <div class="form-row form-row--split">
              <input
                type="text"
                placeholder="Cron expression (e.g. '0 3 * * *' for daily at 3am)"
                bind:value={cronDraft}
                disabled={savingSettings}
              />
              <label class="enabled-toggle">
                <input type="checkbox" bind:checked={enabledDraft} disabled={savingSettings} />
                Enabled
              </label>
              <button type="submit" class="btn-accent" disabled={savingSettings || !cronDraft.trim()}>
                {#if savingSettings}<span class="spinner"></span> Saving…{:else}Save{/if}
              </button>
            </div>
          </form>
          <div class="schedule-dates">
            <span>Last run: {formatDate(settings.last_run)}</span>
            <span>Next run: {formatDate(settings.next_run)}</span>
          </div>
          <div class="section-toolbar">
            <button class="btn-secondary" disabled={runningAll} onclick={handleRunAll}>
              {#if runningAll}<span class="spinner"></span> Running…{:else}Run now{/if}
            </button>
          </div>
          {#if runAllMsg}
            <div class="feedback info">{runAllMsg}</div>
          {/if}
        {/if}
      </section>

      <section class="create-section">
        <h3>Add a link manually</h3>
        <form class="create-form" onsubmit={handleCreate}>
          <div class="form-row form-row--split">
            <input
              type="url"
              placeholder="Artist or playlist URL (Spotify, SoundCloud, YouTube…)"
              bind:value={newUrl}
              disabled={creating}
              required
            />
            <input
              type="text"
              placeholder="Label (optional)"
              bind:value={newLabel}
              disabled={creating}
            />
            <button type="submit" class="btn-accent" disabled={creating || !newUrl.trim()}>
              {#if creating}<span class="spinner"></span> Adding…{:else}Add{/if}
            </button>
          </div>

          {#if createError}
            <p class="feedback error">{createError}</p>
          {/if}
        </form>
      </section>

      {#if triggerMsg}
        <div class="feedback info">{triggerMsg}</div>
      {/if}
      {#if syncError}
        <div class="feedback error">{syncError}</div>
      {/if}

      {#if syncLoading}
        <p class="status">Loading…</p>
      {:else if schedules.length === 0}
        <p class="status">No subscriptions yet.</p>
      {:else}
        <ul class="schedule-list">
          {#each schedules as schedule (schedule.id)}
            <li class="schedule-card" class:disabled={!schedule.enabled}>
              <div class="schedule-header">
                <div class="schedule-info">
                  <span class="schedule-label">{schedule.label ?? schedule.url}</span>
                  {#if schedule.label}
                    <span class="schedule-url">{schedule.url}</span>
                  {/if}
                </div>
                <div class="schedule-meta">
                  <span class="type-badge">{schedule.entity_type}</span>
                  <span class="status-badge" class:enabled={schedule.enabled} class:paused={!schedule.enabled}>
                    {schedule.enabled ? 'Active' : 'Paused'}
                  </span>
                </div>
              </div>
              <div class="schedule-dates">
                <span>Last run: {formatDate(schedule.last_run)}</span>
              </div>
              <div class="schedule-actions">
                <button class="btn-secondary" onclick={() => toggleEnabled(schedule)}>
                  {schedule.enabled ? 'Pause' : 'Resume'}
                </button>
                <button class="btn-accent" disabled={triggeringId === schedule.id} onclick={() => handleTrigger(schedule.id)}>
                  {#if triggeringId === schedule.id}
                    <span class="spinner"></span> Syncing…
                  {:else}
                    Sync now
                  {/if}
                </button>
                <button class="btn-danger" onclick={() => handleDelete(schedule.id)}>Remove</button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tools-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .page-header {
    margin-bottom: 1.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
  }

  /* ── Tabs ──────────────────────────────────────────────────────────────────── */
  .tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
    margin-bottom: 1.75rem;
  }

  .tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--muted);
    font-size: 0.875rem;
    font-family: inherit;
    padding: 0.5rem 1.1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: -1px;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: var(--text);
  }

  .tab.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }

  /* ── Tab content ───────────────────────────────────────────────────────────── */
  .tab-content {
    animation: fadein 0.12s ease;
  }

  @keyframes fadein {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .section-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 1.25rem;
  }

  /* ── Storage ───────────────────────────────────────────────────────────────── */
  .stats-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 1.5rem;
    background: var(--surface);
    border-radius: 8px;
    border: 1px solid var(--border);
    margin-bottom: 1.5rem;
  }

  .total-size {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .stat-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted);
    font-weight: 600;
  }

  .stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text);
  }

  .stat-bytes {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .stat-aside {
    font-size: 0.875rem;
    color: var(--muted);
  }

  .artists-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .artist-row {
    display: grid;
    grid-template-columns: 200px 1fr 120px;
    gap: 1rem;
    align-items: center;
    padding: 0.65rem 0.9rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    transition: background 0.1s;
  }

  .artist-row:hover {
    background: var(--surface);
  }

  .artist-name {
    font-weight: 500;
    font-size: 0.875rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artist-bar {
    display: flex;
    align-items: center;
  }

  .bar-track {
    flex: 1;
    height: 5px;
    background: var(--surface-2);
    border-radius: 3px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .artist-meta {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    align-items: center;
  }

  .meta-pct {
    font-size: 0.78rem;
    color: var(--muted);
    min-width: 38px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .meta-size {
    font-size: 0.82rem;
    color: var(--text);
    font-weight: 500;
    min-width: 50px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  /* ── Sync ──────────────────────────────────────────────────────────────────── */
  .sync-subtitle {
    color: var(--muted);
    margin: 0 0 1.5rem;
    font-size: 0.875rem;
  }

  .create-section h3 {
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    margin: 0 0 0.75rem;
  }

  .create-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 2rem;
  }

  .form-row {
    display: flex;
  }

  .form-row--split {
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .form-row input[type="url"],
  .form-row input[type="text"] {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
  }

  .form-row input[type="url"]:focus,
  .form-row input[type="text"]:focus {
    border-color: var(--accent);
  }

  .form-row input:disabled {
    opacity: 0.5;
  }

  .enabled-toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.875rem;
    color: var(--text);
    white-space: nowrap;
  }

  .schedule-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .schedule-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1rem 1.1rem;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .schedule-card.disabled {
    opacity: 0.6;
  }

  .schedule-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
  }

  .schedule-info {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .schedule-label {
    font-weight: 600;
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .schedule-url {
    font-size: 0.75rem;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .schedule-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .type-badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 20px;
    background: var(--surface-2);
    color: var(--muted);
    text-transform: capitalize;
  }

  .status-badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 20px;
    font-weight: 600;
  }

  .status-badge.enabled {
    background: rgba(100, 200, 120, 0.15);
    color: #6dc87a;
  }

  .status-badge.paused {
    background: rgba(180, 180, 180, 0.08);
    color: var(--muted);
  }

  .schedule-dates {
    display: flex;
    gap: 1.5rem;
    font-size: 0.78rem;
    color: var(--muted);
  }

  .schedule-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  /* ── Shared buttons ────────────────────────────────────────────────────────── */
  .btn-header {
    padding: 0.4rem 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-header:hover:not(:disabled) {
    background: var(--surface-2);
  }

  .btn-header:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-accent {
    padding: 0.5rem 1.1rem;
    border-radius: 6px;
    border: none;
    background: var(--accent);
    color: #fff;
    font-size: 0.875rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: opacity 0.15s;
  }

  .btn-accent:hover:not(:disabled) {
    opacity: 0.88;
  }

  .btn-accent:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 0.5rem 1.1rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    font-size: 0.875rem;
    font-family: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: background 0.15s;
  }

  .btn-secondary:hover {
    background: var(--surface);
  }

  .btn-danger {
    padding: 0.5rem 1.1rem;
    border-radius: 6px;
    border: 1px solid var(--error);
    background: transparent;
    color: var(--error);
    font-size: 0.875rem;
    font-family: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    transition: background 0.15s;
  }

  .btn-danger:hover {
    background: color-mix(in srgb, var(--error) 12%, transparent);
  }

  /* ── Feedback ──────────────────────────────────────────────────────────────── */
  .feedback {
    padding: 0.6rem 0.9rem;
    border-radius: 6px;
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }

  .feedback.error {
    background: color-mix(in srgb, var(--error) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--error) 35%, transparent);
    color: var(--error);
  }

  .feedback.info {
    background: rgba(100, 200, 120, 0.12);
    color: #6dc87a;
  }

  /* ── Misc ──────────────────────────────────────────────────────────────────── */
  .status {
    text-align: center;
    color: var(--muted);
    padding: 2.5rem 0;
    font-size: 0.875rem;
  }

  .spinner {
    display: inline-block;
    width: 11px;
    height: 11px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 768px) {
    .stats-header {
      flex-direction: column;
      gap: 1rem;
    }

    .artist-row {
      grid-template-columns: 1fr;
      gap: 0.4rem;
    }

    .artist-meta {
      justify-content: space-between;
    }
  }
</style>
