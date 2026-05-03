<script>
  import Sidebar from '$lib/components/Sidebar.svelte';
  import PublicBlindtests from '$lib/components/PublicBlindtests.svelte';
  import { blindtestStatus } from '$lib/stores/gameStore.js';
</script>

<svelte:head>
  <title>Blindtest.fun</title>
  <meta name="description" content="A fun multiplayer blindtest game — guess movies, TV shows, animes, games and more!" />
</svelte:head>

<div class="homepage">
  {#if $blindtestStatus === 'stopped'}
    <Sidebar />
  {/if}
  <PublicBlindtests />
</div>

<style>
  .homepage {
    display: flex;
    flex: 1;
    overflow: hidden;
    width: 100%;
  }
</style>
