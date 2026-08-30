<script>
  import { goto } from '$app/navigation';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import PublicBlindtests from '$lib/components/PublicBlindtests.svelte';
  import { blindtestStatus } from '$lib/stores/gameStore.js';
</script>

<svelte:head>
  <title>Blindtest.fun</title>
  <meta name="description" content="A fun multiplayer blindtest game — guess movies, TV shows, animes, games and more!" />
</svelte:head>

<div class="flex h-full w-full flex-1 overflow-hidden">
  {#if $blindtestStatus === 'stopped'}
    <Sidebar />
  {/if}

  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    {#if $blindtestStatus !== 'stopped'}
      <!-- The round survives a reload, so leaving the page must not strand it. -->
      <div class="resume-bar">
        <div>
          <h2>A blindtest is still running</h2>
          <p>Pick it up where you left off, or end it to set up a new one.</p>
        </div>
        <div class="resume-actions">
          <button class="btn-primary" onclick={() => goto('/play')}>Resume round</button>
          <button class="btn-secondary" onclick={() => $blindtestStatus = 'stopped'}>End blindtest</button>
        </div>
      </div>
    {/if}
    <PublicBlindtests />
  </div>
</div>

<style>
  .resume-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    flex-wrap: wrap;
    padding: 24px 32px;
    border-bottom: 2px solid var(--divider);
  }

  .resume-bar h2 {
    font-size: 20px;
    font-weight: 800;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .resume-bar p {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .resume-actions {
    display: flex;
    gap: 8px;
  }

  @media (max-width: 760px) {
    .resume-bar { padding: 16px; }
  }
</style>
