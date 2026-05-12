<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getApi } from '$lib/api.js';
  import { blindtestStatus } from '$lib/stores/gameStore.js';
  import { Shuffle, Play } from 'lucide-svelte';

  let blindtests = $state([]);

  onMount(async () => {
    try {
      const res = await fetch(`${getApi()}/getpubliccustomblindtests`);
      if (res.ok) blindtests = await res.json();
    } catch {}
  });

  function play(bt, random = false) {
    $blindtestStatus = 'started';
    goto(`/play/${bt._id}${random ? '?random=true' : ''}`);
  }
</script>

<div class="public-container">
  {#if blindtests.length > 0}
    <div class="section-header">
      <div class="section-title">Public Blindtests</div>
      <div class="section-count">{blindtests.length} available</div>
    </div>
  {/if}
  <div class="public-list">
    {#each blindtests as bt (bt._id)}
      <div class="public-item">
        <div class="item-header">
          <div class="item-title">{bt.name}</div>
          <div class="item-count">{bt.blindtestList.length}</div>
        </div>
        <div class="item-footer">
          <span class="item-user">{bt.username || 'Unknown'}</span>
          <div class="item-actions">
            <button class="action-btn" title="Shuffle" onclick={() => play(bt, true)}><Shuffle size={15} stroke-width={1.8} /></button>
            <button class="action-btn play" title="Play" onclick={() => play(bt)}><Play size={15} stroke-width={1.8} /></button>
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .public-container { overflow: auto; flex: 1; padding: 24px; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }
  .section-title {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .section-count {
    font-size: 0.75rem;
    color: var(--text-dim);
    background: var(--surface-2);
    border: 1px solid var(--border);
    padding: 2px 10px;
    border-radius: 9999px;
  }
  .public-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }
  .public-item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    transition: box-shadow 0.2s, border-color 0.2s, transform 0.2s;
  }
  .public-item:hover {
    border-color: var(--accent-border);
    box-shadow: var(--shadow-lg);
    transform: translateY(-1px);
  }
  .item-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .item-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.4;
  }
  .item-count {
    font-size: 0.6875rem;
    color: var(--accent);
    background: var(--accent-dim);
    border: 1px solid var(--accent-border);
    padding: 2px 8px;
    border-radius: 9999px;
    white-space: nowrap;
    flex-shrink: 0;
    font-weight: 600;
  }
  .item-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: auto;
  }
  .item-user {
    font-size: 0.75rem;
    color: var(--text-dim);
    font-weight: 500;
  }
  .item-actions { display: flex; gap: 6px; }
  .action-btn {
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    padding: 5px 10px;
    border-radius: var(--radius-md);
    transition: all 0.15s;
    display: flex;
    align-items: center;
  }
  .action-btn:hover {
    border-color: var(--border-2);
    background: var(--border);
  }
  .action-btn.play {
    color: white;
    background: var(--accent);
    border-color: var(--accent);
  }
  .action-btn.play:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
