//! Platform AI capability: the OpenAI-compatible request/response contract
//! shared by the control plane endpoint (`/gate/ai/v1/*`, reached as
//! `https://<app>.repo.box/_repo_box/ai/v1/*`) and the ChatMock broker that
//! runs next to ChatMock on the Hetzner box.
//!
//! v1 contract, enforced on both hops:
//! * `POST /v1/chat/completions` and `GET /v1/models` only;
//! * JSON body of at most [`AI_MAX_BODY_BYTES`], at most [`AI_MAX_MESSAGES`]
//!   text messages (`system`/`developer`/`user`/`assistant`), total message
//!   characters within the policy limit;
//! * only allowed models; `max_tokens`/`max_completion_tokens` within the
//!   policy limit (defaulting to it);
//! * **non-streaming**: `stream: true` is refused; tools, functions and
//!   images are refused (no model tool execution in v1);
//! * the upstream body is rebuilt from the recognised fields only, and the
//!   response is rebuilt into the plain chat.completion shape.
//!
//! Nothing here logs or stores prompts, completions, secrets or bodies: the
//! log line per request carries app, user id, model, status, sizes and time.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde_json::{Map, Value, json};
use tokio::sync::{Mutex, Semaphore};

use crate::model::{AI_MAX_BODY_BYTES, AI_MAX_MESSAGES};

/// Largest upstream response body either hop accepts.
pub const AI_MAX_RESPONSE_BYTES: usize = 1024 * 1024;
/// Completion text is cut at this many characters per allowed output token
/// (ChatMock does not reliably honour `max_tokens`; this bounds the output
/// the app receives regardless). Generous: ~4 chars per token on average.
pub const AI_OUTPUT_CHARS_PER_TOKEN: i64 = 8;
pub const AI_MAX_STOP: usize = 4;

// ------------------------------------------------------------------ errors

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl AiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn bad(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }
    fn kind(&self) -> &'static str {
        match self.status.as_u16() {
            401 => "authentication_error",
            403 => "permission_error",
            404 => "not_found_error",
            413 => "request_too_large",
            429 => "rate_limit_error",
            400..=499 => "invalid_request_error",
            _ => "api_error",
        }
    }
    pub fn body(&self) -> Value {
        json!({ "error": { "message": self.message, "type": self.kind(), "code": self.code } })
    }
}

impl IntoResponse for AiError {
    fn into_response(self) -> Response {
        json_response(self.status, &self.body())
    }
}

pub fn json_response(status: StatusCode, v: &Value) -> Response {
    let mut resp = (status, v.to_string()).into_response();
    let h = resp.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    resp
}

// ------------------------------------------------------------- request

/// The limits one hop applies to a request.
#[derive(Debug, Clone)]
pub struct Limits {
    pub models: Vec<String>,
    /// Used when the request names no model. `None`: model is required.
    pub default_model: Option<String>,
    pub max_input_chars: i64,
    pub max_output_tokens: i64,
}

impl Limits {
    pub fn for_policy(p: &crate::model::AiPolicy) -> Self {
        Self {
            models: p.models.clone(),
            default_model: Some(p.default_model.clone()),
            max_input_chars: p.max_input_chars,
            max_output_tokens: p.max_output_tokens,
        }
    }
}

/// A validated request: the rebuilt upstream body plus what the log line needs.
#[derive(Debug, Clone)]
pub struct CleanRequest {
    pub body: Value,
    pub model: String,
    pub input_chars: i64,
    pub max_tokens: i64,
}

const REFUSED_FIELDS: &[(&str, &str)] = &[
    (
        "tools",
        "tool calling is not part of the v1 platform AI endpoint",
    ),
    (
        "tool_choice",
        "tool calling is not part of the v1 platform AI endpoint",
    ),
    (
        "parallel_tool_calls",
        "tool calling is not part of the v1 platform AI endpoint",
    ),
    (
        "functions",
        "function calling is not part of the v1 platform AI endpoint",
    ),
    (
        "function_call",
        "function calling is not part of the v1 platform AI endpoint",
    ),
    ("audio", "audio is not part of the v1 platform AI endpoint"),
    (
        "modalities",
        "only text is supported by the v1 platform AI endpoint",
    ),
];

