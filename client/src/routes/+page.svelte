<script>
  import Sidebar from '$lib/components/Sidebar.svelte';
  import PublicBlindtests from '$lib/components/PublicBlindtests.svelte';
</script>

<svelte:head>
  <title>Blindtest.fun</title>
  <meta name="description" content="A fun multiplayer blindtest game — guess movies, TV shows, animes, games and more!" />
</svelte:head>

<div class="flex h-full w-full flex-1 overflow-hidden">
  <Sidebar />

  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <PublicBlindtests />
  </div>
</div>
