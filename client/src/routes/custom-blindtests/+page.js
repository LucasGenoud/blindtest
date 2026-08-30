import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';

export async function load({ parent }) {
  const { user } = await parent();
  if (browser && !user) redirect(303, '/');
  return {};
}
