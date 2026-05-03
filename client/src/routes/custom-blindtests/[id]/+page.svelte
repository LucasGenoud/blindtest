<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getApi } from '$lib/api.js';
  import { token } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { debounce, categoryListValueLabel } from '$lib/misc.js';

  let blindtest = $state(null);
  let allAudios = $state([]);
  let search = $state('');
  let filterCat = $state('');
  let saving = $state(false);

  onMount(async () => {
    const id = $page.params.id;
    const [btRes, audiosRes] = await Promise.all([
      fetch(`${getApi()}/getcustomblindtest/${id}`),
      fetch(`${getApi()}/getaudiosnames`, { headers: { Authorization: $token } }),
    ]);
    if (btRes.ok) blindtest = await btRes.json();
    if (audiosRes.ok) allAudios = await audiosRes.json();
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
    await fetch(`${getApi()}/updatecustomblindtest/${blindtest._id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ blindtestList: blindtest.blindtestList }),
    });
    saving = false;
  }

  async function togglePublic() {
    blindtest.public = !blindtest.public;
    await fetch(`${getApi()}/updatecustomblindtest/${blindtest._id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ public: blindtest.public }),
    });
  }
</script>

<svelte:head><title>Edit Blindtest — Blindtest</title></svelte:head>

<div class="editor-page">
  {#if blindtest}
    <div class="editor-header">
      <button class="btn-primary" onclick={() => goto('/custom-blindtests')}>← Back</button>
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
          <input bind:value={search} placeholder="Search audios..." style="flex:1" />
          <select bind:value={filterCat}>
            <option value="">All</option>
            {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
          </select>
        </div>
        <div class="pool-list">
          {#each filteredPool().slice(0, 200) as audio (audio._id)}
            <div class="pool-item" onclick={() => addAudio(audio._id)}>
              <span class="cat-dot" style="background:{audio.category === 'movies' ? 'var(--red)' : audio.category === 'animes' ? 'var(--blue)' : 'var(--green)'}"></span>
              <span class="pool-name">{audio.answer}</span>
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
            <div class="selected-item">
              <span class="item-num">{i + 1}</span>
              <span class="item-name">{getAudioName(audioId)}</span>
              <span class="item-cat">{getAudioCat(audioId)}</span>
              <button class="remove-btn" onclick={() => removeAudio(audioId)}>✕</button>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <div class="loading-state">Loading...</div>
  {/if}
</div>

<style>
  .editor-page { height: 100%; width: 100%; display: flex; flex-direction: column; background: var(--bg); }
  .editor-header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  h2 {
    font-family: var(--mono);
    font-size: 0.9rem;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  .header-right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .save-indicator {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--green);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    animation: pulse-anim 1.5s ease-in-out infinite;
  }
  .editor-split { display: flex; flex: 1; overflow: hidden; }
  .pool, .selected { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .pool { border-right: 1px solid var(--border); }
  .panel-header {
    padding: 12px 16px;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    align-items: center;
  }
  .panel-count {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .pool-list, .selected-list { overflow: auto; flex: 1; }
  .pool-item {
    padding: 8px 16px;
    cursor: pointer;
    font-size: 0.8rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background 0.15s;
  }
  .pool-item:hover { background: var(--accent-dim); }
  .pool-name {
    color: var(--text-secondary);
    font-size: 0.78rem;
  }
  .cat-dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .selected-item {
    padding: 8px 16px;
    font-size: 0.8rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background 0.15s;
  }
  .selected-item:hover { background: var(--surface); }
  .item-num {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-dim);
    width: 28px;
    text-align: right;
    flex-shrink: 0;
  }
  .item-name {
    color: var(--text-secondary);
    font-size: 0.78rem;
    flex: 1;
  }
  .item-cat {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .remove-btn {
    background: transparent;
    border: 1px solid rgba(248, 113, 113, 0.3);
    color: var(--red);
    width: 20px;
    height: 20px;
    border-radius: 4px;
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s;
    padding: 0;
  }
  .remove-btn:hover {
    background: rgba(248, 113, 113, 0.1);
    border-color: var(--red);
  }
  .loading-state {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 40px;
  }
</style>
