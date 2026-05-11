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

<div class="flex h-full w-full shrink-0 flex-col border-r border-border bg-surface md:w-[300px] md:max-w-[300px]">
  <div class="flex flex-1 flex-col gap-1.5 overflow-auto px-4 py-5">
    <div class="section-label">Configuration</div>

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Number of guesses</div>
    <input type="number" min="10" bind:value={$numberOfAudios} style="width:120px" />

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Time to guess: <span class="ml-1 font-bold text-accent">{$timeToGuess}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeToGuess} />

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Time with answer: <span class="ml-1 font-bold text-accent">{$timeWithAnswer}s</span></div>
    <input type="range" min="5" max="30" bind:value={$timeWithAnswer} />

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">
      <span>Categories</span>
      <button class="btn-primary ml-auto px-2.5 py-[3px] text-[0.7rem]" onclick={addAll}>All</button>
      <button class="btn-primary ml-1 px-2.5 py-[3px] text-[0.7rem]" onclick={removeAll}>None</button>
    </div>

    {#each categoryListKeyLabel as cat}
      {@const total = getTotal()}
      {@const pct = total > 0 ? (($dataCategories[cat.key] || 0) / total * 100).toFixed(1) : '0.0'}
      {@const approx = total > 0 ? Math.floor($numberOfAudios * ($dataCategories[cat.key] || 0) / total) : 0}
      <div>
        <div class="mb-1 flex items-center justify-between">
          <span class="text-[0.8rem] font-medium text-text-secondary">{cat.label}</span>
          <span class="font-mono text-[0.7rem] text-text-dim">{pct}% ~{approx}</span>
        </div>
        <input type="range" min="0" max="100" value={$dataCategories[cat.key] || 0}
          oninput={(e) => updateCat(cat.key, parseInt(e.target.value))} />
      </div>
    {/each}

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Show category</div>
    <label class="toggle" class:active={$showCategory}>
      <input type="checkbox" bind:checked={$showCategory} />
      {$showCategory ? 'Show' : 'Hide'}
    </label>

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Use superflus</div>
    <label class="toggle" class:active={$useSuperflus}>
      <input type="checkbox" bind:checked={$useSuperflus} />
      {$useSuperflus ? 'Use' : 'Discard'}
    </label>

    <div class="mt-4 flex items-center text-xs font-semibold text-text-secondary first:mt-0">Prioritize less used</div>
    <label class="toggle" class:active={$prioritizeLessUsedAudios}>
      <input type="checkbox" bind:checked={$prioritizeLessUsedAudios} />
      {$prioritizeLessUsedAudios ? 'Prioritize' : 'Random'}
    </label>
  </div>

  <div class="flex flex-col gap-3 border-t border-border bg-surface-2 p-4">
    <div class="text-center text-xs text-text-dim">
      ~{estimatedTime()} min estimated
    </div>
    <button class="w-full rounded-md px-3 py-3 text-[0.9rem] font-semibold tracking-[0.01em] text-white shadow-[0_2px_8px_var(--accent-dim)] transition-all duration-200 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-40 disabled:shadow-none [background:var(--accent)] hover:-translate-y-px hover:[background:var(--accent-hover)] hover:[box-shadow:0_4px_14px_var(--accent-dim)] disabled:hover:translate-y-0 disabled:hover:[background:var(--accent)] disabled:hover:[box-shadow:none]"
      disabled={!getTotal()}
      onclick={startBlindtest}>
      Start blindtest
    </button>
  </div>
</div>