fn num_in(v: &Value, name: &str, lo: f64, hi: f64) -> Result<f64, AiError> {
    let n = v
        .as_f64()
        .ok_or_else(|| AiError::bad("invalid_parameter", format!("{name} must be a number")))?;
    if !(lo..=hi).contains(&n) || !n.is_finite() {
        return Err(AiError::bad(
            "invalid_parameter",
            format!("{name} must be between {lo} and {hi}"),
        ));
    }
    Ok(n)
}

/// Validate a chat.completions request against `limits` and rebuild the
/// upstream body from recognised fields only.
pub fn clean_request(raw: &[u8], limits: &Limits) -> Result<CleanRequest, AiError> {
    if raw.len() > AI_MAX_BODY_BYTES {
        return Err(AiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "request_too_large",
            format!("request body exceeds {AI_MAX_BODY_BYTES} bytes"),
        ));
    }
    let v: Value = serde_json::from_slice(raw)
        .map_err(|_| AiError::bad("invalid_json", "request body must be a JSON object"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| AiError::bad("invalid_json", "request body must be a JSON object"))?;
    for (field, why) in REFUSED_FIELDS {
        if obj.get(*field).is_some_and(|f| !f.is_null()) {
            return Err(AiError::bad(
                "unsupported_parameter",
                format!("{field}: {why}"),
            ));
        }
    }
    if obj.get("stream").and_then(Value::as_bool) == Some(true) {
        return Err(AiError::bad(
            "streaming_unsupported",
            "stream: true is not supported by the v1 platform AI endpoint; send a non-streaming request",
        ));
    }
    if let Some(n) = obj.get("n").filter(|n| !n.is_null())
        && n.as_i64() != Some(1)
    {
        return Err(AiError::bad("unsupported_parameter", "n must be 1"));
    }

    let model = match obj.get("model").filter(|m| !m.is_null()) {
        Some(m) => m
            .as_str()
            .ok_or_else(|| AiError::bad("invalid_parameter", "model must be a string"))?
            .to_string(),
        None => limits
            .default_model
            .clone()
            .ok_or_else(|| AiError::bad("invalid_parameter", "model is required"))?,
    };
    if !limits.models.contains(&model) {
        return Err(AiError::bad(
            "model_not_allowed",
            format!(
                "model '{}' is not allowed here; allowed: {}",
                model.chars().take(64).collect::<String>(),
                limits.models.join(", ")
            ),
        ));
    }

    let msgs = obj
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AiError::bad("invalid_parameter", "messages must be a non-empty array"))?;
    if msgs.is_empty() {
        return Err(AiError::bad(
            "invalid_parameter",
            "messages must be a non-empty array",
        ));
    }
    if msgs.len() > AI_MAX_MESSAGES {
        return Err(AiError::bad(
            "too_many_messages",
            format!("at most {AI_MAX_MESSAGES} messages per request"),
        ));
    }
    let mut clean_msgs = Vec::with_capacity(msgs.len());
    let mut input_chars: i64 = 0;
    for (i, m) in msgs.iter().enumerate() {
        let m = m.as_object().ok_or_else(|| {
            AiError::bad(
                "invalid_parameter",
                format!("messages[{i}] must be an object"),
            )
        })?;
        let role = m.get("role").and_then(Value::as_str).unwrap_or("");
        if !matches!(role, "system" | "developer" | "user" | "assistant") {
            return Err(AiError::bad(
                "unsupported_parameter",
                format!("messages[{i}].role must be system, developer, user or assistant"),
            ));
        }
        if m.get("tool_calls").is_some_and(|t| !t.is_null())
            || m.get("function_call").is_some_and(|t| !t.is_null())
        {
            return Err(AiError::bad(
                "unsupported_parameter",
                format!("messages[{i}]: tool calls are not part of the v1 platform AI endpoint"),
            ));
        }
        let content = match m.get("content") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(parts)) => {
                let mut texts = Vec::with_capacity(parts.len());
                for p in parts {
                    match (
                        p.get("type").and_then(Value::as_str),
                        p.get("text").and_then(Value::as_str),
                    ) {
                        (Some("text"), Some(t)) => texts.push(t.to_string()),
                        _ => {
                            return Err(AiError::bad(
                                "unsupported_parameter",
                                format!("messages[{i}].content: only text parts are supported"),
                            ));
                        }
                    }
                }
                texts.join("\n")
            }
            _ => {
                return Err(AiError::bad(
                    "invalid_parameter",
                    format!("messages[{i}].content must be a string or text parts"),
                ));
            }
        };
        input_chars += content.chars().count() as i64;
        if input_chars > limits.max_input_chars {
            return Err(AiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "input_too_large",
                format!(
                    "messages exceed {} characters for this app",
                    limits.max_input_chars
                ),
            ));
        }
        // The developer role is folded into system for ChatMock compatibility.
        let role = if role == "developer" { "system" } else { role };
        clean_msgs.push(json!({ "role": role, "content": content }));
    }

    let requested = [obj.get("max_completion_tokens"), obj.get("max_tokens")]
        .into_iter()
        .flatten()
        .find(|v| !v.is_null());
    let max_tokens = match requested {
        Some(v) => {
            let n = v.as_i64().filter(|n| *n >= 1).ok_or_else(|| {
                AiError::bad("invalid_parameter", "max_tokens must be a positive integer")
            })?;
            if n > limits.max_output_tokens {
                return Err(AiError::bad(
                    "max_tokens_too_large",
                    format!(
                        "max_tokens may be at most {} for this app",
                        limits.max_output_tokens
                    ),
                ));
            }
            n
        }
        None => limits.max_output_tokens,
    };

    let mut body = Map::new();
    body.insert("model".into(), json!(model));
    body.insert("messages".into(), Value::Array(clean_msgs));
    body.insert("max_tokens".into(), json!(max_tokens));
    body.insert("stream".into(), json!(false));
    for (name, lo, hi) in [
        ("temperature", 0.0, 2.0),
        ("top_p", 0.0, 1.0),
        ("presence_penalty", -2.0, 2.0),
        ("frequency_penalty", -2.0, 2.0),
    ] {
        if let Some(v) = obj.get(name).filter(|v| !v.is_null()) {
            body.insert(name.into(), json!(num_in(v, name, lo, hi)?));
        }
    }
    if let Some(stop) = obj.get("stop").filter(|v| !v.is_null()) {
        let list: Vec<&str> = match stop {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        let n_given = stop.as_array().map(|a| a.len()).unwrap_or(1);
        if list.is_empty()
            || list.len() != n_given
            || list.len() > AI_MAX_STOP
            || list.iter().any(|s| s.is_empty() || s.chars().count() > 64)
        {
            return Err(AiError::bad(
                "invalid_parameter",
                format!("stop must be a string or up to {AI_MAX_STOP} strings of 1-64 characters"),
            ));
        }
        body.insert("stop".into(), json!(list));
    }
    if let Some(rf) = obj.get("response_format").filter(|v| !v.is_null()) {
        match rf.get("type").and_then(Value::as_str) {
            Some("text") => {}
            Some("json_object") => {
                body.insert("response_format".into(), json!({ "type": "json_object" }));
            }
            _ => {
                return Err(AiError::bad(
                    "unsupported_parameter",
                    "response_format.type must be text or json_object",
                ));
            }
        }
    }
    Ok(CleanRequest {
        body: Value::Object(body),
        model,
        input_chars,
        max_tokens,
    })
}

