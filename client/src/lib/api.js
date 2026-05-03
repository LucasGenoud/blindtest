const DEV = import.meta.env.DEV;

export function getApi() {
  if (typeof window !== 'undefined' && window.__API_URL__) return window.__API_URL__;
  return DEV ? 'http://localhost:3080' : 'http://localhost:3080';
}

export function getWs() {
  if (typeof window !== 'undefined' && window.__WS_URL__) return window.__WS_URL__;
  return DEV ? 'ws://localhost:3080/ws' : 'ws://localhost:3080/ws';
}

export async function apiFetch(path, options = {}) {
  const url = `${getApi()}${path}`;
  const res = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || res.statusText);
  }
  return res.json();
}

export function authHeaders(token) {
  return { Authorization: token };
}
