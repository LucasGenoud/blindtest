<script>
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { categoryListValueLabel } from '$lib/misc.js';
  import { Play, ExternalLink, Pencil, Trash2, X, Search, Flag, ChevronDown, ChevronUp, Download } from 'lucide-svelte';

  let audios = $state([]);
  let search = $state('');
  let filterStatus = $state('all');
  let filterCategory = $state('all');
  let filterSuperflus = $state('all');
  let editAudio = $state(null);
  let showAddForm = $state(false);
  let previewAudio = $state(null);
  let newAudio = $state({ category: 'movies', answer: '', videoUrl: '', startTime: 0, superflus: false });
  let loading = $state(true);

  // Pagination
  const PAGE_SIZE_OPTIONS = [25, 50, 100, 200];
  let pageSize = $state(50);
  let currentPage = $state(1);

  // Sorting
  let sortKey = $state('addedDate');
  let sortDir = $state('desc'); // 'asc' | 'desc'

  onMount(async () => {
    if ($userPermission < 2) { goto('/'); return; }
    await loadAudios();
  });

  async function loadAudios() {
    loading = true;
    const res = await fetch(`${getApi()}/getallaudios`, { headers: { Authorization: $token } });
    if (res.ok) audios = await res.json();
    loading = false;
  }

  function filteredAudios() {
    let list = audios;
    if (search) {
      const s = search.toLowerCase();
      list = list.filter(a => a.answer.toLowerCase().includes(s) || a.category.toLowerCase().includes(s) || (a.submittedByUsername || '').toLowerCase().includes(s));
    }
    if (filterStatus !== 'all') {
      list = list.filter(a => a.processingStatus === filterStatus);
    }
    if (filterCategory !== 'all') {
      list = list.filter(a => a.category === filterCategory);
    }
    if (filterSuperflus === 'yes') {
      list = list.filter(a => a.superflus);
    } else if (filterSuperflus === 'no') {
      list = list.filter(a => !a.superflus);
    }
    // Sort
    list = [...list].sort((a, b) => {
      let av = a[sortKey], bv = b[sortKey];
      if (av == null) av = '';
      if (bv == null) bv = '';
      if (typeof av === 'string') av = av.toLowerCase();
      if (typeof bv === 'string') bv = bv.toLowerCase();
      if (av < bv) return sortDir === 'asc' ? -1 : 1;
      if (av > bv) return sortDir === 'asc' ? 1 : -1;
      return 0;
    });
    return list;
  }

  function pagedAudios() {
    const list = filteredAudios();
    const start = (currentPage - 1) * pageSize;
    return list.slice(start, start + pageSize);
  }

  const totalFiltered = $derived(filteredAudios().length);
  const totalPages = $derived(Math.max(1, Math.ceil(totalFiltered / pageSize)));

  // Reset page when filters/search/sort/pageSize change
  $effect(() => {
    search; filterStatus; filterCategory; filterSuperflus; sortKey; sortDir; pageSize;
    currentPage = 1;
  });

  function setSort(key) {
    if (sortKey === key) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = key;
      sortDir = 'asc';
    }
  }

  function sortIcon(key) {
    if (sortKey !== key) return 'neutral';
    return sortDir;
  }

  async function addAudio() {
    if (!newAudio.answer || !newAudio.videoUrl) return;
    await fetch(`${getApi()}/newaudio`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify(newAudio),
    });
    showAddForm = false;
    newAudio = { category: 'movies', answer: '', videoUrl: '', startTime: 0, superflus: false };
    await loadAudios();
  }

  async function saveEdit() {
    await fetch(`${getApi()}/updateaudio`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify(editAudio),
    });
    editAudio = null;
    await loadAudios();
  }

  async function deleteAudio(id) {
    if (!confirm('Delete this audio?')) return;
    await fetch(`${getApi()}/deleteaudio?id=${id}`, {
      method: 'DELETE',
      headers: { Authorization: $token },
    });
    await loadAudios();
  }

  async function resetFlag(audioId) {
    await fetch(`${getApi()}/resetflag`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ audioId }),
    });
    await loadAudios();
  }

  async function downloadBackup() {
    const res = await fetch(`${getApi()}/backupaudio`, { headers: { Authorization: $token } });
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'backup.zip'; a.click();
  }

  function openPreview(audio) {
    previewAudio = audio;
  }

  function closePreview() {
    previewAudio = null;
  }

  function getStatusColor(status) {
    if (status === 'ready') return 'status-ready';
    if (status === 'processing') return 'status-processing';
    if (status === 'error') return 'status-error';
    return 'status-unknown';
  }

  function getStatusLabel(status) {
    if (status === 'ready') return 'Ready';
    if (status === 'processing') return 'Processing';
    if (status === 'error') return 'Error';
    return 'Unknown';
  }

  const counts = $derived.by(() => ({
    total: audios.length,
    ready: audios.filter(a => a.processingStatus === 'ready').length,
    processing: audios.filter(a => a.processingStatus === 'processing').length,
    error: audios.filter(a => a.processingStatus === 'error').length,
    flagged: audios.filter(a => a.flagged?.length > 0).length,
  }));
