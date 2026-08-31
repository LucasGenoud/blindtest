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
    pub temperature: f32,
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
}

impl LlmError {
    fn new(public: &str, detail: impl Into<String>) -> Self {
        LlmError { public: public.to_string(), detail: detail.into() }
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
            temperature: env_var("LLM_TEMPERATURE")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.4),
            client,
        })
    }

    /// Send one completion and return the assistant's raw text.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String, LlmError> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
        });
        if self.json_mode {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
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

        parsed
            .choices
            .into_iter()
            .next()
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
    ) -> Result<impl futures_util::Stream<Item = Result<String, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "stream": true,
        });
        if self.json_mode {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
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
    }

    let state = State { inner, buf: Vec::new(), pending: Default::default(), finished: false };

    futures_util::stream::unfold(state, |mut st| async move {
        loop {
            if let Some(text) = st.pending.pop_front() {
                return Some((Ok(text), st));
            }
            if st.finished {
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
}

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
pub fn extract_json(raw: &str) -> Option<serde_json::Value> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw.trim()) {
        return Some(value);
    }

    let bytes = raw.as_bytes();
    let start = raw.find('{')?;
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
                    return serde_json::from_str(&raw[start..=i]).ok();
                }
            }
            _ => {}
        }
    }
    None
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
