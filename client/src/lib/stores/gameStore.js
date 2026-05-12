import { writable } from "svelte/store";

export const blindtestStatus = writable("stopped"); // stopped, started, paused
export const volume = writable(
  typeof localStorage !== "undefined"
    ? parseInt(localStorage.getItem("volume") || "50")
    : 50,
);
export const timeToGuess = writable(15);
export const timeWithAnswer = writable(10);
export const numberOfAudios = writable(30);
export const showCategory = writable(true);
export const useSuperflus = writable(false);
export const prioritizeLessUsedAudios = writable(false);
export const currentAudioData = writable(null);
export const currentAudioNumber = writable(0);
export const showAnswer = writable(false);
export const disabledUsers = writable([]);
export const dataCategories = writable({
  movies: 100,
  tvshows: 100,
  animes: 100,
  animatedmovies: 100,
  animatedseries: 100,
  games: 100,
  musics: 100,
  internetculture: 100,
  quotes: 0,
});

// Persist volume
volume.subscribe((val) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("volume", String(val));
  }
});
