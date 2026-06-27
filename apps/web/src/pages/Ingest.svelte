<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    listIngestFiles,
    ingestFile,
    ingestAll,
    getTasks,
    type IngestFileEntry,
    type IngestFilesResponse,
    type IngestResult,
  } from '../lib/api';
  import type { TaskDto } from '../lib/types';

  // ── State ──────────────────────────────────────────────────────────────────

  let response: IngestFilesResponse | null = $state(null);
  let loadingFiles = $state(true);
  let filesError: string | null = $state(null);

  // Which file row is expanded (by absolute path)
  let expandedPath: string | null = $state(null);

  // Per-file ingest state
  let ingestingFile: string | null = $state(null);
  let fileResults: Record<string, { ok: boolean; message: string }> = $state({});

  // Batch ingest state
  let batchTaskId: number | null = $state(null);
  let batchTask: TaskDto | null = $state(null);
  let batchError: string | null = $state(null);
  let batchIngesting = $state(false);

  // Poll interval for task progress
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  // Scheduled auto-ingest
  let pollHours = $state(0);
  let pollMinutes = $state(30);
  let pollEnabled = $state(false);
  let pollTimerId: ReturnType<typeof setInterval> | null = null;
  let pollMsg: string | null = $state(null);

  // ── Files ──────────────────────────────────────────────────────────────────

  async function loadFiles() {
    loadingFiles = true;
    filesError = null;
    try {
      response = await listIngestFiles();
    } catch (e: unknown) {
      filesError = e instanceof Error ? e.message : String(e);
    } finally {
      loadingFiles = false;
    }
  }

  onMount(loadFiles);

  function toggleExpand(path: string) {
    expandedPath = expandedPath === path ? null : path;
  }

  // ── Single-file ingest ─────────────────────────────────────────────────────

  async function handleIngestFile(e: Event, file: IngestFileEntry) {
    e.stopPropagation(); // don't toggle the expand panel
    if (ingestingFile) return;
    ingestingFile = file.path;
    const prev = { ...fileResults };
    delete prev[file.path];
    fileResults = prev;

    try {
      const result: IngestResult = await ingestFile(file.path);
      fileResults = {
        ...fileResults,
        [file.path]: {
          ok: true,
          message: result.needs_validation
            ? `Staged for validation — "${result.title}"`
            : `Ingested — "${result.title}"`,
        },
      };
      await loadFiles();
    } catch (err: unknown) {
      fileResults = {
        ...fileResults,
        [file.path]: {
          ok: false,
          message: err instanceof Error ? err.message : String(err),
        },
      };
    } finally {
      ingestingFile = null;
    }
  }

  // ── Batch ingest ───────────────────────────────────────────────────────────

  async function handleIngestAll() {
    if (batchIngesting) return;
    batchIngesting = true;
    batchError = null;
    batchTask = null;
    batchTaskId = null;
    stopTaskPoll();

    try {
      const res = await ingestAll();
      batchTaskId = res.task_id;
      startTaskPoll();
      await loadFiles();
    } catch (e: unknown) {
      batchError = e instanceof Error ? e.message : String(e);
      batchIngesting = false;
    }
  }

  function startTaskPoll() {
    pollInterval = setInterval(async () => {
      if (batchTaskId === null) return;
      try {
        const tasks = await getTasks();
        const t = tasks.find((x) => x.id === batchTaskId) ?? null;
        batchTask = t;
        if (t && (t.status === 'Completed' || t.status === 'Failed' || t.status === 'Cancelled')) {
          stopTaskPoll();
          batchIngesting = false;
          await loadFiles();
        }
      } catch {
        // ignore transient poll errors
      }
    }, 1500);
  }

  function stopTaskPoll() {
    if (pollInterval !== null) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  // ── Scheduled auto-ingest ──────────────────────────────────────────────────

  function applyPollSchedule() {
    stopSchedule();
    const ms = (pollHours * 3600 + pollMinutes * 60) * 1000;
    if (!pollEnabled || ms <= 0) return;
    pollTimerId = setInterval(async () => {
      try {
        const res = await ingestAll();
        pollMsg = `Auto-ingest started (task #${res.task_id}) at ${new Date().toLocaleTimeString()}`;
        batchTaskId = res.task_id;
        batchIngesting = true;
        startTaskPoll();
        await loadFiles();
      } catch (e: unknown) {
        pollMsg = `Auto-ingest failed: ${e instanceof Error ? e.message : String(e)}`;
      }
    }, ms);
    pollMsg = `Auto-ingest scheduled every ${formatDuration(ms)}.`;
  }

  function stopSchedule() {
    if (pollTimerId !== null) {
      clearInterval(pollTimerId);
      pollTimerId = null;
    }
  }

  onDestroy(() => {
    stopTaskPoll();
    stopSchedule();
  });

  // ── Helpers ────────────────────────────────────────────────────────────────

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`;
    if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`;
    return `${(n / 1024 ** 3).toFixed(2)} GB`;
  }

  function formatDuration(ms: number): string {
    const s = ms / 1000;
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m`;
    return `${(s / 3600).toFixed(1)}h`;
  }

  function formatSeconds(s: number): string {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, '0')}`;
  }

  function taskProgress(t: TaskDto): number {
    if (!t.total || t.total === 0) return 0;
    return Math.round((t.progress / t.total) * 100);
  }
