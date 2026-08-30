import { get } from 'svelte/store';
import { getWs } from './api.js';
import { websocket } from './stores/websocketStore.js';

const RETRY_DELAYS = [1000, 2000, 5000, 10000, 30000];

let retries = 0;
let retryTimer = null;
let activeToken = null;
let wanted = false;

/**
 * One connection attempt. The previous version polled `readyState` every 10 ms and
 * had no failure path at all, so a refused connection left a 100 Hz interval running
 * for the lifetime of the page and a promise that never settled.
 */
function openSocket(token) {
  return new Promise((resolve, reject) => {
    let socket;
    try {
      socket = new WebSocket(getWs(), token || undefined);
    } catch (err) {
      reject(err);
      return;
    }

    const cleanup = () => {
      socket.removeEventListener('open', onOpen);
      socket.removeEventListener('error', onFailure);
      socket.removeEventListener('close', onFailure);
    };
    const onOpen = () => { cleanup(); resolve(socket); };
    const onFailure = () => { cleanup(); reject(new Error('WebSocket connection failed')); };

    socket.addEventListener('open', onOpen);
    socket.addEventListener('error', onFailure);
    socket.addEventListener('close', onFailure);
  });
}

function scheduleReconnect() {
  if (!wanted) return;
  const delay = RETRY_DELAYS[Math.min(retries, RETRY_DELAYS.length - 1)];
  retries += 1;
  clearTimeout(retryTimer);
  retryTimer = setTimeout(() => {
    connectWebSocket(activeToken).catch(() => scheduleReconnect());
  }, delay);
}

/**
 * Connect and keep the connection alive. A socket that dropped mid-game used to stay
 * dead until the page was reloaded, silently killing chat and canvas updates.
 */
export async function connectWebSocket(token) {
  activeToken = token;
  wanted = true;
  clearTimeout(retryTimer);

  const socket = await openSocket(token);
  retries = 0;
  websocket.set(socket);

  socket.addEventListener('close', () => {
    // Ignore a socket that has already been replaced by a newer one.
    if (get(websocket) !== socket) return;
    websocket.set(null);
    scheduleReconnect();
  });

  return socket;
}

export function disconnectWebSocket() {
  wanted = false;
  clearTimeout(retryTimer);
  const socket = get(websocket);
  websocket.set(null);
  if (socket) socket.close();
}
