<script>
  import { onMount, tick } from 'svelte';
  import { api, apiTry, ApiError } from '$lib/api.js';
  import { Send, Trash2 } from 'lucide-svelte';

  /**
   * The conversation that builds a blindtest. The track list it produces lands in
   * the panel next door, which is the only place the tracks are ever shown — the
   * assistant talks about what it did, it does not recite titles.
   */
  let { blindtestId, model = '', onapplied } = $props();

  let messages = $state([]);
  let draft = $state('');
  let sending = $state(false);
  let error = $state('');
  let thread = $state(null);
  /** The reply as it arrives, before the turn is finished and recorded. */
  let streamed = $state('');
  /** What the model is doing while there is still nothing to show. */
  let phase = $state('reading');

  const examples = [
    '20 tracks from 90s cartoons, easy enough for a family game',
    'A hard 15-track anime opening quiz, no shonen',
    'Mix games and movie themes, 25 tracks, difficulty ramping up',
  ];

  onMount(async () => {
    messages = await apiTry(api.get(`/getblindtestagentmessages/${blindtestId}`), []);
    scrollDown();
  });

  async function scrollDown() {
    await tick();
    if (thread) thread.scrollTop = thread.scrollHeight;
  }

  async function send() {
    const prompt = draft.trim();
    if (!prompt || sending) return;

    error = '';
    sending = true;
    draft = '';
    streamed = '';
    phase = 'reading';
    // Shown straight away: the round trip is a model call and can take a while.
    messages = [...messages, { _id: `pending-${Date.now()}`, role: 'user', content: prompt, tracks: [] }];
    scrollDown();

    let result = null;
    let failure = '';
    let heardBack = false;

    try {
      await api.stream(`/streamblindtest/${blindtestId}`, { prompt }, (event, data) => {
        heardBack = true;
        if (event === 'delta') {
          streamed += data.text;
          scrollDown();
        } else if (event === 'thinking') {
          // A reasoning model can spend minutes here before any prose appears.
          phase = 'thinking';
        } else if (event === 'reset') {
          // The answer that prose belonged to is being thrown away and rewritten.
          streamed = '';
          phase = 'thinking';
        } else if (event === 'error') {
          failure = data.message;
        } else if (event === 'done') {
          result = data;
        }
      });
    } catch (e) {
      if (e instanceof ApiError) {
        // The server answered and said why.
        failure = e.message;
      } else if (!heardBack) {
        // Nothing came back at all — a proxy that will not pass an event stream,
        // or a browser that cannot read one. The plain endpoint still works, and
        // since no frame arrived nothing has been recorded twice.
        try {
          result = await api.post(`/generateblindtest/${blindtestId}`, { prompt });
        } catch (e2) {
          failure = e2 instanceof ApiError ? e2.message : 'The assistant could not be reached.';
        }
      } else {
        failure = 'The connection to the assistant dropped.';
      }
    }

    if (result) {
      messages = [
        ...messages,
        {
          _id: result.messageId,
          role: 'assistant',
          content: result.reply,
          tracks: result.blindtestList,
        },
      ];
      if (result.changed) onapplied?.(result.blindtestList);
    } else {
      error = failure || 'The assistant could not be reached.';
      // Put the text back so a failed attempt is not retyped.
      draft = prompt;
      messages = messages.slice(0, -1);
    }

    streamed = '';
    sending = false;
    scrollDown();
  }

  function onKeydown(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  async function clearThread() {
    if (!confirm('Clear this conversation? The track list stays as it is.')) return;
    await apiTry(api.del(`/clearblindtestagentmessages/${blindtestId}`));
    messages = [];
    error = '';
  }

  function useExample(text) {
    draft = text;
  }
</script>

<div class="agent">
  <div class="panel-header">
    <span class="panel-label">Assistant</span>
    {#if model}<span class="model">{model}</span>{/if}
    {#if messages.length > 0}
      <button class="btn-xs clear" onclick={clearThread} title="Clear conversation">
        <Trash2 size={12} stroke-width={1.8} /> Clear
      </button>
    {/if}
  </div>

  <div class="thread" bind:this={thread}>
    {#if messages.length === 0}
      <div class="empty-state">
        <h3>Describe the blindtest</h3>
        <p>
          Say what it should be about, how many tracks, which categories and how hard. The assistant
          picks from the clip library and fills the list beside this one. Keep talking to change it.
        </p>
        <div class="examples">
          {#each examples as example}
            <button class="example" onclick={() => useExample(example)}>{example}</button>
          {/each}
        </div>
      </div>
    {/if}

    {#each messages as message (message._id)}
      <div class="turn" class:from-user={message.role === 'user'}>
        <div class="who">{message.role === 'user' ? 'You' : 'Assistant'}</div>
        <div class="said">{message.content}</div>
        {#if message.role === 'assistant' && message.tracks.length > 0}
          <div class="outcome tabular">{message.tracks.length} tracks in the list</div>
        {/if}
      </div>
    {/each}

    {#if sending}
      <div class="turn">
        <div class="who">Assistant</div>
        {#if streamed}
          <div class="said">{streamed}</div>
        {:else}
          <div class="working"><div class="loading-line"></div></div>
          <div class="said dim">
            {phase === 'thinking' ? 'Thinking it through…' : 'Reading the library…'}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="composer">
    <textarea
      bind:value={draft}
      onkeydown={onKeydown}
      disabled={sending}
      rows="2"
      placeholder="Ask for a blindtest, or change the one you have…"
    ></textarea>
    <button class="btn-primary" onclick={send} disabled={sending || !draft.trim()}>
      <Send size={14} stroke-width={1.8} /> {sending ? 'Working' : 'Send'}
    </button>
  </div>
</div>

<style>
  .agent {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .panel-header {
    padding: 12px 16px;
    display: flex;
    gap: 12px;
    align-items: center;
    border-bottom: 2px solid var(--divider);
  }
  .panel-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-primary);
  }
  .model {
    font-size: 11px;
    color: var(--text-dim);
  }
  .clear {
    margin-left: auto;
  }

  .thread {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }

  .empty-state {
    padding: 32px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
  h3 {
    font-size: 20px;
    font-weight: 800;
    color: var(--text-primary);
    line-height: 1.1;
  }
  .empty-state p {
    font-size: 15px;
    line-height: 1.55;
    color: var(--text-secondary);
    max-width: 52ch;
  }
  .examples {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    margin-top: 12px;
  }
  /* Starters, not commands: a rule under the text, no box. */
  .example {
    background: transparent;
    border: 0;
    padding: 0;
    text-align: left;
    font-size: 13px;
    color: var(--accent-text);
    cursor: pointer;
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 4px;
    transition: text-decoration-thickness var(--duration-fast) ease-out;
  }
  .example:hover {
    text-decoration-thickness: 2px;
  }

  .turn {
    padding: 16px;
    border-bottom: 1px solid var(--divider);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .from-user {
    background: var(--row-hover);
  }
  .who {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-dim);
  }
  .said {
    font-size: 15px;
    line-height: 1.55;
    color: var(--text-primary);
    white-space: pre-wrap;
  }
  .said.dim {
    color: var(--text-secondary);
  }
  .outcome {
    font-size: 13px;
    color: var(--text-secondary);
  }
  .working {
    position: relative;
    height: 2px;
    margin: 4px 0;
  }

  .error {
    padding: 12px 16px;
    border-top: 2px solid var(--divider);
    font-size: 13px;
    color: var(--signal-wrong);
  }

  .composer {
    border-top: 2px solid var(--divider);
    padding: 12px 16px;
    display: flex;
    gap: 8px;
    align-items: flex-end;
  }
  textarea {
    flex: 1;
    resize: none;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 0;
    color: var(--text-primary);
    font-family: var(--sans);
    font-size: 15px;
    line-height: 1.55;
    padding: 8px 12px;
    outline: none;
    transition: border-color var(--duration-fast) ease-out;
  }
  textarea:focus {
    border-color: var(--accent);
  }
  textarea:disabled {
    opacity: 0.4;
  }
</style>
