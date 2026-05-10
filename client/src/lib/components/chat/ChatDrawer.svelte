<script>
  import { onMount } from 'svelte';
  import { token } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { getApi } from '$lib/api.js';
  import { stringToColor } from '$lib/misc.js';
  import dayjs from 'dayjs';

  let showChat = $state(false);
  let messages = $state([]);
  let messageText = $state('');
  let notifCount = $state(0);

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
      if (!showChat) notifCount++;
      messages = [...messages, {
        ...msg.data,
        color: stringToColor(msg.data.username),
        time: dayjs().format('HH:mm:ss'),
      }];
      scrollBottom();
    }
  }

  function scrollBottom() {
    setTimeout(() => {
      const el = document.getElementById('chatMessages');
      if (el) el.scrollTo(0, el.scrollHeight + 100);
    }, 50);
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
    <div class="chat-btn" onclick={openChat}>
      💬
      {#if notifCount > 0}
        <div class="notif">{notifCount}</div>
      {/if}
    </div>
  {/if}

  {#if showChat}
    <div class="chat-popup">
      <div class="chat-header">
        <span class="chat-title">Chat</span>
        <span class="close-btn" onclick={() => showChat = false}>✕</span>
      </div>
      <div id="chatMessages" class="chat-messages">
        {#each messages as m (m._id)}
          <div class="msg">
            <div class="msg-header">
              <span class="msg-user" style="color:{m.color}">{m.username}</span>
              <span class="msg-time">{m.time}</span>
            </div>
            <div class="msg-text">{m.messageValue}</div>
          </div>
        {/each}
      </div>
      <div class="chat-input">
        <input bind:value={messageText} placeholder="Message..."
          onkeydown={(e) => { e.stopPropagation(); if (e.key === 'Enter') sendMessage(); }} />
        <button class="send-btn" onclick={sendMessage} disabled={!messageText}>➤</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-btn {
    position: fixed; right: 20px; bottom: 20px;
    width: 44px; height: 44px;
    background: var(--accent);
    border: none;
    border-radius: var(--radius-lg);
    display: flex; align-items: center; justify-content: center;
    font-size: 20px; cursor: pointer; z-index: 999;
    transition: all 0.2s;
    box-shadow: 0 4px 14px var(--accent-dim);
  }
  .chat-btn:hover {
    background: var(--accent-hover);
    transform: translateY(-2px);
    box-shadow: 0 6px 20px var(--accent-dim);
  }
  .notif {
    position: absolute; bottom: 100%; right: 100%;
    background: var(--red);
    font-size: 0.65rem;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 9999px;
    margin-right: -8px;
    margin-bottom: -8px;
    color: white;
  }
  .chat-popup {
    position: fixed; right: 20px; bottom: 20px;
    width: 360px; height: 500px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl); z-index: 1000;
    display: flex; flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }
  .chat-header {
    padding: 16px 20px;
    display: flex; justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .chat-title {
    font-size: 0.9375rem;
    font-weight: 700;
    color: var(--text-primary);
  }
  .close-btn {
    cursor: pointer;
    color: var(--text-dim);
    font-size: 14px;
    transition: color 0.15s;
    width: 28px; height: 28px;
    display: flex; align-items: center; justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    border: 1px solid var(--border);
  }
  .close-btn:hover { color: var(--text-primary); border-color: var(--border-2); }
  .chat-messages {
    flex: 1; overflow: auto; padding: 16px 20px;
  }
  .msg { margin-bottom: 14px; }
  .msg-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 3px;
  }
  .msg-user {
    font-weight: 600;
    font-size: 0.8125rem;
  }
  .msg-time {
    font-size: 0.6875rem;
    color: var(--text-dim);
    font-family: var(--mono);
  }
  .msg-text {
    font-size: 0.875rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .chat-input {
    display: flex; padding: 12px 16px; gap: 8px;
    border-top: 1px solid var(--border);
    background: var(--surface-2);
  }
  .chat-input input { flex: 1; }
  .send-btn {
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
    flex-shrink: 0;
  }
  .send-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  @media (max-width: 800px) {
    .chat-btn { display: none; }
  }
</style>
