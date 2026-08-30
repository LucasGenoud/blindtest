import { api, apiTry } from '$lib/api.js';

/** Every audio-library request in one place, so the screen only handles state. */
export const audioApi = {
  list: () => apiTry(api.get('/getallaudios'), []),
  create: (audio) => apiTry(api.post('/newaudio', audio)),
  update: (audio) => apiTry(api.post('/updateaudio', audio)),
  remove: (id) => apiTry(api.del(`/deleteaudio?id=${encodeURIComponent(id)}`)),
  reprocess: (id) => apiTry(api.post(`/reprocessaudio?audioId=${encodeURIComponent(id)}`)),
  resetFlag: (audioId) => apiTry(api.post('/resetflag', { audioId })),
  backup: () => apiTry(api.get('/backupaudio', { parse: 'blob' })),
};
