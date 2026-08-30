/** Writing hex colours into an ImageData buffer, kept apart from the component. */

export function writePixel(imageData, size, x, y, hex) {
  const idx = (y * size + x) * 4;
  imageData.data[idx] = parseInt(hex.substring(0, 2), 16);
  imageData.data[idx + 1] = parseInt(hex.substring(2, 4), 16);
  imageData.data[idx + 2] = parseInt(hex.substring(4, 6), 16);
  imageData.data[idx + 3] = 255;
}

/** Paint a whole flat array of hex strings; missing entries read as white. */
export function writeAll(imageData, size, pixels) {
  for (let i = 0; i < pixels.length; i++) {
    writePixel(imageData, size, i % size, Math.floor(i / size), pixels[i] || 'ffffff');
  }
}
