<script lang="ts">
  import { onMount } from 'svelte';
  import { getPendingCount, getActiveTasksCount } from './lib/api';
  import Home from './pages/Home.svelte';
  import Validations from './pages/Validations.svelte';
  import Tasks from './pages/Tasks.svelte';
  import Library from './pages/Library.svelte';
  import SyncSchedules from './pages/SyncSchedules.svelte';
  import HelpModal from './lib/HelpModal.svelte';

  type Page = 'home' | 'validations' | 'tasks' | 'library' | 'sync';

  let page: Page = $state('home');
  let pendingCount = $state(0);
  let activeTasksCount = $state(0);
  let helpOpen = $state(false);

  async function refreshCounts() {
    try {
      pendingCount = await getPendingCount();
    } catch {
      // API might not be up yet in dev
    }
    try {
      activeTasksCount = await getActiveTasksCount();
    } catch {
      // ignore
    }
  }

  onMount(() => {
    refreshCounts();
    const interval = setInterval(refreshCounts, 5_000);

    function onKeydown(e: KeyboardEvent) {
      const tgt = e.target as HTMLElement;
      if (tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.tagName === 'SELECT') return;
      if (e.key === '?') {
        e.preventDefault();
        helpOpen = !helpOpen;
      }
    }
    document.addEventListener('keydown', onKeydown);

    return () => {
      clearInterval(interval);
      document.removeEventListener('keydown', onKeydown);
    };
  });

  function navigate(to: Page) {
    page = to;
    if (to === 'validations') refreshCounts();
  }
</script>

<nav>
  <button class="brand" onclick={() => navigate('home')}>Soundome</button>
  <div class="nav-links">
    <button
      class="nav-link"
      class:active={page === 'home'}
      onclick={() => navigate('home')}
    >
      Download
    </button>
    <button
      class="nav-link"
      class:active={page === 'library'}
      onclick={() => navigate('library')}
    >
      Library
    </button>
    <button
      class="nav-link"
      class:active={page === 'validations'}
      onclick={() => navigate('validations')}
    >
      Validations
      {#if pendingCount > 0}
        <span class="badge">{pendingCount}</span>
      {/if}
    </button>
    <button
      class="nav-link"
      class:active={page === 'tasks'}
      onclick={() => navigate('tasks')}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      Tasks
      {#if activeTasksCount > 0}
        <span class="badge">{activeTasksCount}</span>
      {/if}
    </button>
    <button
      class="nav-link"
      class:active={page === 'sync'}
      onclick={() => navigate('sync')}
    >
      Sync
    </button>
    <button
      class="nav-link nav-help"
      onclick={() => (helpOpen = true)}
      title="Help (press ?)"
      aria-label="Help"
    >
      ?
    </button>
  </div>
</nav>

<main>
  {#if page === 'home'}
    <Home onNavigateTasks={() => navigate('tasks')} />
  {:else if page === 'library'}
    <Library />
  {:else if page === 'validations'}
    <Validations onDownloaded={refreshCounts} />
  {:else if page === 'sync'}
    <SyncSchedules />
  {:else}
    <Tasks />
  {/if}
</main>

<HelpModal open={helpOpen} onClose={() => (helpOpen = false)} />

<style>
  nav {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .brand {
    background: none;
    border: none;
    color: var(--text);
    font-weight: 700;
    font-size: 1rem;
    letter-spacing: 0.05em;
    cursor: pointer;
    padding: 0;
  }

  .nav-links {
    display: flex;
    gap: 0.25rem;
  }

  .nav-link {
    background: none;
    border: none;
    color: var(--muted);
    font-size: 0.875rem;
    padding: 0.35rem 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    transition: color 0.15s, background 0.15s;
  }

  .nav-link:hover {
    color: var(--text);
    background: var(--surface-2);
  }

  .nav-link.active {
    color: var(--text);
    background: var(--surface-2);
  }

  .nav-help {
    font-weight: 700;
    font-size: 1rem;
    min-width: 28px;
    justify-content: center;
    opacity: 0.55;
  }

  .nav-help:hover {
    opacity: 1;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: #e05252;
    color: #fff;
    font-size: 0.68rem;
    font-weight: 700;
    border-radius: 9px;
    line-height: 1;
  }

  main {
    min-height: calc(100vh - 48px);
    background: var(--bg);
  }
</style>
