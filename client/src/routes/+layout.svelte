<script>
  import '../app.css';
  import Topbar from '$lib/components/Topbar.svelte';
  import ChatDrawer from '$lib/components/chat/ChatDrawer.svelte';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { connectWebSocket } from '$lib/websocket.js';
  import { getApi } from '$lib/api.js';
  import { onMount } from 'svelte';

  let { children } = $props();

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
  <div class="app-content">
    {@render children()}
  </div>
  {#if $user}
    <ChatDrawer />
  {/if}
</div>

<style>
  .app-content {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
    background: var(--bg);
  }
</style>
