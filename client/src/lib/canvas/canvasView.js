/**
 * The canvas view transform: world (pixel) coordinates to screen and back.
 *
 * These are pure so the pan/zoom rules can be reasoned about without a DOM: a
 * view is `{ zoom, panX, panY }` and every function returns a new one rather
 * than mutating component state.
 */

export const MIN_ZOOM = 0.5;
export const MAX_ZOOM = 40;
export const ZOOM_FACTOR = 1.15;

export function clampZoom(zoom) {
  return Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom));
}

export function screenToWorld(view, rect, screenX, screenY) {
  return {
    x: (screenX - rect.left - view.panX) / view.zoom,
    y: (screenY - rect.top - view.panY) / view.zoom,
  };
}

export function worldToViewport(view, worldX, worldY) {
  return {
    x: worldX * view.zoom + view.panX,
    y: worldY * view.zoom + view.panY,
  };
}

export function pixelAt(view, rect, screenX, screenY) {
  const { x, y } = screenToWorld(view, rect, screenX, screenY);
  return { x: Math.floor(x), y: Math.floor(y) };
}

/** Zoom about a screen point, keeping the world point under it fixed. */
export function zoomAtPoint(view, rect, screenX, screenY, nextZoom) {
  const mx = screenX - rect.left;
  const my = screenY - rect.top;
  const worldX = (mx - view.panX) / view.zoom;
  const worldY = (my - view.panY) / view.zoom;
  const zoom = clampZoom(nextZoom);
  return { zoom, panX: mx - worldX * zoom, panY: my - worldY * zoom };
}

/** Centre the view on a world pixel, keeping the current zoom. */
export function centerOn(view, rect, pixel) {
  return {
    zoom: view.zoom,
    panX: rect.width / 2 - (pixel.x + 0.5) * view.zoom,
    panY: rect.height / 2 - (pixel.y + 0.5) * view.zoom,
  };
}

/**
 * Fit the whole board in the viewport.
 *
 * Takes the element's content box (clientWidth/clientHeight) rather than its
 * bounding rect: the two differ by borders and any scrollbar, and the fit is
 * against the space the canvas can actually occupy.
 */
export function fit(width, height, size) {
  const zoom = clampZoom(Math.min(width / size, height / size));
  return {
    zoom,
    panX: (width - size * zoom) / 2,
    panY: (height - size * zoom) / 2,
  };
}

export function inBounds(pixel, size) {
  return pixel.x >= 0 && pixel.x < size && pixel.y >= 0 && pixel.y < size;
}
