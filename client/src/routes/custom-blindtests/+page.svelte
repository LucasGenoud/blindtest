<script>
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { Plus, Pencil, Trash2 } from 'lucide-svelte';

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
    <button class="btn-primary" onclick={create}><Plus size={14} stroke-width={1.8} /> Create</button>
  </div>

  {#if blindtests.length === 0}
    <div class="empty-state">No blindtests yet</div>
  {/if}

  <div class="bt-list">
    {#each blindtests as bt (bt._id)}
      <div class="bt-item" in:fly={{ y: 8, duration: 200, delay: 40 }}>
        <div class="bt-info">
          <div class="bt-name">{bt.name}</div>
          <div class="bt-count">{bt.blindtestList.length} audios</div>
        </div>
        <div class="bt-actions">
          <button class="btn-primary" onclick={() => goto(`/custom-blindtests/${bt._id}`)}><Pencil size={13} stroke-width={1.8} /> Edit</button>
          <button class="btn-danger" onclick={() => deleteBt(bt._id)}><Trash2 size={13} stroke-width={1.8} /></button>
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
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
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
  .bt-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bt-item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    padding: 18px 22px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: box-shadow 0.2s, border-color 0.2s;
  }
  .bt-item:hover {
    border-color: var(--accent-border);
    box-shadow: var(--shadow-lg);
  }
  .bt-info { display: flex; flex-direction: column; gap: 4px; }
  .bt-name {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .bt-count {
    font-size: 0.75rem;
    color: var(--text-dim);
    font-weight: 500;
  }
  .bt-actions { display: flex; gap: 8px; }
</style>
