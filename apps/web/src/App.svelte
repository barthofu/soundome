<script lang="ts">
  import Validations from './pages/Validations.svelte';
</script>

<nav>
  <span class="brand">Soundome</span>
</nav>

<main>
  <Validations />
</main>

<style>
  nav {
    height: 48px;
    display: flex;
    align-items: center;
    padding: 0 1.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }

  .brand {
    font-weight: 700;
    font-size: 1rem;
    letter-spacing: 0.05em;
  }

  main {
    min-height: calc(100vh - 48px);
    background: var(--bg);
  }
</style>
