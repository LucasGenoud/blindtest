<script>
  import { onMount } from 'svelte';
  import { getApi } from '$lib/api.js';

  let stats = $state(null);
  onMount(async () => {
    try {
      const res = await fetch(`${getApi()}/getBlindtestStats`);
      if (res.ok) stats = await res.json();
    } catch {}
  });
</script>

<svelte:head><title>Statistics — Blindtest</title></svelte:head>

<div class="stats-page">
  <div class="page-header">
    <h1>Statistics</h1>
  </div>
  {#if stats}
    <div class="metrics-grid">
      <div class="metric-cell">
        <div class="metric-label">Total Audios</div>
        <div class="metric-value highlight">{stats.totalAudios}</div>
      </div>
      <div class="metric-cell">
        <div class="metric-label">Total Users</div>
        <div class="metric-value">{stats.totalUsers}</div>
      </div>
      <div class="metric-cell">
        <div class="metric-label">Total Plays</div>
        <div class="metric-value">{stats.totalPlays}</div>
      </div>
    </div>
  {:else}
    <div class="loading-state">Loading...</div>
  {/if}
</div>

<style>
  .stats-page { padding: 28px 24px; overflow: auto; width: 100%; }
  .page-header {
    margin-bottom: 20px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    font-family: var(--mono);
    font-size: 1rem;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }
  .metric-cell {
    background: var(--surface);
    padding: 24px 28px;
    transition: background 0.15s;
  }
  .metric-cell:hover { background: var(--surface-2); }
  .metric-label {
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 10px;
  }
  .metric-value {
    font-family: var(--mono);
    font-size: 1.4rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .metric-value.highlight { color: var(--accent); }
  .loading-state {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  @media (max-width: 700px) {
    .metrics-grid { grid-template-columns: 1fr; }
  }
</style>
