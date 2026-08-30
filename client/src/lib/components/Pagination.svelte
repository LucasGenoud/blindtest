<script>
  import { pageWindow } from '$lib/audios/audioLibrary.js';

  /** Page numbers plus a rows-per-page choice. Owns no data, only the controls. */
  let {
    page = $bindable(1),
    pageSize = $bindable(50),
    total = 0,
    pageSizeOptions = [25, 50, 100, 200],
  } = $props();

  const totalPages = $derived(Math.max(1, Math.ceil(total / pageSize)));
  const from = $derived((page - 1) * pageSize + 1);
  const to = $derived(Math.min(page * pageSize, total));
</script>

<div class="pagination-bar">
  <div class="pagination-info">
    {from}–{to} of {total}
  </div>
  <div class="pagination-controls">
    <button class="page-btn" disabled={page === 1} onclick={() => (page = 1)} aria-label="First page">«</button>
    <button class="page-btn" disabled={page === 1} onclick={() => page--} aria-label="Previous page">‹</button>
    {#each pageWindow(page, totalPages) as p}
      <button class="page-btn {p === page ? 'active' : ''}" onclick={() => (page = p)}>{p}</button>
    {/each}
    <button class="page-btn" disabled={page === totalPages} onclick={() => page++} aria-label="Next page">›</button>
    <button class="page-btn" disabled={page === totalPages} onclick={() => (page = totalPages)} aria-label="Last page">»</button>
  </div>
  <div class="page-size-select">
    <span class="page-size-label">Rows</span>
    {#each pageSizeOptions as opt}
      <button class="page-size-btn {pageSize === opt ? 'active' : ''}" onclick={() => (pageSize = opt)}>{opt}</button>
    {/each}
  </div>
</div>

<style>
  /* ── Pagination bar ── */
  .pagination-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 0;
    flex-shrink: 0;
    flex-wrap: wrap;
    border-top: 2px solid var(--divider);
  }

  .pagination-info {
    font-size: 13px;
    color: var(--text-secondary);
    min-width: 90px;
    font-variant-numeric: tabular-nums;
  }

  .pagination-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  /* Pages are a row of labels, not a row of boxes. */
  .page-btn,
  .page-size-btn {
    min-width: 24px;
    height: 24px;
    padding: 0 4px;
    border-radius: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition: color var(--duration-fast) ease-out;
  }

  .page-btn:hover:not(:disabled),
  .page-size-btn:hover { color: var(--accent-ink); }

  .page-btn.active,
  .page-size-btn.active { color: var(--accent-ink); font-weight: 800; }

  .page-btn:disabled { opacity: 0.3; cursor: default; }

  .page-size-select {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .page-size-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
  }
</style>
