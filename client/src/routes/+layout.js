import { browser } from '$app/environment';
import { api, ApiError } from '$lib/api.js';

// Runs on the client only: the session lives in localStorage, which does not
// exist during SSR. Child routes read `user` from here via `await parent()`.
export async function load({ fetch }) {
  if (!browser) return { user: null, invalidToken: false };

  const t = localStorage.getItem('token');
  if (!t) return { user: null, invalidToken: false };

  try {
    return { user: await api.get('/getuser', { fetch }), invalidToken: false };
  } catch (e) {
    // A rejected token is worth dropping; an unreachable server is not.
    if (e instanceof ApiError) {
      localStorage.removeItem('token');
      return { user: null, invalidToken: true };
    }
    return { user: null, invalidToken: false };
  }
}
