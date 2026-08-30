<script>
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { api, apiTry } from '$lib/api.js';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { colors, debounce } from '$lib/misc.js';
  import {
    ZOOM_FACTOR,
    centerOn, fit, pixelAt, worldToViewport, zoomAtPoint,
  } from '$lib/canvas/canvasView.js';
  import { writeAll, writePixel } from '$lib/canvas/pixelBuffer.js';
  import PalettePicker from '$lib/components/canvas/PalettePicker.svelte';
  import CanvasHud from '$lib/components/canvas/CanvasHud.svelte';
  import { playSelect, playPaint } from '$lib/sound.js';
  import { Paintbrush } from 'lucide-svelte';

  let paintConfirm = $state(null);
  let paintConfirmTimer;

  const SIZE = 1000;

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
  // One object so the pure helpers in canvasView.js can take and return it whole.
  let view = $state({ zoom: 1, panX: 0, panY: 0 });
  const zoom = $derived(view.zoom);
  const panX = $derived(view.panX);
  const panY = $derived(view.panY);

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
      const data = await apiTry(api.get('/getCanvas'));
      if (data) {
        pixelData = data;
        drawCanvas();
      }
    } catch {}

    centerCanvas();
  });

  // Re-attached whenever the socket is replaced, so live updates survive a reconnect
  // and still arrive if the socket opens after this page has mounted.
  $effect(() => {
    const socket = $websocket;
    if (!socket) return;
    socket.addEventListener('message', onWsMessage);
    return () => socket.removeEventListener('message', onWsMessage);
  });

  function centerCanvas() {
    if (!viewportEl) return;
    view = fit(viewportEl.clientWidth, viewportEl.clientHeight, SIZE);
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
    writeAll(imageData, SIZE, pixelData);
    ctx.putImageData(imageData, 0, 0);
  }

  function setPixelColor(x, y, hex) {
    writePixel(imageData, SIZE, x, y, hex);
  }

  // --- Coordinate conversions (the maths lives in canvasView.js) ---
  const viewportRect = () => viewportEl.getBoundingClientRect();

  function getCanvasPixel(e) {
    return pixelAt(view, viewportRect(), e.clientX, e.clientY);
  }

  function zoomAt(screenX, screenY, nextZoom) {
    view = zoomAtPoint(view, viewportRect(), screenX, screenY, nextZoom);
  }

  function centerOnPixel(pixel) {
    if (!viewportEl || !pixel) return;
    view = centerOn(view, viewportRect(), pixel);
  }

  // --- Event handlers ---
  function handleWheel(e) {
    e.preventDefault();
    const direction = e.deltaY > 0 ? -1 : 1;
    const factor = direction > 0 ? ZOOM_FACTOR : 1 / ZOOM_FACTOR;
    zoomAt(e.clientX, e.clientY, zoom * factor);
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
      view = { ...view, panX: dragStartPan.x + dx, panY: dragStartPan.y + dy };
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

    playPaint();

    // Optimistic: the pixel is drawn below regardless, and the broadcast corrects it.
    apiTry(api.post('/updatePixel', {
      pixel: { selectedPixel: { x, y }, selectedColor },
    }));

    const idx = y * SIZE + x;
    pixelData[idx] = selectedColor.hex;
    setPixelColor(x, y, selectedColor.hex);
    ctx.putImageData(imageData, 0, 0);

    paintConfirm = { x, y, hex: selectedColor.hex };
    if (paintConfirmTimer) clearTimeout(paintConfirmTimer);
    paintConfirmTimer = setTimeout(() => paintConfirm = null, 600);
  }

  function handleClick(e) {
    if (didDrag) return;
    const { x, y } = getCanvasPixel(e);
    if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;
    selectedPixel = { x, y };
    sendPosition(selectedPixel);
    debouncedPixelQuery(selectedPixel, e.clientX, e.clientY);
    playSelect();
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
      view = { ...view, panX: dragStartPan.x + dx, panY: dragStartPan.y + dy };
    } else if (e.touches.length === 2) {
      e.preventDefault();
      const dist = getTouchDist(e.touches);
      const center = getTouchCenter(e.touches);
      const scaleFactor = dist / lastTouchDist;
      zoomAt(center.x, center.y, zoom * scaleFactor);
      // Two-finger drag: follow the midpoint as well as the pinch.
      view = {
        ...view,
        panX: view.panX + (center.x - lastTouchCenter.x),
        panY: view.panY + (center.y - lastTouchCenter.y),
      };
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
    // Ignore keypresses if user is typing in an input
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
        const pos = worldToViewport(view, pixel.x + 0.5, pixel.y + 0.5);
        pixelInfoPos = { x: rect.left + pos.x, y: rect.top + pos.y };
      }
    } else {
      pixelInfoPos = { x: clientX, y: clientY };
    }

    pixelInfo = await apiTry(api.get(`/getPixelData?pixel=${encodeURIComponent(JSON.stringify(pixel))}`), pixelInfo);
  }, 200);

  // --- Zoom control helpers ---
  function zoomIn() {
    if (!viewportEl) return;
    const r = viewportRect();
    zoomAt(r.left + r.width / 2, r.top + r.height / 2, zoom * ZOOM_FACTOR);
  }

  function zoomOut() {
    if (!viewportEl) return;
    const r = viewportRect();
    zoomAt(r.left + r.width / 2, r.top + r.height / 2, zoom / ZOOM_FACTOR);
  }

  let gridVisible = $derived(showGrid && zoom >= 8);
  let gridSize = $derived(zoom);
