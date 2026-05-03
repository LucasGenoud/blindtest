<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getApi } from '$lib/api.js';
  import { blindtestStatus } from '$lib/stores/gameStore.js';

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
    <div class="section-label" style="padding:20px 20px 0">Public blindtests</div>
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
            <button class="action-btn" title="Shuffle" onclick={() => play(bt, true)}>🔀</button>
            <button class="action-btn play" title="Play" onclick={() => play(bt)}>▶</button>
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .public-container { overflow: auto; flex: 1; }
  .public-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    margin: 14px 20px;
  }
  .public-item {
    background: var(--surface);
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    transition: background 0.15s;
  }
  .public-item:hover { background: var(--surface-2); }
  .item-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .item-title {
    font-family: var(--mono);
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }
  .item-count {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-dim);
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 1px 6px;
    border-radius: 3px;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .item-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: auto;
  }
  .item-user {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .item-actions { display: flex; gap: 4px; }
  .action-btn {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.15s;
    display: flex;
    align-items: center;
  }
  .action-btn:hover {
    border-color: var(--border-2);
    background: var(--bg);
  }
  .action-btn.play {
    color: var(--green);
    border-color: rgba(74, 222, 128, 0.3);
  }
  .action-btn.play:hover {
    background: rgba(74, 222, 128, 0.08);
    border-color: var(--green);
  }
</style>
