<script>
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { volume } from '$lib/stores/gameStore.js';
  import { theme } from '$lib/stores/themeStore.js';
  import { stringToColor } from '$lib/misc.js';
  import { goto } from '$app/navigation';
  import LoginPopup from '$lib/components/login/LoginPopup.svelte';

  let showLogin = $state(false);
  let showProfile = $state(false);

  function changeVolume(e) {
    $volume = parseInt(e.target.value);
  }

  function toggleTheme() {
    $theme = $theme === 'dark' ? 'light' : 'dark';
  }

  function logOut() {
    $token = '';
    $user = null;
    showProfile = false;
  }

  function nav(path) {
    goto(path);
    showProfile = false;
  }
</script>

<header class="z-[1000] flex h-14 items-center justify-between border-b border-border bg-surface px-5 shadow-card">
  <div class="flex items-center gap-3">
    <div
      class="flex cursor-pointer items-center gap-1.5 no-underline"
      onclick={() => goto('/')}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === 'Enter' && goto('/')}
    >
      <span class="text-[1.2rem] text-accent">♪</span>
      <span class="text-base font-bold tracking-[-0.02em] text-text-primary">blindtest</span>
      <span class="rounded-full border px-1.5 py-px text-[0.625rem] font-semibold uppercase tracking-[0.05em] text-accent" style="background: var(--accent-dim); border-color: var(--accent-border);">
        v2
      </span>
    </div>

    <button
      class="rounded-full border border-border bg-bg px-3.5 py-[5px] text-[0.8125rem] font-medium text-text-secondary transition-all duration-200 hover:text-accent hover:[background:var(--accent-dim)] hover:[border-color:var(--accent-border)]"
      onclick={() => goto('/canvas')}
    >
      Canvas
    </button>
  </div>

  <div class="flex items-center gap-2.5">
    <div class="hide-mobile flex items-center gap-1.5 rounded-full border border-border bg-bg px-3 py-[5px]">
      <span class="text-[13px] opacity-60">🔊</span>
      <input
        type="range"
        min="0" max="100"
        value={$volume}
        oninput={changeVolume}
        style="width:90px"
      />
    </div>

    <button
      class="flex h-[34px] w-[34px] items-center justify-center rounded-md border border-border bg-surface-2 p-0 text-base text-text-secondary transition-all duration-200 hover:text-accent hover:[background:var(--accent-dim)] hover:[border-color:var(--accent-border)]"
      onclick={toggleTheme}
      title="Toggle theme"
    >
      {$theme === 'dark' ? '☀' : '☾'}
    </button>

    {#if !$token}
      <button class="btn-primary" onclick={() => showLogin = true}>
        Sign in
      </button>
    {:else if $user}
      <div class="relative">
        <div
          class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md text-[0.8rem] font-bold text-white transition-[opacity,box-shadow] duration-150 hover:opacity-90 hover:[box-shadow:0_0_0_3px_var(--accent-dim)]"
          style="background:{stringToColor($user.name)}"
          onclick={() => showProfile = !showProfile}
        >
          {$user.name?.substring(0,1).toUpperCase()}
        </div>

        {#if showProfile}
          <div class="absolute right-0 top-[calc(100%+10px)] z-[1001] flex min-w-[210px] flex-col gap-0.5 rounded-lg border border-border bg-surface p-2 shadow-panel">
            <div class="px-3 py-2 pb-1 text-sm font-semibold text-text-primary">{$user.name}</div>
            <div class="my-1 h-px bg-border"></div>
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/statistics')}>📊 Statistics</button>
            {#if $userPermission >= 2}
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/manage-audios')}>🎵 Manage audios</button>
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/suggestions')}>💬 Suggestions</button>
            {/if}
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/custom-blindtests')}>🎶 Create blindtest</button>
            {#if $userPermission >= 3}
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/manage-users')}>👥 Manage users</button>
            {/if}
            <div class="my-1 h-px bg-border"></div>
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-red transition-[background,color] duration-150 hover:text-red hover:[background:rgba(220,38,38,0.06)]" onclick={logOut}>🚪 Log out</button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</header>

{#if showLogin}
  <LoginPopup onclose={() => showLogin = false} />
{/if}
