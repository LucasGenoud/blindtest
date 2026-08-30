<script>
  import { onMount } from 'svelte';
  import { api, apiTry } from '$lib/api.js';

  let stats = $state(null);
  onMount(async () => {
    stats = await apiTry(api.get('/getBlindtestStats'));
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
    <div class="loading-region"><div class="loading-line"></div></div>
  {/if}
</div>

<style>
  .stats-page { padding: 32px; overflow: auto; width: 100%; }

  .page-header {
    margin-bottom: 24px;
    padding-bottom: 12px;
    border-bottom: 2px solid var(--divider);
  }

  h1 {
    font-size: 32px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 24px;
  }

  /* Grouped by alignment and space, not by a box. */
  .metric-cell {
    background: transparent;
    border: 0;
    border-radius: 0;
    padding: 0;
  }

  .metric-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  .metric-value {
    font-size: 64px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.03em;
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
  }

  /* One accent per screen. */
  .metric-value.highlight { color: var(--accent-text); }

  .loading-region { position: relative; height: 2px; }

  @media (max-width: 760px) {
    .metrics-grid { grid-template-columns: 1fr; gap: 32px; }
    .metric-value { font-size: 32px; }
  }
</style>
