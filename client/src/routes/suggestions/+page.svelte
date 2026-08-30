<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { ExternalLink } from 'lucide-svelte';

  let suggestions = $state([]);

  onMount(async () => {
    const res = await fetch(`${getApi()}/getSuggestions`);
    if (res.ok) suggestions = await res.json();
  });
</script>

<svelte:head><title>Suggestions — Blindtest</title></svelte:head>

<div class="manage-page">
  <div class="page-header">
    <h1>Suggestions</h1>
  </div>
  {#if suggestions.length === 0}
    <div class="empty-state">
      <h2>No suggestions yet</h2>
      <p>Clips submitted by players appear here for a contributor to review.</p>
    </div>
  {:else}
    <table>
      <thead><tr><th>Category</th><th>Answer</th><th>URL</th><th>Submitted by</th><th>Date</th></tr></thead>
      <tbody>
        {#each suggestions as s (s._id)}
          <tr transition:fade={{ duration: 200 }}>
            <td><span class="cat-badge">{s.category}</span></td>
            <td>{s.answer}</td>
            <td><a href={s.videoUrl} target="_blank">Link <ExternalLink size={11} stroke-width={1.8} /></a></td>
            <td>{s.submittedByUsername || 'Unknown'}</td>
            <td class="mono">{new Date(s.addedDate).toLocaleDateString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .manage-page { padding: 32px; overflow: auto; width: 100%; }
  .page-header {
    margin-bottom: 24px;
    padding-bottom: 12px;
    border-bottom: 2px solid var(--divider);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h1 {
    font-size: 32px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  /* Same skeleton every time: heading, one line, one action. Flush left. */
  .empty-state {
    padding: 32px 0;
    text-align: left;
  }
  /* A ruled list, not a filled panel. */
  table {
    width: 100%;
    border-collapse: collapse;
    background: transparent;
    border-radius: 0;
    border: 0;
  }
  th {
    text-align: left;
    padding: 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    background: transparent;
    border-bottom: 2px solid var(--divider);
  }
  td {
    padding: 8px;
    border-bottom: 1px solid var(--divider);
    font-size: 13px;
    color: var(--text-secondary);
  }
  td.mono {
    font-size: 13px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }
  tr:hover td { background: rgba(245, 241, 232, 0.06); }
  .cat-badge {
    font-size: 13px;
    color: var(--text-dim);
  }
</style>