</script>

<div class="ingest-page">
  <h2>Ingest</h2>
  <p class="subtitle">
    Manage local audio files in the ingest directory. Trigger ingestion manually or on a schedule.
  </p>

  <!-- ── Batch actions ──────────────────────────────────────────────────────── -->
  <section class="section">
    <h3>Batch ingest</h3>

    <div class="batch-row">
      <button
        class="btn-primary"
        disabled={batchIngesting || loadingFiles}
        onclick={handleIngestAll}
      >
        {#if batchIngesting}
          <span class="spinner"></span> Ingesting…
        {:else}
          Ingest all files
        {/if}
      </button>
      <button class="btn-secondary" onclick={loadFiles} disabled={loadingFiles}>
        {#if loadingFiles}<span class="spinner"></span>{:else}Refresh{/if}
      </button>
    </div>

    {#if batchError}
      <p class="feedback error">{batchError}</p>
    {/if}

    {#if batchTask}
      <div class="task-status">
        <div class="task-header">
          <span class="task-label">Task #{batchTask.id}</span>
          <span
            class="status-badge"
            class:ok={batchTask.status === 'Completed'}
            class:running={batchTask.status === 'Running' || batchTask.status === 'Pending'}
            class:error={batchTask.status === 'Failed'}
          >
            {batchTask.status}
          </span>
        </div>

        {#if batchTask.total !== null && batchTask.total > 0}
          <div class="progress-bar-wrap">
            <div class="progress-bar" style="width: {taskProgress(batchTask)}%"></div>
          </div>
          <span class="progress-label">{batchTask.progress} / {batchTask.total}</span>
        {/if}

        {#if batchTask.stats}
          <div class="stats-row">
            <span class="stat ok">✓ {batchTask.stats.downloaded} ingested</span>
            <span class="stat warn">⚠ {batchTask.stats.to_validate} to validate</span>
            {#if batchTask.stats.errors.length > 0}
              <span class="stat error">✗ {batchTask.stats.errors.length} errors</span>
            {/if}
          </div>
        {/if}

        {#if batchTask.error}
          <p class="feedback error">{batchTask.error}</p>
        {/if}
      </div>
    {/if}
  </section>

  <!-- ── Scheduled auto-ingest ─────────────────────────────────────────────── -->
  <section class="section">
    <h3>Scheduled auto-ingest</h3>
    <p class="hint">
      Automatically trigger "Ingest all" at a fixed interval (browser-side, resets on page reload).
    </p>

    <div class="schedule-row">
      <label class="toggle-label">
        <input type="checkbox" bind:checked={pollEnabled} onchange={applyPollSchedule} />
        Enabled
      </label>
      <div class="interval-group">
        <input
          type="number"
          min="0"
          max="23"
          bind:value={pollHours}
          disabled={!pollEnabled}
          onchange={applyPollSchedule}
        />
        <span class="unit">h</span>
        <input
          type="number"
          min="0"
          max="59"
          step="5"
          bind:value={pollMinutes}
          disabled={!pollEnabled}
          onchange={applyPollSchedule}
        />
        <span class="unit">m</span>
      </div>
    </div>

    {#if pollMsg}
      <p class="feedback info">{pollMsg}</p>
    {/if}
  </section>

  <!-- ── File list ──────────────────────────────────────────────────────────── -->
  <section class="section">
    <h3>
      Files in ingest directory
      {#if response}
        <span class="dir-path">{response.ingest_dir}</span>
      {/if}
    </h3>

    {#if filesError}
      <p class="feedback error">{filesError}</p>
    {:else if loadingFiles}
      <p class="empty">Loading…</p>
    {:else if !response || response.files.length === 0}
      <p class="empty">No audio files found in the ingest directory.</p>
    {:else}
      <ul class="file-list">
        {#each response.files as file (file.path)}
          {@const result = fileResults[file.path]}
          {@const expanded = expandedPath === file.path}

          <li class="file-card" class:done={result?.ok} class:failed={result && !result.ok}>
            <!-- ── Header row : left side is clickable to expand, right side has the ingest button ── -->
            <div class="file-header">
              <!-- clickable expand area -->
              <div
                class="file-expand-area"
                role="button"
                tabindex="0"
                aria-expanded={expanded}
                onclick={() => toggleExpand(file.path)}
                onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleExpand(file.path)}
              >
                <svg
                  class="chevron"
                  class:open={expanded}
                  xmlns="http://www.w3.org/2000/svg"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <polyline points="9 18 15 12 9 6"></polyline>
                </svg>

                <div class="file-text">
                  <span class="file-name">
                    {#if file.relative_path !== file.name}
                      <span class="file-subdir"
                        >{file.relative_path.slice(0, file.relative_path.lastIndexOf('/') + 1)}</span
                      >
                    {/if}
                    {file.name}
                  </span>
                  <span class="file-meta-line">
                    <span class="file-size">{formatBytes(file.size_bytes)}</span>
                    {#if file.tags?.title}
                      <span class="file-tag-preview"
                        >{file.tags.artists.length > 0
                          ? `${file.tags.artists[0]} — `
                          : ''}{file.tags.title}</span
                      >
                    {/if}
                    {#if file.tags?.duration_secs}
                      <span class="file-duration">{formatSeconds(file.tags.duration_secs)}</span>
                    {/if}
                  </span>
                </div>
              </div>

              <!-- right side: result feedback + ingest button -->
              <div class="file-actions">
                {#if result}
                  <span class="result-msg" class:ok={result.ok} class:error={!result.ok}>
                    {result.message}
                  </span>
                {/if}
                <button
                  class="btn-primary btn-sm"
                  disabled={!!ingestingFile || !!result?.ok}
                  onclick={(e) => handleIngestFile(e, file)}
                >
                  {#if ingestingFile === file.path}
                    <span class="spinner"></span>
                  {:else if result?.ok}
                    Done
                  {:else}
                    Ingest
                  {/if}
                </button>
              </div>
            </div>

            <!-- ── Expandable detail panel ── -->
            {#if expanded}
              <div class="file-detail">
                {#if file.tags}
                  {@const t = file.tags}
                  <dl class="tags-grid">
                    {#if t.title}
                      <dt>Title</dt>
                      <dd>{t.title}</dd>
                    {/if}
                    {#if t.artists.length > 0}
                      <dt>Artists</dt>
                      <dd>{t.artists.join(', ')}</dd>
                    {/if}
                    {#if t.album}
                      <dt>Album</dt>
                      <dd>{t.album}</dd>
                    {/if}
                    {#if t.date}
                      <dt>Date</dt>
                      <dd>{t.date}</dd>
                    {/if}
                    {#if t.genre}
                      <dt>Genre</dt>
                      <dd>{t.genre}</dd>
                    {/if}
                    {#if t.track_number}
                      <dt>Track #</dt>
                      <dd>{t.track_number}</dd>
                    {/if}
                    {#if t.duration_secs}
                      <dt>Duration</dt>
                      <dd>{formatSeconds(t.duration_secs)}</dd>
                    {/if}
                  </dl>
                {:else}
                  <p class="no-tags">No readable tags found in this file.</p>
                {/if}

                <div class="detail-path">
                  <span class="detail-path-label">Path</span>
                  <code>{file.path}</code>
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .ingest-page {
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
    color: var(--muted);
    margin-bottom: 2rem;
    font-size: 0.9rem;
  }

  .section {
    margin-bottom: 2.5rem;
  }

  h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .dir-path {
    font-size: 0.78rem;
    font-weight: 400;
    color: var(--muted);
    font-family: monospace;
    background: var(--surface-2);
    padding: 0.1rem 0.45rem;
    border-radius: 4px;
  }

  .hint {
    font-size: 0.85rem;
    color: var(--muted);
    margin-bottom: 0.75rem;
  }

  /* ── Buttons ── */
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
    font-family: inherit;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent, #7c6af7);
    color: #fff;
  }

  .btn-secondary {
    background: var(--surface-2, #2a2a2a);
    color: inherit;
  }

  .btn-sm {
    padding: 0.3rem 0.75rem;
    font-size: 0.82rem;
  }

  /* ── Batch row ── */
  .batch-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }

  /* ── Task status ── */
  .task-status {
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    padding: 0.9rem 1rem;
    background: var(--surface, #1a1a1a);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .task-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .task-label {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .progress-bar-wrap {
    height: 6px;
    background: var(--surface-2, #2a2a2a);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent, #7c6af7);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .progress-label {
    font-size: 0.78rem;
    color: var(--muted);
    text-align: right;
  }

  .stats-row {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    font-size: 0.82rem;
  }

  .stat.ok   { color: #6dc87a; }
  .stat.warn { color: #e8b04b; }
  .stat.error { color: var(--error, #e55); }

  .status-badge {
    font-size: 0.75rem;
    padding: 0.18rem 0.5rem;
    border-radius: 20px;
    font-weight: 600;
  }
  .status-badge.ok      { background: rgba(100,200,120,0.15); color: #6dc87a; }
  .status-badge.running { background: rgba(124,106,247,0.15); color: var(--accent, #7c6af7); }
  .status-badge.error   { background: rgba(220,50,50,0.12);   color: var(--error, #e55); }

  /* ── Schedule ── */
  .schedule-row {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    flex-wrap: wrap;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.9rem;
    cursor: pointer;
    user-select: none;
  }

  .interval-group {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .interval-group input[type='number'] {
    width: 58px;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--border, #333);
    border-radius: 6px;
    background: var(--surface, #1a1a1a);
    color: var(--text);
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    text-align: center;
  }

  .interval-group input:disabled { opacity: 0.4; }
  .unit { font-size: 0.82rem; color: var(--muted); }

  /* ── Feedback ── */
  .feedback {
    padding: 0.55rem 0.85rem;
    border-radius: 6px;
    font-size: 0.88rem;
    margin-top: 0.6rem;
  }
  .feedback.error { background: rgba(220,50,50,0.12);   color: var(--error, #e55); }
  .feedback.info  { background: rgba(100,200,120,0.10); color: #6dc87a; }

  /* ── File list ── */
  .empty {
    color: var(--muted);
    text-align: center;
    padding: 2.5rem 0;
  }

  .file-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .file-card {
    border: 1px solid var(--border, #333);
    border-radius: 8px;
    background: var(--surface, #1a1a1a);
    overflow: hidden;
    transition: border-color 0.15s;
  }

  .file-card.done   { border-color: rgba(100,200,120,0.35); opacity: 0.75; }
  .file-card.failed { border-color: rgba(220,50,50,0.35); }

  /* ── File card header ── */
  .file-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  /* Left expand zone — acts as the clickable toggle */
  .file-expand-area {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    flex: 1;
    min-width: 0;
    padding: 0.7rem 0 0.7rem 1rem;
    cursor: pointer;
    user-select: none;
  }

  .file-expand-area:hover { background: var(--surface-2, #222); }

  .file-header .file-actions {
    padding: 0.5rem 1rem 0.5rem 0;
  }

  /* chevron icon */
  .chevron {
    flex-shrink: 0;
    color: var(--muted);
    transition: transform 0.2s ease;
  }
  .chevron.open { transform: rotate(90deg); }

  .file-text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }

  .file-name {
    font-size: 0.9rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-subdir {
    font-weight: 400;
    color: var(--muted);
    font-size: 0.85em;
  }

  .file-meta-line {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .file-size     { font-size: 0.75rem; color: var(--muted); }
  .file-duration { font-size: 0.75rem; color: var(--muted); }

  .file-tag-preview {
    font-size: 0.75rem;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 28ch;
  }

  .file-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }

  .result-msg {
    font-size: 0.8rem;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .result-msg.ok    { color: #6dc87a; }
  .result-msg.error { color: var(--error, #e55); }

  /* ── Detail panel ── */
  .file-detail {
    border-top: 1px solid var(--border, #333);
    padding: 0.85rem 1rem 0.85rem 2.4rem;
    background: var(--bg, #141414);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .tags-grid {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.25rem 1rem;
    margin: 0;
  }

  .tags-grid dt {
    font-size: 0.78rem;
    color: var(--muted);
    font-weight: 500;
    white-space: nowrap;
    padding-top: 0.05rem;
  }

  .tags-grid dd {
    font-size: 0.85rem;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-tags {
    font-size: 0.85rem;
    color: var(--muted);
    margin: 0;
  }

  .detail-path {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .detail-path-label {
    font-size: 0.75rem;
    color: var(--muted);
    white-space: nowrap;
  }

  .detail-path code {
    font-size: 0.75rem;
    color: var(--muted);
    word-break: break-all;
  }

  /* ── Spinner ── */
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
