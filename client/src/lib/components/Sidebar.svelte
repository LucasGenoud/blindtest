<script>
  import { goto } from '$app/navigation';
  import { blindtestStatus, numberOfAudios, timeToGuess, timeWithAnswer, showCategory, useSuperflus, prioritizeLessUsedAudios, dataCategories, disabledUsers } from '$lib/stores/gameStore.js';
  import { token } from '$lib/stores/userStore.js';
  import { getApi } from '$lib/api.js';
  import { categoryListKeyLabel } from '$lib/misc.js';
  import { onMount } from 'svelte';

  let contributorUsers = $state([]);
  let selectedDisabled = $state([]);

  $effect(() => {
    $disabledUsers = [...selectedDisabled];
  });

  onMount(async () => {
    try {
      const res = await fetch(`${getApi()}/getcontributorusers`);
      if (res.ok) contributorUsers = await res.json();
    } catch {}
  });

  function getTotal() {
    return Object.values($dataCategories).reduce((a, b) => a + b, 0);
  }

  function updateCat(key, val) {
    $dataCategories = { ...$dataCategories, [key]: val };
  }

  function addAll() {
    $dataCategories = { movies:100, tvshows:100, animes:100, animatedmovies:100, animatedseries:100, games:100, musics:100, internetculture:100, quotes:0 };
  }
  function removeAll() {
    $dataCategories = { movies:0, tvshows:0, animes:0, animatedmovies:0, animatedseries:0, games:0, musics:0, internetculture:0, quotes:0 };
  }

  function startBlindtest() {
    $blindtestStatus = 'started';
    goto('/play');
  }

  function estimatedTime() {
    return Math.round(($timeWithAnswer + $timeToGuess) * $numberOfAudios / 60);
  }
</script>

<div class="sidebar">
  <div class="sidebar-content">
    <div class="section-label">Configuration</div>

    <div class="sidebar-title">Number of guesses</div>
    <input type="number" min="10" bind:value={$numberOfAudios} style="width:120px" />

    <div class="sidebar-title">Time to guess: <span class="val">{$timeToGuess}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeToGuess} />

    <div class="sidebar-title">Time with answer: <span class="val">{$timeWithAnswer}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeWithAnswer} />

    <div class="sidebar-title" style="display:flex;align-items:center">
      <span>Categories</span>
      <button class="btn-primary sm" style="margin-left:auto" onclick={addAll}>All</button>
      <button class="btn-primary sm" style="margin-left:4px" onclick={removeAll}>None</button>
    </div>

    {#each categoryListKeyLabel as cat}
      {@const total = getTotal()}
      {@const pct = total > 0 ? (($dataCategories[cat.key] || 0) / total * 100).toFixed(1) : '0.0'}
      {@const approx = total > 0 ? Math.floor($numberOfAudios * ($dataCategories[cat.key] || 0) / total) : 0}
      <div>
        <div class="cat-row">
          <span class="cat-name">{cat.label}</span>
          <span class="cat-stats">{pct}% ~{approx}</span>
        </div>
        <input type="range" min="0" max="100" value={$dataCategories[cat.key] || 0}
          oninput={(e) => updateCat(cat.key, parseInt(e.target.value))} />
      </div>
    {/each}

    <div class="sidebar-title">Show category</div>
    <label class="toggle" class:active={$showCategory}>
      <input type="checkbox" bind:checked={$showCategory} />
      {$showCategory ? 'Show' : 'Hide'}
    </label>

    <div class="sidebar-title">Use superflus</div>
    <label class="toggle" class:active={$useSuperflus}>
      <input type="checkbox" bind:checked={$useSuperflus} />
      {$useSuperflus ? 'Use' : 'Discard'}
    </label>

    <div class="sidebar-title">Prioritize less used</div>
    <label class="toggle" class:active={$prioritizeLessUsedAudios}>
      <input type="checkbox" bind:checked={$prioritizeLessUsedAudios} />
      {$prioritizeLessUsedAudios ? 'Prioritize' : 'Random'}
    </label>
  </div>

  <div class="sidebar-footer">
    <div class="footer-meta">
      ~{estimatedTime()} min estimated
    </div>
    <button class="start-btn"
      disabled={!getTotal()}
      onclick={startBlindtest}>
      Start blindtest
    </button>
  </div>
</div>

<style>
  .sidebar {
    width: 300px; flex-shrink: 0;
    height: 100%; display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    background: var(--surface);
  }
  .sidebar-content {
    overflow: auto; padding: 20px 16px;
    display: flex; flex-direction: column; gap: 6px;
  }
  .section-label {
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 12px;
  }
  .sidebar-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-top: 18px;
    margin-bottom: 6px;
    display: flex;
    align-items: center;
  }
  .sidebar-title:first-child { margin-top: 0; }
  .sidebar-title .val {
    color: var(--accent);
    font-weight: 700;
    margin-left: 4px;
  }
  .cat-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }
  .cat-name {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
  .cat-stats {
    font-size: 0.7rem;
    color: var(--text-dim);
    font-family: var(--mono);
  }
  .sidebar-footer {
    padding: 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--surface-2);
  }
  .footer-meta {
    font-size: 0.75rem;
    color: var(--text-dim);
    text-align: center;
  }
  .start-btn {
    width: 100%;
    padding: 12px;
    background: var(--accent);
    color: #ffffff;
    font-family: var(--sans);
    font-size: 0.9rem;
    font-weight: 600;
    border: none;
    border-radius: var(--radius-md);
    letter-spacing: 0.01em;
    transition: all 0.2s;
    box-shadow: 0 2px 8px var(--accent-dim);
  }
  .start-btn:hover:not(:disabled) {
    background: var(--accent-hover);
    box-shadow: 0 4px 14px var(--accent-dim);
    transform: translateY(-1px);
  }
  .start-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    box-shadow: none;
  }
  .start-btn:active:not(:disabled) {
    transform: scale(0.98);
  }
  :global(.btn-primary.sm) {
    font-size: 0.7rem;
    padding: 3px 10px;
  }
  @media screen and (max-width: 700px) {
    .sidebar { width: 100%; }
  }
</style>
