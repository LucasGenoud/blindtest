<script>
  import { onMount } from 'svelte';
  import { getApi } from '$lib/api.js';
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';

  let blindtests = $state([]);

  onMount(async () => {
    if (!$token) { goto('/'); return; }
    const res = await fetch(`${getApi()}/getcustomblindtests`, { headers: { Authorization: $token } });
    if (res.ok) blindtests = await res.json();
  });

  async function create() {
    const name = prompt('Enter blindtest name:');
    if (!name) return;
    const res = await fetch(`${getApi()}/createcustomblindtest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ name }),
    });
    if (res.ok) {
      const data = await res.json();
      goto(`/custom-blindtests/${data._id}`);
    }
  }

  async function deleteBt(id) {
    if (!confirm('Delete this blindtest?')) return;
    await fetch(`${getApi()}/deletecustomblindtest/${id}`, {
      method: 'DELETE',
      headers: { Authorization: $token },
    });
    blindtests = blindtests.filter(b => b._id !== id);
  }
</script>

<svelte:head><title>Custom Blindtests — Blindtest</title></svelte:head>

<div class="manage-page">
  <div class="page-header">
    <h1>My Custom Blindtests</h1>
    <button class="btn-primary" onclick={create}>+ Create</button>
  </div>

  {#if blindtests.length === 0}
    <div class="empty-state">No blindtests yet</div>
  {/if}

  <div class="bt-list">
    {#each blindtests as bt (bt._id)}
      <div class="bt-item">
        <div class="bt-info">
          <div class="bt-name">{bt.name}</div>
          <div class="bt-count">{bt.blindtestList.length} audios</div>
        </div>
        <div class="bt-actions">
          <button class="btn-primary" onclick={() => goto(`/custom-blindtests/${bt._id}`)}>✏ Edit</button>
          <button class="btn-danger" onclick={() => deleteBt(bt._id)}>🗑</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .manage-page { padding: 28px 24px; overflow: auto; width: 100%; }
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
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
  .bt-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }
  .bt-item {
    background: var(--surface);
    padding: 18px 22px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: background 0.15s;
  }
  .bt-item:hover { background: var(--surface-2); }
  .bt-info { display: flex; flex-direction: column; gap: 4px; }
  .bt-name {
    font-family: var(--mono);
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
  }
  .bt-count {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .bt-actions { display: flex; gap: 6px; }
</style>
