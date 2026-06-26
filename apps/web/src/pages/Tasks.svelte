<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getTasks, retryTask, cancelTask } from '../lib/api';
  import type { TaskDto } from '../lib/types';

  interface Props {
    onNavigateValidations?: () => void;
  }
  let { onNavigateValidations }: Props = $props();

  let tasks: TaskDto[] = $state([]);
  let loading = $state(true);
  let retrying: Set<number> = $state(new Set());
  let cancelling: Set<number> = $state(new Set());
  // Track which task error lists are expanded
  let expandedErrors: Set<number> = $state(new Set());
  let expandedValidations: Set<number> = $state(new Set());
  let interval: ReturnType<typeof setInterval>;

  async function refresh() {
    try {
      tasks = await getTasks();
      // Clear cancelling state for tasks that have transitioned to Cancelled/Cancelling
      if (cancelling.size > 0) {
        const resolved = tasks
          .filter((t) => cancelling.has(t.id) && (t.status === 'Cancelled' || t.status === 'Cancelling'))
          .map((t) => t.id);
        if (resolved.length > 0) {
          cancelling = new Set([...cancelling].filter((id) => !resolved.includes(id)));
        }
      }
    } catch {
      // silent
    } finally {
      loading = false;
    }
  }

  async function handleRetry(task: TaskDto) {
    retrying = new Set([...retrying, task.id]);
    try {
      await retryTask(task.id);
      await refresh();
     } catch (e) {
       alert(`Retry failed: ${e instanceof Error ? e.message : e}`);
     } finally {
      retrying = new Set([...retrying].filter((id) => id !== task.id));
    }
  }

  async function handleCancel(task: TaskDto) {
    cancelling = new Set([...cancelling, task.id]);
    try {
      await cancelTask(task.id);
      await refresh();
     } catch (e) {
       alert(`Cancel failed: ${e instanceof Error ? e.message : e}`);
       cancelling = new Set([...cancelling].filter((id) => id !== task.id));
    }
    // Keep in cancelling state until task status reflects cancellation (handled in refresh)
  }

  function toggleErrors(taskId: number) {
    if (expandedErrors.has(taskId)) {
      expandedErrors = new Set([...expandedErrors].filter((id) => id !== taskId));
    } else {
      expandedErrors = new Set([...expandedErrors, taskId]);
    }
  }

  function toggleValidations(taskId: number) {
    if (expandedValidations.has(taskId)) {
      expandedValidations = new Set([...expandedValidations].filter((id) => id !== taskId));
    } else {
      expandedValidations = new Set([...expandedValidations, taskId]);
    }
  }

  onMount(() => {
    refresh();
    interval = setInterval(refresh, 3_000);
  });

  onDestroy(() => clearInterval(interval));

  function statusLabel(status: TaskDto['status']) {
    return { Pending: 'Pending', Running: 'Running', Completed: 'Completed', Failed: 'Failed', Cancelled: 'Cancelled', Cancelling: 'Cancelling…' }[status] ?? status;
  }

  function statusClass(status: TaskDto['status']) {
    return { Pending: 'pending', Running: 'running', Completed: 'completed', Failed: 'failed', Cancelled: 'cancelled', Cancelling: 'cancelling' }[status] ?? '';
  }

  function progressPercent(task: TaskDto) {
    if (!task.total || task.total === 0) return 0;
    return Math.round((task.progress / task.total) * 100);
  }

  function taskLabel(task: TaskDto) {
    if (task.label) return task.label;
    if (task.task_type === 'SyncPlaylist') return 'Sync playlist';
    if (task.task_type === 'SyncArtist') return 'Sync artist';
    if (task.task_type === 'SyncAlbum') return 'Sync album';
    return 'Download track';
  }

  function canRetry(status: TaskDto['status']) {
    return status === 'Pending' || status === 'Failed' || status === 'Running' || status === 'Cancelled';
  }

  function canCancel(status: TaskDto['status']) {
    return status === 'Running' || status === 'Pending';
  }

  function hasStats(task: TaskDto) {
    return task.stats != null && (
      task.stats.downloaded > 0 ||
      task.stats.to_validate > 0 ||
      task.stats.skipped > 0 ||
      task.stats.errors.length > 0
    );
  }

  function reasonLabel(reason: string | null): string {
    if (reason === 'soundcloud_drm_protected') return 'DRM protected';
    if (reason === 'metadata_partial_match') return 'partial metadata match';
    if (reason === 'metadata_no_match') return 'no metadata match';
    return reason ?? 'needs review';
  }
