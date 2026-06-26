<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getSyncSchedules,
    createSyncSchedule,
    updateSyncSchedule,
    deleteSyncSchedule,
    triggerSyncSchedule,
  } from '../lib/api';
  import type { SyncScheduleDto } from '../lib/api';

  let schedules: SyncScheduleDto[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);

  // Create form
  let newUrl = $state('');
  let newLabel = $state('');
  let newScheduleType = $state<'interval' | 'cron'>('interval');
  let newIntervalHours = $state(1);
  let newCronExpression = $state('0 12 * * *');
  let creating = $state(false);
  let createError: string | null = $state(null);

  // Trigger feedback
  let triggeringId: number | null = $state(null);
  let triggerMsg: string | null = $state(null);

  async function load() {
    loading = true;
    error = null;
    try {
      schedules = await getSyncSchedules();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function handleCreate(e: SubmitEvent) {
    e.preventDefault();
    if (!newUrl.trim()) return;
    creating = true;
    createError = null;
    try {
      const body: any = {
        playlist_url: newUrl.trim(),
        label: newLabel.trim() || null,
      };
      
      if (newScheduleType === 'interval') {
        body.interval_hours = newIntervalHours;
      } else {
        body.cron_expression = newCronExpression;
      }
      
      await createSyncSchedule(body);
      newUrl = '';
      newLabel = '';
      newIntervalHours = 1;
      newCronExpression = '0 12 * * *';
      newScheduleType = 'interval';
      await load();
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
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleDelete(id: number) {
    if (!confirm('Delete this sync schedule?')) return;
    try {
      await deleteSyncSchedule(id);
      schedules = schedules.filter((s) => s.id !== id);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleTrigger(id: number) {
    triggeringId = id;
    triggerMsg = null;
    try {
      const res = await triggerSyncSchedule(id);
      triggerMsg = `Sync started (task #${res.task_id})`;
      await load();
    } catch (e: unknown) {
      triggerMsg = e instanceof Error ? e.message : String(e);
    } finally {
      triggeringId = null;
      setTimeout(() => (triggerMsg = null), 5000);
    }
  }

  function formatInterval(s: number): string {
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m`;
    if (s < 86400) return `${Math.floor(s / 3600)}h`;
    return `${Math.floor(s / 86400)}d`;
  }

  function formatDate(dt: string | null): string {
    if (!dt) return '—';
    const d = new Date(dt.replace(' ', 'T'));
    return d.toLocaleString();
  }

  function formatSchedule(schedule: SyncScheduleDto): string {
    if (schedule.interval_hours !== null && schedule.interval_hours !== undefined) {
      const h = schedule.interval_hours;
      if (h < 1) return `every ${Math.round(h * 60)}m`;
      if (h === 1) return 'every hour';
      if (h === Math.floor(h)) return `every ${Math.floor(h)}h`;
      return `every ${h}h`;
    }
    return `cron: ${schedule.cron_expression || '?'}`;
  }
</script>

<div class="sync-page">
  <h2>Sync Schedules</h2>
  <p class="subtitle">Define playlists to synchronize automatically using intervals or cron expressions.</p>

  <!-- Create form -->
  <section class="create-section">
    <h3>Add a schedule</h3>
    <form class="create-form" onsubmit={handleCreate}>
      <div class="form-row">
        <input
          type="url"
          placeholder="Playlist URL (Spotify, SoundCloud, YouTube…)"
          bind:value={newUrl}
          disabled={creating}
          required
        />
      </div>
      <div class="form-row form-row--split">
        <input
          type="text"
          placeholder="Label (optional)"
          bind:value={newLabel}
          disabled={creating}
        />
        <div class="schedule-type-toggle">
          <button
            type="button"
            class="toggle-btn"
            class:active={newScheduleType === 'interval'}
            disabled={creating}
            onclick={() => (newScheduleType = 'interval')}
          >
            Interval
          </button>
          <button
            type="button"
            class="toggle-btn"
            class:active={newScheduleType === 'cron'}
            disabled={creating}
            onclick={() => (newScheduleType = 'cron')}
          >
            Cron
          </button>
        </div>
      </div>

      {#if newScheduleType === 'interval'}
        <div class="form-row form-row--split">
          <div class="interval-group">
            <input
              type="number"
              min="0.25"
              step="0.25"
              bind:value={newIntervalHours}
              disabled={creating}
            />
            <span class="interval-hint">hours</span>
          </div>
          <button type="submit" disabled={creating || !newUrl.trim()}>
            {#if creating}<span class="spinner"></span> Adding…{:else}Add{/if}
          </button>
        </div>
      {:else}
        <div class="form-row form-row--split">
          <input
            type="text"
            placeholder="Cron expression (e.g. '0 12 * * *' for daily at noon)"
            bind:value={newCronExpression}
            disabled={creating}
          />
          <button type="submit" disabled={creating || !newUrl.trim()}>
            {#if creating}<span class="spinner"></span> Adding…{:else}Add{/if}
          </button>
        </div>
      {/if}

      {#if createError}
        <p class="feedback error">{createError}</p>
      {/if}
    </form>
  </section>

  {#if triggerMsg}
    <div class="feedback info">{triggerMsg}</div>
  {/if}

  {#if error}
    <div class="feedback error">{error}</div>
  {/if}

  <!-- Schedule list -->
  {#if loading}
    <p class="empty">Loading…</p>
  {:else if schedules.length === 0}
    <p class="empty">No schedules yet.</p>
  {:else}
    <ul class="schedule-list">
      {#each schedules as schedule (schedule.id)}
        <li class="schedule-card" class:disabled={!schedule.enabled}>
          <div class="schedule-header">
            <div class="schedule-info">
              <span class="schedule-label">{schedule.label ?? schedule.playlist_url}</span>
              {#if schedule.label}
                <span class="schedule-url">{schedule.playlist_url}</span>
              {/if}
            </div>
            <div class="schedule-meta">
              <span class="interval-badge">{formatSchedule(schedule)}</span>
              <span class="status-badge" class:enabled={schedule.enabled} class:paused={!schedule.enabled}>
                {schedule.enabled ? 'Active' : 'Paused'}
              </span>
            </div>
          </div>

          <div class="schedule-dates">
            <span>Last run: {formatDate(schedule.last_run)}</span>
            <span>Next run: {formatDate(schedule.next_run)}</span>
          </div>

          <div class="schedule-actions">
            <button class="btn-secondary" onclick={() => toggleEnabled(schedule)}>
              {schedule.enabled ? 'Pause' : 'Resume'}
            </button>
            <button
              class="btn-primary"
              disabled={triggeringId === schedule.id}
              onclick={() => handleTrigger(schedule.id)}
            >
              {#if triggeringId === schedule.id}
                <span class="spinner"></span> Syncing…
              {:else}
                Sync now
              {/if}
            </button>
            <button class="btn-danger" onclick={() => handleDelete(schedule.id)}>Delete</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .sync-page {
    max-width: 820px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  h2 {
    font-size: 1.5rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: var(--color-muted, #888);
    margin-bottom: 2rem;
    font-size: 0.9rem;
  }

  .create-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
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
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .form-row input[type="url"],
  .form-row input[type="text"] {
    flex: 1;
    padding: 0.55rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.9rem;
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

  .schedule-type-toggle {
    display: flex;
    gap: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .toggle-btn {
    flex: 1;
    padding: 0.55rem 0.75rem;
    border: none;
    background: var(--surface);
    color: var(--color-muted, #888);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-btn:not(:last-child) {
    border-right: 1px solid var(--border);
  }

  .toggle-btn.active {
    background: var(--color-accent, #7c6af7);
    color: #fff;
  }

  .toggle-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .interval-group {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .interval-group input[type="number"] {
    width: 80px;
    padding: 0.55rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
  }

  .interval-group input[type="number"]:focus {
    border-color: var(--accent);
  }

  .interval-group input[type="number"]:disabled {
    opacity: 0.5;
  }

  .interval-hint {
    font-size: 0.8rem;
    color: var(--color-muted, #888);
    white-space: nowrap;
  }

  button {
    padding: 0.55rem 1.1rem;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  button[type="submit"],
  .btn-primary {
    background: var(--color-accent, #7c6af7);
    color: #fff;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--color-surface2, #2a2a2a);
    color: inherit;
  }

  .btn-danger {
    background: transparent;
    color: var(--color-error, #e55);
    border: 1px solid var(--color-error, #e55);
  }

  .feedback {
    padding: 0.6rem 0.9rem;
    border-radius: 6px;
    font-size: 0.9rem;
    margin-bottom: 1rem;
  }

  .feedback.error {
    background: rgba(220, 50, 50, 0.12);
    color: var(--color-error, #e55);
  }

  .feedback.info {
    background: rgba(100, 200, 120, 0.12);
    color: #6dc87a;
  }

  .empty {
    color: var(--color-muted, #888);
    text-align: center;
    padding: 3rem 0;
  }

  .schedule-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .schedule-card {
    border: 1px solid var(--color-border, #333);
    border-radius: 10px;
    padding: 1rem 1.2rem;
    background: var(--color-surface, #1a1a1a);
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
    font-size: 0.95rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .schedule-url {
    font-size: 0.78rem;
    color: var(--color-muted, #888);
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

  .interval-badge {
    font-size: 0.78rem;
    padding: 0.2rem 0.55rem;
    border-radius: 20px;
    background: var(--color-surface2, #2a2a2a);
    color: var(--color-muted, #aaa);
  }

  .status-badge {
    font-size: 0.78rem;
    padding: 0.2rem 0.55rem;
    border-radius: 20px;
    font-weight: 600;
  }

  .status-badge.enabled {
    background: rgba(100, 200, 120, 0.15);
    color: #6dc87a;
  }

  .status-badge.paused {
    background: rgba(180, 180, 180, 0.1);
    color: #aaa;
  }

  .schedule-dates {
    display: flex;
    gap: 1.5rem;
    font-size: 0.8rem;
    color: var(--color-muted, #888);
  }

  .schedule-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
