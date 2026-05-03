import { getWs } from './api.js';

export async function connectWebSocket(token) {
  return new Promise((resolve) => {
    const ws = new WebSocket(getWs(), token || undefined);
    const timer = setInterval(() => {
      if (ws.readyState === 1) {
        clearInterval(timer);
        resolve(ws);
      }
    }, 10);
  });
}
