<script>
  import { goto } from '$app/navigation';
  import { blindtestStatus, numberOfAudios, timeToGuess, timeWithAnswer, showCategory, useSuperflus, prioritizeLessUsedAudios, dataCategories, disabledUsers } from '$lib/stores/gameStore.js';
  import { token } from '$lib/stores/userStore.js';
  import { getApi } from '$lib/api.js';
  import { categoryListKeyLabel } from '$lib/misc.js';
  import { onMount } from 'svelte';
  import { Play } from 'lucide-svelte';

  let contributorUsers = $state([]);
  let selectedDisabled = $state([]);
  let dropdownOpen = $state(false);
  let dropdownEl = $state(null);

  $effect(() => {
    $disabledUsers = [...selectedDisabled];
  });

  function toggleContributor(id) {
    if (selectedDisabled.includes(id)) {
      selectedDisabled = selectedDisabled.filter(x => x !== id);
    } else {
      selectedDisabled = [...selectedDisabled, id];
    }
  }

  function handleClickOutside(e) {
    if (dropdownEl && !dropdownEl.contains(e.target)) {
      dropdownOpen = false;
    }
  }

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

<svelte:window onclick={(e) => { if (dropdownOpen) handleClickOutside(e); }} />

<div class="sidebar-panel flex h-full w-full shrink-0 flex-col md:w-[300px] md:max-w-[300px]">
  <div class="flex flex-1 flex-col gap-1.5 overflow-auto px-4 py-5">
    <div class="section-label">Configuration</div>

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Number of guesses</div>
    <input type="number" min="10" bind:value={$numberOfAudios} style="width:120px" />

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Time to guess: <span class="ml-1 font-bold text-accent">{$timeToGuess}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeToGuess} />

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Time with answer: <span class="ml-1 font-bold text-accent">{$timeWithAnswer}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeWithAnswer} />

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">
      <span>Categories</span>
      <button class="btn-secondary sm ml-auto" onclick={addAll}>All</button>
      <button class="btn-secondary sm ml-2" onclick={removeAll}>None</button>
    </div>

    {#each categoryListKeyLabel as cat}
      {@const total = getTotal()}
      {@const pct = total > 0 ? (($dataCategories[cat.key] || 0) / total * 100).toFixed(1) : '0.0'}
      {@const approx = total > 0 ? Math.floor($numberOfAudios * ($dataCategories[cat.key] || 0) / total) : 0}
      <div>
        <div class="mb-1 flex items-center justify-between">
          <span class="text-sm text-text-primary">{cat.label}</span>
          <span class="font-mono text-xs text-text-dim">{pct}% ~{approx}</span>
        </div>
        <input type="range" min="0" max="100" value={$dataCategories[cat.key] || 0}
          oninput={(e) => updateCat(cat.key, parseInt(e.target.value))} />
      </div>
    {/each}

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Show category</div>
    <label class="toggle" class:active={$showCategory}>
      <input type="checkbox" bind:checked={$showCategory} />
      {$showCategory ? 'Show' : 'Hide'}
    </label>

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Use superflus</div>
    <label class="toggle" class:active={$useSuperflus}>
      <input type="checkbox" bind:checked={$useSuperflus} />
      {$useSuperflus ? 'Use' : 'Discard'}
    </label>

    <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Prioritize less used</div>
    <label class="toggle" class:active={$prioritizeLessUsedAudios}>
      <input type="checkbox" bind:checked={$prioritizeLessUsedAudios} />
      {$prioritizeLessUsedAudios ? 'Prioritize' : 'Random'}
    </label>

    {#if contributorUsers.length > 0}
      <div class="mt-6 flex items-center text-xs font-semibold uppercase tracking-[0.1em] text-text-secondary first:mt-0">Exclude contributors</div>
      <div class="contributor-dropdown" bind:this={dropdownEl}>
        <button
          type="button"
          class="contributor-trigger"
          onclick={() => dropdownOpen = !dropdownOpen}
          aria-haspopup="listbox"
          aria-expanded={dropdownOpen}
        >
          <span>
            {#if selectedDisabled.length === 0}
              None excluded
            {:else}
              {selectedDisabled.length} excluded
            {/if}
          </span>
          <svg class="chevron" class:open={dropdownOpen} width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M2 4l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        {#if dropdownOpen}
          <ul class="contributor-menu" role="listbox" aria-multiselectable="true">
            {#each contributorUsers as u}
              {@const checked = selectedDisabled.includes(u._id)}
              <li role="option" aria-selected={checked}>
                <label class="contributor-option" class:selected={checked}>
                  <input
                    type="checkbox"
                    checked={checked}
                    onchange={() => toggleContributor(u._id)}
                  />
                  <span>{u.name}</span>
                </label>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>

  <div class="sidebar-footer flex flex-col gap-3 p-4">
    <div class="text-xs font-semibold uppercase tracking-[0.1em] text-text-dim">
      <span class="tabular">~{estimatedTime()}</span> min estimated
    </div>
    <button class="btn-primary w-full" disabled={!getTotal()} onclick={startBlindtest}>
      <Play size={16} stroke-width={2} /> Start blindtest
    </button>
  </div>
</div>

<style>
  .sidebar-panel {
    background: var(--bg);
    border-right: 2px solid var(--divider);
  }

  .sidebar-footer {
    border-top: 2px solid var(--divider);
  }

  .contributor-dropdown {
    position: relative;
    width: 100%;
  }

  .contributor-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: 36px;
    padding: 8px 12px;
    background: var(--surface);
    border: 1px solid var(--divider);
    border-radius: 0;
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 400;
    cursor: pointer;
    text-align: left;
    transition: border-color var(--duration-fast) ease-out;
  }

  .contributor-trigger:hover,
  .contributor-trigger:focus-visible {
    border-color: var(--accent);
    outline: none;
  }

  .chevron {
    flex-shrink: 0;
    transition: transform var(--duration-fast) ease-out;
    color: var(--text-dim);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .contributor-menu {
    position: absolute;
    z-index: 50;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--surface-2);
    border-top: 2px solid var(--divider);
    border-radius: 0;
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .contributor-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 0;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
    transition: color var(--duration-fast) ease-out;
    user-select: none;
    width: 100%;
  }

  .contributor-option:hover {
    color: var(--accent-ink);
  }

  .contributor-option.selected {
    color: var(--accent);
  }

  .contributor-option input[type="checkbox"] {
    accent-color: var(--accent);
    width: 13px;
    height: 13px;
    min-height: 0;
    cursor: pointer;
    flex-shrink: 0;
  }
</style>
