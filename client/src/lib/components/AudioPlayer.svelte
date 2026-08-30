<script>
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { getApi } from '$lib/api.js';
  import { getVideoId, categoryListValueLabel } from '$lib/misc.js';
  import { token, user, userPermission } from '$lib/stores/userStore.js';
  import { blindtestStatus, timeToGuess, timeWithAnswer, numberOfAudios, currentAudioData, currentAudioNumber, showAnswer, useSuperflus, prioritizeLessUsedAudios, dataCategories, disabledUsers, showCategory, volume } from '$lib/stores/gameStore.js';
  import confetti from 'canvas-confetti';
  import { Pause, Play, ExternalLink, Flag, Volume2, VolumeX } from 'lucide-svelte';

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
  let currentAnswer = $state('');
  let loadFailures = $state(0);
  let loadError = $state('');
  let timer;
  let player;

  // Stop chasing audios once the server has failed this many times in a row, instead
  // of retrying forever past the end of the game.
  const MAX_CONSECUTIVE_FAILURES = 5;

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
      // Private blindtests are owner-only now, so the token has to travel with this.
      const res = await fetch(`${getApi()}/getcustomblindtest/${blindtestId}`, {
        headers: $token ? { Authorization: $token } : {},
      });
      if (!res.ok) {
        loadError = 'This blindtest could not be loaded.';
        videoBuffering = false;
        return;
      }
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
      // The token identifies who is playing; the server no longer trusts a userId
      // sent in the query string.
      const res = await fetch(`${getApi()}/getnextaudio?${qs}`, {
        headers: $token ? { Authorization: $token } : {},
      });
      if (!res.ok) { failedToLoad(); return; }
      const data = await res.json();

      videoBuffering = true;
      loadFailures = 0;
      loadError = '';
      $currentAudioNumber++;
      audioFlagged = false;
      $showAnswer = false;
      currentAnswer = '';

      videoId = data.videoData._id;
      $currentAudioData = data.videoData;
      passedAudiosIds = [...passedAudiosIds, data.videoData._id];
      currentCategory = categoryListValueLabel.find(c => c.value === data.videoData.category);
      countDown = $timeToGuess;
      preciseCountDown = $timeToGuess;

    } catch (e) {
      failedToLoad();
    }
  }

  /// Skip to the next audio after a failure, but give up rather than looping forever
  /// when the server itself is unavailable.
  function failedToLoad() {
    stopTimer();
    loadFailures++;
    $currentAudioNumber++;

    if (loadFailures >= MAX_CONSECUTIVE_FAILURES) {
      videoBuffering = false;
      loadError = 'Could not reach the server. The blindtest has been stopped.';
      return;
    }
    if ($currentAudioNumber >= totalAudios) {
      stopBlindtest();
      return;
    }
    videoBuffering = true;
    setTimeout(playAudio, 2000);
  }

  /// The answer is not part of the audio payload — it would be readable in the
  /// network tab before anyone had guessed — so it is fetched at reveal time.
  async function fetchAnswer(id) {
    try {
      const res = await fetch(`${getApi()}/getaudioanswer?audioId=${encodeURIComponent(id)}`);
      if (res.ok && videoId === id) {
        const data = await res.json();
        currentAnswer = data.answer ?? '';
      }
    } catch {
      // Leave the answer blank rather than breaking the reveal.
    }
  }

  $effect(() => {
    if ($showAnswer && videoId && !currentAnswer) fetchAnswer(videoId);
  });

  // Loads the clip when the round changes. Keep this effect's dependencies to
  // videoId and player only.
  $effect(() => {
    if (videoId && player) loadVideo();
  });

  function loadVideo() {
    player.src = `${getApi()}/media/${videoId}`;
    player.load();
    // Do not read $volume here. This runs inside the effect below, and Svelte
    // tracks reads made by anything an effect calls, so touching the volume store
    // would make it a dependency: changing the volume would reload the clip and
    // restart the countdown. The element keeps its volume across a src change,
    // and the dedicated effect below owns it.

    player.oncanplay = () => {
      videoBuffering = false;
      player.play().catch(e => console.error('Autoplay prevented', e));
      startCountdown();
    };

    player.onended = () => {
      player.currentTime = 0;
      player.play();
    };

    player.onerror = () => {
      if ($token) {
        // Recorded for contributors to review, but marked automatic: an automatic
        // flag no longer removes the audio from everyone else's rotation, so a bad
        // stretch of server trouble cannot quietly empty the pool.
        flagAudio(true);
      }
      failedToLoad();
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
    const message = auto ? 'Automatic report for broken audio' : reportMessage;
    try {
      await fetch(`${getApi()}/flagaudio`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: $token },
        body: JSON.stringify({ audio: $currentAudioData, reportMessage: message, auto }),
      });
    } catch {
      // A failed report should not strand the player on the current audio.
    }
    reportMessage = '';
    // The automatic path is driven by failedToLoad(); only a manual flag skips here.
    if (!auto) {
      stopTimer();
      playAudio();
    }
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
  $effect(() => { if (player) { player.volume = $volume / 100; } });

  let volumeBeforeMute = $state(50);

  function toggleMute() {
    if ($volume > 0) {
      volumeBeforeMute = $volume;
      $volume = 0;
    } else {
      $volume = volumeBeforeMute || 50;
    }
  }
