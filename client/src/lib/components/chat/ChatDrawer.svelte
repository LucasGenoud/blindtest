<script>
  import { onMount, tick } from 'svelte';
  import { token } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { getApi } from '$lib/api.js';
  import { stringToColor } from '$lib/misc.js';
  import { slide, fade } from 'svelte/transition';
  import { MessageCircle, X, Send } from 'lucide-svelte';
  import dayjs from 'dayjs';

  let showChat = $state(false);
  let messages = $state([]);
  let messageText = $state('');
  let notifCount = $state(0);
  let badgePop = $state(false);

  onMount(async () => {
    try {
      const res = await fetch(`${getApi()}/getLatestChatMessages`, {
        headers: { Authorization: $token },
      });
      if (res.ok) {
        const data = await res.json();
        messages = data.map(m => ({
          ...m,
          color: stringToColor(m.username),
          time: dayjs(m.date).format('HH:mm:ss'),
        }));
      }
    } catch {}

    if ($websocket) {
      $websocket.addEventListener('message', onWsMessage);
    }
  });

  function onWsMessage(e) {
    const msg = JSON.parse(e.data);
    if (msg.type === 'chatMessage') {
      if (!showChat) {
        notifCount++;
        badgePop = true;
        setTimeout(() => badgePop = false, 300);
      }
      messages = [...messages, {
        ...msg.data,
        color: stringToColor(msg.data.username),
        time: dayjs().format('HH:mm:ss'),
      }];
      scrollBottom();
    }
  }

  async function scrollBottom() {
    await tick();
    const el = document.getElementById('chatMessages');
    if (el) el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
  }

  function openChat() {
    showChat = true;
    notifCount = 0;
    scrollBottom();
  }

  async function sendMessage() {
    if (!messageText) return;
    try {
      await fetch(`${getApi()}/sendChatMessage`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: $token },
        body: JSON.stringify({ messageValue: messageText }),
      });
      messageText = '';
    } catch {}
  }
</script>

<div>
  {#if !showChat}
    <div
      class="fixed bottom-5 right-5 z-[999] flex h-11 w-11 cursor-pointer items-center justify-center rounded-lg text-[20px] text-white shadow-[0_4px_14px_var(--accent-dim)] transition-all duration-200 hover:-translate-y-0.5 hover:shadow-[0_6px_20px_var(--accent-dim)] max-[800px]:hidden [background:var(--accent)] hover:[background:var(--accent-hover)]"
      onclick={openChat}
    >
      <MessageCircle size={20} stroke-width={1.8} />
      {#if notifCount > 0}
        <div class={`absolute -right-1 -top-1 flex min-w-[1.1rem] items-center justify-center rounded-full bg-red px-1.5 py-0.5 text-[0.65rem] font-bold text-white transition-opacity duration-150 {badgePop ? 'badge-pop' : ''}`}>{Math.min(notifCount, 99)}</div>
      {/if}
    </div>
  {/if}

  {#if showChat}
    <div class="fixed bottom-5 right-5 z-[1000] flex h-[500px] w-[360px] flex-col overflow-hidden rounded-xl border border-border bg-surface shadow-panel" in:slide={{ duration: 200 }} out:fade={{ duration: 100 }}>
      <div class="flex items-center justify-between border-b border-border bg-surface px-5 py-4 shrink-0">
        <span class="text-[0.9375rem] font-bold text-text-primary">Chat</span>
        <button class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-sm border border-border bg-surface-2 text-sm text-text-dim transition-colors duration-150 hover:border-border-2 hover:text-text-primary" onclick={() => showChat = false} aria-label="Close chat"><X size={12} stroke-width={1.8} /></button>
      </div>
      <div id="chatMessages" class="flex-1 overflow-auto px-5 py-4 pb-16">
        {#each messages as m (m._id)}
          <div class="mb-3.5 last:mb-0" in:slide={{ duration: 150, delay: 0 }}>
            <div class="mb-[3px] flex items-baseline gap-2">
              <span class="text-[0.8125rem] font-semibold" style="color:{m.color}">{m.username}</span>
              <span class="font-mono text-[0.6875rem] text-text-dim">{m.time}</span>
            </div>
            <div class="text-sm leading-6 text-text-secondary break-words">{m.messageValue}</div>
          </div>
        {/each}
      </div>
      <div class="absolute bottom-0 left-0 right-0 flex gap-2 border-t border-border bg-surface-2 px-4 py-3">
        <input
          bind:value={messageText}
          placeholder="Message..."
          autocomplete="off"
          onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); } }}
        />
        <button
          class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md text-sm text-white transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-40 [background:var(--accent)] hover:[background:var(--accent-hover)] disabled:hover:[background:var(--accent)]"
          onclick={sendMessage}
          disabled={!messageText.trim()}
          aria-label="Send message"
        ><Send size={14} stroke-width={1.8} /></button>
      </div>
    </div>
  {/if}
</div>
