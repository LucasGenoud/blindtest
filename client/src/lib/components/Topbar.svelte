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

<header class="topbar">
  <div class="topbar-left">
    <div class="topbar-brand" onclick={() => goto('/')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && goto('/')}>
      <span class="brand-icon">♪</span>
      <span class="brand-name">blindtest</span>
      <span class="brand-badge">v2</span>
    </div>
    <button class="nav-pill" onclick={() => goto('/canvas')}>
      Canvas
    </button>
  </div>

  <div class="topbar-right">
    <div class="volume-wrap hide-mobile">
      <span class="vol-icon">🔊</span>
      <input
        type="range"
        min="0" max="100"
        value={$volume}
        oninput={changeVolume}
        style="width:90px"
      />
    </div>

    <button class="theme-toggle" onclick={toggleTheme} title="Toggle theme">
      {$theme === 'dark' ? '☀' : '☾'}
    </button>

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
            <div class="profile-name">{$user.name}</div>
            <div class="popup-divider"></div>
            <button class="profile-btn" onclick={() => nav('/statistics')}>📊 Statistics</button>
            {#if $userPermission >= 2}
              <button class="profile-btn" onclick={() => nav('/manage-audios')}>🎵 Manage audios</button>
              <button class="profile-btn" onclick={() => nav('/suggestions')}>💬 Suggestions</button>
            {/if}
            <button class="profile-btn" onclick={() => nav('/custom-blindtests')}>🎶 Create blindtest</button>
            {#if $userPermission >= 3}
              <button class="profile-btn" onclick={() => nav('/manage-users')}>👥 Manage users</button>
            {/if}
            <div class="popup-divider"></div>
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
    padding: 0 20px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface);
    z-index: 1000;
    box-shadow: var(--shadow-card);
  }
  .topbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .topbar-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .topbar-brand {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    text-decoration: none;
  }
  .brand-icon {
    font-size: 1.2rem;
    color: var(--accent);
  }
  .brand-name {
    font-family: var(--sans);
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .brand-badge {
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--accent);
    background: var(--accent-dim);
    border: 1px solid var(--accent-border);
    padding: 1px 6px;
    border-radius: 9999px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .nav-pill {
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 0.8125rem;
    font-weight: 500;
    padding: 5px 14px;
    border-radius: 9999px;
    cursor: pointer;
    transition: all 0.2s;
  }
  .nav-pill:hover {
    background: var(--accent-dim);
    border-color: var(--accent-border);
    color: var(--accent);
  }
  .volume-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 5px 12px;
    border-radius: 9999px;
  }
  .vol-icon {
    font-size: 13px;
    opacity: 0.6;
  }
  .theme-toggle {
    width: 34px;
    height: 34px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-secondary);
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    padding: 0;
  }
  .theme-toggle:hover {
    border-color: var(--accent-border);
    background: var(--accent-dim);
    color: var(--accent);
  }
  .profile-bubble {
    width: 32px; height: 32px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center; justify-content: center;
    color: white; font-weight: 700;
    cursor: pointer;
    font-size: 0.8rem;
    transition: opacity 0.15s, box-shadow 0.15s;
  }
  .profile-bubble:hover {
    opacity: 0.9;
    box-shadow: 0 0 0 3px var(--accent-dim);
  }
  .profile-popup {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 210px;
    z-index: 1001;
  }
  .profile-name {
    padding: 8px 12px 4px;
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text-primary);
  }
  .popup-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
  .profile-btn {
    width: 100%;
    text-align: left;
    padding: 9px 12px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-family: var(--sans);
    font-size: 0.8125rem;
    font-weight: 500;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .profile-btn:hover {
    background: var(--surface-2);
    color: var(--text-primary);
  }
  .profile-btn.danger {
    color: var(--red);
  }
  .profile-btn.danger:hover {
    background: rgba(220, 38, 38, 0.06);
  }

  @media (max-width: 600px) {
    .hide-mobile { display: none; }
  }
</style>