// ------------------------------------------------------------ response

/// Rebuild an upstream chat.completion into the plain shape apps receive:
/// one assistant text choice, finish reason, usage counts. Anything else the
/// upstream sent (tool calls, provider metadata) is dropped. Output text is
/// cut to `max_tokens * AI_OUTPUT_CHARS_PER_TOKEN` characters.
pub fn clean_response(raw: &[u8], model: &str, max_tokens: i64) -> Result<Value, AiError> {
    let bad = || {
        AiError::new(
            StatusCode::BAD_GATEWAY,
            "upstream_invalid",
            "the model provider returned an unusable response",
        )
    };
    let v: Value = serde_json::from_slice(raw).map_err(|_| bad())?;
    let choice = v
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .ok_or_else(bad)?;
    let content = choice
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let cap = (max_tokens.max(1) * AI_OUTPUT_CHARS_PER_TOKEN) as usize;
    let (content, cut) = if content.chars().count() > cap {
        (content.chars().take(cap).collect::<String>(), true)
    } else {
        (content.to_string(), false)
    };
    let finish = if cut {
        "length".to_string()
    } else {
        match choice.get("finish_reason").and_then(Value::as_str) {
            Some(r @ ("stop" | "length" | "content_filter")) => r.to_string(),
            _ => "stop".to_string(),
        }
    };
    let id = v
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| s.len() <= 128 && s.bytes().all(|b| b.is_ascii_graphic()))
        .map(|s| format!("chatcmpl-rb-{}", s.trim_start_matches("chatcmpl-")))
        .unwrap_or_else(|| "chatcmpl-rb".into());
    let created = v
        .get("created")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let usage = v.get("usage");
    let n = |k: &str| {
        usage
            .and_then(|u| u.get(k))
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0)
    };
    Ok(json!({
        "id": id,
        "object": "chat.completion",
        "created": created,
        "model": model,
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": finish,
        }],
        "usage": {
            "prompt_tokens": n("prompt_tokens"),
            "completion_tokens": n("completion_tokens"),
            "total_tokens": n("total_tokens"),
        },
    }))
}

