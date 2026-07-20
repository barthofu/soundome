<script lang="ts">
  import { onMount } from 'svelte'
  import { onPWAUpdate, refreshPWA, isPWAAvailable } from '../lib/pwa'

  let showUpdatePrompt = false
  let unsubscribe: (() => void) | undefined

  onMount(() => {
    if (!isPWAAvailable()) {
      return
    }

    // Listen for PWA updates
    unsubscribe = onPWAUpdate(() => {
      showUpdatePrompt = true
    })

    return () => {
      unsubscribe?.()
    }
  })

  function handleUpdate() {
    refreshPWA()
    showUpdatePrompt = false
  }

  function handleDismiss() {
    showUpdatePrompt = false
  }
</script>

{#if showUpdatePrompt}
  <div class="pwa-update-prompt">
    <div class="pwa-update-content">
      <h3>Mise à jour disponible</h3>
      <p>Une nouvelle version de Soundome est disponible.</p>
      <div class="pwa-update-actions">
        <button class="btn-primary" on:click={handleUpdate}>
          Mettre à jour
        </button>
        <button class="btn-secondary" on:click={handleDismiss}>
          Plus tard
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .pwa-update-prompt {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 1000;
    animation: slideIn 0.3s ease-in-out;
  }

  .pwa-update-content {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 16px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    max-width: 320px;
  }

  h3 {
    margin: 0 0 8px 0;
    font-size: 16px;
    font-weight: 600;
    color: #333;
  }

  p {
    margin: 0 0 16px 0;
    font-size: 14px;
    color: #666;
  }

  .pwa-update-actions {
    display: flex;
    gap: 8px;
  }

  button {
    flex: 1;
    padding: 8px 12px;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-primary {
    background: #863bff;
    color: white;
  }

  .btn-primary:hover {
    background: #7029d6;
  }

  .btn-secondary {
    background: #f0f0f0;
    color: #333;
  }

  .btn-secondary:hover {
    background: #e0e0e0;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 480px) {
    .pwa-update-prompt {
      bottom: 10px;
      right: 10px;
      left: 10px;
    }

    .pwa-update-content {
      max-width: none;
    }
  }
</style>
