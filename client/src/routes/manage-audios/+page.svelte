<script>
  import { onMount } from 'svelte';
  import { Download, X } from 'lucide-svelte';
  import { userPermission } from '$lib/stores/userStore.js';
  import { audioApi } from '$lib/audios/audioApi.js';
  import {
    PAGE_SIZE_OPTIONS,
    countByStatus,
    filterAudios,
    pageOf,
    sortAudios,
  } from '$lib/audios/audioLibrary.js';
  import AudioFilters from '$lib/components/audios/AudioFilters.svelte';
  import AudioTable from '$lib/components/audios/AudioTable.svelte';
  import AudioEditModal from '$lib/components/audios/AudioEditModal.svelte';
  import AudioPreviewModal from '$lib/components/audios/AudioPreviewModal.svelte';
  import Pagination from '$lib/components/Pagination.svelte';

  const EMPTY_AUDIO = { category: 'movies', answer: '', videoUrl: '', startTime: 0, superflus: false };

  let audios = $state([]);
  let loading = $state(true);

  let search = $state('');
  let filterStatus = $state('all');
  let filterCategory = $state('all');
  let filterSuperflus = $state('all');

  let sortKey = $state('addedDate');
  let sortDir = $state('desc');

  let pageSize = $state(50);
  let currentPage = $state(1);

  let showAddForm = $state(false);
  let newAudio = $state({ ...EMPTY_AUDIO });
  let editAudio = $state(null);
  let previewAudio = $state(null);

  const counts = $derived(countByStatus(audios));
  const visible = $derived(
    sortAudios(
      filterAudios(audios, {
        search,
        status: filterStatus,
        category: filterCategory,
        superflus: filterSuperflus,
      }),
      sortKey,
      sortDir,
    ),
  );
  const paged = $derived(pageOf(visible, currentPage, pageSize));

  // Any change to what is being shown starts again at the first page.
  $effect(() => {
    search; filterStatus; filterCategory; filterSuperflus; sortKey; sortDir; pageSize;
    currentPage = 1;
  });

  onMount(load);

  async function load() {
    loading = true;
    audios = await audioApi.list();
    loading = false;
  }

  async function addAudio() {
    if (!newAudio.answer || !newAudio.videoUrl) return;
    await audioApi.create(newAudio);
    showAddForm = false;
    newAudio = { ...EMPTY_AUDIO };
    await load();
  }

  async function saveEdit() {
    await audioApi.update(editAudio);
    editAudio = null;
    await load();
  }

  async function deleteAudio(id) {
    if (!confirm('Delete this audio?')) return;
    await audioApi.remove(id);
    await load();
  }

  async function reprocessAudio(id) {
    await audioApi.reprocess(id);
    await load();
  }

  async function resetFlag(id) {
    await audioApi.resetFlag(id);
    await load();
  }

  async function downloadBackup() {
    const blob = await audioApi.backup();
    if (!blob) return;
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'backup.zip';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<svelte:head><title>Manage Audios — Blindtest</title></svelte:head>

<div class="manage-page">
  <div class="page-header">
    <div class="header-left">
      <h1>Manage audios</h1>
      <span class="total-badge tabular">{audios.length} tracks</span>
    </div>
    <div class="header-actions">
      <button class="btn-primary" onclick={() => (showAddForm = !showAddForm)}>
        {#if showAddForm}<X size={16} stroke-width={2} /> Cancel{:else}Add audio{/if}
      </button>
      {#if $userPermission >= 3}
        <button class="btn-secondary" onclick={downloadBackup}>
          <Download size={16} stroke-width={2} /> Backup
        </button>
      {/if}
    </div>
  </div>

  <AudioFilters
    {counts}
    bind:status={filterStatus}
    bind:category={filterCategory}
    bind:superflus={filterSuperflus}
    bind:search
    bind:showAddForm
    bind:newAudio
    onadd={addAudio}
  />

  <div class="table-wrap">
    {#if loading}
      <div class="loading-region"><div class="loading-line"></div></div>
    {:else if visible.length === 0}
      <div class="empty-state">
        <h2>No audios match your filters</h2>
        <p>Clear the search or pick a different status to see the rest of the library.</p>
      </div>
    {:else}
      <AudioTable
        audios={paged}
        bind:sortKey
        bind:sortDir
        onpreview={(a) => (previewAudio = a)}
        onreprocess={reprocessAudio}
        onedit={(a) => (editAudio = { ...a })}
        ondelete={deleteAudio}
        onresetflag={resetFlag}
      />
    {/if}
  </div>

  {#if !loading && visible.length > 0}
    <Pagination bind:page={currentPage} bind:pageSize total={visible.length} pageSizeOptions={PAGE_SIZE_OPTIONS} />
  {/if}
</div>

{#if editAudio}
  <AudioEditModal bind:audio={editAudio} onsave={saveEdit} onclose={() => (editAudio = null)} />
{/if}

{#if previewAudio}
  <AudioPreviewModal audio={previewAudio} onclose={() => (previewAudio = null)} />
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

  .table-wrap {
    flex: 1;
    overflow: auto;
  }

  .loading-region { position: relative; height: 2px; }
</style>
