import { writable, derived } from 'svelte/store';

export const token = writable(typeof localStorage !== 'undefined' ? localStorage.getItem('token') || '' : '');
export const user = writable(null);

/// 0 signed out, 1 user, 2 contributor, 3 administrator.
/// Usable outside a component (route guards) as well as through the store below.
export function permissionOf(u) {
  if (!u) return 0;
  if (u.role === 'administrator') return 3;
  if (u.role === 'contributor') return 2;
  return 1;
}

export const userPermission = derived(user, permissionOf);

// Persist token
token.subscribe((val) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('token', val || '');
  }
});
