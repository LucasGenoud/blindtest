<script>
  import { fly } from 'svelte/transition';
  import { Flag, Search } from 'lucide-svelte';
  import { categoryListValueLabel } from '$lib/misc.js';

  let {
    counts,
    status = $bindable('all'),
    category = $bindable('all'),
    superflus = $bindable('all'),
    search = $bindable(''),
    showAddForm = $bindable(false),
    newAudio = $bindable(),
    onadd,
  } = $props();

  const STATUS_FILTERS = [
    { value: 'all', label: 'All', count: 'total' },
    { value: 'ready', label: 'In S3', count: 'ready' },
    { value: 'processing', label: 'Processing', count: 'processing' },
    { value: 'error', label: 'Error', count: 'error' },
  ];
</script>

<div class="stats-bar">
  {#each STATUS_FILTERS as f (f.value)}
    <button class="stat-chip {status === f.value ? 'active' : ''}" onclick={() => (status = f.value)}>
      {f.label} <strong class="tabular">{counts[f.count]}</strong>
    </button>
  {/each}
  {#if counts.flagged > 0}
    <button
      class="stat-chip flag-chip {status === 'flagged' ? 'active' : ''}"
      aria-pressed={status === 'flagged'}
      onclick={() => (status = status === 'flagged' ? 'all' : 'flagged')}
    >
      <Flag size={16} stroke-width={2} /> Flagged <strong class="tabular">{counts.flagged}</strong>
    </button>
  {/if}

  <select class="filter-select" bind:value={category} aria-label="Category">
    <option value="all">All categories</option>
    {#each categoryListValueLabel as c}
      <option value={c.value}>{c.label}</option>
    {/each}
  </select>
  <select class="filter-select" bind:value={superflus} aria-label="Superflus">
    <option value="all">Superflus: all</option>
    <option value="no">Superflus: no</option>
    <option value="yes">Superflus: yes</option>
  </select>
  <div class="search-wrap">
    <Search size={14} stroke-width={1.8} class="search-icon" />
    <input bind:value={search} placeholder="Search answer, category, user…" class="search-input" aria-label="Search audios" />
  </div>
</div>

{#if showAddForm}
  <div class="add-card" in:fly={{ y: -8, duration: 150 }}>
    <div class="add-card-title">New audio</div>
    <div class="form-grid">
      <select bind:value={newAudio.category} aria-label="Category">
        {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
      </select>
      <input bind:value={newAudio.answer} placeholder="Answer (e.g. Inception)" />
      <input bind:value={newAudio.videoUrl} placeholder="YouTube URL" class="url-input" />
      <input type="number" bind:value={newAudio.startTime} placeholder="Start (s)" class="short-input" />
      <label class="toggle" class:active={newAudio.superflus}>
        <input type="checkbox" bind:checked={newAudio.superflus} /> Superflus
      </label>
      <div class="form-actions">
        <button class="btn-primary" onclick={onadd}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Stats bar ── */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    padding: 10px 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
  }

  .stat-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: 0;
    font-size: 13px;
    color: var(--text-primary);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    font-weight: 500;
  }

  .stat-chip:hover { background: var(--surface-2); }

  .stat-chip.active {
    color: var(--accent-ink);
    border-bottom-color: var(--accent);
  }

  .stat-chip strong { font-weight: 700; }

  .flag-chip { color: var(--red); }

  .flag-chip.active { color: var(--red); border-bottom-color: var(--red); }

  .flag-chip strong { color: var(--red); }

  .search-wrap {
    margin-left: auto;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-dim);
    font-size: 15px;
    pointer-events: none;
  }

  .search-input {
    padding-left: 28px;
    width: 220px;
    font-size: 11px;
  }

  /* ── Add card ── */
  .add-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    padding: 18px 20px;
  }

  .add-card-title {
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 12px;
  }

  .form-grid {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }

  .url-input { flex: 1; min-width: 200px; }

  .short-input { width: 90px; }

  .form-actions { display: flex; gap: 6px; }

  /* ── Category / superflus filter selects ── */
  .filter-select {
    font-size: 11px;
    padding: 5px 8px;
    border-radius: 0;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-secondary);
    cursor: pointer;
    height: 30px;
  }
</style>
