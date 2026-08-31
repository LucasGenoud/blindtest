//! A minimal client for any OpenAI-compatible `/chat/completions` endpoint.
//!
//! The provider is chosen entirely by environment: OpenAI, OpenRouter, Groq,
//! together.ai, a local Ollama or llama.cpp — anything that speaks the same
//! shape. Nothing in the app depends on which one is configured, and when none
//! is configured the assistant simply reports itself disabled.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Where the model lives and how to talk to it. Absent when the deployment did
/// not configure one, which is how the assistant gets switched off.
#[derive(Clone)]
pub struct LlmConfig {
    /// Base URL up to but excluding `/chat/completions`, e.g. `https://api.openai.com/v1`.
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    /// Ask for `response_format: json_object`. Some compatible servers reject the
    /// field outright, so it can be turned off.
    pub json_mode: bool,
    /// Upper bound on how many library entries are put in front of the model.
    pub max_catalog: usize,
    /// The context window this *endpoint* serves, which is not always what the
    /// model supports: a vLLM or llama.cpp server started with a smaller
    /// `--max-model-len` will refuse a prompt the model itself could hold.
    pub context_tokens: usize,
    /// The most output ever asked for. The answer proper is small — a sentence and
    /// a list of numbers, ~350 tokens — but a model that thinks out loud spends
    /// this before writing a word of it, so the default is roomy. What is actually
    /// requested per call is whatever the window has left, up to this.
    pub reserve_tokens: usize,
    pub temperature: f32,
    /// Merged into every request body. The way to reach provider-specific
    /// switches without teaching this client about any of them — turning a
    /// reasoning model's thinking off, for one.
    pub extra_body: Option<serde_json::Value>,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        ChatMessage { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        ChatMessage { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        ChatMessage { role: "assistant".into(), content: content.into() }
    }
}

/// Split so the route can log the provider's own words while telling the caller
/// something short and safe.
#[derive(Debug)]
pub struct LlmError {
    /// Shown to the user.
    pub public: String,
    /// Logged only — may quote the provider's response.
    pub detail: String,
    /// The answer was cut off by the output ceiling. Worth distinguishing: the
    /// caller can make room and try again instead of giving up.
    pub truncated: bool,
}

impl LlmError {
    fn new(public: &str, detail: impl Into<String>) -> Self {
        LlmError { public: public.to_string(), detail: detail.into(), truncated: false }
    }

