<script>
  import AudioPlayer from '$lib/components/AudioPlayer.svelte';
  import { page } from '$app/stores';
</script>

<svelte:head>
  <title>Blindtest — Custom</title>
</svelte:head>

<AudioPlayer blindtestId={$page.params.blindtestId} randomOrder={$page.url.searchParams.get('random') === 'true'} />
