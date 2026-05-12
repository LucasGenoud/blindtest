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
    <div class="empty-state">No suggestions yet</div>
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
  .manage-page { padding: 28px 24px; overflow: auto; width: 100%; }
  .page-header {
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 12px;
  }
  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .empty-state {
    font-size: 0.875rem;
    color: var(--text-dim);
    padding: 48px 0;
    text-align: center;
  }
  table { width: 100%; border-collapse: collapse; background: var(--surface); border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--border); box-shadow: var(--shadow-card); }
  th {
    text-align: left;
    padding: 12px 16px;
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    font-size: 0.875rem;
    color: var(--text-secondary);
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
  }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: var(--surface-2); }
  .cat-badge {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--accent);
    background: var(--accent-dim);
    border: 1px solid var(--accent-border);
    padding: 2px 10px;
    border-radius: 9999px;
  }
</style>
