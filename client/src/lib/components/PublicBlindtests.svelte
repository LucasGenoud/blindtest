<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getApi } from '$lib/api.js';
  import { blindtestStatus } from '$lib/stores/gameStore.js';

  let blindtests = $state([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      const res = await fetch(`${getApi()}/getpubliccustomblindtests`);
      if (res.ok) blindtests = await res.json();
    } catch {
      // The empty state below covers it.
    }
    loading = false;
  });

  function play(bt, random = false) {
    $blindtestStatus = 'started';
    goto(`/play/${bt._id}${random ? '?random=true' : ''}`);
  }
</script>

<div class="public-container">
  <div class="section-header">
    <h2 class="section-title">Public blindtests</h2>
    <span class="section-count tabular">{blindtests.length}</span>
  </div>

  {#if loading}
    <div class="loading-region">
      <div class="loading-line"></div>
    </div>
  {:else if blindtests.length === 0}
    <div class="empty-state">
      <h2>No public blindtests yet</h2>
      <p>Anyone can build one from the clip library and share it here.</p>
    </div>
  {:else}
    <!-- A small set of distinct, clickable objects: cards are right here. -->
    <div class="public-list">
      {#each blindtests as bt (bt._id)}
        <div class="card public-item">
          <div class="item-header">
            <h3 class="item-title">{bt.name}</h3>
            <span class="item-count tabular">{bt.blindtestList.length}</span>
          </div>
          <span class="item-user">{bt.username || 'Unknown'}</span>
          <div class="item-actions">
            <button class="btn-ghost sm" onclick={() => play(bt)}>Play</button>
            <button class="btn-ghost sm" onclick={() => play(bt, true)}>Shuffle</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .public-container {
    overflow: auto;
    flex: 1;
    padding: 32px;
  }

  .section-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 12px;
    margin-bottom: 24px;
    border-bottom: 2px solid var(--divider);
  }

  .section-title {
    font-size: 32px;
    font-weight: 800;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }

  .section-count {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .public-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }

  .public-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .item-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .item-title {
    font-size: 20px;
    font-weight: 800;
    color: var(--text-primary);
    line-height: 1.1;
  }

  .item-count {
    font-size: 13px;
    color: var(--accent-ink);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .item-user {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .item-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .loading-region {
    position: relative;
    height: 2px;
  }

  @media (max-width: 760px) {
    .public-container { padding: 16px; }
    .section-title { font-size: 20px; }
  }
</style>
