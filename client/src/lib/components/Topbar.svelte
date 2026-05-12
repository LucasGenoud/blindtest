<script>
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { volume } from '$lib/stores/gameStore.js';
  import { theme } from '$lib/stores/themeStore.js';
  import { stringToColor } from '$lib/misc.js';
  import { goto } from '$app/navigation';
  import LoginPopup from '$lib/components/login/LoginPopup.svelte';
  import { Music, Volume2, Sun, Moon, BarChart3, Disc3, MessageSquare, Users, LogOut, Palette } from 'lucide-svelte';

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

<header class="z-[1000] flex h-14 items-center justify-between border-b px-5">
  <div class="flex items-center gap-3">
    <div
      class="flex cursor-pointer items-center gap-1.5 no-underline"
      onclick={() => goto('/')}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === 'Enter' && goto('/')}
    >
      <Music size={18} class="text-accent" stroke-width={1.8} />
      <span class="text-base font-bold tracking-[-0.02em] text-text-primary">blindtest</span>
      <span class="rounded-full border px-1.5 py-px text-[0.625rem] font-semibold uppercase tracking-[0.05em] text-accent" style="background: var(--accent-dim); border-color: var(--accent-border);">
        v2
      </span>
    </div>

    <button
      class="rounded-full border border-border bg-bg px-3.5 py-[5px] text-[0.8125rem] font-medium text-text-secondary transition-all duration-200 hover:text-accent hover:[background:var(--accent-dim)] hover:[border-color:var(--accent-border)]"
      onclick={() => goto('/canvas')}
    >
      <Palette size={14} class="shrink-0" stroke-width={1.8} /> Canvas
    </button>
  </div>

  <div class="flex items-center gap-2.5">
    <div class="hide-mobile flex items-center gap-1.5 rounded-full border border-border bg-bg px-3 py-[5px]">
      <Volume2 size={13} class="opacity-60" stroke-width={1.8} />
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
      {$theme === 'dark' ? <Sun size={16} stroke-width={1.8} /> : <Moon size={16} stroke-width={1.8} />}
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
          <div class="profile-dropdown absolute right-0 top-[calc(100%+10px)] z-[1001] flex min-w-[210px] flex-col gap-0.5 rounded-lg border p-2">
            <div class="px-3 py-2 pb-1 text-sm font-semibold text-text-primary">{$user.name}</div>
            <div class="my-1 h-px bg-border"></div>
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/statistics')}><BarChart3 size={15} class="inline-block mr-2" stroke-width={1.8} /> Statistics</button>
            {#if $userPermission >= 2}
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/manage-audios')}><Disc3 size={15} class="inline-block mr-2" stroke-width={1.8} /> Manage audios</button>
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/suggestions')}><MessageSquare size={15} class="inline-block mr-2" stroke-width={1.8} /> Suggestions</button>
            {/if}
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/custom-blindtests')}><Music size={15} class="inline-block mr-2" stroke-width={1.8} /> Create blindtest</button>
            {#if $userPermission >= 3}
              <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-text-secondary transition-[background,color] duration-150 hover:bg-surface-2 hover:text-text-primary" onclick={() => nav('/manage-users')}><Users size={15} class="inline-block mr-2" stroke-width={1.8} /> Manage users</button>
            {/if}
            <div class="my-1 h-px bg-border"></div>
            <button class="w-full rounded-md bg-transparent px-3 py-[9px] text-left text-[0.8125rem] font-medium text-red transition-[background,color] duration-150 hover:text-red hover:[background:rgba(220,38,38,0.06)]" onclick={logOut}><LogOut size={15} class="inline-block mr-2" stroke-width={1.8} /> Log out</button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</header>

{#if showLogin}
  <LoginPopup onclose={() => showLogin = false} />
{/if}

<style>
  header {
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border-bottom-color: var(--glass-border);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.15), 0 4px 20px rgba(0, 0, 0, 0.06);
  }

  .profile-dropdown {
    background: var(--surface);
    border-color: var(--border);
    box-shadow: var(--shadow-lg);
  }
</style>
