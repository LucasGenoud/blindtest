<script>
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { volume } from '$lib/stores/gameStore.js';
  import { stringToColor } from '$lib/misc.js';
  import { goto } from '$app/navigation';
  import LoginPopup from '$lib/components/login/LoginPopup.svelte';

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
</script>

<header class="topbar">
  <div class="topbar-left">
    <div class="topbar-brand" onclick={() => goto('/')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && goto('/')}>
      <span class="brand-name">blindtest</span>
      <span class="brand-badge">v2</span>
    </div>
    <button class="btn-primary" onclick={() => goto('/canvas')}>
      Canvas
    </button>
  </div>

  <div class="topbar-right">
    <span class="topbar-volume hide-mobile">vol</span>
    <input
      type="range"
      class="hide-mobile"
      min="0" max="100"
      value={$volume}
      oninput={changeVolume}
      style="width:100px"
    />

    {#if !$token}
      <button class="btn-primary" onclick={() => showLogin = true}>
        Sign in
      </button>
    {:else if $user}
      <div style="position:relative">
        <div
          class="profile-bubble"
          style="background:{stringToColor($user.name)}"
          onclick={() => showProfile = !showProfile}
        >
          {$user.name?.substring(0,1).toUpperCase()}
        </div>

        {#if showProfile}
          <div class="profile-popup">
            <button class="profile-btn" onclick={() => nav('/statistics')}>📊 Statistics</button>
            {#if $userPermission >= 2}
              <button class="profile-btn" onclick={() => nav('/manage-audios')}>🎵 Manage audios</button>
              <button class="profile-btn" onclick={() => nav('/suggestions')}>💬 Suggestions</button>
            {/if}
            <button class="profile-btn" onclick={() => nav('/custom-blindtests')}>🎶 Create blindtest</button>
            {#if $userPermission >= 3}
              <button class="profile-btn" onclick={() => nav('/manage-users')}>👥 Manage users</button>
            {/if}
            <button class="profile-btn danger" onclick={logOut}>🚪 Log out</button>
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
  .topbar {
    border-bottom: 1px solid var(--border);
    padding: 12px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
    z-index: 1000;
  }
  .topbar-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .topbar-brand {
    display: flex;
    align-items: baseline;
    gap: 8px;
    cursor: pointer;
  }
  .brand-name {
    font-family: var(--mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .brand-badge {
    font-family: var(--mono);
    font-size: 0.6rem;
    font-weight: 600;
    color: var(--accent);
    background: var(--accent-dim);
    border: 1px solid var(--accent-border);
    padding: 1px 6px;
    border-radius: 3px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .topbar-volume {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .profile-bubble {
    width: 30px; height: 30px;
    border-radius: 6px;
    display: flex;
    align-items: center; justify-content: center;
    color: white; font-weight: 600;
    cursor: pointer;
    font-family: var(--mono);
    font-size: 0.75rem;
    transition: opacity 0.15s;
  }
  .profile-bubble:hover { opacity: 0.85; }
  .profile-popup {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 200px;
    z-index: 1001;
  }
  .profile-btn {
    width: 100%;
    text-align: left;
    padding: 10px 14px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: var(--mono);
    font-size: 0.75rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.15s;
  }
  .profile-btn:hover {
    background: var(--surface-2);
  }
  .profile-btn.danger {
    color: var(--red);
  }
  .profile-btn.danger:hover {
    background: rgba(248, 113, 113, 0.08);
  }
</style>
