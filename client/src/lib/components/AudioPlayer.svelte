<script>
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { api, apiTry, getApi } from '$lib/api.js';
  import { categoryListValueLabel } from '$lib/misc.js';
  import { allocateCategories } from '$lib/gameOrder.js';
  import { token, userPermission } from '$lib/stores/userStore.js';
  import { blindtestStatus, timeToGuess, timeWithAnswer, numberOfAudios, currentAudioData, currentAudioNumber, showAnswer, useSuperflus, prioritizeLessUsedAudios, dataCategories, disabledUsers, showCategory, volume, resetBlindtestState } from '$lib/stores/gameStore.js';
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
  let answerToken = $state('');
  let loadFailures = $state(0);
  let loadError = $state('');
  let timer;
  let timerDeadline = 0;
  let phaseRemaining = 0;
  let retryTimer;
  let requestGeneration = 0;
  let sourceGeneration = 0;
  let player;

  // Stop chasing audios once the server has failed this many times in a row, instead
  // of retrying forever past the end of the game.
  const MAX_CONSECUTIVE_FAILURES = 5;

  // Initialize game
  onMount(() => {
    if ($blindtestStatus !== 'started') { goto('/'); return; }
    $currentAudioData = null;
    $currentAudioNumber = 0;
    $showAnswer = false;
    initGame();
  });

  onDestroy(() => {
    requestGeneration++;
    sourceGeneration++;
    stopTimer();
    clearTimeout(retryTimer);
    if (player) { player.pause(); player.src = ""; player.load(); }
    resetBlindtestState();
  });

  async function initGame() {
    const initRequest = ++requestGeneration;
    if (blindtestId) {
      // Private blindtests are owner-only, so this needs the token the client adds.
      customBlindtest = await apiTry(api.get(`/getcustomblindtest/${blindtestId}`));
      if (initRequest !== requestGeneration) return;
      if (!customBlindtest) {
        loadError = 'This blindtest could not be loaded.';
        videoBuffering = false;
        return;
      }
      totalAudios = customBlindtest.blindtestList.length;
      if (randomOrder) shuffleArray(customBlindtest.blindtestList);
    } else {
      buildCategoryOrder();
    }
    if (!totalAudios) {
      videoBuffering = false;
      loadError = 'This blindtest has no playable clips.';
      return;
    }
    playAudio();
  }

  function buildCategoryOrder() {
    const order = allocateCategories($dataCategories, $numberOfAudios);
    shuffleArray(order);
    predefinedCategoryOrder = order;
    totalAudios = order.length;
  }

  async function playAudio() {
    if ($currentAudioNumber >= totalAudios) { stopBlindtest(); return; }
    clearTimeout(retryTimer);
    stopTimer();
    videoBuffering = true;
    const request = ++requestGeneration;
    const index = $currentAudioNumber;
    let params = {};
    if (customBlindtest) {
      params = { audioId: customBlindtest.blindtestList[index] };
    } else {
      params = {
        category: predefinedCategoryOrder[index] || '',
        passedAudiosIds: JSON.stringify(passedAudiosIds),
        useSuperflus: String($useSuperflus),
        prioritizeLessUsedAudios: String($prioritizeLessUsedAudios),
        disabledUsers: JSON.stringify($disabledUsers),
      };
    }
    params.revealAfter = String($timeToGuess);
    const qs = new URLSearchParams(params).toString();
    try {
      // The token identifies who is playing.
      const data = await api.get(`/getnextaudio?${qs}`);
      if (request !== requestGeneration) return;

      loadError = '';
      $currentAudioNumber = index + 1;
      audioFlagged = false;
      $showAnswer = false;
      currentAnswer = '';
      answerToken = data.answerToken;

      videoId = data.videoData._id;
      $currentAudioData = data.videoData;
      passedAudiosIds = [...passedAudiosIds, data.videoData._id];
      currentCategory = categoryListValueLabel.find(c => c.value === data.videoData.category);
      countDown = $timeToGuess;
      preciseCountDown = $timeToGuess;

    } catch (e) {
      if (request === requestGeneration) failedToLoad(true);
    }
  }

  /// Skip to the next audio after a failure, but give up rather than looping forever
  /// when the server itself is unavailable.
  function failedToLoad(advance) {
    requestGeneration++;
    sourceGeneration++;
    stopTimer();
    loadFailures++;
    if (advance) $currentAudioNumber++;

    if (loadFailures >= MAX_CONSECUTIVE_FAILURES) {
      videoBuffering = false;
      loadError = 'No playable clips could be loaded.';
      $blindtestStatus = 'paused';
      return;
    }
    if ($currentAudioNumber >= totalAudios) {
      stopBlindtest();
      return;
    }
    videoBuffering = true;
    retryTimer = setTimeout(playAudio, 2000);
  }

  /// The answer is not part of the audio payload — it would be readable in the
  /// network tab before anyone had guessed — so it is fetched at reveal time.
  async function fetchAnswer(id, revealToken) {
    // Leave the answer blank rather than breaking the reveal.
    const data = await apiTry(api.get(`/getaudioanswer?token=${encodeURIComponent(revealToken)}`));
    if (data && videoId === id) currentAnswer = data.answer ?? '';
  }

  $effect(() => {
    if ($showAnswer && videoId && answerToken && !currentAnswer) fetchAnswer(videoId, answerToken);
  });

  // Loads the clip when the round changes. Keep this effect's dependencies to
  // videoId and player only.
  $effect(() => {
    if (videoId && player) loadVideo();
  });

  function loadVideo() {
    const source = ++sourceGeneration;
    let started = false;
    // Do not read $volume here. This runs inside the effect below, and Svelte
    // tracks reads made by anything an effect calls, so touching the volume store
    // would make it a dependency: changing the volume would reload the clip and
    // restart the countdown. The element keeps its volume across a src change,
    // and the dedicated effect below owns it.

    player.oncanplay = async () => {
      if (source !== sourceGeneration || started) return;
      started = true;
      phaseRemaining = $timeToGuess;
      try {
        await player.play();
        if (source !== sourceGeneration) return;
        loadFailures = 0;
        videoBuffering = false;
        startCountdown(phaseRemaining);
      } catch {
        started = false;
        videoBuffering = false;
        loadError = 'Playback was blocked. Press Resume to continue.';
        $blindtestStatus = 'paused';
      }
    };

    player.onended = async () => {
      if (source !== sourceGeneration) return;
      player.currentTime = 0;
      try {
        await player.play();
      } catch {
        pauseBlindtest();
        loadError = 'Playback stopped. Press Resume to continue.';
      }
    };

    player.onerror = () => {
      if (source !== sourceGeneration) return;
      sourceGeneration++;
      if ($token) {
        // Recorded for contributors to review, but marked automatic: an automatic
        // flag no longer removes the audio from everyone else's rotation, so a bad
        // stretch of server trouble cannot quietly empty the pool.
        flagAudio(true);
      }
      failedToLoad(false);
    };

    player.src = `${getApi()}/media/${videoId}`;
    player.load();
  }

  function startCountdown(duration = $showAnswer ? $timeWithAnswer : $timeToGuess) {
    stopTimer();
    phaseRemaining = duration;
    timerDeadline = Date.now() + duration * 1000;
    timer = setInterval(() => {
      phaseRemaining = Math.max(0, (timerDeadline - Date.now()) / 1000);
      if (!$showAnswer) {
        preciseCountDown = phaseRemaining;
        countDown = Math.ceil(preciseCountDown);
      }
      if (phaseRemaining <= 0) timerEnded();
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
      startCountdown($timeWithAnswer);
    } else if ($currentAudioNumber < totalAudios) {
      playAudio();
    } else {
      stopBlindtest();
    }
  }

  function pauseBlindtest() {
    if (timer) phaseRemaining = Math.max(0, (timerDeadline - Date.now()) / 1000);
    stopTimer();
    $blindtestStatus = 'paused';
    if (player) player.pause();
  }
  async function resumeBlindtest() {
    if (!player) return;
    try {
      await player.play();
      loadError = '';
      $blindtestStatus = 'started';
      startCountdown(phaseRemaining || ($showAnswer ? $timeWithAnswer : $timeToGuess));
    } catch {
      loadError = 'Playback could not start.';
    }
  }
  function skipAudio() { stopTimer(); playAudio(); }

  function stopBlindtest() {
    requestGeneration++;
    sourceGeneration++;
    clearTimeout(retryTimer);
    stopTimer();
    resetBlindtestState();
    videoId = null;
    if (player) {
      player.oncanplay = null;
      player.onended = null;
      player.onerror = null;
      player.pause();
      player.src = "";
      player.load();
    }
    goto('/');
  }

  async function flagAudio(auto = false) {
    const message = auto ? 'Automatic report for broken audio' : reportMessage;
    const audio = $currentAudioData;
    if (auto) {
      try { await api.post('/flagaudio', { audio, reportMessage: message, auto: true }); } catch {}
      return;
    }
    audioFlagged = true;
    try {
      await api.post('/flagaudio', { audio, reportMessage: message, auto: false });
      reportMessage = '';
      // The automatic path is driven by failedToLoad(); only a manual flag skips here.
      if (!auto) {
        stopTimer();
        playAudio();
      }
    } catch (e) {
      audioFlagged = false;
      if (!auto) loadError = e.message || 'Could not report the audio.';
    }
  }

  function openYoutube() {
    if (player && $currentAudioData) {
      const t = Math.round(player.currentTime || 0);
      try {
        const url = new URL($currentAudioData.videoUrl);
        url.searchParams.set('t', String(t));
        window.open(url.toString(), '_blank', 'noopener');
        pauseBlindtest();
      } catch {
        loadError = 'The source URL is invalid.';
      }
    }
  }

  function shuffleArray(arr) {
    for (let i = arr.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [arr[i], arr[j]] = [arr[j], arr[i]];
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
      {#if !loadError && $blindtestStatus === 'started'}
        <button class="btn-circle" title="Pause" aria-label="Pause" onclick={pauseBlindtest}><Pause size={16} stroke-width={2} /></button>
      {:else if !loadError && $blindtestStatus === 'paused'}
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
      {#if $currentAudioNumber < totalAudios}
        <button class="btn-secondary" disabled={videoBuffering} onclick={skipAudio}>Skip clip</button>
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