</script>

<svelte:window onkeydown={handleKeyDown} />
<svelte:head><title>Community Canvas — Blindtest</title></svelte:head>

<div class="canvas-page"
  oncontextmenu={(e) => e.preventDefault()}
>
  <PalettePicker bind:selected={selectedColor} />

  <!-- Canvas viewport -->
  <div class="canvas-viewport"
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
      {@const pos = worldToViewport(view, hoverPixel.x, hoverPixel.y)}
      <div class="hover-highlight" style="
        left: {pos.x}px;
        top: {pos.y}px;
        width: {zoom}px;
        height: {zoom}px;
      "></div>
    {/if}

    <!-- Selected pixel highlight -->
    {#if selectedPixel && zoom >= 2}
      {@const pos = worldToViewport(view, selectedPixel.x, selectedPixel.y)}
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

    <!-- Paint confirmation burst -->
    {#if paintConfirm && zoom >= 3}
      {@const pPos = worldToViewport(view, paintConfirm.x + 0.5, paintConfirm.y + 0.5)}
      <div class="paint-confirm" style="left:{pPos.x}px;top:{pPos.y}px;transform:translate(-50%,-50%)" out:fade={{ duration: 500 }}>
        <Paintbrush size={16} stroke-width={1.8} />
      </div>
    {/if}

    <!-- Other users cursors -->
    {#each Object.values(otherUsers) as u (u.wsId)}
      {#if !$user || u.username !== $user.name}
        {@const pos = worldToViewport(view, u.x + 0.5, u.y + 0.5)}
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
      {@const pos = worldToViewport(view, cursorPixel.x + 0.5, cursorPixel.y + 0.5)}
      <div class="user-cursor local-cursor" style="left:{pos.x}px;top:{pos.y}px">
        <svg width="12" height="16" viewBox="0 0 12 16" fill="none">
          <path d="M1 1L1 12L4.5 8.5L8 14L10 13L6.5 7L11 7L1 1Z" fill="var(--blue)" stroke="var(--bg)" stroke-width="1"/>
        </svg>
        <span class="cursor-name">{$user ? $user.name : 'Guest'} (You)</span>
      </div>
    {/if}
  </div>

  <CanvasHud
    {zoom}
    bind:showGrid
    pixel={selectedPixel ?? hoverPixel}
    pixelIsSelected={!!selectedPixel}
    onzoomin={zoomIn}
    onzoomout={zoomOut}
    onreset={centerCanvas}
  />

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

  .paint-confirm {
    position: absolute;
    pointer-events: none;
    z-index: 7;
    animation: paintConfirm 600ms var(--easing-primary) forwards;
    color: var(--accent-ink);
  }

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
    font-size: 11px;
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 1px 5px;
    border-radius: 0;
    color: var(--accent-ink);
    white-space: nowrap;
    margin-top: 2px;
    opacity: 0.9;
  }

  .coord-hint {
    color: var(--accent-ink);
    opacity: 0.8;
  }

  .pixel-tooltip {
    position: fixed; z-index: 100;
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 6px 12px;
    border-radius: 0;
    pointer-events: none;
    display: flex; flex-direction: column; gap: 2px;
  }

  .tooltip-user {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .tooltip-date {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-dim);
  }
</style>
