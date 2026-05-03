import { writable, derived } from 'svelte/store';

export const token = writable(typeof localStorage !== 'undefined' ? localStorage.getItem('token') || '' : '');
export const user = writable(null);

export const userPermission = derived(user, ($user) => {
  if (!$user) return 0;
  if ($user.role === 'administrator') return 3;
  if ($user.role === 'contributor') return 2;
  return 1;
});

// Persist token
token.subscribe((val) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('token', val || '');
  }
});