// -------------------------------------------------------------- client

/// Loopback-only HTTP/1 client for the two internal hops (platform → broker
/// tunnel, broker → ChatMock). Plain HTTP is acceptable only because both
/// ends are loopback; `validate_loopback_url` enforces that.
#[derive(Clone)]
pub struct Upstream {
    client: Client<HttpConnector, Full<Bytes>>,
    base: String,
}

pub struct UpstreamReply {
    pub status: StatusCode,
    pub body: Bytes,
}

/// Accept only `http://127.0.0.1:PORT` / `http://[::1]:PORT` (no path).
pub fn validate_loopback_url(url: &str) -> Result<String, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("{url}: must be http://127.0.0.1:PORT (loopback only)"))?
        .trim_end_matches('/');
    let addr: SocketAddr = rest
        .parse()
        .map_err(|_| format!("{url}: must be http://127.0.0.1:PORT (loopback only)"))?;
    if !addr.ip().is_loopback() || addr.port() == 0 {
        return Err(format!("{url}: must be a loopback address"));
    }
    Ok(format!("http://{addr}"))
}

impl Upstream {
    pub fn new(base: &str) -> Result<Self, String> {
        let base = validate_loopback_url(base)?;
        let mut connector = HttpConnector::new();
        connector.set_connect_timeout(Some(Duration::from_secs(5)));
        connector.enforce_http(true);
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build(connector);
        Ok(Self { client, base })
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub async fn send(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, String)],
        body: Option<Vec<u8>>,
        timeout: Duration,
    ) -> Result<UpstreamReply, AiError> {
        let unavailable = |why: &str| {
            AiError::new(
                StatusCode::BAD_GATEWAY,
                "upstream_unavailable",
                format!("the model provider is unavailable ({why})"),
            )
        };
        let mut req = hyper::Request::builder()
            .method(method)
            .uri(format!("{}{}", self.base, path));
        for (k, v) in headers {
            req = req.header(*k, v);
        }
        let has_body = body.is_some();
        if has_body {
            req = req.header(header::CONTENT_TYPE, "application/json");
        }
        let req = req
            .body(Full::new(Bytes::from(body.unwrap_or_default())))
            .map_err(|_| unavailable("request"))?;
        let fut = async {
            let resp = self
                .client
                .request(req)
                .await
                .map_err(|_| unavailable("connect"))?;
            let status = resp.status();
            let body = Limited::new(resp.into_body(), AI_MAX_RESPONSE_BYTES)
                .collect()
                .await
                .map_err(|_| unavailable("response too large or interrupted"))?
                .to_bytes();
            Ok(UpstreamReply { status, body })
        };
        match tokio::time::timeout(timeout, fut).await {
            Ok(r) => r,
            Err(_) => Err(AiError::new(
                StatusCode::GATEWAY_TIMEOUT,
                "upstream_timeout",
                "the model provider did not answer in time",
            )),
        }
    }
}

