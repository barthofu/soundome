<script lang="ts">
  import type { PendingValidationDto } from '../lib/types';

  interface Props {
    track: PendingValidationDto;
  }

  let { track }: Props = $props();

  function formatDuration(seconds: number | null): string {
    if (!seconds) return '—';
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function artistNames(track: PendingValidationDto): string {
    return track.artists.map((a) => a.name).join(', ') || '—';
  }
</script>

<article class="track-card">
  <div class="cover">
    {#if track.cover}
      <img src={track.cover} alt="cover" />
    {:else}
      <div class="cover-placeholder">♪</div>
    {/if}
  </div>

  <div class="info">
    <div class="row main">
      <span class="title">{track.title}</span>
      <span class="artists">{artistNames(track)}</span>
    </div>

    <div class="row meta">
      {#if track.album}
        <span class="chip">💿 {track.album.title}</span>
      {/if}
      {#if track.date}
        <span class="chip">📅 {track.date}</span>
      {/if}
      {#if track.genre}
        <span class="chip">🎵 {track.genre}</span>
      {/if}
      {#if track.duration}
        <span class="chip">⏱ {formatDuration(track.duration)}</span>
      {/if}
    </div>

    {#if track.validation_reason}
      <div class="reason">
        ⚠️ <code>{track.validation_reason}</code>
      </div>
    {/if}

    {#if track.file_path}
      <div class="filepath">
        <code>{track.file_path}</code>
      </div>
    {/if}
  </div>
</article>

<style>
  .track-card {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .cover {
    flex-shrink: 0;
    width: 72px;
    height: 72px;
    border-radius: 6px;
    overflow: hidden;
    background: var(--surface-2);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    font-size: 1.8rem;
    color: var(--muted);
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: baseline;
  }

  .title {
    font-weight: 600;
    font-size: 1rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .artists {
    font-size: 0.875rem;
    color: var(--muted);
  }

  .chip {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    background: var(--surface-2);
    border-radius: 4px;
    white-space: nowrap;
  }

  .reason {
    font-size: 0.75rem;
    color: var(--warning);
  }

  .filepath {
    font-size: 0.7rem;
    color: var(--muted);
    word-break: break-all;
  }
</style>
