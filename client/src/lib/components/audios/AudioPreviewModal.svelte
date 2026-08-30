<script>
  import { fade, fly } from 'svelte/transition';
  import { X } from 'lucide-svelte';
  import { getApi } from '$lib/api.js';

  let { audio, onclose } = $props();
</script>

<div class="overlay" in:fade={{ duration: 150 }}
     onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="popup preview-popup" in:fly={{ y: 20, duration: 200 }}>
    <div class="popup-header">
      <div class="preview-title-group">
        <span class="cat-badge">{audio.category}</span>
        <span class="popup-title">{audio.answer}</span>
      </div>
      <button class="close-btn" onclick={onclose} aria-label="Close"><X size={14} stroke-width={1.8} /></button>
    </div>
    <div class="video-container">
      <!-- svelte-ignore a11y_media_has_caption -->
      <video src="{getApi()}/media/{audio._id}" controls autoplay class="video-player">
        Your browser does not support video playback.
      </video>
    </div>
    <div class="preview-meta">
      <span class="meta-item"><span class="meta-label">Start</span> {audio.startTime}s</span>
      <span class="meta-item"><span class="meta-label">Plays</span> {audio.count}</span>
      <span class="meta-item"><span class="meta-label">Added by</span> {audio.submittedByUsername || '—'}</span>
    </div>
  </div>
</div>

<style>
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
  .cat-badge {
    font-size: 13px;
    color: var(--text-dim);
  }
</style>
