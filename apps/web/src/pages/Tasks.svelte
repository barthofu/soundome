<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getTasks, retryTask } from '../lib/api';
  import type { TaskDto } from '../lib/types';

  let tasks: TaskDto[] = $state([]);
  let loading = $state(true);
  let retrying: Set<number> = $state(new Set());
  let interval: ReturnType<typeof setInterval>;

  async function refresh() {
    try {
      tasks = await getTasks();
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
      alert(`Échec du retry : ${e instanceof Error ? e.message : e}`);
    } finally {
      retrying = new Set([...retrying].filter((id) => id !== task.id));
    }
  }

  onMount(() => {
    refresh();
    interval = setInterval(refresh, 3_000);
  });

  onDestroy(() => clearInterval(interval));

  function statusLabel(status: TaskDto['status']) {
    return { Pending: 'En attente', Running: 'En cours', Completed: 'Terminé', Failed: 'Échec' }[status] ?? status;
  }

  function statusClass(status: TaskDto['status']) {
    return { Pending: 'pending', Running: 'running', Completed: 'completed', Failed: 'failed' }[status] ?? '';
  }

  function progressPercent(task: TaskDto) {
    if (!task.total || task.total === 0) return 0;
    return Math.round((task.progress / task.total) * 100);
  }

  function taskLabel(task: TaskDto) {
    if (task.label) return task.label;
    if (task.task_type === 'SyncPlaylist') return 'Synchronisation de playlist';
    return 'Téléchargement';
  }

  function canRetry(status: TaskDto['status']) {
    return status === 'Pending' || status === 'Failed' || status === 'Running';
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
              {#if canRetry(task.status)}
                <button
                  class="retry-btn"
                  disabled={retrying.has(task.id)}
                  onclick={() => handleRetry(task)}
                >
                  {retrying.has(task.id) ? '…' : 'Relancer'}
                </button>
              {/if}
              <span class="status-badge {statusClass(task.status)}">{statusLabel(task.status)}</span>
            </div>
          </div>

          {#if task.status === 'Running' || task.status === 'Completed'}
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

          {#if task.error}
            <p class="task-error">{task.error}</p>
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
  .status-badge.pending  { background: #3b3b3b; color: #aaa; }
  .status-badge.running  { background: #1e3a5f; color: #60a5fa; }
  .status-badge.completed { background: #1a3326; color: #4ade80; }
  .status-badge.failed   { background: #3b1a1a; color: #f87171; }

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
  .progress-fill.running   { background: #60a5fa; }
  .progress-fill.completed { background: #4ade80; }

  .progress-text {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
    min-width: 50px;
    text-align: right;
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
</style>
