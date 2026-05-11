<script>
  import { onMount, onDestroy } from 'svelte';
  import { getApi } from '$lib/api.js';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { colors, debounce } from '$lib/misc.js';

  const SIZE = 1000;
  const MIN_ZOOM = 0.5;
  const MAX_ZOOM = 40;
  const ZOOM_FACTOR = 1.15;

  let canvasEl = $state(null);
  let viewportEl = $state(null);
  let ctx = $state(null);
  let imageData = $state(null);
  let selectedColor = $state(colors[0]);
  let pixelInfo = $state(null);
  let pixelInfoPos = $state({ x: 0, y: 0 });
  let otherUsers = $state({});
  let pixelData = $state([]); // flat hex array

  // --- Transform state (world coordinates) ---
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);

  // --- Interaction state ---
  let dragging = $state(false);
  let dragStartScreen = $state({ x: 0, y: 0 });
  let dragStartPan = $state({ x: 0, y: 0 });
  let didDrag = $state(false);
  let hoverPixel = $state(null); // { x, y } from mouse movement
  let selectedPixel = $state(null); // { x, y } selected via click or arrow keys
  let showGrid = $state(true);

  // --- Touch state ---
  let lastTouchDist = $state(0);
  let lastTouchCenter = $state({ x: 0, y: 0 });

  onMount(async () => {
    ctx = canvasEl.getContext('2d');
    imageData = ctx.createImageData(SIZE, SIZE);

    try {
      const res = await fetch(`${getApi()}/getCanvas`);
      if (res.ok) {
        pixelData = await res.json();
        drawCanvas();
      }
    } catch {}

    if ($websocket) {
      $websocket.addEventListener('message', onWsMessage);
    }

    centerCanvas();
  });

  onDestroy(() => {
    if ($websocket) {
      $websocket.removeEventListener('message', onWsMessage);
    }
  });

  function centerCanvas() {
    if (!viewportEl) return;
    const vw = viewportEl.clientWidth;
    const vh = viewportEl.clientHeight;
    const fitZoom = Math.min(vw / SIZE, vh / SIZE) * 0.85;
    zoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, fitZoom));
    panX = (vw - SIZE * zoom) / 2;
    panY = (vh - SIZE * zoom) / 2;
  }

  function onWsMessage(e) {
    const msg = JSON.parse(e.data);
    if (msg.type === 'updatePixel') {
      const { selectedPixel, selectedColor: sc } = msg.data;
      const hex = sc.hex || 'ffffff';
      const idx = selectedPixel.y * SIZE + selectedPixel.x;
      pixelData[idx] = hex;
      setPixelColor(selectedPixel.x, selectedPixel.y, hex);
      ctx.putImageData(imageData, 0, 0);
    } else if (msg.type === 'userPosition') {
      if ($user && msg.user.username === $user.name) return;
      otherUsers = { ...otherUsers, [msg.user.wsId]: { ...msg.user, ...msg.data } };
    } else if (msg.type === 'removeUser') {
      const { [msg.wsId]: _, ...rest } = otherUsers;
      otherUsers = rest;
    }
  }

  function drawCanvas() {
    for (let i = 0; i < pixelData.length; i++) {
      const hex = pixelData[i] || 'ffffff';
      const x = i % SIZE;
      const y = Math.floor(i / SIZE);
      setPixelColor(x, y, hex);
    }
    ctx.putImageData(imageData, 0, 0);
  }

  function setPixelColor(x, y, hex) {
    const idx = (y * SIZE + x) * 4;
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    imageData.data[idx] = r;
    imageData.data[idx + 1] = g;
    imageData.data[idx + 2] = b;
    imageData.data[idx + 3] = 255;
  }

  // --- Coordinate conversions ---
  function screenToWorld(sx, sy) {
    const rect = viewportEl.getBoundingClientRect();
    const rx = sx - rect.left;
    const ry = sy - rect.top;
    return {
      x: (rx - panX) / zoom,
      y: (ry - panY) / zoom,
    };
  }

  function worldToViewport(wx, wy) {
    return {
      x: wx * zoom + panX,
      y: wy * zoom + panY,
    };
  }

  function getCanvasPixel(e) {
    const { x, y } = screenToWorld(e.clientX, e.clientY);
    return { x: Math.floor(x), y: Math.floor(y) };
  }

  function zoomAtPoint(screenX, screenY, newZoom) {
    const rect = viewportEl.getBoundingClientRect();
    const mx = screenX - rect.left;
    const my = screenY - rect.top;

    const wx = (mx - panX) / zoom;
    const wy = (my - panY) / zoom;

    const clampedZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newZoom));

    panX = mx - wx * clampedZoom;
    panY = my - wy * clampedZoom;
    zoom = clampedZoom;
  }

  function centerOnPixel(pixel) {
    if (!viewportEl || !pixel) return;
    const rect = viewportEl.getBoundingClientRect();
    panX = rect.width / 2 - (pixel.x + 0.5) * zoom;
    panY = rect.height / 2 - (pixel.y + 0.5) * zoom;
  }

  // --- Event handlers ---
  function handleWheel(e) {
    e.preventDefault();
    const direction = e.deltaY > 0 ? -1 : 1;
    const factor = direction > 0 ? ZOOM_FACTOR : 1 / ZOOM_FACTOR;
    zoomAtPoint(e.clientX, e.clientY, zoom * factor);
  }

  function handleMouseDown(e) {
    // Left, middle, or right click all can pan
    startDrag(e);
    e.preventDefault();
  }

  function startDrag(e) {
    dragging = true;
    didDrag = false;
    dragStartScreen = { x: e.clientX, y: e.clientY };
    dragStartPan = { x: panX, y: panY };
  }

  function sendPosition(pixel) {
    if ($websocket && $websocket.readyState === 1 && pixel) {
      $websocket.send(JSON.stringify({ type: 'userPosition', data: { x: pixel.x, y: pixel.y } }));
    }
  }

  function handleMouseMove(e) {
    const pixel = getCanvasPixel(e);
    if (pixel.x >= 0 && pixel.x < SIZE && pixel.y >= 0 && pixel.y < SIZE) {
      hoverPixel = pixel;
    } else {
      hoverPixel = null;
    }

    if (dragging) {
      const dx = e.clientX - dragStartScreen.x;
      const dy = e.clientY - dragStartScreen.y;
      if (Math.abs(dx) > 2 || Math.abs(dy) > 2) didDrag = true;
      panX = dragStartPan.x + dx;
      panY = dragStartPan.y + dy;
    }

    sendPosition(hoverPixel);
    debouncedPixelQuery(hoverPixel, e.clientX, e.clientY);
  }

  function handleMouseUp() {
    dragging = false;
  }

  function placePixel(x, y) {
    if (!$token) return;
    if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;

    fetch(`${getApi()}/updatePixel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({
        pixel: {
          selectedPixel: { x, y },
          selectedColor: selectedColor,
        },
      }),
    });

    const idx = y * SIZE + x;
    pixelData[idx] = selectedColor.hex;
    setPixelColor(x, y, selectedColor.hex);
    ctx.putImageData(imageData, 0, 0);
  }

  function handleClick(e) {
    if (didDrag) return;
    const { x, y } = getCanvasPixel(e);
    if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;
    selectedPixel = { x, y };
    centerOnPixel(selectedPixel);
    sendPosition(selectedPixel);
    debouncedPixelQuery(selectedPixel, e.clientX, e.clientY);
  }

  // --- Touch support ---
  function getTouchDist(touches) {
    const dx = touches[0].clientX - touches[1].clientX;
    const dy = touches[0].clientY - touches[1].clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  function getTouchCenter(touches) {
    return {
      x: (touches[0].clientX + touches[1].clientX) / 2,
      y: (touches[0].clientY + touches[1].clientY) / 2,
    };
  }

  function handleTouchStart(e) {
    if (e.touches.length === 1) {
      startDrag({ clientX: e.touches[0].clientX, clientY: e.touches[0].clientY });
    } else if (e.touches.length === 2) {
      e.preventDefault();
      lastTouchDist = getTouchDist(e.touches);
      lastTouchCenter = getTouchCenter(e.touches);
      dragging = false;
    }
  }

  function handleTouchMove(e) {
    if (e.touches.length === 1 && dragging) {
      const dx = e.touches[0].clientX - dragStartScreen.x;
      const dy = e.touches[0].clientY - dragStartScreen.y;
      if (Math.abs(dx) > 2 || Math.abs(dy) > 2) didDrag = true;
      panX = dragStartPan.x + dx;
      panY = dragStartPan.y + dy;
    } else if (e.touches.length === 2) {
      e.preventDefault();
      const dist = getTouchDist(e.touches);
      const center = getTouchCenter(e.touches);
      const scaleFactor = dist / lastTouchDist;
      zoomAtPoint(center.x, center.y, zoom * scaleFactor);
      panX += center.x - lastTouchCenter.x;
      panY += center.y - lastTouchCenter.y;
      lastTouchDist = dist;
      lastTouchCenter = center;
    }
  }

  function handleTouchEnd(e) {
    if (e.touches.length < 2) {
      dragging = false;
    }
  }

  // --- Keyboard shortcuts ---
  function handleKeyDown(e) {
    // Ignore keypresses if user is typing in a chat or input
    if (document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA') {
      return;
    }

    if (e.key === 'r' || e.key === 'R') {
      centerCanvas();
      return;
    }
    if (e.key === '+' || e.key === '=') {
      zoomIn();
      return;
    }
    if (e.key === '-') {
      zoomOut();
      return;
    }

    // Navigation and placement
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (selectedPixel) selectedPixel = { x: selectedPixel.x, y: Math.max(0, selectedPixel.y - 1) };
      else selectedPixel = { x: Math.floor(SIZE/2), y: Math.floor(SIZE/2) };
      centerOnPixel(selectedPixel);
      sendPosition(selectedPixel);
      debouncedPixelQuery(selectedPixel);
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (selectedPixel) selectedPixel = { x: selectedPixel.x, y: Math.min(SIZE - 1, selectedPixel.y + 1) };
      else selectedPixel = { x: Math.floor(SIZE/2), y: Math.floor(SIZE/2) };
      centerOnPixel(selectedPixel);
      sendPosition(selectedPixel);
      debouncedPixelQuery(selectedPixel);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      if (selectedPixel) selectedPixel = { x: Math.max(0, selectedPixel.x - 1), y: selectedPixel.y };
      else selectedPixel = { x: Math.floor(SIZE/2), y: Math.floor(SIZE/2) };
      centerOnPixel(selectedPixel);
      sendPosition(selectedPixel);
      debouncedPixelQuery(selectedPixel);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      if (selectedPixel) selectedPixel = { x: Math.min(SIZE - 1, selectedPixel.x + 1), y: selectedPixel.y };
      else selectedPixel = { x: Math.floor(SIZE/2), y: Math.floor(SIZE/2) };
      centerOnPixel(selectedPixel);
      sendPosition(selectedPixel);
      debouncedPixelQuery(selectedPixel);
    } else if (e.key === ' ') {
      e.preventDefault();
      if (selectedPixel) {
        placePixel(selectedPixel.x, selectedPixel.y);
      }
    }
  }

  const debouncedPixelQuery = debounce(async (pixel, clientX, clientY) => {
    if (!pixel || pixel.x < 0 || pixel.x >= SIZE || pixel.y < 0 || pixel.y >= SIZE) { 
      pixelInfo = null; 
      return; 
    }
    
    if (clientX === undefined || clientY === undefined) {
      if (viewportEl) {
        const rect = viewportEl.getBoundingClientRect();
        const pos = worldToViewport(pixel.x + 0.5, pixel.y + 0.5);
        pixelInfoPos = { x: rect.left + pos.x, y: rect.top + pos.y };
      }
    } else {
      pixelInfoPos = { x: clientX, y: clientY };
    }

    try {
      const res = await fetch(`${getApi()}/getPixelData?pixel=${JSON.stringify(pixel)}`);
      if (res.ok) pixelInfo = await res.json();
    } catch {}
  }, 200);

  // --- Zoom control helpers ---
  function zoomIn() {
    if (!viewportEl) return;
    const rect = viewportEl.getBoundingClientRect();
    zoomAtPoint(rect.left + rect.width / 2, rect.top + rect.height / 2, zoom * ZOOM_FACTOR);
  }

  function zoomOut() {
    if (!viewportEl) return;
    const rect = viewportEl.getBoundingClientRect();
    zoomAtPoint(rect.left + rect.width / 2, rect.top + rect.height / 2, zoom / ZOOM_FACTOR);
  }

  let gridVisible = $derived(showGrid && zoom >= 8);
  let gridSize = $derived(zoom);
</script>

<svelte:window onkeydown={handleKeyDown} />
<svelte:head><title>Community Canvas — Blindtest</title></svelte:head>

<div class="canvas-page"
  oncontextmenu={(e) => e.preventDefault()}
>
  <!-- Color Palette -->
  <div class="palette">
    <span class="palette-label">Palette</span>
    <div class="palette-colors">
      {#each colors as c (c.index)}
        <div class="color-swatch" class:selected={selectedColor.index === c.index}
          style="background:#{c.hex}" onclick={() => selectedColor = c}
          title={c.name}></div>
      {/each}
    </div>
    <div class="palette-info">
      <div class="selected-color-preview" style="background:#{selectedColor.hex}"></div>
      <span class="selected-color-name">{selectedColor.name}</span>
    </div>
  </div>

  <!-- Canvas viewport -->
  <div class="canvas-viewport"
    class:hide-cursor={!!hoverPixel}
    bind:this={viewportEl}
    onwheel={handleWheel}
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    onclick={handleClick}
    ontouchstart={handleTouchStart}
    ontouchmove={handleTouchMove}
    ontouchend={handleTouchEnd}
  >
    <!-- Canvas element -->
    <canvas
      bind:this={canvasEl}
      width={SIZE} height={SIZE}
      style="transform: matrix({zoom}, 0, 0, {zoom}, {panX}, {panY}); image-rendering: pixelated;"
    ></canvas>

    <!-- Grid overlay -->
    {#if gridVisible}
      <svg class="grid-overlay" style="
        position: absolute; top: 0; left: 0;
        width: 100%; height: 100%;
        pointer-events: none;
      ">
        <defs>
          <pattern id="pixelGrid" width={gridSize} height={gridSize} patternUnits="userSpaceOnUse"
            x={panX % gridSize} y={panY % gridSize}>
            <path d="M {gridSize} 0 L 0 0 0 {gridSize}" fill="none"
              stroke="rgba(255,255,255,0.08)" stroke-width="0.5"/>
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#pixelGrid)"/>
      </svg>
    {/if}

    <!-- Hover highlight -->
    {#if hoverPixel && zoom >= 3}
      {@const pos = worldToViewport(hoverPixel.x, hoverPixel.y)}
      <div class="hover-highlight" style="
        left: {pos.x}px;
        top: {pos.y}px;
        width: {zoom}px;
        height: {zoom}px;
      "></div>
    {/if}

    <!-- Selected pixel highlight -->
    {#if selectedPixel && zoom >= 2}
      {@const pos = worldToViewport(selectedPixel.x, selectedPixel.y)}
      <div class="selected-highlight" style="
        left: {pos.x}px;
        top: {pos.y}px;
        width: {zoom}px;
        height: {zoom}px;
      ">
        <span class="selected-crosshair tl"></span>
        <span class="selected-crosshair tr"></span>
        <span class="selected-crosshair bl"></span>
        <span class="selected-crosshair br"></span>
      </div>
    {/if}

    <!-- Other users cursors -->
    {#each Object.values(otherUsers) as u (u.wsId)}
      {#if !$user || u.username !== $user.name}
        {@const pos = worldToViewport(u.x + 0.5, u.y + 0.5)}
        <div class="user-cursor" style="left:{pos.x}px;top:{pos.y}px">
          <svg width="12" height="16" viewBox="0 0 12 16" fill="none">
            <path d="M1 1L1 12L4.5 8.5L8 14L10 13L6.5 7L11 7L1 1Z" fill="var(--accent)" stroke="var(--bg)" stroke-width="1"/>
          </svg>
          <span class="cursor-name">{u.username}</span>
        </div>
      {/if}
    {/each}

    <!-- Local user cursor (shown at selected pixel or hover) -->
    {#if selectedPixel || hoverPixel}
      {@const cursorPixel = selectedPixel ?? hoverPixel}
      {@const pos = worldToViewport(cursorPixel.x + 0.5, cursorPixel.y + 0.5)}
      <div class="user-cursor local-cursor" style="left:{pos.x}px;top:{pos.y}px">
        <svg width="12" height="16" viewBox="0 0 12 16" fill="none">
          <path d="M1 1L1 12L4.5 8.5L8 14L10 13L6.5 7L11 7L1 1Z" fill="var(--blue)" stroke="var(--bg)" stroke-width="1"/>
        </svg>
        <span class="cursor-name">{$user ? $user.name : 'Guest'} (You)</span>
      </div>
    {/if}
  </div>

  <!-- HUD: Zoom controls -->
  <div class="hud-controls">
    <button class="hud-btn" onclick={zoomOut} title="Zoom out (-)">−</button>
    <span class="hud-zoom">{Math.round(zoom * 100)}%</span>
    <button class="hud-btn" onclick={zoomIn} title="Zoom in (+)">+</button>
    <div class="hud-divider"></div>
    <button class="hud-btn" onclick={centerCanvas} title="Reset view (R)">⌂</button>
    <button class="hud-btn hud-btn-text" class:active={showGrid} onclick={() => showGrid = !showGrid} title="Toggle grid">
      Grid
    </button>
    <div class="hud-divider"></div>
    <span class="hud-hint">Click to select · Arrows to move · Space to paint</span>
  </div>

  <!-- HUD: Coordinates -->
  {#if selectedPixel || hoverPixel}
    {@const displayPixel = selectedPixel ?? hoverPixel}
    <div class="hud-coords">
      <span class="coord">X: {displayPixel.x}</span>
      <span class="coord">Y: {displayPixel.y}</span>
      {#if selectedPixel}
        <span class="coord coord-hint">· SPACE to paint</span>
      {/if}
    </div>
  {/if}

  <!-- Pixel info tooltip -->
  {#if pixelInfo && pixelInfo.username}
    <div class="pixel-tooltip" style="left:{pixelInfoPos.x + 16}px;top:{pixelInfoPos.y}px">
      <span class="tooltip-user">{pixelInfo.username}</span>
      {#if pixelInfo.d}
        <span class="tooltip-date">{new Date(pixelInfo.d).toLocaleString()}</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .canvas-page {
    width: 100%; height: 100%;
    display: flex; flex-direction: column;
    overflow: hidden; position: relative;
    background: var(--bg);
  }

  .palette {
    display: flex; align-items: center; gap: 12px;
    padding: 8px 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    z-index: 10;
  }
  .palette-label {
    font-family: var(--mono);
    font-size: 0.6rem;
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
    width: 22px; height: 22px; border-radius: 4px;
    cursor: pointer; border: 2px solid transparent;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .color-swatch:hover { transform: scale(1.18); z-index: 2; }
  .color-swatch.selected {
    border-color: var(--text-primary);
    box-shadow: 0 0 8px rgba(245, 245, 247, 0.3);
    transform: scale(1.12);
  }
  .palette-info {
    display: flex; align-items: center; gap: 8px;
    padding-left: 12px;
    border-left: 1px solid var(--border);
    flex-shrink: 0;
  }
  .selected-color-preview {
    width: 28px; height: 28px;
    border-radius: 6px;
    border: 1px solid var(--border-2);
  }
  .selected-color-name {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    min-width: 80px;
  }

  .canvas-viewport {
    flex: 1; overflow: hidden; position: relative;
    background:
      radial-gradient(circle at 50% 50%, rgba(30, 30, 40, 1) 0%, rgba(10, 10, 11, 1) 100%);
  }
  .canvas-viewport.hide-cursor {
    cursor: none;
  }
  canvas {
    position: absolute;
    top: 0; left: 0;
    transform-origin: 0 0;
  }

  .grid-overlay {
    position: absolute; inset: 0;
    pointer-events: none;
    z-index: 2;
  }

  .hover-highlight {
    position: absolute;
    pointer-events: none;
    z-index: 3;
    border: 1.5px solid rgba(255,255,255,0.4);
    mix-blend-mode: normal;
  }

  .selected-highlight {
    position: absolute;
    pointer-events: none;
    z-index: 4;
    outline: 2px solid #fff;
    outline-offset: 1px;
    box-shadow: 0 0 0 3px rgba(0,0,0,0.5), 0 0 12px rgba(255,255,255,0.2);
  }
  .selected-crosshair {
    position: absolute;
    width: 4px;
    height: 4px;
    border-color: #fff;
    border-style: solid;
  }
  .selected-crosshair.tl { top: -3px; left: -3px; border-width: 2px 0 0 2px; }
  .selected-crosshair.tr { top: -3px; right: -3px; border-width: 2px 2px 0 0; }
  .selected-crosshair.bl { bottom: -3px; left: -3px; border-width: 0 0 2px 2px; }
  .selected-crosshair.br { bottom: -3px; right: -3px; border-width: 0 2px 2px 0; }

  .user-cursor {
    position: absolute; pointer-events: none;
    z-index: 5;
    display: flex; align-items: flex-start; gap: 2px;
    transform: translate(-1px, -1px);
    transition: left 0.1s linear, top 0.1s linear;
  }
  .local-cursor {
    z-index: 6;
    transition: none; /* Local cursor should feel instant */
  }
  .local-cursor .cursor-name {
    color: var(--blue);
  }
  .cursor-name {
    font-family: var(--mono);
    font-size: 0.55rem;
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 1px 5px;
    border-radius: 3px;
    color: var(--accent);
    white-space: nowrap;
    margin-top: 2px;
    opacity: 0.9;
  }

  .hud-controls {
    position: absolute;
    bottom: 16px; left: 50%;
    transform: translateX(-50%);
    display: flex; align-items: center; gap: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px 6px;
    z-index: 20;
    backdrop-filter: blur(10px);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }
  .hud-btn {
    width: 30px; height: 30px;
    border-radius: 5px;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--mono);
    font-size: 16px;
    cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: all 0.15s;
    padding: 0;
  }
  .hud-btn:hover {
    background: var(--surface-2);
  }
  .hud-btn.hud-btn-text {
    width: auto;
    font-size: 0.6rem;
    padding: 0 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-dim);
  }
  .hud-btn.hud-btn-text.active {
    color: var(--accent);
    background: var(--accent-dim);
  }
  .hud-zoom {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--text-secondary);
    min-width: 48px;
    text-align: center;
    user-select: none;
  }
  .hud-hint {
    font-family: var(--mono);
    font-size: 0.55rem;
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
    border-radius: 6px;
    padding: 5px 10px;
    z-index: 20;
    backdrop-filter: blur(10px);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }
  .coord {
    font-family: var(--mono);
    font-size: 0.62rem;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
  }
  .coord-hint {
    color: var(--accent);
    opacity: 0.8;
  }

  .pixel-tooltip {
    position: fixed; z-index: 100;
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 6px 12px;
    border-radius: 6px;
    pointer-events: none;
    display: flex; flex-direction: column; gap: 2px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  }
  .tooltip-user {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-primary);
    font-weight: 500;
  }
  .tooltip-date {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-dim);
  }
</style>