</script>

<svelte:head><title>Manage Audios — Blindtest</title></svelte:head>

<div class="manage-page">
  <!-- Header -->
  <div class="page-header">
    <div class="header-left">
      <h1>Manage Audios</h1>
      <span class="total-badge">{audios.length} tracks</span>
    </div>
    <div class="header-actions">
      <button class="btn-primary btn-icon" onclick={() => showAddForm = !showAddForm}>
        {#if showAddForm}
          <X size={14} stroke-width={1.8} /> Cancel
        {:else}
          + Add Audio
        {/if}
      </button>
      {#if $userPermission >= 3}
        <button class="btn-primary btn-icon" onclick={downloadBackup}>
          <Download size={14} stroke-width={1.8} /> Backup
        </button>
      {/if}
    </div>
  </div>

  <!-- Stats bar -->
  <div class="stats-bar">
    <button class="stat-chip {filterStatus === 'all' ? 'active' : ''}" onclick={() => filterStatus = 'all'}>
      <span class="chip-dot dot-all"></span> All <strong>{counts.total}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'ready' ? 'active' : ''}" onclick={() => filterStatus = 'ready'}>
      <span class="chip-dot dot-ready"></span> In S3 <strong>{counts.ready}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'processing' ? 'active' : ''}" onclick={() => filterStatus = 'processing'}>
      <span class="chip-dot dot-processing"></span> Processing <strong>{counts.processing}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'error' ? 'active' : ''}" onclick={() => filterStatus = 'error'}>
      <span class="chip-dot dot-error"></span> Error <strong>{counts.error}</strong>
    </button>
    {#if counts.flagged > 0}
      <div class="stat-chip flag-chip">
        <Flag size={13} stroke-width={1.8} /> Flagged <strong>{counts.flagged}</strong>
      </div>
    {/if}
    <!-- Extra filters -->
    <select class="filter-select" bind:value={filterCategory}>
      <option value="all">All categories</option>
      {#each categoryListValueLabel as c}
        <option value={c.value}>{c.label}</option>
      {/each}
    </select>
    <select class="filter-select" bind:value={filterSuperflus}>
      <option value="all">Superflus: all</option>
      <option value="no">Superflus: no</option>
      <option value="yes">Superflus: yes</option>
    </select>
    <div class="search-wrap">
      <Search size={14} stroke-width={1.8} class="search-icon" />
      <input bind:value={search} placeholder="Search answer, category, user…" class="search-input" />
    </div>
  </div>

  <!-- Add form -->
  {#if showAddForm}
    <div class="add-card" in:fly={{ y: -8, duration: 150 }}>
      <div class="add-card-title">New Audio</div>
      <div class="form-grid">
        <select bind:value={newAudio.category}>
          {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
        </select>
        <input bind:value={newAudio.answer} placeholder="Answer (e.g. Inception)" />
        <input bind:value={newAudio.videoUrl} placeholder="YouTube URL" class="url-input" />
        <input type="number" bind:value={newAudio.startTime} placeholder="Start (s)" class="short-input" />
        <label class="toggle" class:active={newAudio.superflus}>
          <input type="checkbox" bind:checked={newAudio.superflus} /> Superflus
        </label>
        <div class="form-actions">
          <button class="btn-primary" onclick={addAudio}>Save</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Table -->
  <div class="table-wrap">
    {#if loading}
      <div class="loading-state">
        <span class="loading-spin">⟳</span> Loading audios…
      </div>
    {:else if filteredAudios().length === 0}
      <div class="empty-state">No audios match your filters.</div>
    {:else}
      <table>
        <thead>
          <tr>
            <th><button class="sort-btn" onclick={() => setSort('processingStatus')}>Status {#if sortIcon('processingStatus') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('processingStatus') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th><button class="sort-btn" onclick={() => setSort('category')}>Category {#if sortIcon('category') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('category') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th><button class="sort-btn" onclick={() => setSort('answer')}>Answer {#if sortIcon('answer') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('answer') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th><button class="sort-btn" onclick={() => setSort('startTime')}>Start {#if sortIcon('startTime') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('startTime') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th><button class="sort-btn" onclick={() => setSort('count')}>Plays {#if sortIcon('count') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('count') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th>Flags</th>
            <th><button class="sort-btn" onclick={() => setSort('submittedByUsername')}>By {#if sortIcon('submittedByUsername') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('submittedByUsername') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th><button class="sort-btn" onclick={() => setSort('addedDate')}>Added {#if sortIcon('addedDate') === 'asc'}<ChevronUp size={11} stroke-width={1.8} />{:else if sortIcon('addedDate') === 'desc'}<ChevronDown size={11} stroke-width={1.8} />{/if}</button></th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each pagedAudios() as audio (audio._id)}
            <tr class:flagged={audio.flagged?.length > 0} transition:fade={{ duration: 150 }}>
              <!-- S3 Status -->
              <td>
                <span class="status-badge {getStatusColor(audio.processingStatus)}">
                  {getStatusLabel(audio.processingStatus)}
                </span>
              </td>
              <!-- Category -->
              <td><span class="cat-badge">{audio.category}</span></td>
              <!-- Answer -->
              <td class="answer-cell">
                <span class="answer-text">{audio.answer}</span>
                {#if audio.superflus}
                  <span class="sup-tag">superflus</span>
                {/if}
              </td>
              <!-- Start -->
              <td class="mono">{audio.startTime}s</td>
              <!-- Plays -->
              <td class="mono">{audio.count}</td>
              <!-- Flags -->
              <td>
                {#if audio.flagged?.length > 0}
                  <div class="flag-cell">
                    <span class="flag-count"><Flag size={12} stroke-width={1.8} /> {audio.flagged.length}</span>
                    <button class="btn-xs btn-warning" onclick={() => resetFlag(audio._id)}>Reset</button>
                  </div>
                {:else}
                  <span class="dim">—</span>
                {/if}
              </td>
              <!-- By -->
              <td class="mono dim-cell">{audio.submittedByUsername || '—'}</td>
              <!-- Added -->
              <td class="mono dim-cell">{audio.addedDate ? audio.addedDate.slice(0, 10) : '—'}</td>
              <!-- Actions -->
              <td>
                <div class="action-group">
                  {#if audio.processingStatus === 'ready' && audio.s3ObjectKey}
                    <button class="btn-xs btn-play" title="Preview audio" onclick={() => openPreview(audio)}><Play size={11} stroke-width={1.8} /></button>
                  {/if}
                  <a href={audio.videoUrl} target="_blank" class="btn-xs btn-link" title="Open YouTube"><ExternalLink size={11} stroke-width={1.8} /></a>
                  <button class="btn-xs btn-edit" title="Edit" onclick={() => editAudio = {...audio}}><Pencil size={11} stroke-width={1.8} /></button>
                  {#if $userPermission >= 3}
                    <button class="btn-xs btn-del" title="Delete" onclick={() => deleteAudio(audio._id)}><Trash2 size={11} stroke-width={1.8} /></button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <!-- Pagination bar -->
  {#if !loading && totalFiltered > 0}
    <div class="pagination-bar">
      <div class="pagination-info">
        {(currentPage - 1) * pageSize + 1}–{Math.min(currentPage * pageSize, totalFiltered)} of {totalFiltered}
      </div>
      <div class="pagination-controls">
        <button class="page-btn" disabled={currentPage === 1} onclick={() => currentPage = 1}>«</button>
        <button class="page-btn" disabled={currentPage === 1} onclick={() => currentPage--}>‹</button>
        {#each Array.from({length: Math.min(7, totalPages)}, (_, i) => {
          const half = 3;
          let start = Math.max(1, currentPage - half);
          let end = Math.min(totalPages, start + 6);
          start = Math.max(1, end - 6);
          return start + i;
        }).filter(p => p >= 1 && p <= totalPages) as p}
          <button class="page-btn {p === currentPage ? 'active' : ''}" onclick={() => currentPage = p}>{p}</button>
        {/each}
        <button class="page-btn" disabled={currentPage === totalPages} onclick={() => currentPage++}>›</button>
        <button class="page-btn" disabled={currentPage === totalPages} onclick={() => currentPage = totalPages}>»</button>
      </div>
      <div class="page-size-select">
        <span class="page-size-label">Rows</span>
        {#each PAGE_SIZE_OPTIONS as opt}
          <button class="page-size-btn {pageSize === opt ? 'active' : ''}" onclick={() => pageSize = opt}>{opt}</button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- Edit Popup -->
{#if editAudio}
  <div class="overlay" in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} onclick={(e) => e.target === e.currentTarget && (editAudio = null)}>
    <div class="popup edit-popup" in:fly={{ y: 20, duration: 200 }}>
      <div class="popup-header">
        <span class="popup-title">Edit Audio</span>
        <button class="close-btn" onclick={() => editAudio = null}><X size={14} stroke-width={1.8} /></button>
      </div>
      <div class="popup-body">
        <label class="field-label">Category</label>
        <select bind:value={editAudio.category}>
          {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
        </select>
        <label class="field-label">Answer</label>
        <input bind:value={editAudio.answer} placeholder="Answer" />
        <label class="field-label">YouTube URL</label>
        <input bind:value={editAudio.videoUrl} placeholder="YouTube URL" />
        <label class="field-label">Start time (seconds)</label>
        <input type="number" bind:value={editAudio.startTime} placeholder="Start time" />
        <label class="toggle" class:active={editAudio.superflus}>
          <input type="checkbox" bind:checked={editAudio.superflus} /> Superflus
        </label>
      </div>
      <div class="popup-footer">
        <button class="btn-primary" onclick={saveEdit}>Save changes</button>
        <button class="btn-danger" onclick={() => editAudio = null}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<!-- Preview Popup -->
{#if previewAudio}
  <div class="overlay" in:fade={{ duration: 150 }} out:fade={{ duration: 100 }} onclick={(e) => e.target === e.currentTarget && closePreview()}>
    <div class="popup preview-popup" in:fly={{ y: 20, duration: 200 }}>
      <div class="popup-header">
        <div class="preview-title-group">
          <span class="cat-badge">{previewAudio.category}</span>
          <span class="popup-title">{previewAudio.answer}</span>
        </div>
        <button class="close-btn" onclick={closePreview}><X size={14} stroke-width={1.8} /></button>
      </div>
      <div class="video-container">
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
          src="{getApi()}/media/{previewAudio._id}"
          controls
          autoplay
          class="video-player"
        >
          Your browser does not support video playback.
        </video>
      </div>
      <div class="preview-meta">
        <span class="meta-item"><span class="meta-label">Start</span> {previewAudio.startTime}s</span>
        <span class="meta-item"><span class="meta-label">Plays</span> {previewAudio.count}</span>
        <span class="meta-item"><span class="meta-label">Added by</span> {previewAudio.submittedByUsername || '—'}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .manage-page {
    padding: 28px 24px;
    overflow: auto;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  /* ── Header ── */
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .header-left {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }
  .total-badge {
    font-size: 0.75rem;
    color: var(--text-dim);
    background: var(--surface-2);
    border: 1px solid var(--border);
    padding: 2px 10px;
    border-radius: 9999px;
    font-weight: 500;
  }
  .header-actions {
    display: flex;
    gap: 8px;
  }

  /* ── Stats bar ── */
  .stats-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    padding: 10px 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
  }
  .stat-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s;
    font-weight: 500;
  }
  .stat-chip:hover { background: var(--surface-2); }
  .stat-chip.active {
    background: var(--accent-dim);
    border-color: var(--accent-border);
    color: var(--accent);
  }
  .stat-chip strong { font-weight: 700; }
  .flag-chip { cursor: default; color: var(--red); }
  .flag-chip strong { color: var(--red); }
  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .dot-all { background: var(--text-dim); }
  .dot-ready { background: var(--green); }
  .dot-processing { background: var(--orange); }
  .dot-error { background: var(--red); }

  .search-wrap {
    margin-left: auto;
    position: relative;
    display: flex;
    align-items: center;
  }
  .search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-dim);
    font-size: 1rem;
    pointer-events: none;
  }
  .search-input {
    padding-left: 28px;
    width: 220px;
    font-size: 0.75rem;
  }

  /* ── Add card ── */
  .add-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 18px 20px;
  }
  .add-card-title {
    font-family: var(--mono);
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-bottom: 12px;
  }
  .form-grid {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  .url-input { flex: 1; min-width: 200px; }
  .short-input { width: 90px; }
  .form-actions { display: flex; gap: 6px; }

  /* ── Sort buttons ── */
  .sort-btn {
    background: transparent;
    border: none;
    padding: 0;
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .sort-btn:hover { color: var(--text-primary); }

  /* ── Category / superflus filter selects ── */
  .filter-select {
    font-size: 0.75rem;
    padding: 5px 8px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-secondary);
    cursor: pointer;
    height: 30px;
  }

  /* ── Dim cell ── */
  .dim-cell {
    color: var(--text-dim);
    font-size: 0.7rem;
  }

  /* ── Pagination bar ── */
  .pagination-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 4px;
    flex-shrink: 0;
    flex-wrap: wrap;
  }
  .pagination-info {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-dim);
    min-width: 90px;
  }
  .pagination-controls {
    display: flex;
    gap: 3px;
    align-items: center;
  }
  .page-btn {
    min-width: 28px;
    height: 28px;
    padding: 0 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--mono);
    font-size: 0.72rem;
    cursor: pointer;
    transition: all 0.15s;
  }
  .page-btn:hover:not(:disabled) { background: var(--surface-2); border-color: var(--border-2); }
  .page-btn.active { background: var(--accent-dim); border-color: var(--accent-border); color: var(--accent); font-weight: 700; }
  .page-btn:disabled { opacity: 0.3; cursor: default; }
  .page-size-select {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 3px;
  }
  .page-size-label {
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    letter-spacing: 0.06em;
    margin-right: 4px;
  }
  .page-size-btn {
    min-width: 36px;
    height: 24px;
    padding: 0 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    font-family: var(--mono);
    font-size: 0.68rem;
    cursor: pointer;
    transition: all 0.15s;
  }
  .page-size-btn:hover { background: var(--surface-2); }
  .page-size-btn.active { background: var(--accent-dim); border-color: var(--accent-border); color: var(--accent); font-weight: 700; }

  /* ── Table ── */
  .table-wrap {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .loading-state, .empty-state {
    padding: 48px;
    text-align: center;
    color: var(--text-dim);
    font-family: var(--mono);
    font-size: 0.78rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    padding: 10px 14px;
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--surface);
    z-index: 1;
    white-space: nowrap;
  }
  td {
    padding: 9px 14px;
    border-bottom: 1px solid var(--border);
    font-size: 0.78rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.72rem;
    color: var(--text-dim);
  }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: var(--surface); }
  tr.flagged td { background: rgba(248, 113, 113, 0.04); }
  tr.flagged:hover td { background: rgba(248, 113, 113, 0.08); }

  /* ── Status badges ── */
  .status-badge {
    font-family: var(--mono);
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 3px 8px;
    border-radius: 4px;
    white-space: nowrap;
  }
  .status-ready {
    background: rgba(74, 222, 128, 0.1);
    color: var(--green);
    border: 1px solid rgba(74, 222, 128, 0.25);
  }
  .status-processing {
    background: rgba(251, 146, 60, 0.1);
    color: var(--orange);
    border: 1px solid rgba(251, 146, 60, 0.25);
  }
  .status-error {
    background: rgba(248, 113, 113, 0.1);
    color: var(--red);
    border: 1px solid rgba(248, 113, 113, 0.25);
  }
  .status-unknown {
    background: var(--surface-2);
    color: var(--text-dim);
    border: 1px solid var(--border);
  }

  /* ── Category badge ── */
  .cat-badge {
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    background: var(--surface-2);
    padding: 2px 7px;
    border-radius: 3px;
    letter-spacing: 0.04em;
  }

  /* ── Answer cell ── */
  .answer-cell {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .answer-text {
    color: var(--text-primary);
    font-size: 0.8rem;
  }
  .sup-tag {
    font-family: var(--mono);
    font-size: 0.58rem;
    color: var(--text-dim);
    background: var(--surface-2);
    padding: 1px 5px;
    border-radius: 3px;
    margin-left: 6px;
  }

  /* ── Flags cell ── */
  .flag-cell {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .flag-count {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--red);
  }
  .dim { color: var(--text-dim); }

  /* ── Action buttons ── */
  .action-group {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .btn-xs {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.65rem;
    font-family: var(--mono);
    font-weight: 500;
    letter-spacing: 0.04em;
    cursor: pointer;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-secondary);
    transition: all 0.15s;
    text-decoration: none;
  }
  .btn-xs:hover { background: var(--surface-2); border-color: var(--border-2); }
  .btn-play {
    border-color: rgba(74, 222, 128, 0.3);
    color: var(--green);
  }
  .btn-play:hover {
    background: rgba(74, 222, 128, 0.1);
    border-color: var(--green);
  }
  .btn-link {
    border-color: rgba(96, 165, 250, 0.3);
    color: var(--blue);
  }
  .btn-link:hover {
    background: rgba(96, 165, 250, 0.1);
    border-color: var(--blue);
  }
  .btn-edit { color: var(--accent); border-color: var(--accent-border); }
  .btn-edit:hover { background: var(--accent-dim); border-color: var(--accent); }
  .btn-del { color: var(--red); border-color: rgba(248, 113, 113, 0.3); }
  .btn-del:hover { background: rgba(248, 113, 113, 0.1); border-color: var(--red); }
  .btn-warning {
    color: var(--orange);
    border-color: rgba(251, 146, 60, 0.3);
    background: transparent;
  }
  .btn-warning:hover { background: rgba(251, 146, 60, 0.1); border-color: var(--orange); }

  /* ── Overlay & Popups ── */
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(6px);
  }
  .popup {
    background: var(--surface);
    border: 1px solid var(--border-2);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0,0,0,0.6);
  }
  .popup-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }
  .popup-title {
    font-family: var(--mono);
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .preview-title-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-dim);
    font-size: 0.9rem;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    transition: all 0.15s;
  }
  .close-btn:hover { color: var(--text-primary); background: var(--surface-2); }

  /* Edit popup */
  .edit-popup { width: 420px; }
  .popup-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field-label {
    font-family: var(--mono);
    font-size: 0.6rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
    margin-top: 4px;
  }
  .popup-footer {
    padding: 14px 20px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 8px;
  }

  /* Preview popup */
  .preview-popup { width: 720px; max-width: 95vw; }
  .video-container {
    background: #000;
    position: relative;
  }
  .video-player {
    width: 100%;
    display: block;
    max-height: 60vh;
    background: #000;
  }
  .preview-meta {
    display: flex;
    gap: 20px;
    padding: 14px 20px;
    border-top: 1px solid var(--border);
    flex-wrap: wrap;
  }
  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .meta-label {
    font-family: var(--mono);
    font-size: 0.58rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }
  .meta-item {
    font-family: var(--mono);
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
</style>
