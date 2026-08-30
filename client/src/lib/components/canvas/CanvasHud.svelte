<script>
  import { Minus, Plus, Home, Grid3X3 } from 'lucide-svelte';

  let {
    zoom = 1,
    showGrid = $bindable(true),
    pixel = null,
    pixelIsSelected = false,
    onzoomin,
    onzoomout,
    onreset,
  } = $props();
</script>

<div class="hud-controls">
  <button class="hud-btn" onclick={onzoomout} title="Zoom out (-)" aria-label="Zoom out"><Minus size={14} stroke-width={1.8} /></button>
  <span class="hud-zoom tabular">{Math.round(zoom * 100)}%</span>
  <button class="hud-btn" onclick={onzoomin} title="Zoom in (+)" aria-label="Zoom in"><Plus size={14} stroke-width={1.8} /></button>
  <div class="hud-divider"></div>
  <button class="hud-btn" onclick={onreset} title="Reset view (R)" aria-label="Reset view"><Home size={14} stroke-width={1.8} /></button>
  <button class="hud-btn" class:active={showGrid} onclick={() => (showGrid = !showGrid)} title="Toggle grid" aria-label="Toggle grid" aria-pressed={showGrid}>
    <Grid3X3 size={14} stroke-width={1.8} />
  </button>
  <div class="hud-divider"></div>
  <span class="hud-hint">Click to select · Arrows to move · Space to paint</span>
</div>

{#if pixel}
  <div class="hud-coords">
    <span class="coord tabular">X: {pixel.x}</span>
    <span class="coord tabular">Y: {pixel.y}</span>
    {#if pixelIsSelected}
      <span class="coord coord-hint">· SPACE to paint</span>
    {/if}
  </div>
{/if}

<style>
  .hud-controls {
    position: absolute;
    bottom: 16px; left: 50%;
    transform: translateX(-50%);
    display: flex; align-items: center; gap: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    padding: 4px 6px;
    z-index: 20;
  }

  .hud-btn {
    width: 30px; height: 30px;
    border-radius: 0;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--mono);
    font-size: 15px;
    cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    padding: 0;
  }

  .hud-btn:hover {
    background: var(--surface-2);
  }

  .hud-btn.hud-btn-text {
    width: auto;
    font-size: 11px;
    padding: 0 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-dim);
  }

  .hud-btn.hud-btn-text.active {
    color: var(--accent-ink);
    background: var(--surface-2);
  }

  .hud-zoom {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-secondary);
    min-width: 48px;
    text-align: center;
    user-select: none;
  }

  .hud-hint {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-dim);
    letter-spacing: 0.04em;
    white-space: nowrap;
    padding: 0 4px;
    user-select: none;
  }

  .hud-divider {
    width: 1px; height: 18px;
    background: var(--border);
    margin: 0 4px;
  }

  .hud-coords {
    position: absolute;
    bottom: 16px; right: 16px;
    display: flex; gap: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    padding: 5px 10px;
    z-index: 20;
  }

  .coord {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
  }
</style>
