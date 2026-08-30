import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { permissionOf } from '$lib/stores/userStore.js';

export async function load({ parent }) {
  const { user } = await parent();
  if (browser && permissionOf(user) < 3) redirect(303, '/');
  return {};
}
