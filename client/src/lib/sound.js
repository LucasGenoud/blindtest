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

function play(frequency, duration, attack, level) {
  try {
    if (currentVolume === 0) return;
    const ctx = getAudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.value = frequency;
    const now = ctx.currentTime;
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(currentVolume / 100 * level, now + attack);
    gain.gain.exponentialRampToValueAtTime(0.001, now + duration);
    osc.start(now);
    osc.stop(now + duration);
  } catch {}
}

export const playSelect = () => play(800, 0.04, 0.003, 0.15);
export const playPaint = () => play(500, 0.06, 0.005, 0.2);
