export const categoryListValueLabel = [
  { value: 'movies', label: 'Movie' },
  { value: 'tvshows', label: 'TV show' },
  { value: 'animes', label: 'Anime' },
  { value: 'animatedmovies', label: 'Animated movie' },
  { value: 'animatedseries', label: 'Animated serie' },
  { value: 'games', label: 'Game' },
  { value: 'musics', label: 'Music' },
  { value: 'internetculture', label: 'Internet culture' },
  { value: 'quotes', label: 'Quotes' },
];

export const categoryListKeyLabel = [
  { key: 'movies', label: 'Movies' },
  { key: 'tvshows', label: 'TV shows' },
  { key: 'animes', label: 'Animes' },
  { key: 'animatedmovies', label: 'Animated movies' },
  { key: 'animatedseries', label: 'Animated series' },
  { key: 'games', label: 'Games' },
  { key: 'musics', label: 'Musics' },
  { key: 'internetculture', label: 'Internet culture' },
  { key: 'quotes', label: 'Quotes' },
];

export function getVideoId(url) {
  if (!url) return null;
  let id = url.split('v=')[1];
  if (!id) id = url.split('.be/')[1];
  if (!id) return null;
  const ampPos = id.indexOf('&');
  if (ampPos !== -1) id = id.substring(0, ampPos);
  return id;
}

export function checkEmail(email) {
  return email.toLowerCase().match(/^(([^<>()[\]\\.,;:\s@"]+(\.[^<>()[\]\\.,;:\s@"]+)*)|(".+"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/);
}

export function debounce(fn, delay) {
  let timeout = null;
  return function (...args) {
    clearTimeout(timeout);
    timeout = setTimeout(() => fn.apply(this, args), delay);
  };
}

export function hexToRgb(hex) {
  const bigint = parseInt(hex, 16);
  return {
    r: (bigint >> 16) & 255,
    g: (bigint >> 8) & 255,
    b: bigint & 255,
  };
}

export function stringToColor(str) {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  let colour = '#';
  for (let i = 0; i < 3; i++) {
    const value = (hash >> (i * 8)) & 0xff;
    colour += ('00' + value.toString(16)).substr(-2);
  }
  return colour;
}

export const colors = [
  { index: 0, name: 'Burgandy', r: 109, g: 0, b: 26, hex: '6d001a' },
  { index: 1, name: 'Dark red', r: 190, g: 0, b: 57, hex: 'be0039' },
  { index: 2, name: 'Red', r: 255, g: 69, b: 0, hex: 'ff4500' },
  { index: 3, name: 'Pink', r: 255, g: 56, b: 129, hex: 'ff3881' },
  { index: 4, name: 'Orange', r: 255, g: 168, b: 0, hex: 'ffa800' },
  { index: 5, name: 'Yellow', r: 255, g: 214, b: 53, hex: 'ffd635' },
  { index: 6, name: 'Dark teal', r: 0, g: 117, b: 111, hex: '00756f' },
  { index: 7, name: 'Teal', r: 0, g: 158, b: 170, hex: '009eaa' },
  { index: 8, name: 'Light teal', r: 0, g: 204, b: 192, hex: '00ccc0' },
  { index: 9, name: 'Lavender', r: 148, g: 179, b: 255, hex: '94b3ff' },
  { index: 10, name: 'Dark green', r: 0, g: 163, b: 104, hex: '00a368' },
  { index: 11, name: 'Green', r: 0, g: 204, b: 120, hex: '00cc78' },
  { index: 12, name: 'Light green', r: 126, g: 237, b: 86, hex: '7eed56' },
  { index: 13, name: 'Dark blue', r: 36, g: 80, b: 164, hex: '2450a4' },
  { index: 14, name: 'Blue', r: 54, g: 144, b: 234, hex: '3690ea' },
  { index: 15, name: 'Light blue', r: 81, g: 233, b: 244, hex: '51e9f4' },
  { index: 16, name: 'Dark purple', r: 129, g: 30, b: 159, hex: '811e9f' },
  { index: 17, name: 'Indigo', r: 73, g: 58, b: 193, hex: '493ac1' },
  { index: 18, name: 'PeriWinkle', r: 106, g: 92, b: 255, hex: '6a5cff' },
  { index: 19, name: 'Pale purple', r: 228, g: 171, b: 255, hex: 'e4abff' },
  { index: 20, name: 'Magenta', r: 222, g: 16, b: 127, hex: 'de107f' },
  { index: 21, name: 'Purple', r: 180, g: 74, b: 192, hex: 'b44ac0' },
  { index: 22, name: 'Light pink', r: 255, g: 153, b: 170, hex: 'ff99aa' },
  { index: 23, name: 'Brown', r: 156, g: 105, b: 38, hex: '9c6926' },
  { index: 24, name: 'Dark brown', r: 156, g: 72, b: 47, hex: '9c482f' },
  { index: 25, name: 'Black', r: 0, g: 0, b: 0, hex: '000000' },
  { index: 26, name: 'Dark gray', r: 81, g: 82, b: 82, hex: '515252' },
  { index: 27, name: 'Gray', r: 137, g: 141, b: 144, hex: '898d90' },
  { index: 28, name: 'Light grey', r: 212, g: 215, b: 217, hex: 'd4d7d9' },
  { index: 29, name: 'Pale yellow', r: 255, g: 248, b: 184, hex: 'fff8b8' },
  { index: 30, name: 'Beige', r: 255, g: 180, b: 112, hex: 'ffb470' },
  { index: 31, name: 'White', r: 255, g: 255, b: 255, hex: 'ffffff' },
];
