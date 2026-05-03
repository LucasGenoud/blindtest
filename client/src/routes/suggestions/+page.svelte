<script>
  import { onMount } from 'svelte';
  import { getApi } from '$lib/api.js';

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
          <tr>
            <td><span class="cat-badge">{s.category}</span></td>
            <td>{s.answer}</td>
            <td><a href={s.videoUrl} target="_blank">Link</a></td>
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
  .empty-state {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 40px 0;
    text-align: center;
  }
  table { width: 100%; border-collapse: collapse; }
  th {
    text-align: left;
    padding: 10px 12px;
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--text-dim);
  }
  tr:hover { background: var(--surface); }
  .cat-badge {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-secondary);
    background: var(--surface-2);
    padding: 2px 8px;
    border-radius: 3px;
    letter-spacing: 0.04em;
  }
</style>
