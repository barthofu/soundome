<script lang="ts">
  import { onMount } from 'svelte';
  import { getPendingCount, getActiveTasksCount, getVersion } from './lib/api';
  import Home from './pages/Home.svelte';
  import Validations from './pages/Validations.svelte';
  import Tasks from './pages/Tasks.svelte';
  import Library from './pages/Library.svelte';
  import Tools from './pages/Tools.svelte';
  import Ingest from './pages/Ingest.svelte';
  import HelpModal from './lib/HelpModal.svelte';
  import PWAUpdatePrompt from './components/PWAUpdatePrompt.svelte';

  type Page = 'download' | 'validations' | 'tasks' | 'library' | 'tools' | 'ingest';

  let page: Page = $state('library');
  let pendingCount = $state(0);
  let activeTasksCount = $state(0);
  let helpOpen = $state(false);
  let version = $state('');
  let mobileMenuOpen = $state(false);

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
    getVersion().then((v) => (version = v));
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
    mobileMenuOpen = false;
    if (to === 'validations') refreshCounts();
  }
</script>

<nav>
  <!-- Brand -->
  <button class="brand" onclick={() => navigate('library')}>Soundome

    {#if version}
      <span class="version">v{version}</span>
    {/if}
  </button>
  
  <!-- Hamburger menu button (visible on mobile) -->
  <button class="hamburger" onclick={() => (mobileMenuOpen = !mobileMenuOpen)} aria-label="Toggle menu" aria-expanded={mobileMenuOpen}>
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <line x1="3" y1="6" x2="21" y2="6"></line>
      <line x1="3" y1="12" x2="21" y2="12"></line>
      <line x1="3" y1="18" x2="21" y2="18"></line>
    </svg>
  </button>

  <div class="nav-links" class:mobile-open={mobileMenuOpen}>
    <!-- Group 1: Library (leftmost, standalone) -->
    <button
      class="nav-link"
      class:active={page === 'library'}
      onclick={() => navigate('library')}
    >
      Library
    </button>

    <!-- Separator -->
    <span class="nav-sep" aria-hidden="true"></span>

    <!-- Group 2: Main workflow -->
    <button
      class="nav-link"
      class:active={page === 'download'}
      onclick={() => navigate('download')}
    >
      Download
    </button>
    <button
      class="nav-link"
      class:active={page === 'ingest'}
      onclick={() => navigate('ingest')}
    >
      Ingest
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

    <!-- Separator -->
    <span class="nav-sep" aria-hidden="true"></span>

    <!-- Group 3: Utility icons only -->

    <!-- Tasks icon -->
    <button
      class="nav-icon"
      class:active={page === 'tasks'}
      onclick={() => navigate('tasks')}
      title="Tasks"
      aria-label="Tasks"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/>
        <polyline points="12 6 12 12 16 14"/>
      </svg>
      {#if activeTasksCount > 0}
        <span class="badge">{activeTasksCount}</span>
      {/if}
    </button>

    <!-- Tools icon -->
    <button
      class="nav-icon"
      class:active={page === 'tools'}
      onclick={() => navigate('tools')}
      title="Tools"
      aria-label="Tools"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
      </svg>
    </button>

    <!-- Help icon -->
    <button
      class="nav-icon nav-help"
      onclick={() => (helpOpen = true)}
      title="Help (press ?)"
      aria-label="Help"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="10"/>
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
    </button>
  </div>
</nav>

<main>
  {#if page === 'download'}
    <Home onNavigateTasks={() => navigate('tasks')} />
  {:else if page === 'library'}
    <Library />
  {:else if page === 'tools'}
    <Tools />
  {:else if page === 'validations'}
    <Validations onDownloaded={refreshCounts} />
  {:else if page === 'ingest'}
    <Ingest />
  {:else}
    <Tasks onNavigateValidations={() => navigate('validations')} />
  {/if}
</main>

<HelpModal open={helpOpen} onClose={() => (helpOpen = false)} />

<PWAUpdatePrompt />

<style>
  nav {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  @media (max-width: 640px) {
    nav {
      padding: 0 0.75rem;
      height: 44px;
    }
  }

  .brand {
    background: none;
    border: none;
    color: var(--text);
    font-weight: 700;
    font-size: 0.95rem;
    letter-spacing: 0.05em;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  @media (min-width: 641px) {
    .brand {
      font-size: 1rem;
    }
  }

  .hamburger {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text);
    cursor: pointer;
    padding: 0.25rem;
    margin-left: 0.5rem;
    flex-shrink: 0;
  }

  @media (min-width: 641px) {
    .hamburger {
      display: none;
    }
  }

  .version {
    font-size: 0.7rem;
    color: var(--muted);
    opacity: 0.6;
    margin-left: 0.4rem;
    flex-shrink: 0;
    user-select: none;
  }

  .nav-links {
    display: none;
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-direction: column;
    gap: 0;
    padding: 0.5rem 0;
    max-height: calc(100vh - 44px);
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
  }

  .nav-links.mobile-open {
    display: flex;
  }

  @media (min-width: 641px) {
    .nav-links {
      display: flex;
      position: static;
      flex-direction: row;
      align-items: center;
      gap: 0.15rem;
      padding: 0;
      border: none;
      max-height: none;
    }
  }

  /* Thin separator */
  .nav-sep {
    display: none;
    width: 1px;
    height: 16px;
    background: var(--border);
    margin: 0 0.4rem;
    flex-shrink: 0;
  }

  @media (min-width: 641px) {
    .nav-sep {
      display: block;
    }
  }

  /* Text nav items */
  .nav-link {
    background: none;
    border: none;
    color: var(--muted);
    font-size: 0.8rem;
    font-family: inherit;
    padding: 0.5rem 1rem;
    border-radius: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    transition: color 0.15s, background 0.15s;
    white-space: nowrap;
    width: 100%;
    justify-content: flex-start;
  }

  @media (min-width: 641px) {
    .nav-link {
      padding: 0.35rem 0.7rem;
      border-radius: 6px;
      width: auto;
      justify-content: center;
      font-size: 0.875rem;
    }
  }

  .nav-link:hover {
    color: var(--text);
    background: var(--surface-2);
  }

  .nav-link.active {
    color: var(--text);
    background: var(--surface-2);
  }

  /* Icon-only nav items */
  .nav-icon {
    background: none;
    border: none;
    color: var(--muted);
    padding: 0.5rem 1rem;
    border-radius: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    position: relative;
    transition: color 0.15s, background 0.15s;
    width: 100%;
  }

  @media (min-width: 641px) {
    .nav-icon {
      padding: 0.35rem 0.45rem;
      border-radius: 6px;
      width: auto;
      justify-content: center;
    }
  }

  .nav-icon:hover {
    color: var(--text);
    background: var(--surface-2);
  }

  .nav-icon.active {
    color: var(--text);
    background: var(--surface-2);
  }

  .nav-help {
    opacity: 0.6;
  }

  .nav-help:hover {
    opacity: 1;
  }

  /* Badge */
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 17px;
    height: 17px;
    padding: 0 4px;
    background: #e05252;
    color: #fff;
    font-size: 0.66rem;
    font-weight: 700;
    border-radius: 9px;
    line-height: 1;
  }

  /* Badge on icon buttons: absolute top-right */
  .nav-icon .badge {
    position: absolute;
    top: 1px;
    right: 1px;
    min-width: 14px;
    height: 14px;
    font-size: 0.6rem;
    padding: 0 3px;
  }

  main {
    min-height: calc(100vh - 48px);
    background: var(--bg);
    overflow-x: hidden;
  }

  @media (max-width: 640px) {
    main {
      min-height: calc(100vh - 44px);
    }
  }
</style>
