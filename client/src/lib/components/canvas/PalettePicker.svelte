<script>
  import { colors } from '$lib/misc.js';

  /** The drawing palette. These colours are the artwork's, not the interface's. */
  let { selected = $bindable(colors[0]) } = $props();
</script>

<div class="palette">
  <span class="palette-label">Palette</span>
  <div class="palette-colors">
    {#each colors as c (c.index)}
      <button
        class="color-swatch"
        class:selected={selected.index === c.index}
        style="background:#{c.hex}"
        onclick={() => (selected = c)}
        title={c.name}
        aria-label={c.name}
        aria-pressed={selected.index === c.index}
      ></button>
    {/each}
  </div>
  <div class="palette-info">
    <div class="selected-color-preview" style="background:#{selected.hex}"></div>
    <span class="selected-color-name">{selected.name}</span>
  </div>
</div>

<style>
  .palette {
    display: flex; align-items: center; gap: 12px;
    padding: 8px 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    z-index: 10;
  }

  .palette-label {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    flex-shrink: 0;
  }

  .palette-colors {
    display: flex; flex-wrap: wrap; gap: 3px;
    flex: 1;
  }

  .color-swatch {
    width: 22px; height: 22px; border-radius: 0;
    cursor: pointer; border: 2px solid transparent;
    transition: color var(--duration-fast) ease-out, background-color var(--duration-fast) ease-out, border-color var(--duration-fast) ease-out;
    flex-shrink: 0;
  }

  .color-swatch:hover { transform: scale(1.18); z-index: 2; }

  .color-swatch.selected {
    border-color: var(--text-primary);
  }

  .palette-info {
    display: flex; align-items: center; gap: 8px;
    padding-left: 12px;
    border-left: 1px solid var(--border);
    flex-shrink: 0;
  }

  .selected-color-preview {
    width: 28px; height: 28px;
    border-radius: 0;
    border: 1px solid var(--border-2);
  }

  .selected-color-name {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    min-width: 80px;
  }
</style>
