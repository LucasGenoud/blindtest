# Canvas Click Sound — Design

## Goal
Add subtle, satisfying UI click sounds to the canvas page's two interaction points:
1. **Select** — clicking to highlight a pixel
2. **Paint** — pressing Space to place a pixel

## Architecture

### Sound Utility (`client/src/lib/sound.js`)
- Single module that holds one `AudioContext` (created lazily on first use)
- Exports `playSelect()` and `playPaint()` functions
- Each function plays a short oscillator tone:
  - **Select**: 800Hz sine, ~40ms duration, quick attack/decay
  - **Paint**: 500Hz sine, ~60ms duration, quick attack/decay
- Both respect the existing `volume` store from `gameStore.js` (0-100 scale)
- If volume is 0 or AudioContext is unavailable, silently skip

### Canvas Page Changes (`client/src/routes/canvas/+page.svelte`)
- Import `playSelect`, `playPaint` from `sound.js`
- Call `playSelect()` at the end of `handleClick()` (pixel selection path only)
- Call `playPaint()` at the start of `placePixel()` (pixel placement via Space bar)

## No External Files
All sounds are generated procedurally via Web Audio API. No `.mp3`/`.wav` assets.

## Error Handling
- AudioContext creation is wrapped in try/catch — failure is silent (graceful degradation)
- `playSelect`/`playPaint` are wrapped in try/catch so a sound error never breaks canvas interaction
