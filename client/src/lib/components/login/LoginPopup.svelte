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
    <div class="relative flex border-b border-border px-6">
      <div
        class={`absolute bottom-0 h-0.5 rounded-t-sm transition-[left] duration-300 ${mode === 'signin' ? 'left-6' : 'left-1/2'}`}
        style="width: calc(50% - 24px); background: var(--accent);"
      ></div>
      <div class={`flex-1 cursor-pointer py-4 text-center text-sm transition-colors duration-200 ${mode === 'signin' ? 'font-semibold text-accent' : 'font-medium text-text-dim hover:text-text-secondary'}`} onclick={() => mode = 'signin'}>Sign in</div>
      <div class={`flex-1 cursor-pointer py-4 text-center text-sm transition-colors duration-200 ${mode === 'signup' ? 'font-semibold text-accent' : 'font-medium text-text-dim hover:text-text-secondary'}`} onclick={() => mode = 'signup'}>Sign up</div>
    </div>

    <div class="flex flex-col gap-3 px-6 py-7">
      {#if mode === 'signup'}
        <input bind:value={name} placeholder="Username" />
      {/if}
      <input bind:value={email} placeholder="Email" type="email" />
      <input bind:value={password} placeholder="Password" type="password"
        onkeydown={(e) => e.key === 'Enter' && (mode === 'signin' ? signin() : signup())} />

      {#if error}
        <div class="rounded-md border px-3.5 py-2.5 text-[0.8125rem] text-red" style="background: rgba(220, 38, 38, 0.06); border-color: rgba(220, 38, 38, 0.2);">{error}</div>
      {/if}

      <button class="mt-1 w-full rounded-md px-3 py-3 text-[0.9rem] font-semibold text-white transition-all duration-200 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-40 [background:var(--accent)] hover:-translate-y-px hover:[background:var(--accent-hover)] hover:[box-shadow:0_4px_12px_var(--accent-dim)] disabled:hover:translate-y-0 disabled:hover:[background:var(--accent)] disabled:hover:[box-shadow:none]"
        disabled={loading}
        onclick={() => mode === 'signin' ? signin() : signup()}>
        {mode === 'signin' ? 'Sign in' : 'Sign up'}
      </button>
    </div>
  </div>
</div>