</script>

<!-- In game the nav bar is replaced by a single line: round counter left, exit
     right. Media is full-bleed, controls sit in a fixed bottom bar, and the view
     never scrolls. -->
<div class="player-container">
  <div class="round-line">
    <div class="round-counter">
      <span class="round-current tabular">{$currentAudioNumber}</span>
      <span class="round-total tabular">/ {totalAudios}</span>
      {#if $currentAudioData && $showCategory && currentCategory}
        <span class="round-category">{currentCategory.label}</span>
      {/if}
    </div>
    <button class="btn-secondary sm" onclick={stopBlindtest}>Exit</button>
  </div>

  <div class="progress-bar">
    <div class="fill" style="width:{totalAudios ? Math.round($currentAudioNumber / totalAudios * 100) : 0}%"></div>
  </div>

  <!-- Main area -->
  <div class="blindtest-main">
    {#if videoBuffering && !loadError && !$showAnswer}
      <div class="loading-line"></div>
    {/if}

    {#if !$showAnswer}
      {#if loadError}
        <div class="state-block">
          <h2>Could not load the next clip</h2>
          <p>{loadError}</p>
          <button class="btn-secondary" onclick={stopBlindtest}>Leave game</button>
        </div>
      {:else if !videoBuffering}
        <!-- The countdown ring is the second of the two circular exceptions. -->
        <div class="countdown-circle" role="timer" aria-live="off">
          <svg width="260" height="260" viewBox="0 0 260 260">
            <circle cx="130" cy="130" r="120" fill="none" stroke="var(--divider)" stroke-width="2"/>
            <circle cx="130" cy="130" r="120" fill="none" stroke="var(--accent)" stroke-width="2"
              stroke-dasharray="{2 * Math.PI * 120}"
              stroke-dashoffset="{2 * Math.PI * 120 * (1 - Math.max(0, Math.min(1, $timeToGuess > 0 ? preciseCountDown / $timeToGuess : 0)))}"
              style="transition:stroke-dashoffset 0.1s linear" />
          </svg>
          <span class="value">{countDown}</span>
        </div>
      {/if}
    {:else}
      <div class="answer-box answer-enter" aria-live="polite">{currentAnswer}</div>
    {/if}

    <!-- Native video player (always rendered, visibility toggled) -->
    <div class="yt-wrapper" class:visible={$showAnswer && $currentAudioData}>
      <video bind:this={player} class="native-player" playsinline></video>
    </div>

    {#if $showAnswer && $currentAudioData}
      <div class="audio-meta">
        <span class="meta-label">by</span>
        <span class="meta-val">{$currentAudioData.submittedByUsername || 'Unknown'}</span>
        <span class="meta-label">plays</span>
        <span class="meta-val tabular">{($currentAudioData.count || 0) + 1}</span>
      </div>
    {/if}
  </div>

  <!-- Fixed 96px control bar. Only play/pause is icon-only; every other action
       carries a word. -->
  <div class="control-bar">
    <div class="control-group">
      {#if $blindtestStatus === 'started'}
        <button class="btn-circle" title="Pause" aria-label="Pause" onclick={pauseBlindtest}><Pause size={16} stroke-width={2} /></button>
      {:else if $blindtestStatus === 'paused'}
        <button class="btn-circle" title="Resume" aria-label="Resume" onclick={resumeBlindtest}><Play size={16} stroke-width={2} /></button>
      {/if}

      <div class="volume-control">
        <button class="btn-circle" title={$volume === 0 ? 'Unmute' : 'Mute'} aria-label={$volume === 0 ? 'Unmute' : 'Mute'} onclick={toggleMute}>
          {#if $volume === 0}
            <VolumeX size={16} stroke-width={2} />
          {:else}
            <Volume2 size={16} stroke-width={2} />
          {/if}
        </button>
        <input type="range" min="0" max="100" bind:value={$volume} aria-label="Volume" />
      </div>
      <button class="btn-secondary" disabled={$showAnswer} onclick={revealAnswer}>Reveal answer</button>
      {#if $currentAudioNumber < totalAudios}
        <button class="btn-secondary" onclick={skipAudio}>Skip clip</button>
      {/if}
      <button class="btn-secondary" onclick={openYoutube}>
        <ExternalLink size={16} stroke-width={2} /> Open source
      </button>
    </div>

    <div class="control-group">
      {#if $userPermission > 0 && !audioFlagged}
        <input bind:value={reportMessage} placeholder="What is wrong with this clip?" aria-label="Report message" />
        <button class="btn-danger" onclick={() => flagAudio()}>
          <Flag size={16} stroke-width={2} /> Report
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .player-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg);
  }

  .round-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 32px;
    flex-shrink: 0;
  }

  .round-counter {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .round-current {
    font-size: 20px;
    font-weight: 800;
    color: var(--accent-text);
  }

  .round-total {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .round-category {
    margin-left: 8px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
  }

  .blindtest-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
    gap: 24px;
    min-height: 0;
    padding: 24px 32px;
  }

  /* Flush left, one heading, one line, one action. */
  .state-block {
    align-self: flex-start;
    max-width: 480px;
  }

  .state-block h2 {
    font-size: 20px;
    font-weight: 800;
    color: var(--text-primary);
    margin-bottom: 8px;
  }

  .state-block p {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 16px;
  }

  /* Centred: one of the two deliberate exceptions. */
  .countdown-circle {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .countdown-circle .value {
    position: absolute;
    font-size: 64px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }

  /* The other exception, and the only moment that gets a movement. */
  .answer-box {
    font-size: 64px;
    font-weight: 800;
    line-height: 1.1;
    letter-spacing: -0.03em;
    color: var(--accent-text);
    text-align: center;
    max-width: 1200px;
  }

  .answer-enter {
    animation: answerReveal var(--duration-reveal) var(--easing-primary) forwards;
  }

  /* The media is the picture: full-bleed, nothing framing it. */
  .yt-wrapper {
    width: 100%;
    max-width: 1200px;
    aspect-ratio: 16/9;
    min-height: 0;
    pointer-events: none;
    display: none;
    overflow: hidden;
  }

  .yt-wrapper.visible { display: block; }

  .native-player {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .audio-meta {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 13px;
  }

  .meta-label { color: var(--text-secondary); }
  .meta-val { color: var(--text-primary); font-weight: 600; }

  .control-bar {
    height: 96px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 0 32px;
    border-top: 2px solid var(--divider);
  }

  .control-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .volume-control {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 8px;
  }

  .volume-control input[type="range"] {
    width: 96px;
  }

  @media screen and (max-width: 760px) {
    .round-line, .blindtest-main { padding-left: 16px; padding-right: 16px; }
    .countdown-circle .value { font-size: 32px; }
    .answer-box { font-size: 32px; }
    .control-bar {
      height: auto;
      flex-wrap: wrap;
      gap: 8px;
      padding: 16px;
    }
  }
</style>