    fn cut_off(detail: impl Into<String>) -> Self {
        LlmError { public: TRUNCATED.to_string(), detail: detail.into(), truncated: true }
    }
}

fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl LlmConfig {
    /// `None` disables the assistant. A base URL and a model name are the minimum;
    /// the key is optional because local servers do not ask for one.
    pub fn from_env() -> Option<Self> {
        let base_url = env_var("LLM_BASE_URL")?.trim_end_matches('/').to_string();
        let model = env_var("LLM_MODEL")?;

        let timeout = env_var("LLM_TIMEOUT_SECS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(120);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| log::error!("Could not build the LLM HTTP client: {}", e))
            .ok()?;

        Some(LlmConfig {
            base_url,
            api_key: env_var("LLM_API_KEY"),
            model,
            json_mode: env_var("LLM_JSON_MODE").as_deref() != Some("false"),
            max_catalog: env_var("LLM_MAX_CATALOG")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(4000),
            // What a self-hosted vLLM or llama.cpp is most often started with.
            // Overshooting the window is a hard 400 rather than a worse answer, so
            // the default assumes the small case; raise it to match the endpoint.
            context_tokens: env_var("LLM_CONTEXT_TOKENS")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(32_768),
            reserve_tokens: env_var("LLM_RESERVE_TOKENS")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(4_000),
            temperature: env_var("LLM_TEMPERATURE")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.4),
            extra_body: env_var("LLM_EXTRA_BODY").and_then(|raw| {
                match serde_json::from_str::<serde_json::Value>(&raw) {
                    Ok(value) if value.is_object() => Some(value),
                    _ => {
                        log::error!("LLM_EXTRA_BODY is not a JSON object; ignoring it");
                        None
                    }
                }
            }),
            client,
        })
    }

    /// Send one completion and return the assistant's raw text.
    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        max_output: usize,
    ) -> Result<String, LlmError> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": max_output,
        });
        if self.json_mode {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        if let Some(serde_json::Value::Object(extra)) = self.extra_body.clone() {
            if let Some(target) = body.as_object_mut() {
                target.extend(extra);
            }
        }

        let url = format!("{}/chat/completions", self.base_url);
        let mut request = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await.map_err(|e| {
            LlmError::new("Could not reach the language model.", e.to_string())
        })?;

        let status = response.status();
        let text = response.text().await.map_err(|e| {
            LlmError::new("The language model sent an unreadable response.", e.to_string())
        })?;

        if !status.is_success() {
            return Err(LlmError::new(
                &format!("The language model refused the request ({}).", status.as_u16()),
                text,
            ));
        }

        let parsed: ChatResponse = serde_json::from_str(&text).map_err(|e| {
            LlmError::new(
                "The language model sent a response in an unexpected shape.",
                format!("{}: {}", e, truncate(&text, 500)),
            )
        })?;

        let choice = parsed.choices.into_iter().next();
        if choice.as_ref().and_then(|c| c.finish_reason.as_deref()) == Some("length") {
            return Err(LlmError::cut_off(truncate(&text, 500)));
        }

        choice
            .and_then(|c| c.message.content)
            .filter(|c| !c.trim().is_empty())
            .ok_or_else(|| LlmError::new("The language model returned nothing.", truncate(&text, 500)))
    }

    /// The same completion, delivered as it is written.
    ///
    /// Yields content deltas, not whole messages: the caller accumulates them and
    /// is responsible for making sense of a half-finished document.
    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        max_output: usize,
    ) -> Result<impl futures_util::Stream<Item = Result<String, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": max_output,
            "stream": true,
        });
        if self.json_mode {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }

        if let Some(serde_json::Value::Object(extra)) = self.extra_body.clone() {
            if let Some(target) = body.as_object_mut() {
                target.extend(extra);
            }
        }

        let url = format!("{}/chat/completions", self.base_url);
        let mut request = self.client.post(&url).json(&body);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await.map_err(|e| {
            LlmError::new("Could not reach the language model.", e.to_string())
        })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::new(
                &format!("The language model refused the request ({}).", status.as_u16()),
                text,
            ));
        }

        Ok(sse_deltas(response.bytes_stream()))
    }
}

