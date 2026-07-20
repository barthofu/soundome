<script lang="ts">
  interface Props {
    open: boolean;
    onClose: () => void;
  }
  let { open, onClose }: Props = $props();

  let dialogEl: HTMLDialogElement | undefined = $state(undefined);

  $effect(() => {
    if (!dialogEl) return;
    if (open) { if (!dialogEl.open) dialogEl.showModal(); }
    else { if (dialogEl.open) dialogEl.close(); }
  });
</script>

<dialog
  bind:this={dialogEl}
  onclose={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
  class="help-dialog"
>
  <div class="dialog-header">
    <h3>Soundome — help</h3>
    <button class="dialog-close" onclick={onClose} aria-label="Close">&times;</button>
  </div>

  <div class="dialog-body">

    <!-- ── Pages ────────────────────────────────────────────────────────── -->
    <section>
      <h4>Pages</h4>
      <table class="help-table">
        <tbody>
          <tr>
            <td class="page-name">Download</td>
            <td>Paste a track, album, or playlist URL. Result appears inline; tracks needing review are flagged.</td>
          </tr>
          <tr>
            <td class="page-name">Library</td>
            <td>Browse and edit artists, albums, tracks, and playlists. Drill into an artist or album to see its content. Merge duplicate artists via multi-select.</td>
          </tr>
          <tr>
            <td class="page-name">Validations</td>
            <td>Tracks whose metadata could not be matched automatically. Review, optionally edit, then approve or reject each one.</td>
          </tr>
          <tr>
            <td class="page-name">Tasks</td>
            <td>Background jobs (playlist syncs, downloads). Shows progress, status, and allows retry or cancellation.</td>
          </tr>
          <tr>
            <td class="page-name">Sync</td>
            <td>Scheduled playlist sync jobs. Add a URL with an interval; pause, resume, or trigger a sync manually.</td>
          </tr>
        </tbody>
      </table>
    </section>

    <!-- ── Keyboard shortcuts ───────────────────────────────────────────── -->
    <section>
      <h4>Keyboard shortcuts</h4>

      <h5>Library</h5>
      <table class="shortcut-table">
        <tbody>
          <tr><td><kbd>S</kbd></td><td>Focus the search field</td></tr>
          <tr><td><kbd>E</kbd></td><td>Edit the item under the cursor</td></tr>
          <tr><td><kbd>⌫ Backspace</kbd></td><td>Go up one level (album → artist → list)</td></tr>
          <tr><td><kbd>Shift</kbd> + click</td><td>Select an artist or album for merge</td></tr>
          <tr><td><kbd>M</kbd></td><td>Start merge (requires ≥ 2 artists or albums selected)</td></tr>
          <tr><td><kbd>Esc</kbd></td><td>Cancel merge / clear selection</td></tr>
        </tbody>
      </table>

      <h5>Validations</h5>
      <table class="shortcut-table">
        <tbody>
          <tr><td><kbd>E</kbd></td><td>Open inline edit for the hovered card</td></tr>
          <tr><td><kbd>Esc</kbd></td><td>Close the inline edit form</td></tr>
        </tbody>
      </table>

      <h5>Edit modal</h5>
      <table class="shortcut-table">
        <tbody>
          <tr><td><kbd>Enter</kbd></td><td>Save changes</td></tr>
          <tr><td><kbd>Esc</kbd></td><td>Cancel without saving</td></tr>
        </tbody>
      </table>

      <h5>Global</h5>
      <table class="shortcut-table">
        <tbody>
          <tr><td><kbd>?</kbd></td><td>Open / close this help panel</td></tr>
        </tbody>
      </table>
    </section>

    <!-- ── Tips ─────────────────────────────────────────────────────────── -->
    <section>
      <h4>Tips</h4>
      <ul class="tips-list">
        <li>Playlist URLs queue a background sync task — follow progress in <strong>Tasks</strong>.</li>
        <li>Tracks marked <span class="badge-warn">review</span> in the recent-downloads list are waiting in <strong>Validations</strong>.</li>
        <li>In the Validations page, <strong>Show matches</strong> fetches alternative metadata candidates when the reason is a partial match.</li>
        <li>In Library → Artists or Albums, the <strong>Similar</strong> filter highlights items whose names/titles are close — useful for spotting duplicates before merging.</li>
        <li>Full API docs are available at <a href="/swagger" target="_blank" rel="noopener noreferrer">/swagger</a>.</li>
      </ul>
    </section>

  </div>
</dialog>

<style>
  .help-dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 0;
    width: min(640px, 94vw);
    max-height: 88vh;
    overflow-y: auto;
    color: var(--text);
    font-family: inherit;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
  }

  .help-dialog::backdrop {
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(3px);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.1rem 1.4rem 0.8rem;
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--surface);
    z-index: 1;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .dialog-close {
    background: none;
    border: none;
    color: var(--muted);
    font-size: 1.4rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 0.2rem;
  }

  .dialog-close:hover {
    color: var(--text);
  }

  .dialog-body {
    padding: 1.2rem 1.4rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  h4 {
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted);
    margin: 0;
  }

  h5 {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--muted);
    margin: 0.4rem 0 0.2rem;
  }

  /* ── Pages table ──────────────────────────────────────────────────── */

  .help-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.855rem;
  }

  .help-table td {
    padding: 0.42rem 0.6rem;
    vertical-align: top;
    border-bottom: 1px solid var(--border);
  }

  .help-table tr:last-child td {
    border-bottom: none;
  }

  .page-name {
    font-weight: 600;
    white-space: nowrap;
    width: 1%;
    padding-right: 1rem;
    color: var(--accent);
  }

  /* ── Shortcut table ───────────────────────────────────────────────── */

  .shortcut-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.845rem;
  }

  .shortcut-table td {
    padding: 0.3rem 0.4rem;
    vertical-align: middle;
  }

  .shortcut-table td:first-child {
    white-space: nowrap;
    width: 1%;
    padding-right: 1.2rem;
  }

  kbd {
    display: inline-block;
    padding: 0.12rem 0.4rem;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-family: inherit;
    font-size: 0.8rem;
    line-height: 1.4;
    color: var(--text);
  }

  /* ── Tips list ────────────────────────────────────────────────────── */

  .tips-list {
    margin: 0;
    padding: 0 0 0 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.845rem;
    color: var(--muted);
  }

  .tips-list li {
    line-height: 1.5;
  }

  .tips-list strong {
    color: var(--text);
    font-weight: 600;
  }

  .tips-list a {
    color: var(--accent);
    text-decoration: none;
  }

  .tips-list a:hover {
    text-decoration: underline;
  }

  .badge-warn {
    font-size: 0.7rem;
    padding: 0.1rem 0.35rem;
    background: color-mix(in srgb, var(--warning, #f59e0b) 20%, transparent);
    border: 1px solid color-mix(in srgb, var(--warning, #f59e0b) 40%, transparent);
    color: var(--warning, #f59e0b);
    border-radius: 4px;
  }
</style>
