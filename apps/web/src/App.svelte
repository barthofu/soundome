<script lang="ts">
  import { onMount } from 'svelte';
  import { getPendingCount } from './lib/api';
  import Home from './pages/Home.svelte';
  import Validations from './pages/Validations.svelte';

  type Page = 'home' | 'validations';

  let page: Page = $state('home');
  let pendingCount = $state(0);

  async function refreshCount() {
    try {
      pendingCount = await getPendingCount();
    } catch {
      // API might not be up yet in dev
    }
  }

  onMount(() => {
    refreshCount();
    const interval = setInterval(refreshCount, 30_000);
    return () => clearInterval(interval);
  });

  function navigate(to: Page) {
    page = to;
    if (to === 'validations') refreshCount();
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
      class:active={page === 'validations'}
      onclick={() => navigate('validations')}
    >
      Validations
      {#if pendingCount > 0}
        <span class="badge">{pendingCount}</span>
      {/if}
    </button>
  </div>
</nav>

<main>
  {#if page === 'home'}
    <Home />
  {:else}
    <Validations onDownloaded={refreshCount} />
  {/if}
</main>

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
