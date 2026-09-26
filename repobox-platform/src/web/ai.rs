//! The same-origin AI endpoint of every managed app:
//! `https://<app>.repo.box/_repo_box/ai/v1/chat/completions` (and
//! `/v1/models`). The generated Caddy route strips browser `X-RepoBox-*`,
//! runs the normal gate, copies the gate-issued identity and rewrites the
//! request to `/gate/ai/v1/*` on this loopback service.
//!
//! This handler does not trust headers alone. A request is served only if
//! all of these hold:
//! * the route marker and route-named app are present (set by Caddy, never
//!   by the browser, whose copies were stripped);
//! * the gate-issued identity says `session` and names a user id;
//! * the browser's host-only `__Host-rb_app` cookie resolves to a live app
//!   session *for that app*, whose user still has access and is the same
//!   user the gate named. A forged header on a direct loopback call without
//!   that cookie is refused;
//! * the request is same-origin (JSON content type, which forces a CORS
//!   preflight that is never answered; `Origin`/`Sec-Fetch-Site` checked
//!   when present), so a sibling `*.repo.box` page cannot spend the quota;
//! * the app is enabled, its AI policy is on, and the request fits the
//!   policy's models, limits and daily quotas.
//!
//! Then the rebuilt request goes to the broker through the loopback tunnel
//! with the bridge secret. Prompts, completions and cookies are never
//! logged or stored; only counters are.

use std::time::Instant;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::{APP_COOKIE, S};
use crate::ai::{AiError, CleanRequest, Limits, clean_request, clean_response, json_response};
use crate::model::{App, User, validate_app_name};
use crate::render::{GATE_APP_HEADER, GATE_MARKER_HEADER};
use crate::store::SessionKind;

fn hdr<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn unauthenticated() -> AiError {
    AiError::new(
        StatusCode::UNAUTHORIZED,
        "unauthenticated",
        "a signed-in platform session for this app is required; open the app from auth.repo.box first",
    )
}

/// Resolve and authorise the caller. Returns the app and the user.
fn authorise(s: &S, headers: &HeaderMap, method: &Method) -> Result<(App, User), AiError> {
    let not_found = || AiError::new(StatusCode::NOT_FOUND, "not_found", "not found");
    if hdr(headers, GATE_MARKER_HEADER) != Some("1") {
        return Err(not_found());
    }
    let app_name = hdr(headers, GATE_APP_HEADER).ok_or_else(not_found)?;
    validate_app_name(app_name).map_err(|_| not_found())?;
    let app = s
        .store
        .app_by_name(app_name)
        .ok()
        .flatten()
        .ok_or_else(not_found)?;
    if !app.enabled {
        return Err(AiError::new(
            StatusCode::NOT_FOUND,
            "app_disabled",
            "this app is switched off",
        ));
    }

    // Identity: the gate's answer (copied by Caddy) and the browser's app
    // session must agree.
    if hdr(headers, "x-repobox-auth") != Some("session") {
        return Err(unauthenticated());
    }
    let gate_user: i64 = hdr(headers, "x-repobox-user-id")
        .and_then(|v| v.parse().ok())
        .ok_or_else(unauthenticated)?;
    let (_, user) = super::cookie_value(headers, APP_COOKIE)
        .and_then(|raw| {
            s.store
                .session_lookup(SessionKind::App, &raw)
                .ok()
                .flatten()
        })
        .filter(|(sess, _)| sess.app_id == Some(app.id))
        .filter(|(_, user)| s.store.has_access(user, &app).unwrap_or(false))
        .ok_or_else(unauthenticated)?;
    if user.id != gate_user {
        return Err(unauthenticated());
    }

    // Same-origin only.
    // Scheme and host must match; the port is not compared (production is
    // 443 only, and the local edge test runs Caddy on a high port).
    let host = app.host(&s.cfg.domain);
    if let Some(o) = hdr(headers, "origin")
        && o.strip_prefix("https://")
            .map(|rest| rest.trim_end_matches('/'))
            .map(|rest| rest.split_once(':').map_or(rest, |(h, _)| h))
            != Some(host.as_str())
    {
        return Err(AiError::new(
            StatusCode::FORBIDDEN,
            "cross_origin",
            "the AI endpoint only accepts requests from the app's own origin",
        ));
    }
    if let Some(site) = hdr(headers, "sec-fetch-site")
        && site != "same-origin"
    {
        return Err(AiError::new(
            StatusCode::FORBIDDEN,
            "cross_origin",
            "the AI endpoint only accepts requests from the app's own origin",
        ));
    }
    if *method == Method::POST {
        let ct = hdr(headers, "content-type").unwrap_or("");
        if !ct
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .eq_ignore_ascii_case("application/json")
        {
            return Err(AiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "content_type",
                "Content-Type must be application/json",
            ));
        }
    }

    if !app.ai.enabled {
        return Err(AiError::new(
            StatusCode::FORBIDDEN,
            "ai_disabled",
            "the platform AI capability is not enabled for this app",
        ));
    }
    Ok((app, user))
}

