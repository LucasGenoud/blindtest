<script>
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { volume } from '$lib/stores/gameStore.js';
  import { stringToColor } from '$lib/misc.js';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import LoginPopup from '$lib/components/login/LoginPopup.svelte';
  import { Volume2, LogOut } from 'lucide-svelte';

  let showLogin = $state(false);
  let showProfile = $state(false);

  function changeVolume(e) {
    $volume = parseInt(e.target.value);
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

  const links = $derived([
    { label: 'Play', path: '/' },
    { label: 'Canvas', path: '/canvas' },
    ...($token ? [{ label: 'Blindtests', path: '/custom-blindtests' }] : []),
  ]);
</script>

<!-- One bar: brand flush left, links in text colour, one primary action at the right end. -->
<header class="z-[1000] flex h-14 shrink-0 items-center justify-between gap-6 bg-bg px-8">
  <div class="flex items-center gap-6">
    <button
      class="flex items-baseline gap-2 bg-transparent p-0 text-left"
      onclick={() => goto('/')}
    >
      <span class="text-lg font-extrabold tracking-[-0.02em] text-text-primary">blindtest</span>
      <span class="text-xs font-semibold uppercase tracking-[0.1em] text-accent-ink">v2</span>
    </button>

    <nav class="hide-mobile flex items-center gap-6">
      {#each links as link (link.path)}
        <button
          class="bg-transparent p-0 text-[14px] font-normal {$page.url.pathname === link.path ? 'text-accent' : 'text-text-primary'} hover:text-accent-ink"
          onclick={() => goto(link.path)}
        >
          {link.label}
        </button>
      {/each}
    </nav>
  </div>

  <div class="flex items-center gap-4">
    <div class="hide-mobile flex items-center gap-2">
      <Volume2 size={16} class="text-text-secondary" stroke-width={2} />
      <input
        type="range"
        min="0" max="100"
        value={$volume}
        oninput={changeVolume}
        aria-label="Volume"
        style="width:90px"
      />
    </div>

    {#if !$token}
      <button class="btn-primary" onclick={() => showLogin = true}>
        Sign in
      </button>
    {:else if $user}
      <div class="relative">
        <!-- The avatar is one of the two circular exceptions. -->
        <button
          class="flex h-9 w-9 items-center justify-center rounded-full p-0 text-sm font-extrabold text-bg"
          style="background:{stringToColor($user.name)}"
          onclick={() => showProfile = !showProfile}
          aria-label="Account menu"
        >
          {$user.name?.substring(0,1).toUpperCase()}
        </button>

        {#if showProfile}
          <div class="profile-dropdown absolute right-0 top-[calc(100%+8px)] z-[1001] flex min-w-[200px] flex-col">
            <div class="px-3 py-2 text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary">{$user.name}</div>
            <button class="menu-item" onclick={() => nav('/statistics')}>Statistics</button>
            {#if $userPermission >= 2}
              <button class="menu-item" onclick={() => nav('/manage-audios')}>Manage audios</button>
              <button class="menu-item" onclick={() => nav('/suggestions')}>Suggestions</button>
            {/if}
            <button class="menu-item" onclick={() => nav('/custom-blindtests')}>Create blindtest</button>
            {#if $userPermission >= 3}
              <button class="menu-item" onclick={() => nav('/manage-users')}>Manage users</button>
            {/if}
            <button class="menu-item menu-item-leave" onclick={logOut}>
              <LogOut size={16} stroke-width={2} /> Log out
            </button>
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
    border-bottom: 2px solid var(--divider);
  }

  /* Menus separate by a lighter surface plus a rule, not by a shadow. */
  .profile-dropdown {
    background: var(--surface-2);
    border-top: 2px solid var(--divider);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 400;
    text-align: left;
    transition: color var(--duration-fast) ease-out;
  }

  .menu-item:hover {
    color: var(--accent-ink);
  }

  .menu-item-leave {
    color: var(--signal-wrong);
    border-top: 2px solid var(--divider);
    margin-top: 4px;
  }
</style>
