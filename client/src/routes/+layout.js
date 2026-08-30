import { browser } from '$app/environment';
import { getApi } from '$lib/api.js';

// Runs on the client only: the session lives in localStorage, which does not exist
// during SSR. Child routes read `user` from here via `await parent()`.
export async function load({ fetch }) {
  if (!browser) return { user: null, invalidToken: false };

  const t = localStorage.getItem('token');
  if (!t) return { user: null, invalidToken: false };

  try {
    const res = await fetch(`${getApi()}/getuser`, {
      headers: { Authorization: t },
    });
    if (res.ok) {
      return { user: await res.json(), invalidToken: false };
    }
  } catch {
    // Network trouble should not sign the user out; leave the token in place.
    return { user: null, invalidToken: false };
  }

  // The token was rejected, so it is no longer worth keeping.
  localStorage.removeItem('token');
  return { user: null, invalidToken: true };
}