fn bridge(s: &S) -> Result<&crate::ai::Bridge, AiError> {
    s.cfg.ai.as_deref().ok_or_else(|| {
        AiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "ai_unavailable",
            "the platform AI broker is not configured on this host",
        )
    })
}

fn bridge_headers(b: &crate::ai::Bridge, app: &App, user: &User) -> Vec<(&'static str, String)> {
    vec![
        ("authorization", b.secret.bearer()),
        ("x-repobox-app", app.name.clone()),
        ("x-repobox-user-id", user.id.to_string()),
    ]
}

pub async fn chat(State(s): State<S>, headers: HeaderMap, body: Body) -> Response {
    let started = Instant::now();
    let mut log_app = String::from("-");
    let mut log_user = String::from("-");
    let result: Result<(CleanRequest, serde_json::Value), AiError> = async {
        let (app, user) = authorise(&s, &headers, &Method::POST)?;
        log_app = app.name.clone();
        log_user = user.id.to_string();
        let raw = crate::ai::read_body(body).await?;
        let req = clean_request(&raw, &Limits::for_policy(&app.ai))?;
        let b = bridge(&s)?;
        let _permit = b.permits.try_acquire().map_err(|_| {
            AiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "busy",
                "the platform AI endpoint is at capacity; retry shortly",
            )
        })?;
        match s.store.ai_take_quota(&app, user.id) {
            Ok(Ok(_)) => {}
            Ok(Err(which)) => {
                return Err(AiError::new(
                    StatusCode::TOO_MANY_REQUESTS,
                    "quota_exceeded",
                    match which {
                        "app_daily_quota" => "this app has used its AI requests for today (UTC)",
                        _ => "you have used your AI requests for this app today (UTC)",
                    },
                ));
            }
            Err(_) => {
                return Err(AiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "could not check the AI quota",
                ));
            }
        }
        let reply = b
            .upstream
            .send(
                Method::POST,
                "/v1/chat/completions",
                &bridge_headers(b, &app, &user),
                Some(req.body.to_string().into_bytes()),
                b.timeout,
            )
            .await?;
        if !reply.status.is_success() {
            return Err(crate::ai::upstream_error(&reply));
        }
        let out = clean_response(&reply.body, &req.model, req.max_tokens)?;
        Ok((req, out))
    }
    .await;
    let ms = started.elapsed().as_millis();
    match result {
        Ok((req, out)) => {
            tracing::info!(
                target: "ai",
                "chat app={log_app} user={log_user} model={} status=200 in_chars={} max_tokens={} ms={ms}",
                req.model,
                req.input_chars,
                req.max_tokens
            );
            json_response(StatusCode::OK, &out)
        }
        Err(e) => {
            tracing::info!(
                target: "ai",
                "chat app={log_app} user={log_user} status={} code={} ms={ms}",
                e.status.as_u16(),
                e.code
            );
            e.into_response()
        }
    }
}

/// The app's allowed models that the broker currently routes.
pub async fn models(State(s): State<S>, headers: HeaderMap) -> Response {
    let result = async {
        let (app, user) = authorise(&s, &headers, &Method::GET)?;
        let b = bridge(&s)?;
        let reply = b
            .upstream
            .send(
                Method::GET,
                "/v1/models",
                &bridge_headers(b, &app, &user),
                None,
                std::time::Duration::from_secs(15),
            )
            .await?;
        if !reply.status.is_success() {
            return Err(crate::ai::upstream_error(&reply));
        }
        let live: Vec<String> = serde_json::from_slice::<serde_json::Value>(&reply.body)
            .ok()
            .and_then(|v| {
                v.get("data").and_then(|d| d.as_array()).map(|a| {
                    a.iter()
                        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
                        .map(str::to_string)
                        .collect()
                })
            })
            .unwrap_or_default();
        let models: Vec<String> = app
            .ai
            .models
            .iter()
            .filter(|m| live.contains(m))
            .cloned()
            .collect();
        Ok(crate::ai::models_list(&models))
    }
    .await;
    match result {
        Ok(v) => json_response(StatusCode::OK, &v),
        Err(e) => e.into_response(),
    }
}

/// Anything else under the AI prefix.
pub async fn not_found() -> Response {
    let mut r = AiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "unknown platform AI path; see https://auth.repo.box/api/platform/v1",
    )
    .into_response();
    r.headers_mut()
        .insert(header::ALLOW, "GET, POST".parse().expect("static"));
    r
}