/// Decode an OpenAI-style `text/event-stream` into the text each chunk carries.
///
/// Bytes are buffered rather than decoded as they arrive: a chunk boundary can
/// fall in the middle of a multi-byte character, and a line can arrive in pieces.
fn sse_deltas<S, B>(inner: S) -> impl futures_util::Stream<Item = Result<String, LlmError>>
where
    S: futures_util::Stream<Item = reqwest::Result<B>> + Unpin,
    B: AsRef<[u8]>,
{
    use futures_util::StreamExt;

    struct State<S> {
        inner: S,
        buf: Vec<u8>,
        pending: std::collections::VecDeque<String>,
        finished: bool,
        /// The model hit its output ceiling; whatever arrived is a fragment.
        truncated: bool,
        reported: bool,
    }

    let state = State {
        inner,
        buf: Vec::new(),
        pending: Default::default(),
        finished: false,
        truncated: false,
        reported: false,
    };

    futures_util::stream::unfold(state, |mut st| async move {
        loop {
            if let Some(text) = st.pending.pop_front() {
                return Some((Ok(text), st));
            }
            if st.finished {
                if st.truncated && !st.reported {
                    st.reported = true;
                    return Some((Err(LlmError::cut_off("finish_reason=length")), st));
                }
                return None;
            }

            match st.inner.next().await {
                Some(Ok(chunk)) => {
                    st.buf.extend_from_slice(chunk.as_ref());
                    while let Some(nl) = st.buf.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = st.buf.drain(..=nl).collect();
                        let line = String::from_utf8_lossy(&line);
                        let Some(data) = line.trim().strip_prefix("data:") else { continue };
                        let data = data.trim();
                        if data == "[DONE]" {
                            st.finished = true;
                            break;
                        }
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                            if value["choices"][0]["finish_reason"] == "length" {
                                st.truncated = true;
                            }
                            match value["choices"][0]["delta"]["content"].as_str() {
                                Some(text) if !text.is_empty() => {
                                    st.pending.push_back(text.to_string())
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    st.finished = true;
                    return Some((
                        Err(LlmError::new(
                            "The connection to the language model dropped.",
                            e.to_string(),
                        )),
                        st,
                    ));
                }
                None => st.finished = true,
            }
        }
    })
}

/// Read the `reply` string out of a JSON document that is still being written.
///
/// The model answers in one JSON object, so there is no prose to forward until the
/// `reply` value starts arriving — and it arrives escaped, a fragment at a time.
/// Returns what has been written so far, whether or not the string has closed;
/// `None` until the value actually begins.
pub fn reply_prefix(raw: &str) -> Option<String> {
    // Nothing is shown while the model is still thinking: its working out is not
    // the answer, and it may well mention "reply" on the way there.
    let raw = visible_answer(raw)?;
    let key = raw.find("\"reply\"")?;
    let after = &raw[key + "\"reply\"".len()..];
    let colon = after.find(':')?;
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }

    let mut out = String::new();
    let mut chars = rest[1..].chars();

    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => {
                // A backslash at the very end is half an escape: stop before it
                // rather than forwarding a character the model has not chosen yet.
                let Some(escape) = chars.next() else { return Some(out) };
                match escape {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'u' => {
                        let hex: String = chars.by_ref().take(4).collect();
                        if hex.chars().count() < 4 {
                            return Some(out);
                        }
                        if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                        {
                            out.push(ch);
                        }
                    }
                    other => out.push(other),
                }
            }
            c => out.push(c),
        }
    }

    Some(out)
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// Said when the answer was cut off mid-sentence. Worth its own message: the
/// remedy is a setting, not a retry.
const TRUNCATED: &str = "The language model ran out of room before finishing its answer. \
Raise LLM_RESERVE_TOKENS, or turn off the model's thinking mode.";

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect::<String>() + "…"
}

/// Pull the first JSON object out of a completion.
///
/// Models wrap their answer in prose or a ```json fence often enough that
/// deserialising the raw string fails for reasons that have nothing to do with
/// the content, so scan for the outermost braces first.
/// The part of a completion that is meant to be the answer.
///
/// A reasoning model writes its working out first, wrapped in `<think>`. That is
/// not the answer, and it is full of draft JSON that a naive scan would seize on.
/// `None` means the model is still thinking and has not started answering.
fn visible_answer(raw: &str) -> Option<&str> {
    const TAGS: [(&str, &str); 3] =
        [("<think>", "</think>"), ("<thinking>", "</thinking>"), ("<reasoning>", "</reasoning>")];

    for (open, close) in TAGS {
        if let Some(at) = raw.rfind(close) {
            return Some(&raw[at + close.len()..]);
        }
        // Opened and never closed: whatever follows is still reasoning.
        if raw.contains(open) {
            return None;
        }
    }
    Some(raw)
}

/// Read one balanced JSON object starting at `start`, which must index a `{`.
fn balanced_object(text: &str, start: usize) -> Option<serde_json::Value> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for i in start..bytes.len() {
        let c = bytes[i] as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_str(&text[start..=i]).ok();
                }
            }
            _ => {}
        }
    }
    None
}

/// Pull the answer object out of a completion.
///
/// Models wrap their answer in prose or a ```json fence often enough that
/// deserialising the raw string fails for reasons that have nothing to do with
/// the content. Candidates are preferred by shape rather than by position: the
/// first `{` in a reply is frequently a fragment the model was thinking aloud
/// about, so an object carrying `reply` or `tracks` wins over an earlier one.
pub fn extract_json(raw: &str) -> Option<serde_json::Value> {
    let text = visible_answer(raw)?.trim();

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        return Some(value);
    }

    let mut fallback = None;
    for (start, _) in text.match_indices('{') {
        let Some(value) = balanced_object(text, start) else { continue };
        if value.get("reply").is_some()
            || value.get("tracks").is_some()
            || value.get("songs").is_some()
        {
            return Some(value);
        }
        fallback.get_or_insert(value);
    }

    fallback
}

#[cfg(test)]
mod tests {
    use super::extract_json;

