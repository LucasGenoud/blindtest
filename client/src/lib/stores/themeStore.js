import { writable } from 'svelte/store';

/**
 * Sodium ships dark: that is the ground the palette, the greyscale-media rule and
 * the contrast values were built for. Light is available as a choice and is
 * derived from the light ground the guidelines define in section 2.
 */
const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('theme') : null;

export const theme = writable(stored === 'light' ? 'light' : 'dark');

theme.subscribe((value) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('theme', value);
  }
});
