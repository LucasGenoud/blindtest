import { get } from 'svelte/store';
import { token } from './stores/userStore.js';

const DEV = import.meta.env.DEV;

export function getApi() {
  if (typeof window !== 'undefined' && window.__API_URL__) return window.__API_URL__;
  if (import.meta.env.VITE_API_URL) return import.meta.env.VITE_API_URL;
  return DEV ? 'http://localhost:3080' : 'http://localhost:3080';
}

export function getWs() {
  if (typeof window !== 'undefined' && window.__WS_URL__) return window.__WS_URL__;
  if (import.meta.env.VITE_WS_URL) return import.meta.env.VITE_WS_URL;
  return DEV ? 'ws://localhost:3080/ws' : 'ws://localhost:3080/ws';
}

/** A non-2xx response. `status` lets callers tell 401 from 500 without parsing text. */
export class ApiError extends Error {
  constructor(message, status) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

async function errorFrom(res) {
  let detail = '';
  try {
    detail = await res.text();
    // The server answers with JSON strings ("Forbidden"), so unwrap the quotes.
    const parsed = JSON.parse(detail);
    if (typeof parsed === 'string') detail = parsed;
    else if (parsed?.error) detail = parsed.error;
  } catch {
    // Not JSON; the raw text is the best message available.
  }
  return new ApiError(detail || res.statusText, res.status);
}

/**
 * The single place a request is built.
 *
 * Every call site used to spell out the base URL, the Authorization header and
 * its own res.ok check, which is how two screens ended up sending no token at
 * all. The token is read from the store here so no caller has to remember it.
 *
 * @param {string} path      path beginning with '/'
 * @param {object} [options]
 * @param {string} [options.method='GET']
 * @param {any}    [options.body]        serialised as JSON when present
 * @param {boolean}[options.auth=true]   attach the token when one exists
 * @param {'json'|'blob'|'none'} [options.parse='json']
 * @param {typeof fetch} [options.fetch] SvelteKit's fetch inside a load()
 */
async function request(path, options = {}) {
  const {
    method = 'GET',
    body,
    auth = true,
    parse = 'json',
    fetch: fetchFn = globalThis.fetch,
    signal,
  } = options;

  const headers = {};
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  if (auth) {
    const t = get(token);
    if (t) headers.Authorization = t;
  }

  const res = await fetchFn(`${getApi()}${path}`, {
    method,
    headers,
    signal,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (!res.ok) throw await errorFrom(res);
  if (parse === 'none') return null;
  if (parse === 'blob') return res.blob();
  return res.json();
}

export const api = {
  get: (path, options) => request(path, { ...options, method: 'GET' }),
  post: (path, body, options) => request(path, { ...options, method: 'POST', body }),
  del: (path, options) => request(path, { ...options, method: 'DELETE' }),
};

/**
 * For the many places that only want the data and treat any failure as "nothing
 * to show". Keeps `try {} catch {}` out of every component.
 */
export async function apiTry(promise, fallback = null) {
  try {
    return await promise;
  } catch {
    return fallback;
  }
}