    #[test]
    fn reads_a_bare_object() {
        let v = extract_json(r#"{"reply":"hi","tracks":[1,2]}"#).unwrap();
        assert_eq!(v["reply"], "hi");
    }

    #[test]
    fn reads_an_object_inside_a_fence() {
        let v = extract_json("Sure!\n```json\n{\"tracks\": [3]}\n```\n").unwrap();
        assert_eq!(v["tracks"][0], 3);
    }

    #[test]
    fn braces_inside_strings_do_not_end_the_object() {
        let v = extract_json(r#"prose {"reply":"a } b","tracks":[]} trailing"#).unwrap();
        assert_eq!(v["reply"], "a } b");
    }

    #[test]
    fn returns_none_without_an_object() {
        assert!(extract_json("no json here").is_none());
    }

    /// What a reasoning model actually sends: it drafts an answer inside its
    /// thinking, so the first `{` in the text is not the one that counts.
    #[test]
    fn skips_a_reasoning_block_and_the_draft_inside_it() {
        let raw = "<think>Let me try {\"tracks\": [1,2]} — no, too easy. \
                   Maybe {\"reply\":\"draft\"} instead.</think>\
                   {\"reply\":\"Twelve dramas.\",\"tracks\":[7,8]}";
        let value = extract_json(raw).unwrap();
        assert_eq!(value["reply"], "Twelve dramas.");
        assert_eq!(value["tracks"][0], 7);
    }

    #[test]
    fn a_fenced_answer_after_reasoning_is_still_found() {
        let raw = "<think>weighing it up</think>\n```json\n{\"reply\":\"Done\",\"tracks\":[3]}\n```";
        assert_eq!(extract_json(raw).unwrap()["reply"], "Done");
    }

    /// Cut off mid-thought: there is no answer yet, and the reasoning must not be
    /// mistaken for one.
    #[test]
    fn unfinished_reasoning_yields_nothing() {
        assert!(extract_json("<think>still working {\"reply\":\"maybe\"").is_none());
        assert!(reply_prefix("<think>hmm, the \"reply\" should say").is_none());
    }

    #[test]
    fn prose_is_not_streamed_until_the_thinking_closes() {
        let raw = "<think>planning</think>{\"reply\":\"Twelve dra";
        assert_eq!(reply_prefix(raw).unwrap(), "Twelve dra");
    }

    /// An object with neither key is better than nothing, but only as a last resort.
    #[test]
    fn an_answer_shaped_object_beats_an_earlier_one() {
        let raw = r#"{"note":"ignore me"} then {"reply":"real","tracks":[1]}"#;
        assert_eq!(extract_json(raw).unwrap()["reply"], "real");
    }

    use super::reply_prefix;

    #[test]
    fn forwards_the_reply_as_far_as_it_has_been_written() {
        assert_eq!(reply_prefix(r#"{"reply": "Twelve dra"#).unwrap(), "Twelve dra");
    }

    #[test]
    fn forwards_a_finished_reply_without_the_rest_of_the_document() {
        let raw = r#"{"reply":"Done.","tracks":[1,2]}"#;
        assert_eq!(reply_prefix(raw).unwrap(), "Done.");
    }

    #[test]
    fn unescapes_as_it_goes() {
        let raw = r#"{"reply":"He said \"go\"\nthen left"#;
        assert_eq!(reply_prefix(raw).unwrap(), "He said \"go\"\nthen left");
    }

    /// The escape is only half-arrived; forwarding it would print a stray
    /// backslash that the model never meant to write.
    #[test]
    fn stops_before_a_half_written_escape() {
        assert_eq!(reply_prefix(r#"{"reply":"line\"#).unwrap(), "line");
        assert_eq!(reply_prefix(r#"{"reply":"snow\u26"#).unwrap(), "snow");
    }

    #[test]
    fn waits_until_the_value_actually_starts() {
        assert!(reply_prefix(r#"{"tra"#).is_none());
        assert!(reply_prefix(r#"{"reply""#).is_none());
        assert!(reply_prefix(r#"{"reply":"#).is_none());
    }

    #[test]
    fn finds_the_reply_after_the_track_list() {
        assert_eq!(reply_prefix(r#"{"tracks":[1,2],"reply":"After"#).unwrap(), "After");
    }
}
