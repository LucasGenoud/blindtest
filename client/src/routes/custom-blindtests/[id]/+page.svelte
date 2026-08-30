<script>
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { page } from '$app/stores';
  import { api, apiTry } from '$lib/api.js';
  import { goto } from '$app/navigation';
  import { debounce, categoryListValueLabel } from '$lib/misc.js';
  import { ArrowLeft, X, Search, Plus } from 'lucide-svelte';

  let blindtest = $state(null);
  let allAudios = $state([]);
  let search = $state('');
  let filterCat = $state('');
  let saving = $state(false);

  onMount(async () => {
    const id = $page.params.id;
    [blindtest, allAudios] = await Promise.all([
      apiTry(api.get(`/getcustomblindtest/${id}`)),
      apiTry(api.get('/getaudiosnames'), []),
    ]);
  });

  function filteredPool() {
    return allAudios.filter(a => {
      if (blindtest?.blindtestList.includes(a._id)) return false;
      if (filterCat && a.category !== filterCat) return false;
      if (search && !a.answer.toLowerCase().includes(search.toLowerCase())) return false;
      return true;
    });
  }

  function addAudio(audioId) {
    if (!blindtest) return;
    blindtest.blindtestList = [...blindtest.blindtestList, audioId];
    saveSoon();
  }

  function removeAudio(audioId) {
    if (!blindtest) return;
    blindtest.blindtestList = blindtest.blindtestList.filter(id => id !== audioId);
    saveSoon();
  }

  function getAudioName(id) {
    return allAudios.find(a => a._id === id)?.answer || id;
  }

  function getAudioCat(id) {
    return allAudios.find(a => a._id === id)?.category || '';
  }

  const saveSoon = debounce(save, 500);

  async function save() {
    if (!blindtest) return;
    saving = true;
    await apiTry(api.post(`/updatecustomblindtest/${blindtest._id}`, { blindtestList: blindtest.blindtestList }));
    saving = false;
  }

  async function togglePublic() {
    blindtest.public = !blindtest.public;
    await apiTry(api.post(`/updatecustomblindtest/${blindtest._id}`, { public: blindtest.public }));
  }
</script>

<svelte:head><title>Edit Blindtest — Blindtest</title></svelte:head>

<div class="editor-page">
  {#if blindtest}
    <div class="editor-content" in:fade={{ duration: 250 }}>
      <div class="editor-header">
        <button class="btn-primary" onclick={() => goto('/custom-blindtests')}><ArrowLeft size={14} stroke-width={1.8} /> Back</button>
        <h2>{blindtest.name}</h2>
        <div class="header-right">
          {#if saving}<span class="save-indicator">Saving...</span>{/if}
          <label class="toggle" class:active={blindtest.public}>
            <input type="checkbox" checked={blindtest.public} onchange={togglePublic} />
            {blindtest.public ? 'Public' : 'Private'}
          </label>
        </div>
      </div>

      <div class="editor-split">
        <!-- Pool -->
        <div class="pool">
          <div class="panel-header">
            <Search size={14} stroke-width={1.8} class="search-icon" />
            <input bind:value={search} placeholder="Search audios..." style="flex:1" />
            <select bind:value={filterCat}>
              <option value="">All</option>
              {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
            </select>
          </div>
          <div class="pool-list">
            {#each filteredPool().slice(0, 200) as audio (audio._id)}
              <div class="pool-item" onclick={() => addAudio(audio._id)} transition:fade={{ duration: 150 }}>
                <span class="cat-dot" style="background:{audio.category === 'movies' ? 'var(--red)' : audio.category === 'animes' ? 'var(--blue)' : 'var(--green)'}"></span>
                <span class="pool-name">{audio.answer}</span>
                <Plus size={12} stroke-width={1.8} class="add-icon" />
              </div>
            {/each}
          </div>
        </div>

        <!-- Selected -->
        <div class="selected">
          <div class="panel-header">
            <span class="panel-count">{blindtest.blindtestList.length} selected</span>
          </div>
          <div class="selected-list">
            {#each blindtest.blindtestList as audioId, i (audioId + i)}
              <div class="selected-item" transition:fade={{ duration: 150, delay: 40 }}>
                <span class="item-num">{i + 1}</span>
                <span class="item-name">{getAudioName(audioId)}</span>
                <span class="item-cat">{getAudioCat(audioId)}</span>
                <button class="remove-btn" onclick={() => removeAudio(audioId)}><X size={11} stroke-width={1.8} /></button>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="loading-region"><div class="loading-line"></div></div>
  {/if}
</div>

<style>
  .editor-page { height: 100%; width: 100%; display: flex; flex-direction: column; background: var(--bg); }
  .editor-content { display: flex; flex-direction: column; flex: 1; min-height: 0; }
  .editor-header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 32px;
    border-bottom: 2px solid var(--divider);
  }
  h2 {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }
  .header-right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  /* No pulse loops: it either says saved or it does not. */
  .save-indicator {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--signal-correct);
  }
  .editor-split { display: flex; flex: 1; overflow: hidden; }
  .pool, .selected { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .pool { border-right: 2px solid var(--divider); }
  .panel-header {
    padding: 12px 16px;
    display: flex;
    gap: 8px;
    border-bottom: 2px solid var(--divider);
    align-items: center;
  }
  .panel-count {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
  }
  .pool-list, .selected-list { overflow: auto; flex: 1; }
  .pool-item {
    padding: 10px 16px;
    cursor: pointer;
    font-size: 13px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 10px;
    transition: background 0.15s;
  }
  .pool-item:hover { background: var(--surface-2); }
  .pool-item:hover .add-icon { opacity: 1; }
  .add-icon {
    margin-left: auto;
    opacity: 0;
    color: var(--accent-text);
    transition: opacity 0.15s;
    flex-shrink: 0;
  }
  .panel-header .search-icon {
    color: var(--text-dim);
    flex-shrink: 0;
  }
  .pool-name {
    color: var(--text-primary);
    font-size: 13px;
  }
  /* Category reads as a word, not a colour swatch. */
  .cat-dot { display: none; }
  .selected-item {
    padding: 10px 16px;
    font-size: 13px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background 0.15s;
  }
  .selected-item:hover { background: var(--surface-2); }
  .item-num {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-dim);
    width: 28px;
    text-align: right;
    flex-shrink: 0;
  }
  .item-name {
    color: var(--text-primary);
    font-size: 13px;
    flex: 1;
    font-weight: 500;
  }
  .item-cat {
    font-size: 11px;
    color: var(--text-dim);
    font-weight: 500;
  }
  .remove-btn {
    background: transparent;
    border: 1px solid var(--divider);
    color: var(--red);
    width: 22px;
    height: 22px;
    border-radius: 0;
    font-size: 11px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    padding: 0;
  }
  .remove-btn:hover {
    border-color: var(--red);
  }
  .loading-region { position: relative; height: 2px; }
</style>
