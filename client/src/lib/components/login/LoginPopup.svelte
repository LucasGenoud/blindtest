<script>
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { getApi } from '$lib/api.js';
  import { connectWebSocket } from '$lib/websocket.js';
  import { checkEmail } from '$lib/misc.js';

  let { onclose } = $props();
  let mode = $state('signin');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);

  async function signin() {
    error = '';
    if (!email || !password) { error = 'Fill all fields'; return; }
    loading = true;
    try {
      const res = await fetch(`${getApi()}/signin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });
      if (!res.ok) { error = await res.text(); loading = false; return; }
      const data = await res.json();
      $token = data.token;
      $user = data.user;
      const ws = await connectWebSocket(data.token);
      $websocket = ws;
      onclose();
    } catch (e) {
      error = e.message;
    }
    loading = false;
  }

  async function signup() {
    error = '';
    if (!email || !password || !name) { error = 'Fill all fields'; return; }
    if (!checkEmail(email)) { error = 'Invalid email'; return; }
    if (password.length < 6) { error = 'Password must be at least 6 characters'; return; }
    loading = true;
    try {
      const res = await fetch(`${getApi()}/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, name }),
      });
      if (!res.ok) { error = await res.text(); loading = false; return; }
      mode = 'signin';
      error = '';
    } catch (e) {
      error = e.message;
    }
    loading = false;
  }
</script>

<div class="popup-overlay" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="popup-box">
    <div class="tabs">
      <div class="tab" class:active={mode === 'signin'} onclick={() => mode = 'signin'}>Sign in</div>
      <div class="tab" class:active={mode === 'signup'} onclick={() => mode = 'signup'}>Sign up</div>
      <div class="tab-indicator" class:right={mode === 'signup'}></div>
    </div>

    <div class="form">
      {#if mode === 'signup'}
        <input bind:value={name} placeholder="Username" />
      {/if}
      <input bind:value={email} placeholder="Email" type="email" />
      <input bind:value={password} placeholder="Password" type="password"
        onkeydown={(e) => e.key === 'Enter' && (mode === 'signin' ? signin() : signup())} />

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <button class="submit-btn"
        disabled={loading}
        onclick={() => mode === 'signin' ? signin() : signup()}>
        {mode === 'signin' ? 'Sign in' : 'Sign up'}
      </button>
    </div>
  </div>
</div>

<style>
  .tabs {
    display: flex;
    position: relative;
    border-bottom: 1px solid var(--border);
  }
  .tab {
    flex: 1;
    text-align: center;
    padding: 14px;
    cursor: pointer;
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    transition: color 0.2s;
  }
  .tab.active { color: var(--accent); }
  .tab:hover { color: var(--text-secondary); }
  .tab-indicator {
    position: absolute;
    bottom: 0;
    left: 0;
    width: 50%;
    height: 2px;
    background: var(--accent);
    transition: left 0.3s ease;
  }
  .tab-indicator.right { left: 50%; }
  .form {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .error {
    font-family: var(--mono);
    font-size: 0.72rem;
    color: var(--red);
    padding: 8px 12px;
    background: rgba(248, 113, 113, 0.06);
    border: 1px solid rgba(248, 113, 113, 0.2);
    border-radius: 4px;
  }
  .submit-btn {
    width: 100%;
    padding: 12px;
    background: var(--accent);
    color: var(--bg);
    font-family: var(--mono);
    font-size: 0.8rem;
    font-weight: 600;
    border: none;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    cursor: pointer;
    transition: all 0.2s;
    margin-top: 4px;
  }
  .submit-btn:hover:not(:disabled) {
    box-shadow: 0 0 20px rgba(232, 255, 90, 0.2);
  }
  .submit-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .submit-btn:active:not(:disabled) {
    transform: scale(0.98);
  }
</style>
