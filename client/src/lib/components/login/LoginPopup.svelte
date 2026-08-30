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
      <button class="tab" class:active={mode === 'signin'} onclick={() => mode = 'signin'}>Sign in</button>
      <button class="tab" class:active={mode === 'signup'} onclick={() => mode = 'signup'}>Sign up</button>
    </div>

    <div class="flex flex-col gap-4 p-6">
      {#if mode === 'signup'}
        <label class="field">
          <span class="field-label">Username</span>
          <input bind:value={name} placeholder="How other players see you" />
        </label>
      {/if}

      <label class="field">
        <span class="field-label">Email</span>
        <input bind:value={email} type="email" />
      </label>

      <label class="field">
        <span class="field-label">Password</span>
        <input bind:value={password} type="password"
          onkeydown={(e) => e.key === 'Enter' && (mode === 'signin' ? signin() : signup())} />
      </label>

      {#if error}
        <!-- One line saying what happened and what to do, no box, no colour alone. -->
        <p class="field-error">{error}</p>
      {/if}

      <button class="btn-primary w-full"
        disabled={loading}
        onclick={() => mode === 'signin' ? signin() : signup()}>
        {mode === 'signin' ? 'Sign in' : 'Create account'}
      </button>
    </div>
  </div>
</div>

<style>
  .tabs {
    display: flex;
    border-bottom: 2px solid var(--divider);
  }

  .tab {
    flex: 1;
    padding: 16px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 600;
    text-align: center;
    transition: color var(--duration-fast) ease-out;
  }

  .tab:hover { color: var(--text-primary); }

  .tab.active {
    color: var(--accent-ink);
    box-shadow: inset 0 -2px 0 var(--accent);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
  }

  .field-error {
    font-size: 13px;
    color: var(--signal-wrong);
  }
</style>
