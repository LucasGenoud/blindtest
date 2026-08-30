<script>
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { api, apiTry } from '$lib/api.js';
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { Plus } from 'lucide-svelte';

  let blindtests = $state([]);

  onMount(async () => {
    if (!$token) { goto('/'); return; }
    blindtests = await apiTry(api.get('/getcustomblindtests'), []);
  });

  async function create() {
    const name = prompt('Enter blindtest name:');
    if (!name) return;
    const created = await apiTry(api.post('/createcustomblindtest', { name }));
    if (created) goto(`/custom-blindtests/${created._id}`);
  }

  async function deleteBt(id) {
    if (!confirm('Delete this blindtest?')) return;
    await apiTry(api.del(`/deletecustomblindtest/${id}`));
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
    <div class="empty-state">
      <h2>No blindtests yet</h2>
      <p>Build one from the clip library and choose whether to share it.</p>
    </div>
  {/if}

  <div class="bt-list">
    {#each blindtests as bt (bt._id)}
      <div class="bt-item" in:fly={{ y: 8, duration: 200, delay: 40 }}>
        <div class="bt-info">
          <div class="bt-name">{bt.name}</div>
          <div class="bt-count">{bt.blindtestList.length} audios</div>
        </div>
        <div class="bt-actions">
          <button class="btn-secondary sm" onclick={() => goto(`/custom-blindtests/${bt._id}`)}>Edit</button>
          <button class="btn-danger sm" onclick={() => deleteBt(bt._id)}>Delete</button>
        </div>
      </div>
    {/each}
  </div>
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
  /* Rows of the same kind of thing: a ruled list. */
  .bt-list {
    display: flex;
    flex-direction: column;
  }
  .bt-item {
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--divider);
    border-radius: 0;
    padding: 12px 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    transition: background-color var(--duration-fast) ease-out;
  }
  .bt-item:hover {
    background: var(--row-hover);
  }
  .bt-info { display: flex; flex-direction: column; gap: 4px; }
  .bt-name {
    font-size: 20px;
    font-weight: 800;
    color: var(--text-primary);
    line-height: 1.1;
  }
  .bt-count {
    font-size: 13px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }
  .bt-actions { display: flex; gap: 8px; align-items: center; }
</style>
