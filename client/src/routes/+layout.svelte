<script>
  import '../app.css';
  import Topbar from '$lib/components/Topbar.svelte';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { theme } from '$lib/stores/themeStore.js';
  import { connectWebSocket } from '$lib/websocket.js';
  import { getApi } from '$lib/api.js';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';

  let { children } = $props();

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

  // Apply theme on mount and whenever it changes
  $effect(() => {
    if ($theme === 'dark') {
      document.documentElement.setAttribute('data-theme', 'dark');
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
  });

  onMount(async () => {
    // Auto-login if token exists
    if ($token) {
      try {
        const res = await fetch(`${getApi()}/getuser`, {
          headers: { Authorization: $token },
        });
        if (res.ok) {
          const data = await res.json();
          $user = data;
          const ws = await connectWebSocket($token);
          $websocket = ws;
        } else {
          $token = '';
          $user = null;
        }
      } catch {
        // silent fail
      }
    }
  });
</script>

<div id="app">
  <Topbar />
  <div class="relative flex flex-1 overflow-hidden bg-bg {entering ? 'page-enter' : ''}">
    {@render children()}
  </div>
</div>