/// Map a non-2xx broker/provider answer to what the caller sees. Client
/// errors carry the upstream's error code/message (both hops produce this
/// module's error shape); anything else is a generic gateway error so no
/// provider detail leaks.
pub fn upstream_error(reply: &UpstreamReply) -> AiError {
    if reply.status.is_client_error() && reply.status != StatusCode::UNAUTHORIZED {
        if let Ok(v) = serde_json::from_slice::<Value>(&reply.body)
            && let Some(e) = v.get("error")
        {
            let code = match e.get("code").and_then(Value::as_str) {
                Some("model_not_allowed") => "model_not_allowed",
                Some("model_unavailable") => "model_unavailable",
                Some("input_too_large") => "input_too_large",
                Some("max_tokens_too_large") => "max_tokens_too_large",
                Some("busy") => "busy",
                _ => "upstream_rejected",
            };
            let msg: String = e
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("rejected")
                .chars()
                .take(300)
                .collect();
            return AiError::new(reply.status, code, msg);
        }
        return AiError::new(
            reply.status,
            "upstream_rejected",
            "the broker rejected the request",
        );
    }
    AiError::new(
        StatusCode::BAD_GATEWAY,
        "upstream_unavailable",
        format!("the model provider answered {}", reply.status.as_u16()),
    )
}

// -------------------------------------------------------------- secret

/// The bridge secret shared by the control plane and the broker. Debug
/// never prints it.
#[derive(Clone)]
pub struct Secret(Arc<String>);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl Secret {
    pub fn new(value: String) -> Result<Self, String> {
        let v = value.trim().to_string();
        if v.len() < 32 || !v.bytes().all(|b| b.is_ascii_graphic()) {
            return Err("bridge secret must be at least 32 printable characters".into());
        }
        Ok(Self(Arc::new(v)))
    }

    /// Read from `path`, or from `$CREDENTIALS_DIRECTORY/<credential>`
    /// (systemd `LoadCredential=`) when no path is given.
    pub fn load(path: Option<&std::path::Path>, credential: &str) -> Result<Self, String> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => {
                let dir = std::env::var_os("CREDENTIALS_DIRECTORY").ok_or_else(|| {
                    format!("no secret file given and no systemd credential '{credential}'")
                })?;
                std::path::Path::new(&dir).join(credential)
            }
        };
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read bridge secret {}: {e}", path.display()))?;
        Self::new(raw)
    }

    pub fn bearer(&self) -> String {
        format!("Bearer {}", self.0)
    }

    /// Constant-time comparison against an `Authorization` header value.
    pub fn matches_header(&self, value: Option<&str>) -> bool {
        let want = self.bearer();
        let got = value.unwrap_or("");
        if got.len() != want.len() {
            return false;
        }
        got.bytes()
            .zip(want.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }
}

// -------------------------------------------------------------- bridge

/// The control plane's side of the bridge: the loopback end of the tunnel to
/// the broker, the shared secret, and a concurrency cap.
pub struct Bridge {
    pub upstream: Upstream,
    pub secret: Secret,
    pub permits: Semaphore,
    pub timeout: Duration,
}

impl std::fmt::Debug for Bridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bridge")
            .field("upstream", &self.upstream.base())
            .field("secret", &self.secret)
            .finish()
    }
}

impl Bridge {
    pub fn new(base: &str, secret: Secret, max_concurrency: usize) -> Result<Self, String> {
        Ok(Self {
            upstream: Upstream::new(base)?,
            secret,
            permits: Semaphore::new(max_concurrency.max(1)),
            timeout: Duration::from_secs(90),
        })
    }
}

// -------------------------------------------------------------- broker

/// Configuration of the broker that sits next to ChatMock.
#[derive(Clone, Debug)]
pub struct BrokerConfig {
    /// ChatMock base URL (loopback).
    pub upstream: String,
    pub secret: Secret,
    /// Models the broker will route (intersected with ChatMock's live list).
    pub models: Vec<String>,
    pub max_concurrency: usize,
    pub timeout: Duration,
}

pub struct Broker {
    cfg: BrokerConfig,
    upstream: Upstream,
    permits: Semaphore,
    live: Mutex<Option<(Instant, HashSet<String>)>>,
}

/// How long the broker trusts its view of ChatMock's live model list.
const LIVE_MODELS_TTL: Duration = Duration::from_secs(300);

