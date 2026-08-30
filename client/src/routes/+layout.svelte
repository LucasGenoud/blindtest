<script>
  import '../app.css';
  import Topbar from '$lib/components/Topbar.svelte';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { connectWebSocket } from '$lib/websocket.js';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

  let { children, data } = $props();

  let prevPath = $state('');
  let entering = $state(false);

  $effect(() => {
    const current = $page.url.pathname;
    if (current !== prevPath && prevPath !== '') {
      entering = true;
      prevPath = current;
      const timeout = setTimeout(() => entering = false, 160);
      return () => clearTimeout(timeout);
    }
    if (prevPath === '') {
      prevPath = current;
    }
  });

  // Set user from load() data and connect WebSocket
  $effect(() => {
    if (data.user) {
      $user = data.user;
    } else if (data.invalidToken) {
      // The load already dropped it from storage; keep the store in step.
      $token = '';
      $user = null;
    }
  });

  onMount(async () => {
    if ($token && data.user) {
      try {
        const ws = await connectWebSocket($token);
        $websocket = ws;
      } catch {
        // silent fail
      }
    }
  });
</script>

<div id="app">
  {#if !$page.url.pathname.startsWith('/play')}
    <Topbar />
  {/if}
  <div class="relative flex flex-1 overflow-hidden bg-bg {entering ? 'page-enter' : ''}">
    {@render children()}
  </div>
</div>
