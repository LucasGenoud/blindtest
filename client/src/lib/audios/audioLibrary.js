/**
 * Pure helpers for the audio library screen: filtering, sorting, paging and the
 * status vocabulary. No fetching and no component state, so each can be reasoned
 * about (and tested) on its own.
 */

export const PAGE_SIZE_OPTIONS = [25, 50, 100, 200];

export function matchesSearch(audio, search) {
  if (!search) return true;
  const s = search.toLowerCase();
  return (
    audio.answer.toLowerCase().includes(s) ||
    audio.category.toLowerCase().includes(s) ||
    (audio.submittedByUsername || '').toLowerCase().includes(s)
  );
}

export function filterAudios(audios, { search, status, category, superflus }) {
  return audios.filter((a) => {
    if (!matchesSearch(a, search)) return false;
    if (status !== 'all' && a.processingStatus !== status) return false;
    if (category !== 'all' && a.category !== category) return false;
    if (superflus === 'yes' && !a.superflus) return false;
    if (superflus === 'no' && a.superflus) return false;
    return true;
  });
}

export function sortAudios(audios, key, direction) {
  const order = direction === 'asc' ? 1 : -1;
  return [...audios].sort((a, b) => {
    let av = a[key] ?? '';
    let bv = b[key] ?? '';
    if (typeof av === 'string') av = av.toLowerCase();
    if (typeof bv === 'string') bv = bv.toLowerCase();
    if (av < bv) return -order;
    if (av > bv) return order;
    return 0;
  });
}

export function pageOf(list, page, size) {
  const start = (page - 1) * size;
  return list.slice(start, start + size);
}

/** Page numbers to show around the current one, clamped to what exists. */
export function pageWindow(currentPage, totalPages, span = 7) {
  const count = Math.min(span, totalPages);
  let start = Math.max(1, currentPage - Math.floor(span / 2));
  start = Math.max(1, Math.min(start, totalPages - count + 1));
  return Array.from({ length: count }, (_, i) => start + i);
}

export function countByStatus(audios) {
  return {
    total: audios.length,
    ready: audios.filter((a) => a.processingStatus === 'ready').length,
    processing: audios.filter((a) => a.processingStatus === 'processing').length,
    error: audios.filter((a) => a.processingStatus === 'error').length,
    flagged: audios.filter((a) => a.flagged?.length > 0).length,
  };
}

export function statusClass(status) {
  if (status === 'ready') return 'status-ready';
  if (status === 'processing') return 'status-processing';
  if (status === 'error') return 'status-error';
  return 'status-unknown';
}

export function statusLabel(status) {
  if (status === 'ready') return 'Ready';
  if (status === 'processing') return 'Processing';
  if (status === 'error') return 'Error';
  return 'Unknown';
}