impl Broker {
    pub fn new(cfg: BrokerConfig) -> Result<Arc<Self>, String> {
        for m in &cfg.models {
            if !crate::model::AI_KNOWN_MODELS.contains(&m.as_str()) {
                return Err(format!("broker model '{m}' is not a known platform model"));
            }
        }
        if cfg.models.is_empty() {
            return Err("broker needs at least one model".into());
        }
        let upstream = Upstream::new(&cfg.upstream)?;
        Ok(Arc::new(Self {
            permits: Semaphore::new(cfg.max_concurrency.max(1)),
            cfg,
            upstream,
            live: Mutex::new(None),
        }))
    }

    /// Allowed models that ChatMock currently exposes.
    async fn live_models(&self) -> Result<Vec<String>, AiError> {
        let mut cache = self.live.lock().await;
        let fresh = cache
            .as_ref()
            .is_some_and(|(at, _)| at.elapsed() < LIVE_MODELS_TTL);
        if !fresh {
            let reply = self
                .upstream
                .send(
                    Method::GET,
                    "/v1/models",
                    &[],
                    None,
                    Duration::from_secs(10),
                )
                .await?;
            if !reply.status.is_success() {
                return Err(upstream_error(&reply));
            }
            let v: Value = serde_json::from_slice(&reply.body).map_err(|_| {
                AiError::new(
                    StatusCode::BAD_GATEWAY,
                    "upstream_invalid",
                    "the model provider returned an unusable model list",
                )
            })?;
            let ids: HashSet<String> = v
                .get("data")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|m| m.get("id").and_then(Value::as_str))
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            *cache = Some((Instant::now(), ids));
        }
        let live = &cache.as_ref().expect("filled above").1;
        Ok(self
            .cfg
            .models
            .iter()
            .filter(|m| live.contains(*m))
            .cloned()
            .collect())
    }
}

pub fn broker_router(broker: Arc<Broker>) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok\n" }))
        .route("/v1/models", get(broker_models))
        .route("/v1/chat/completions", post(broker_chat))
        .fallback(|| async {
            AiError::new(StatusCode::NOT_FOUND, "not_found", "unknown broker path")
        })
        .with_state(broker)
}

fn broker_auth(b: &Broker, headers: &HeaderMap) -> Result<(), AiError> {
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    if b.cfg.secret.matches_header(auth) {
        Ok(())
    } else {
        Err(AiError::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "broker credential required",
        ))
    }
}

async fn broker_models(State(b): State<Arc<Broker>>, headers: HeaderMap) -> Response {
    if let Err(e) = broker_auth(&b, &headers) {
        return e.into_response();
    }
    match b.live_models().await {
        Ok(models) => json_response(StatusCode::OK, &models_list(&models)),
        Err(e) => e.into_response(),
    }
}

pub fn models_list(models: &[String]) -> Value {
    json!({
        "object": "list",
        "data": models.iter().map(|m| json!({"id": m, "object": "model", "owned_by": "repo.box"})).collect::<Vec<_>>(),
    })
}

fn tag(headers: &HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .filter(|v| {
            v.len() <= 63
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .unwrap_or("-")
        .to_string()
}

async fn broker_chat(State(b): State<Arc<Broker>>, headers: HeaderMap, body: Body) -> Response {
    let started = Instant::now();
    let app = tag(&headers, "x-repobox-app");
    let user = tag(&headers, "x-repobox-user-id");
    let result = async {
        broker_auth(&b, &headers)?;
        let raw = read_body(body).await?;
        let live = b.live_models().await?;
        let limits = Limits {
            models: b.cfg.models.clone(),
            default_model: None,
            max_input_chars: crate::model::AI_MAX_INPUT_CHARS_CEILING,
            max_output_tokens: crate::model::AI_MAX_OUTPUT_TOKENS_CEILING,
        };
        let req = clean_request(&raw, &limits)?;
        if !live.contains(&req.model) {
            return Err(AiError::bad(
                "model_unavailable",
                format!(
                    "model '{}' is not currently exposed by the provider",
                    req.model
                ),
            ));
        }
        let _permit = b.permits.try_acquire().map_err(|_| {
            AiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "busy",
                "the AI broker is at capacity; retry shortly",
            )
        })?;
        let reply = b
            .upstream
            .send(
                Method::POST,
                "/v1/chat/completions",
                // ChatMock accepts any compatibility key; none is forwarded.
                &[],
                Some(req.body.to_string().into_bytes()),
                b.cfg.timeout,
            )
            .await?;
        if !reply.status.is_success() {
            return Err(upstream_error(&reply));
        }
        let out = clean_response(&reply.body, &req.model, req.max_tokens)?;
        Ok((req, out))
    }
    .await;
    let ms = started.elapsed().as_millis();
    match result {
        Ok((req, out)) => {
            tracing::info!(
                target: "ai_broker",
                "chat app={app} user={user} model={} status=200 in_chars={} max_tokens={} ms={ms}",
                req.model,
                req.input_chars,
                req.max_tokens
            );
            json_response(StatusCode::OK, &out)
        }
        Err(e) => {
            tracing::info!(
                target: "ai_broker",
                "chat app={app} user={user} status={} code={} ms={ms}",
                e.status.as_u16(),
                e.code
            );
            e.into_response()
        }
    }
}

/// Read a request body with the platform's size ceiling.
pub async fn read_body(body: Body) -> Result<Bytes, AiError> {
    axum::body::to_bytes(body, AI_MAX_BODY_BYTES)
        .await
        .map_err(|_| {
            AiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "request_too_large",
                format!("request body exceeds {AI_MAX_BODY_BYTES} bytes"),
            )
        })
}

