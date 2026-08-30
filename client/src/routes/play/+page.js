import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { get } from 'svelte/store';
import { blindtestStatus } from '$lib/stores/gameStore.js';

// blindtestStatus is restored from localStorage, so a reload mid-game stays on /play.
export async function load() {
  if (browser && get(blindtestStatus) !== 'started') redirect(303, '/');
  return {};
}
