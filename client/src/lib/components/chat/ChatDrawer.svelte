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
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    display: flex; align-items: center; justify-content: center;
    font-size: 20px; cursor: pointer; z-index: 999;
    transition: all 0.15s;
  }
  .chat-btn:hover {
    border-color: var(--border-2);
    background: var(--surface-2);
  }
  .notif {
    position: absolute; bottom: 100%; right: 100%;
    background: var(--red);
    font-family: var(--mono);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 4px;
    margin-right: -8px;
    margin-bottom: -8px;
    color: white;
  }
  .chat-popup {
    position: fixed; right: 20px; bottom: 20px;
    width: 360px; height: 500px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px; z-index: 1000;
    display: flex; flex-direction: column;
    overflow: hidden;
  }
  .chat-header {
    padding: 14px 18px;
    display: flex; justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border);
  }
  .chat-title {
    font-family: var(--mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .close-btn {
    cursor: pointer;
    color: var(--text-dim);
    font-size: 14px;
    transition: color 0.15s;
  }
  .close-btn:hover { color: var(--text-secondary); }
  .chat-messages {
    flex: 1; overflow: auto; padding: 12px 18px;
  }
  .msg { margin-bottom: 12px; }
  .msg-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .msg-user {
    font-family: var(--mono);
    font-weight: 600;
    font-size: 0.72rem;
  }
  .msg-time {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-dim);
  }
  .msg-text {
    font-size: 0.82rem;
    margin-top: 3px;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .chat-input {
    display: flex; padding: 12px 18px; gap: 8px;
    border-top: 1px solid var(--border);
  }
  .chat-input input { flex: 1; }
  .send-btn {
    background: var(--accent);
    color: var(--bg);
    border: none;
    border-radius: 6px;
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .send-btn:hover:not(:disabled) {
    box-shadow: 0 0 10px rgba(232, 255, 90, 0.2);
  }
  .send-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
  @media (max-width: 800px) {
    .chat-btn { display: none; }
  }
</style>
