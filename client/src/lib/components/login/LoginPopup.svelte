<script>
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { api } from '$lib/api.js';
  import { connectWebSocket } from '$lib/websocket.js';
  import { X } from 'lucide-svelte';

  let { onclose } = $props();
  let mode = $state('signin');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);
  let emailInput;

  async function signin() {
    error = '';
    if (!email || !password) { error = 'Fill all fields'; return; }
    loading = true;
    try {
      // Signing in is the one call with no token to send yet.
      const data = await api.post('/signin', { email, password }, { auth: false });
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
    if (!emailInput.checkValidity()) { error = 'Invalid email'; return; }
    if (password.length < 6) { error = 'Password must be at least 6 characters'; return; }
    loading = true;
    try {
      await api.post('/signup', { email, password, name }, { auth: false });
      mode = 'signin';
      error = '';
    } catch (e) {
      error = e.message;
    }
    loading = false;
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<div class="popup-overlay">
  <div class="popup-box" role="dialog" aria-modal="true" aria-label="Account">
    <button class="btn-circle absolute right-2 top-2 z-10" onclick={onclose} aria-label="Close"><X size={16} /></button>
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
        <input bind:this={emailInput} bind:value={email} type="email" required />
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
