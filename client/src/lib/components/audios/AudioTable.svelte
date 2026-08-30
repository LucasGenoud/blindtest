<script>
  import { Play, Flag, ChevronUp, ChevronDown } from 'lucide-svelte';
  import { statusClass, statusLabel } from '$lib/audios/audioLibrary.js';
  import { userPermission } from '$lib/stores/userStore.js';

  let {
    audios = [],
    sortKey = $bindable('addedDate'),
    sortDir = $bindable('desc'),
    onpreview,
    onreprocess,
    onedit,
    ondelete,
    onresetflag,
  } = $props();

  const COLUMNS = [
    { key: 'processingStatus', label: 'Status' },
    { key: 'category', label: 'Category' },
    { key: 'answer', label: 'Answer' },
    { key: 'startTime', label: 'Start' },
    { key: 'count', label: 'Plays' },
    { key: null, label: 'Flags' },
    { key: 'submittedByUsername', label: 'By' },
    { key: 'addedDate', label: 'Added' },
    { key: null, label: 'Actions' },
  ];

  function setSort(key) {
    if (sortKey === key) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = key;
      sortDir = 'asc';
    }
  }
</script>

<table>
  <thead>
    <tr>
      {#each COLUMNS as col}
        <th>
          {#if col.key}
            <button class="sort-btn" onclick={() => setSort(col.key)}>
              {col.label}
              {#if sortKey === col.key && sortDir === 'asc'}<ChevronUp size={11} stroke-width={1.8} />
              {:else if sortKey === col.key}<ChevronDown size={11} stroke-width={1.8} />{/if}
            </button>
          {:else}
            {col.label}
          {/if}
        </th>
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each audios as audio (audio._id)}
      <tr class:flagged={audio.flagged?.length > 0}>
        <td><span class="status {statusClass(audio.processingStatus)}">{statusLabel(audio.processingStatus)}</span></td>
        <td><span class="cat-badge">{audio.category}</span></td>
        <td class="answer-cell">
          <span class="answer-text">{audio.answer}</span>
          {#if audio.superflus}<span class="sup-tag">superflus</span>{/if}
        </td>
        <td class="mono">{audio.startTime}s</td>
        <td class="mono">{audio.count}</td>
        <td>
          {#if audio.flagged?.length > 0}
            <div class="flag-cell">
              <span class="flag-count"><Flag size={12} stroke-width={1.8} /> {audio.flagged.length}</span>
              <button class="btn-xs" onclick={() => onresetflag(audio._id)}>Reset</button>
            </div>
          {:else}
            <span class="dim">—</span>
          {/if}
        </td>
        <td class="mono dim-cell">{audio.submittedByUsername || '—'}</td>
        <td class="mono dim-cell">{audio.addedDate ? audio.addedDate.slice(0, 10) : '—'}</td>
        <td>
          <div class="action-group">
            {#if audio.processingStatus === 'ready' && audio.s3ObjectKey}
              <button class="btn-xs" title="Preview audio" aria-label="Preview audio" onclick={() => onpreview(audio)}><Play size={14} stroke-width={2} /></button>
            {/if}
            {#if audio.processingStatus === 'error'}
              <button class="btn-xs" title="Retry processing" onclick={() => onreprocess(audio._id)}>Retry</button>
            {/if}
            <a href={audio.videoUrl} target="_blank" class="btn-xs" title="Open source">Source</a>
            <button class="btn-xs" title="Edit" onclick={() => onedit(audio)}>Edit</button>
            {#if $userPermission >= 3}
              <button class="btn-xs btn-del" title="Delete" onclick={() => ondelete(audio._id)}>Delete</button>
            {/if}
          </div>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
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

  /* ── Dim cell ── */
  .dim-cell {
    color: var(--text-dim);
    font-size: 11px;
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
</style>