</script>

<div class="tasks-page">
  <h2>Tasks</h2>

  {#if loading}
    <p class="empty">Loading…</p>
  {:else if tasks.length === 0}
    <p class="empty">No tasks.</p>
  {:else}
    <ul class="task-list">
      {#each tasks as task (task.id)}
        <li class="task-card">
          <div class="task-header">
            <span class="task-label">{taskLabel(task)}</span>
            <div class="task-header-right">
              {#if canCancel(task.status)}
                <button
                  class="cancel-btn"
                  disabled={cancelling.has(task.id)}
                  onclick={() => handleCancel(task)}
                >
                  {#if cancelling.has(task.id)}
                    <span class="spinner"></span> Cancelling…
                  {:else}
                    Cancel
                  {/if}
                </button>
              {/if}
              {#if canRetry(task.status)}
                <button
                  class="retry-btn"
                  disabled={retrying.has(task.id)}
                  onclick={() => handleRetry(task)}
                >
                  {retrying.has(task.id) ? '…' : 'Retry'}
                </button>
              {/if}
              <span class="status-badge {statusClass(task.status)}">{statusLabel(task.status)}</span>
            </div>
          </div>

          {#if task.status === 'Running' || task.status === 'Completed' || task.status === 'Cancelled'}
            <div class="progress-row">
              <div class="progress-bar">
                <div
                  class="progress-fill {statusClass(task.status)}"
                  style="width: {progressPercent(task)}%"
                ></div>
              </div>
              <span class="progress-text">
                {task.progress}{task.total != null ? ` / ${task.total}` : ''}
              </span>
            </div>
          {/if}

          {#if hasStats(task)}
            <div class="stats-row">
              {#if task.stats!.downloaded > 0}
                <span class="stat-chip downloaded">✓ {task.stats!.downloaded} downloaded</span>
              {/if}
              {#if task.stats!.to_validate > 0}
                {#if task.stats!.to_validate_tracks && task.stats!.to_validate_tracks.length > 0}
                  <button
                    class="stat-chip to-validate"
                    onclick={() => toggleValidations(task.id)}
                    title="View tracks pending validation"
                  >
                    ⚠ {task.stats!.to_validate} pending validation
                    <span class="chevron">{expandedValidations.has(task.id) ? '▲' : '▼'}</span>
                  </button>
                {:else}
                  <button
                    class="stat-chip to-validate"
                    onclick={() => onNavigateValidations?.()}
                    title="Go to Validations"
                  >
                    ⚠ {task.stats!.to_validate} pending validation
                  </button>
                {/if}
              {/if}
              {#if task.stats!.skipped > 0}
                <span class="stat-chip skipped">↩ {task.stats!.skipped} skipped</span>
              {/if}
              {#if task.stats!.errors.length > 0}
                <button
                  class="stat-chip errors"
                  onclick={() => toggleErrors(task.id)}
                  title="View error details"
                >
                  ✕ {task.stats!.errors.length} error{task.stats!.errors.length > 1 ? 's' : ''}
                  <span class="chevron">{expandedErrors.has(task.id) ? '▲' : '▼'}</span>
                </button>
              {/if}
            </div>

            {#if task.stats!.errors.length > 0 && expandedErrors.has(task.id)}
              <ul class="error-list">
                {#each task.stats!.errors as err}
                  <li class="error-item">
                    {#if err.provider_url}
                      <a href={err.provider_url} target="_blank" rel="noopener noreferrer" class="error-track-link">
                        {err.track}
                        <span class="external-icon">↗</span>
                      </a>
                    {:else}
                      <span class="error-track">{err.track}</span>
                    {/if}
                    <span class="error-reason">{err.reason}</span>
                  </li>
                {/each}
              </ul>
            {/if}

            {#if task.stats!.to_validate_tracks && task.stats!.to_validate_tracks.length > 0 && expandedValidations.has(task.id)}
              <ul class="validation-list">
                {#each task.stats!.to_validate_tracks as item}
                  <li class="validation-item">
                    <span class="validation-track">{item.track}</span>
                    <span class="validation-reason">{reasonLabel(item.reason)}</span>
                  </li>
                {/each}
                <li class="validation-item validation-action">
                  <button class="go-validate-btn" onclick={() => onNavigateValidations?.()}>
                    Review in Validations →
                  </button>
                </li>
              </ul>
            {/if}
          {/if}

          {#if task.error}
            <p class="task-error">⚠ {task.error}</p>
          {/if}

          {#if task.updated_at}
            <p class="task-date">{new Date(task.updated_at).toLocaleString()}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tasks-page {
    max-width: 640px;
    margin: 2rem auto;
    padding: 0 1rem;
  }

  h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin-bottom: 1.25rem;
    color: var(--text);
  }

  .empty {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .task-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .task-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.85rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .task-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }

  .task-header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .retry-btn {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 99px;
    border: 1px solid #555;
    background: transparent;
    color: #aaa;
    cursor: pointer;
  }
  .retry-btn:hover:not(:disabled) {
    background: #2a2a2a;
    color: #fff;
  }
  .retry-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cancel-btn {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 99px;
    border: 1px solid #b91c1c;
    background: transparent;
    color: #f87171;
    cursor: pointer;
  }
  .cancel-btn:hover:not(:disabled) {
    background: #3b1a1a;
    color: #fca5a5;
  }
  .cancel-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .task-label {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-badge {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 99px;
    flex-shrink: 0;
  }
  .status-badge.pending    { background: #3b3b3b; color: #aaa; }
  .status-badge.running    { background: #1e3a5f; color: #60a5fa; }
  .status-badge.completed  { background: #1a3326; color: #4ade80; }
  .status-badge.failed     { background: #3b1a1a; color: #f87171; }
  .status-badge.cancelled  { background: #3b2a1a; color: #fb923c; }
  .status-badge.cancelling { background: #3b2a1a; color: #fbbf24; }

  .progress-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .progress-bar {
    flex: 1;
    height: 6px;
    background: var(--surface-2, #2a2a2a);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.4s ease;
  }
  .progress-fill.running    { background: #60a5fa; }
  .progress-fill.completed  { background: #4ade80; }
  .progress-fill.cancelled  { background: #fb923c; }

  .progress-text {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    min-width: 50px;
    text-align: right;
  }

  /* ── Stats chips ── */
  .stats-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-top: 0.1rem;
  }

  .stat-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.72rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 99px;
    white-space: nowrap;
  }

  .stat-chip.downloaded { background: #1a3326; color: #4ade80; }
  .stat-chip.to-validate {
    background: #3b3000;
    color: #fbbf24;
    border: none;
    cursor: pointer;
  }
  .stat-chip.to-validate:hover { background: #4a3d00; }

  .stat-chip.skipped    { background: #2a2a2a; color: #9ca3af; }
  .stat-chip.errors     {
    background: #3b1a1a;
    color: #f87171;
    border: none;
    cursor: pointer;
  }
  .stat-chip.errors:hover { background: #4a2020; }

  .chevron {
    font-size: 0.6rem;
    opacity: 0.7;
  }

  /* ── Error detail list ── */
  .error-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    border-left: 2px solid #5a2020;
    padding-left: 0.75rem;
  }

  .error-item {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .error-track {
    font-size: 0.78rem;
    font-weight: 600;
    color: #f87171;
  }

  .error-track-link {
    font-size: 0.78rem;
    font-weight: 600;
    color: #f87171;
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    border-bottom: 1px solid #f8717166;
    cursor: pointer;
    transition: color 0.2s, border-color 0.2s;
  }

  .error-track-link:hover {
    color: #fb8181;
    border-bottom-color: #f87171;
  }

  .external-icon {
    font-size: 0.6rem;
    opacity: 0.7;
  }

  .error-reason {
    font-size: 0.72rem;
    color: #9ca3af;
    word-break: break-word;
  }

  /* ── Validation detail list ── */
  .validation-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    border-left: 2px solid #78520a;
    padding-left: 0.75rem;
  }

  .validation-item {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .validation-track {
    font-size: 0.78rem;
    font-weight: 600;
    color: #fbbf24;
  }

  .validation-reason {
    font-size: 0.72rem;
    color: #9ca3af;
  }

  .validation-action {
    margin-top: 0.2rem;
  }

  .go-validate-btn {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 99px;
    border: 1px solid #fbbf24;
    background: transparent;
    color: #fbbf24;
    cursor: pointer;
  }
  .go-validate-btn:hover {
    background: #3b3000;
  }

  .task-error {
    font-size: 0.78rem;
    color: #f87171;
    margin: 0;
  }

  .task-date {
    font-size: 0.72rem;
    color: var(--muted);
    margin: 0;
  }

  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid #f8717155;
    border-top-color: #f87171;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
