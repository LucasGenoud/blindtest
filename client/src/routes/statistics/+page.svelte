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
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }
  .metric-cell {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    padding: 28px 24px;
    transition: box-shadow 0.2s, transform 0.2s;
  }
  .metric-cell:hover {
    box-shadow: var(--shadow-lg);
    transform: translateY(-1px);
  }
  .metric-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    margin-bottom: 12px;
  }
  .metric-value {
    font-family: var(--mono);
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.03em;
  }
  .metric-value.highlight { color: var(--accent); }
  .loading-state {
    font-size: 0.875rem;
    color: var(--text-dim);
    padding: 48px 0;
    text-align: center;
  }
  @media (max-width: 700px) {
    .metrics-grid { grid-template-columns: 1fr; }
  }
</style>
