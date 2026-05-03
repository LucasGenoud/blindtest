<script>
  import { onMount } from 'svelte';
  import { getApi } from '$lib/api.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { goto } from '$app/navigation';
  import { categoryListValueLabel } from '$lib/misc.js';

  let audios = $state([]);
  let search = $state('');
  let editAudio = $state(null);
  let showAddForm = $state(false);
  let newAudio = $state({ category: 'movies', answer: '', videoUrl: '', startTime: 0, superflus: false });

  onMount(async () => {
    if ($userPermission < 2) { goto('/'); return; }
    await loadAudios();
  });

  async function loadAudios() {
    const res = await fetch(`${getApi()}/getallaudios`, { headers: { Authorization: $token } });
    if (res.ok) audios = await res.json();
  }

  $effect(() => {
    // filtered is derived
  });

  function filteredAudios() {
    if (!search) return audios;
    const s = search.toLowerCase();
    return audios.filter(a => a.answer.toLowerCase().includes(s) || a.category.toLowerCase().includes(s));
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
</script>

<svelte:head><title>Manage Audios — Blindtest</title></svelte:head>

<div class="manage-page">
  <div class="page-header">
    <h1>Manage Audios</h1>
    <div class="header-actions">
      <button class="btn-primary" onclick={() => showAddForm = true}>+ Add</button>
      {#if $userPermission >= 3}
        <button class="btn-primary" onclick={downloadBackup}>⬇ Backup</button>
      {/if}
      <input bind:value={search} placeholder="Search..." style="width:200px" />
    </div>
  </div>

  {#if showAddForm}
    <div class="card add-form">
      <div class="section-label">New Audio</div>
      <div class="form-row">
        <select bind:value={newAudio.category}>
          {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
        </select>
        <input bind:value={newAudio.answer} placeholder="Answer" />
        <input bind:value={newAudio.videoUrl} placeholder="YouTube URL" />
        <input type="number" bind:value={newAudio.startTime} placeholder="Start" style="width:80px" />
        <label class="toggle" class:active={newAudio.superflus}>
          <input type="checkbox" bind:checked={newAudio.superflus} /> Superflus
        </label>
        <button class="btn-primary" onclick={addAudio}>Save</button>
        <button class="btn-danger" onclick={() => showAddForm = false}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="table-wrapper">
    <table>
      <thead>
        <tr>
          <th>Category</th><th>Answer</th><th>URL</th><th>Start</th><th>Plays</th><th>Rating</th><th>Flags</th><th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredAudios() as audio (audio._id)}
          <tr class:flagged={audio.flagged?.length > 0}>
            <td><span class="cat-badge">{audio.category}</span></td>
            <td>{audio.answer}</td>
            <td><a href={audio.videoUrl} target="_blank">Link</a></td>
            <td class="mono">{audio.startTime}s</td>
            <td class="mono">{audio.count}</td>
            <td class="mono">{audio.rating?.toFixed(1) || '-'}</td>
            <td>
              {#if audio.flagged?.length > 0}
                <span class="flag-count">🚩 {audio.flagged.length}</span>
                <button class="btn-primary sm" onclick={() => resetFlag(audio._id)}>Reset</button>
              {:else}
                <span class="mono dim">-</span>
              {/if}
            </td>
            <td>
              <div class="action-group">
                <button class="btn-primary sm" onclick={() => editAudio = {...audio}}>✏</button>
                {#if $userPermission >= 3}
                  <button class="btn-danger sm" onclick={() => deleteAudio(audio._id)}>🗑</button>
                {/if}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if editAudio}
    <div class="popup-overlay" onclick={(e) => e.target === e.currentTarget && (editAudio = null)}>
      <div class="popup-box" style="padding:24px;display:flex;flex-direction:column;gap:12px">
        <div class="section-label">Edit Audio</div>
        <select bind:value={editAudio.category}>
          {#each categoryListValueLabel as c}<option value={c.value}>{c.label}</option>{/each}
        </select>
        <input bind:value={editAudio.answer} placeholder="Answer" />
        <input bind:value={editAudio.videoUrl} placeholder="YouTube URL" />
        <input type="number" bind:value={editAudio.startTime} placeholder="Start time" />
        <label class="toggle" class:active={editAudio.superflus}>
          <input type="checkbox" bind:checked={editAudio.superflus} /> Superflus
        </label>
        <div style="display:flex;gap:8px">
          <button class="btn-primary" onclick={saveEdit}>Save</button>
          <button class="btn-danger" onclick={() => editAudio = null}>Cancel</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .manage-page { padding: 28px 24px; overflow: auto; width: 100%; }
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    font-family: var(--mono);
    font-size: 1rem;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .add-form {
    margin-bottom: 16px;
  }
  .form-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
    margin-top: 10px;
  }
  .table-wrapper { overflow: auto; }
  table { width: 100%; border-collapse: collapse; }
  th {
    text-align: left;
    padding: 10px 12px;
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-dim);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border);
  }
  td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  td.mono {
    font-family: var(--mono);
    font-size: 0.75rem;
  }
  td.mono.dim { color: var(--text-dim); }
  tr:hover { background: var(--surface); }
  tr.flagged { background: rgba(248, 113, 113, 0.06); }
  .cat-badge {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-secondary);
    background: var(--surface-2);
    padding: 2px 8px;
    border-radius: 3px;
    letter-spacing: 0.04em;
  }
  .flag-count {
    font-family: var(--mono);
    font-size: 0.72rem;
    color: var(--red);
    margin-right: 6px;
  }
  .action-group { display: flex; gap: 4px; }
  :global(.btn-primary.sm), :global(.btn-danger.sm) {
    font-size: 0.6rem;
    padding: 3px 8px;
  }
</style>
