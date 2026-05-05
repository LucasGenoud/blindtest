<script>
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { getApi } from '$lib/api.js';
  import { getVideoId, categoryListValueLabel } from '$lib/misc.js';
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { blindtestStatus, timeToGuess, timeWithAnswer, numberOfAudios, currentAudioData, currentAudioNumber, showAnswer, useSuperflus, prioritizeLessUsedAudios, dataCategories, disabledUsers, showCategory, audioRating, volume } from '$lib/stores/gameStore.js';
  import confetti from 'canvas-confetti';

  let { blindtestId = null, randomOrder = false } = $props();

  let totalAudios = $state(0);
  let countDown = $state(0);
  let preciseCountDown = $state(0);
  let videoId = $state(null);
  let currentCategory = $state(null);
  let passedAudiosIds = $state([]);
  let predefinedCategoryOrder = $state([]);
  let videoBuffering = $state(true);
  let audioFlagged = $state(false);
  let reportMessage = $state('');
  let customBlindtest = $state(null);
  let timer = $state(null);
  let player = $state(null);
  let playerReady = $state(false);
  let ytApiLoaded = $state(false);

  // Initialize game
  onMount(() => {
    if ($blindtestStatus !== 'started') { goto('/'); return; }
    initGame();
  });

  onDestroy(() => {
    stopTimer();
    if (player) { player.pause(); player.src = ""; player.load(); }
  });

  async function initGame() {
    if (blindtestId) {
      const res = await fetch(`${getApi()}/getcustomblindtest/${blindtestId}`);
      customBlindtest = await res.json();
      totalAudios = customBlindtest.blindtestList.length;
      if (randomOrder) shuffleArray(customBlindtest.blindtestList);
    } else {
      buildCategoryOrder();
    }
    playAudio();
  }

  function buildCategoryOrder() {
    let total = 0;
    const cats = [];
    Object.keys($dataCategories).forEach(c => { total += $dataCategories[c]; if ($dataCategories[c] > 0) cats.push(c); });
    const order = [];
    Object.keys($dataCategories).forEach(c => {
      const pct = ($dataCategories[c] / total) * 100;
      const n = Math.floor($numberOfAudios * pct / 100);
      for (let i = 0; i < n; i++) order.push(c);
    });
    while (order.length < $numberOfAudios && cats.length) {
      order.push(cats[Math.floor(Math.random() * cats.length)]);
    }
    shuffleArray(order);
    predefinedCategoryOrder = order;
    totalAudios = order.length;
  }

  async function playAudio() {
    let params = {};
    if (customBlindtest) {
      params = { audioId: customBlindtest.blindtestList[$currentAudioNumber], userId: $user?._id || '' };
    } else {
      params = {
        category: predefinedCategoryOrder[$currentAudioNumber] || '',
        passedAudiosIds: JSON.stringify(passedAudiosIds),
        useSuperflus: String($useSuperflus),
        prioritizeLessUsedAudios: String($prioritizeLessUsedAudios),
        userId: $user?._id || '',
        disabledUsers: JSON.stringify($disabledUsers),
      };
    }
    const qs = new URLSearchParams(params).toString();
    try {
      const res = await fetch(`${getApi()}/getnextaudio?${qs}`);
      if (!res.ok) { $currentAudioNumber++; setTimeout(playAudio, 2000); return; }
      const data = await res.json();

      videoBuffering = true;
      $currentAudioNumber++;
      audioFlagged = false;
      $showAnswer = false;

      videoId = data.videoData._id;
      $currentAudioData = data.videoData;
      $audioRating = data.rating || null;
      passedAudiosIds = [...passedAudiosIds, data.videoData._id];
      currentCategory = categoryListValueLabel.find(c => c.value === data.videoData.category);
      countDown = $timeToGuess;
      preciseCountDown = $timeToGuess;

      loadVideo();
    } catch (e) {
      $currentAudioNumber++;
      setTimeout(playAudio, 2000);
    }
  }

  function loadVideo() {
    if (!videoId || !player) return;
    player.src = `${getApi()}/media/${videoId}`;
    player.load();
    player.volume = $volume;
    
    player.oncanplay = () => {
      videoBuffering = false;
      player.play().catch(e => console.error("Autoplay prevented", e));
      startCountdown();
    };
    
    player.onended = () => {
      player.currentTime = 0;
      player.play();
    };
    
    player.onerror = (e) => {
      if ($token) { reportMessage = 'Automatic report for broken audio'; flagAudio(true); }
    };
  }

  function startCountdown() {
    stopTimer();
    const startTime = Date.now();
    const target = $showAnswer ? $timeWithAnswer : $timeToGuess;
    timer = setInterval(() => {
      const elapsed = (Date.now() - startTime) / 1000;
      if (!$showAnswer) {
        preciseCountDown = Math.max(0, $timeToGuess - elapsed);
        countDown = Math.ceil(preciseCountDown);
      }
      if (elapsed >= target) timerEnded();
    }, 100);
  }

  function stopTimer() {
    if (timer) { clearInterval(timer); timer = null; }
  }

  function timerEnded() {
    stopTimer();
    if (!$showAnswer) {
      $showAnswer = true;
      countDown = $timeToGuess;
      preciseCountDown = $timeToGuess;
      startCountdown();
    } else if ($currentAudioNumber < totalAudios) {
      playAudio();
    } else {
      stopBlindtest();
      const confInterval = setInterval(() => {
        confetti({ particleCount: 100, origin: { x: Math.random(), y: Math.random() - 0.2 }, startVelocity: 30, spread: 360 });
      }, 100);
      setTimeout(() => clearInterval(confInterval), 5000);
    }
  }

  function pauseBlindtest() { stopTimer(); $blindtestStatus = 'paused'; if (player) player.pause(); }
  function resumeBlindtest() { $blindtestStatus = 'started'; if (player) player.play(); startCountdown(); }
  function skipAudio() { stopTimer(); playAudio(); }
  function revealAnswer() { stopTimer(); $showAnswer = true; startCountdown(); }

  function stopBlindtest() {
    stopTimer();
    $currentAudioData = null; $currentAudioNumber = 0; $showAnswer = false;
    $blindtestStatus = 'stopped'; $disabledUsers = [];
    videoId = null;
    if (player) { player.pause(); player.src = ""; player.load(); }
    goto('/');
  }

  async function flagAudio(auto = false) {
    audioFlagged = true;
    await fetch(`${getApi()}/flagaudio`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ audio: $currentAudioData, reportMessage }),
    });
    reportMessage = '';
    stopTimer();
    playAudio();
  }

  async function rateAudio(rating) {
    await fetch(`${getApi()}/rateAudio/${$currentAudioData._id}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: $token },
      body: JSON.stringify({ rating }),
    });
  }

  function openYoutube() {
    if (player && $currentAudioData) {
      const t = Math.round(player.currentTime || 0);
      window.open(`${$currentAudioData.videoUrl}&t=${t}`, '_blank');
      pauseBlindtest();
    }
  }

  function shuffleArray(arr) {
    for (let r = 0; r < 3; r++) {
      for (let i = arr.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [arr[i], arr[j]] = [arr[j], arr[i]];
      }
    }
  }

  // Watch volume
  $effect(() => { if (player) { player.volume = $volume; } });
</script>

<div class="player-container">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-left">
      {#if $blindtestStatus === 'started'}
        <button class="btn-circle" title="Pause" onclick={pauseBlindtest}>⏸</button>
      {:else if $blindtestStatus === 'paused'}
        <button class="btn-circle" title="Resume" onclick={resumeBlindtest}>▶</button>
      {/if}
      <button class="btn-circle" title="Reveal" disabled={$showAnswer} onclick={revealAnswer}>✨</button>
      {#if $currentAudioNumber < totalAudios}
        <button class="btn-circle" title="Skip" onclick={skipAudio}>⏭</button>
      {/if}
      <button class="btn-circle" title="YouTube" onclick={openYoutube}>▶️</button>

      {#if $user && $showAnswer}
        <div class="rating-stars">
          {#each [1,2,3,4,5] as star}
            <span class="star" class:filled={star <= ($audioRating?.rating || 0)} onclick={() => rateAudio(star)}>★</span>
          {/each}
        </div>
      {/if}
    </div>

    <div class="toolbar-right">
      {#if totalAudios > 0}
        <span class="progress-label">{$currentAudioNumber}/{totalAudios}</span>
      {/if}
      {#if $userPermission > 0 && !audioFlagged}
        <input bind:value={reportMessage} placeholder="Report..." style="width:140px" />
        <button class="btn-circle warn" title="Flag" onclick={() => flagAudio()}>🚩</button>
      {/if}
      <button class="btn-circle danger" title="Stop" onclick={stopBlindtest}>✕</button>
    </div>
  </div>

  <!-- Progress bar -->
  {#if totalAudios > 0}
    <div class="progress-track">
      <div class="progress-fill" style="width:{Math.round($currentAudioNumber / totalAudios * 100)}%"></div>
    </div>
  {/if}

  <!-- Category -->
  {#if $currentAudioData && $showCategory && currentCategory}
    <div class="category-label">{currentCategory.label}</div>
  {/if}

  <!-- Main area -->
  <div class="blindtest-main">
    {#if !$showAnswer}
      {#if videoBuffering}
        <div class="loading-state">
          <div class="loading-text">Loading</div>
          <div class="loading-spin" style="font-size:48px">💿</div>
        </div>
      {:else}
        <div class="countdown-circle">
          <svg width="260" height="260" viewBox="0 0 260 260">
            <circle cx="130" cy="130" r="120" fill="none" stroke="var(--border)" stroke-width="3"/>
            <circle cx="130" cy="130" r="120" fill="none" stroke="var(--accent)" stroke-width="3"
              stroke-dasharray="{2 * Math.PI * 120}"
              stroke-dashoffset="{2 * Math.PI * 120 * (1 - preciseCountDown / $timeToGuess)}"
              stroke-linecap="round"
              style="transition:stroke-dashoffset 0.1s linear" />
          </svg>
          <span class="value">{countDown}</span>
        </div>
      {/if}
    {:else}
      <div class="answer-box">{$currentAudioData?.answer}</div>
    {/if}

    <!-- Native Video player (always rendered, visibility toggled) -->
    <div class="yt-wrapper" class:visible={$showAnswer && $currentAudioData}>
      <video bind:this={player} class="native-player" playsinline></video>
    </div>

    {#if $showAnswer && $currentAudioData}
      <div class="audio-meta">
        <span class="meta-label">by</span>
        <span class="meta-val">{$currentAudioData.submittedByUsername || 'Unknown'}</span>
        <span class="meta-sep">·</span>
        <span class="meta-label">views</span>
        <span class="meta-val">{($currentAudioData.count || 0) + 1}</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .player-container {
    width: 100%; height: 100%;
    padding: 0; text-align: center;
    display: flex; flex-direction: column;
    background: var(--bg);
  }
  .toolbar {
    display: flex; align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    gap: 8px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .btn-circle.warn {
    border-color: rgba(251, 146, 60, 0.4);
    color: var(--orange);
  }
  .btn-circle.warn:hover {
    background: rgba(251, 146, 60, 0.08);
  }
  .btn-circle.danger {
    border-color: rgba(248, 113, 113, 0.4);
    color: var(--red);
  }
  .btn-circle.danger:hover {
    background: rgba(248, 113, 113, 0.08);
  }
  .progress-label {
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-dim);
    letter-spacing: 0.04em;
  }
  .progress-track {
    height: 2px;
    background: var(--border);
    width: 100%;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.3s ease;
  }
  .blindtest-main {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    position: relative;
  }
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }
  .loading-text {
    font-family: var(--mono);
    font-size: 0.8rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }
  .countdown-circle {
    position: relative;
    display: inline-flex; align-items: center; justify-content: center;
  }
  .countdown-circle .value {
    position: absolute;
    font-family: var(--mono);
    font-size: 96px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.04em;
  }
  .answer-box {
    font-family: var(--mono);
    font-size: 36px;
    font-weight: 600;
    color: var(--accent);
    padding: 16px 32px;
    border: 1px solid var(--accent-border);
    background: var(--accent-dim);
    border-radius: 8px;
    margin-bottom: 16px;
    letter-spacing: -0.02em;
  }
  .yt-wrapper {
    width: 80%; max-width: 800px; aspect-ratio: 16/9;
    pointer-events: none; display: none;
  }
  .yt-wrapper.visible { display: block; }
  .native-player {
    width: 100%; height: 100%; object-fit: cover;
  }
  .category-label {
    font-family: var(--mono);
    font-size: 1rem;
    color: var(--text-secondary);
    padding: 10px 0;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .audio-meta {
    position: fixed; bottom: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    padding: 6px 14px;
    border-radius: 6px;
    font-family: var(--mono);
    font-size: 0.7rem;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .meta-label { color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.06em; }
  .meta-val { color: var(--text-primary); font-weight: 500; }
  .meta-sep { color: var(--text-dim); }
  .rating-stars { display: flex; gap: 2px; margin-left: 8px; }
  .star {
    font-size: 18px; cursor: pointer;
    color: var(--text-dim);
    transition: color 0.15s;
  }
  .star.filled { color: var(--accent); }
  .star:hover { color: var(--accent); }
  @media screen and (max-width: 700px) {
    .countdown-circle .value { font-size: 64px; }
    .answer-box { font-size: 20px; padding: 10px 16px; }
    .category-label { font-size: 0.8rem; }
    .toolbar { flex-wrap: wrap; }
  }
</style>