/// Run the broker on a loopback address until the process is stopped.
pub async fn serve_broker(
    bind: SocketAddr,
    cfg: BrokerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !bind.ip().is_loopback() {
        return Err(format!("ai-broker must bind to loopback, got {bind}").into());
    }
    let broker = Broker::new(cfg)?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(
        "ai broker listening on {} -> {} (models: {})",
        listener.local_addr()?,
        broker.upstream.base(),
        broker.cfg.models.join(", ")
    );
    axum::serve(listener, broker_router(broker)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            models: vec!["gpt-5.6-terra".into(), "gpt-5.6-luna".into()],
            default_model: Some("gpt-5.6-terra".into()),
            max_input_chars: 100,
            max_output_tokens: 50,
        }
    }

    fn req(v: Value) -> Result<CleanRequest, AiError> {
        clean_request(v.to_string().as_bytes(), &limits())
    }

    #[test]
    fn rebuilds_a_minimal_upstream_body() {
        let r = req(json!({
            "messages": [
                {"role": "developer", "content": "be brief"},
                {"role": "user", "content": [{"type": "text", "text": "hi"}], "name": "x"}
            ],
            "temperature": 0.2,
            "user": "someone",
            "metadata": {"a": 1},
            "stop": "END",
        }))
        .unwrap();
        assert_eq!(r.model, "gpt-5.6-terra", "default model");
        assert_eq!(r.max_tokens, 50, "defaults to the policy limit");
        assert_eq!(r.input_chars, 10);
        assert_eq!(
            r.body,
            json!({
                "model": "gpt-5.6-terra",
                "messages": [{"role": "system", "content": "be brief"}, {"role": "user", "content": "hi"}],
                "max_tokens": 50,
                "stream": false,
                "temperature": 0.2,
                "stop": ["END"],
            })
        );
    }

    #[test]
    fn refuses_what_v1_does_not_do() {
        let m = json!([{"role": "user", "content": "hi"}]);
        let code = |v: Value| req(v).unwrap_err().code;
        assert_eq!(
            code(json!({"messages": m, "stream": true})),
            "streaming_unsupported"
        );
        assert_eq!(
            code(json!({"messages": m, "tools": [{}]})),
            "unsupported_parameter"
        );
        assert_eq!(
            code(json!({"messages": m, "functions": [{}]})),
            "unsupported_parameter"
        );
        assert_eq!(
            code(json!({"messages": m, "n": 2})),
            "unsupported_parameter"
        );
        assert_eq!(
            code(json!({"messages": m, "model": "gpt-4o"})),
            "model_not_allowed"
        );
        assert_eq!(
            code(json!({"messages": m, "model": "gpt-5.6-sol"})),
            "model_not_allowed"
        );
        assert_eq!(
            code(json!({"messages": m, "max_tokens": 51})),
            "max_tokens_too_large"
        );
        assert_eq!(
            code(json!({"messages": m, "max_tokens": 0})),
            "invalid_parameter"
        );
        assert_eq!(
            code(json!({"messages": m, "temperature": 3})),
            "invalid_parameter"
        );
        assert_eq!(code(json!({"messages": []})), "invalid_parameter");
        assert_eq!(
            code(json!({"messages": [{"role": "tool", "content": "x"}]})),
            "unsupported_parameter"
        );
        assert_eq!(
            code(
                json!({"messages": [{"role": "user", "content": [{"type": "image_url", "image_url": {"url": "x"}}]}]})
            ),
            "unsupported_parameter"
        );
        assert_eq!(
            code(json!({"messages": [{"role": "assistant", "content": "", "tool_calls": [{}]}]})),
            "unsupported_parameter"
        );
        let big = "x".repeat(101);
        let e = req(json!({"messages": [{"role": "user", "content": big}]})).unwrap_err();
        assert_eq!(
            (e.status, e.code),
            (StatusCode::PAYLOAD_TOO_LARGE, "input_too_large")
        );
        let many: Vec<Value> = (0..=AI_MAX_MESSAGES)
            .map(|_| json!({"role": "user", "content": "a"}))
            .collect();
        assert_eq!(code(json!({"messages": many})), "too_many_messages");
        assert_eq!(
            clean_request(b"not json", &limits()).unwrap_err().code,
            "invalid_json"
        );
        let huge = vec![b' '; AI_MAX_BODY_BYTES + 1];
        assert_eq!(
            clean_request(&huge, &limits()).unwrap_err().code,
            "request_too_large"
        );
        // Without a default the model is mandatory (broker hop).
        let mut l = limits();
        l.default_model = None;
        assert_eq!(
            clean_request(json!({"messages": m}).to_string().as_bytes(), &l)
                .unwrap_err()
                .code,
            "invalid_parameter"
        );
    }

    #[test]
    fn responses_are_rebuilt_and_bounded() {
        let upstream = json!({
            "id": "resp_1", "created": 5, "model": "internal",
            "choices": [{"index": 0, "finish_reason": "stop",
                "message": {"role": "assistant", "content": "hello", "tool_calls": [{"x": 1}]}}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 1, "total_tokens": 4},
            "provider_secret": "never forwarded",
        });
        let out = clean_response(upstream.to_string().as_bytes(), "gpt-5.6-terra", 10).unwrap();
        assert_eq!(
            out["choices"][0]["message"],
            json!({"role": "assistant", "content": "hello"})
        );
        assert_eq!(out["model"], "gpt-5.6-terra");
        assert_eq!(out["usage"]["total_tokens"], 4);
        assert!(!out.to_string().contains("never forwarded"));
        assert!(!out.to_string().contains("tool_calls"));
        let long = json!({"choices": [{"message": {"content": "y".repeat(100)}, "finish_reason": "stop"}]});
        let out = clean_response(long.to_string().as_bytes(), "m", 2).unwrap();
        assert_eq!(
            out["choices"][0]["message"]["content"]
                .as_str()
                .unwrap()
                .len(),
            16
        );
        assert_eq!(out["choices"][0]["finish_reason"], "length");
        assert_eq!(
            clean_response(b"{}", "m", 2).unwrap_err().code,
            "upstream_invalid"
        );
    }

    #[test]
    fn loopback_urls_and_secrets() {
        assert_eq!(
            validate_loopback_url("http://127.0.0.1:3232/").unwrap(),
            "http://127.0.0.1:3232"
        );
        assert!(validate_loopback_url("http://[::1]:1").is_ok());
        assert!(validate_loopback_url("http://10.0.0.1:3232").is_err());
        assert!(validate_loopback_url("http://example.com:80").is_err());
        assert!(validate_loopback_url("https://127.0.0.1:1").is_err());
        assert!(validate_loopback_url("http://127.0.0.1:1/v1").is_err());
        assert!(Secret::new("short".into()).is_err());
        let s = Secret::new("a".repeat(40)).unwrap();
        assert!(s.matches_header(Some(&format!("Bearer {}", "a".repeat(40)))));
        assert!(!s.matches_header(Some(&format!("Bearer {}", "b".repeat(40)))));
        assert!(!s.matches_header(None));
        assert_eq!(format!("{s:?}"), "Secret(<redacted>)");
    }
}
