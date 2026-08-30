# Canvas Click Sound Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add subtle UI click sounds to the canvas page — one for selecting a pixel, one for painting it.

**Architecture:** A single `sound.js` utility module creates a lazy `AudioContext` and exposes two functions that play short oscillator tones. The canvas page imports both functions and calls them at the appropriate interaction points. All sounds respect the existing `volume` store (0–100 → gain 0.0–1.0).

**Tech Stack:** Web Audio API, Svelte 5 runes, vanilla JS (`$lib` module).

---

### File Map

| File | Action |
|---|---|
| `client/src/lib/sound.js` | **Create** — AudioContext + `playSelect()`, `playPaint()` |
| `client/src/routes/canvas/+page.svelte` | **Modify** — import sounds, call in `handleClick` (line 239) and `placePixel` (line 211) |

---

### Task 1: Create sound utility

**Files:**
- Create: `client/src/lib/sound.js`

- [ ] **Step 1: Write `client/src/lib/sound.js`**

Create the file with the following content:

```js
import { volume } from '$lib/stores/gameStore.js';

let audioCtx = null;

function getAudioContext() {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume();
  }
  return audioCtx;
}

export function playSelect() {
  try {
    const vol = volume; // read current value — this works without $ prefix in non-Svelte files
    if (vol === 0) return;
    const ctx = getAudioContext();
    const gainVal = vol / 100;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.value = 800;
    const now = ctx.currentTime;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(gainVal * 0.15, now + 0.003);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);
    osc.start(now);
    osc.stop(now + 0.04);
  } catch {}
}

export function playPaint() {
  try {
    const vol = volume;
    if (vol === 0) return;
    const ctx = getAudioContext();
    const gainVal = vol / 100;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.value = 500;
    const now = ctx.currentTime;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(gainVal * 0.2, now + 0.005);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.06);
    osc.start(now);
    osc.stop(now + 0.06);
  } catch {}
}
```

Note: `volume` is a Svelte store. In a plain JS file, you read the value via `volume` (it's the store object). However, since this isn't a `.svelte` file, the `$` prefix won't work. The code above references `volume` directly — we need to fix this to read from the store properly.

**Correction:** In a non-Svelte JS file, you read a writable store's current value by subscribing or using a getter. The cleanest approach for a lazy-read utility is to expose a small getter:

Update `getVolume()` inline:

```js
import { volume } from '$lib/stores/gameStore.js';

let currentVolume = 50;
volume.subscribe((v) => { currentVolume = v; });

let audioCtx = null;

function getAudioContext() {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume();
  }
  return audioCtx;
}

export function playSelect() {
  try {
    if (currentVolume === 0) return;
    const ctx = getAudioContext();
    const gainVal = currentVolume / 100;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.value = 800;
    const now = ctx.currentTime;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(gainVal * 0.15, now + 0.003);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);
    osc.start(now);
    osc.stop(now + 0.04);
  } catch {}
}

export function playPaint() {
  try {
    if (currentVolume === 0) return;
    const ctx = getAudioContext();
    const gainVal = currentVolume / 100;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.value = 500;
    const now = ctx.currentTime;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(gainVal * 0.2, now + 0.005);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.06);
    osc.start(now);
    osc.stop(now + 0.06);
  } catch {}
}
```

- [ ] **Step 2: Verify the file exists**

Run: `ls -la client/src/lib/sound.js`
Expected: file exists with the content above.

- [ ] **Step 3: Commit**

```bash
git add client/src/lib/sound.js
git commit -m "feat: add sound utility for canvas UI feedback"
```

---

### Task 2: Wire sounds into canvas page

**Files:**
- Modify: `client/src/routes/canvas/+page.svelte`

- [ ] **Step 1: Add the sound import**

At line 6, after the existing imports, add:

```js
import { playSelect, playPaint } from '$lib/sound.js';
```

Full import block becomes:
```js
  import { onMount, onDestroy } from 'svelte';
  import { getApi } from '$lib/api.js';
  import { token, user } from '$lib/stores/userStore.js';
  import { websocket } from '$lib/stores/websocketStore.js';
  import { colors, debounce } from '$lib/misc.js';
  import { playSelect, playPaint } from '$lib/sound.js';
```

- [ ] **Step 2: Call `playSelect()` in `handleClick`**

At the end of `handleClick` (after line 239, before the closing `}` on line 239), add the sound call. The function currently ends at line 239. Add `playSelect();` as the last line:

Replace lines 232-239:
```js
  function handleClick(e) {
    if (didDrag) return;
    const { x, y } = getCanvasPixel(e);
    if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;
    selectedPixel = { x, y };
    sendPosition(selectedPixel);
    debouncedPixelQuery(selectedPixel, e.clientX, e.clientY);
    playSelect();
  }
```

- [ ] **Step 3: Call `playPaint()` in `placePixel`**

At the start of `placePixel` (after the guard on line 213), add `playPaint();`:

Replace lines 211-230:
```js
  function placePixel(x, y) {
    if (!$token) return;
    if (x < 0 || x >= SIZE || y < 0 || y >= SIZE) return;

    playPaint();

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
```

- [ ] **Step 4: Verify changes**

Open the app locally (`cd client && npm run dev`), navigate to `/canvas`, and test:
1. Click a pixel — should hear a short 800Hz "tick"
2. Press Space on the selected pixel — should hear a deeper 500Hz "pop"
3. Set volume to 0 in game settings — no sound should play

- [ ] **Step 5: Commit**

```bash
git add client/src/routes/canvas/+page.svelte
git commit -m "feat: play click and paint sounds on canvas interactions"
```

---

## Self-Review

- **Spec coverage:** Both `playSelect` (click/select) and `playPaint` (space/paint) are implemented. Volume is read from the existing store. AudioContext is lazy and wrapped in try/catch. All spec requirements covered.
- **No placeholders:** All code is complete with exact frequencies, durations, and gain curves. No TBDs.
- **Type consistency:** `sound.js` imports `volume` from `gameStore.js` using the standard Svelte store subscription pattern. Canvas page imports both functions correctly using `$lib/` alias.
- **Scope check:** Two files, two tasks, no decomposition needed. Fits in one plan.
