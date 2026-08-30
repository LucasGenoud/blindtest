<script>
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { categoryListValueLabel } from '$lib/misc.js';
  import { Play, ExternalLink, Pencil, Trash2, X, Search, Flag, ChevronDown, ChevronUp, Download, RefreshCw } from 'lucide-svelte';

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

  /// Requeue an audio whose processing failed; previously the only way to retry was
  /// to edit the video URL into something else and back again.
  async function reprocessAudio(audioId) {
    await fetch(`${getApi()}/reprocessaudio?audioId=${encodeURIComponent(audioId)}`, {
      method: 'POST',
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
      <h1>Manage audios</h1>
      <span class="total-badge tabular">{audios.length} tracks</span>
    </div>
    <div class="header-actions">
      <button class="btn-primary" onclick={() => showAddForm = !showAddForm}>
        {#if showAddForm}
          <X size={16} stroke-width={2} /> Cancel
        {:else}
          Add audio
        {/if}
      </button>
      {#if $userPermission >= 3}
        <button class="btn-secondary" onclick={downloadBackup}>
          <Download size={16} stroke-width={2} /> Backup
        </button>
      {/if}
    </div>
  </div>

  <!-- Stats bar -->
  <div class="stats-bar">
    <button class="stat-chip {filterStatus === 'all' ? 'active' : ''}" onclick={() => filterStatus = 'all'}>
All <strong class="tabular">{counts.total}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'ready' ? 'active' : ''}" onclick={() => filterStatus = 'ready'}>
In S3 <strong class="tabular">{counts.ready}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'processing' ? 'active' : ''}" onclick={() => filterStatus = 'processing'}>
Processing <strong class="tabular">{counts.processing}</strong>
    </button>
    <button class="stat-chip {filterStatus === 'error' ? 'active' : ''}" onclick={() => filterStatus = 'error'}>
Error <strong class="tabular">{counts.error}</strong>
    </button>
    {#if counts.flagged > 0}
      <div class="stat-chip flag-chip">
        <Flag size={16} stroke-width={2} /> Flagged <strong class="tabular">{counts.flagged}</strong>
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
      <div class="loading-region"><div class="loading-line"></div></div>
    {:else if filteredAudios().length === 0}
      <div class="empty-state">
        <h2>No audios match your filters</h2>
        <p>Clear the search or pick a different status to see the rest of the library.</p>
      </div>
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
                <span class="status {getStatusColor(audio.processingStatus)}">
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
                    <button class="btn-xs" onclick={() => resetFlag(audio._id)}>Reset</button>
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
                    <button class="btn-xs" title="Preview audio" aria-label="Preview audio" onclick={() => openPreview(audio)}><Play size={14} stroke-width={2} /></button>
                  {/if}
                  {#if audio.processingStatus === 'error'}
                    <button class="btn-xs" title="Retry processing" onclick={() => reprocessAudio(audio._id)}>Retry</button>
                  {/if}
                  <a href={audio.videoUrl} target="_blank" class="btn-xs" title="Open source">Source</a>
                  <button class="btn-xs" title="Edit" onclick={() => editAudio = {...audio}}>Edit</button>
                  {#if $userPermission >= 3}
                    <button class="btn-xs btn-del" title="Delete" onclick={() => deleteAudio(audio._id)}>Delete</button>
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
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }
  .total-badge {
    font-size: 11px;
    color: var(--text-dim);
    background: var(--surface-2);
    border: 1px solid var(--border);
    padding: 2px 10px;
    border-radius: 0;
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
    border-radius: 0;
  }
  .stat-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: 0;
    font-size: 13px;
    color: var(--text-primary);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    font-weight: 500;
  }
  .stat-chip:hover { background: var(--surface-2); }
  .stat-chip.active {
    color: var(--accent-ink);
    border-bottom-color: var(--accent);
  }
  .stat-chip strong { font-weight: 700; }
  .flag-chip { cursor: default; color: var(--red); }
  .flag-chip strong { color: var(--red); }

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
    font-size: 15px;
    pointer-events: none;
  }
  .search-input {
    padding-left: 28px;
    width: 220px;
    font-size: 11px;
  }

  /* ── Add card ── */
  .add-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    padding: 18px 20px;
  }
  .add-card-title {
    font-family: var(--mono);
    font-size: 11px;
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
    font-size: 11px;
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
    font-size: 11px;
    padding: 5px 8px;
    border-radius: 0;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-secondary);
    cursor: pointer;
    height: 30px;
  }

  /* ── Dim cell ── */
  .dim-cell {
    color: var(--text-dim);
    font-size: 11px;
  }

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

  /* ── Table ── */
  .table-wrap {
    flex: 1;
    overflow: auto;
    border: 0;
    border-radius: 0;
  }
  .loading-region { position: relative; height: 2px; }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th {
    text-align: left;
    padding: 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    border-bottom: 2px solid var(--divider);
    position: sticky;
    top: 0;
    background: var(--bg);
    z-index: 1;
    white-space: nowrap;
  }
  td {
    padding: 8px;
    border-bottom: 1px solid var(--divider);
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
  }
  td.mono {
    font-size: 13px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }
  /* Numbers right-aligned and tabular. */
  th.num, td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  tr:hover td { background: var(--row-hover); }
  tr.flagged td { color: var(--text-primary); }

  /* ── Status ── */
  .status {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    white-space: nowrap;
    color: var(--text-secondary);
  }
  .status-ready { color: var(--text-primary); }
  .status-processing { color: var(--text-secondary); }
  .status-error { color: var(--signal-wrong); }
  .status-unknown { color: var(--text-dim); }

  /* ── Category badge ── */
  .cat-badge {
    font-size: 13px;
    color: var(--text-dim);
  }

  /* ── Answer cell ── */
  .answer-cell {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .answer-text {
    color: var(--text-primary);
    font-size: 13px;
  }
  .sup-tag {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-dim);
    margin-left: 8px;
  }

  /* ── Flags cell ── */
  .flag-cell {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .flag-count {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--red);
  }
  .dim { color: var(--text-dim); }

  /* ── Action buttons ── */
  .action-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  /* These are controls, not labels: they carry a border so they read as
     clickable against a dense row of text. */
  .btn-xs {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 4px 8px;
    min-height: 28px;
    border-radius: 0;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--divider);
    background: transparent;
    color: var(--text-primary);
    transition: color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    text-decoration: none;
  }
  .btn-xs:hover {
    color: var(--accent-ink);
    border-color: var(--accent-ink);
    text-decoration: none;
  }
  .btn-del { color: var(--signal-wrong); }
  .btn-del:hover { border-color: var(--signal-wrong); color: var(--signal-wrong); }

  /* ── Overlay & Popups ── */
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .popup {
    background: var(--surface);
    border: 1px solid var(--border-2);
    border-radius: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
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
    font-size: 13px;
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
    font-size: 15px;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 0;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
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
    font-size: 11px;
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
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }
  .meta-item {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-secondary);
  }
</style>
