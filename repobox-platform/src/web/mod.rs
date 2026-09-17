//! HTTP surface: the auth.repo.box UI and the loopback gate that Caddy calls
//! through `forward_auth` for every managed app request.

pub mod css;
pub mod gate;
pub mod html;
pub mod pages;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};

use crate::model::User;
use crate::store::{Session, SessionKind, Store};

/// Host-only cookie names. The `__Host-` prefix makes browsers refuse the
/// cookie unless it is Secure, Path=/ and has no Domain attribute, which is
/// exactly the "no domain-wide bearer cookie" rule enforced by construction.
pub const AUTH_COOKIE: &str = "__Host-rb_auth";
pub const APP_COOKIE: &str = "__Host-rb_app";
/// Query parameter that carries the one-time launch code from the launcher to
/// the app host. Deliberately not `token`: apps use that name for their own
/// invite/setup links, and the gate must never swallow those.
pub const LAUNCH_PARAM: &str = "rb_launch";

#[derive(Debug, Clone)]
pub struct Config {
    /// e.g. `https://auth.repo.box` (no trailing slash)
    pub public_base: String,
    /// apex domain apps live under, e.g. `repo.box`
    pub domain: String,
    pub launch_ttl: i64,
    pub auth_session_ttl: i64,
    pub app_session_ttl: i64,
    pub link_ttl: i64,
}

impl Config {
    pub fn defaults(public_base: &str, domain: &str) -> Self {
        Self {
            public_base: public_base.trim_end_matches('/').to_string(),
            domain: domain.to_string(),
            launch_ttl: 90,
            auth_session_ttl: 30 * 86400,
            app_session_ttl: 24 * 3600,
            link_ttl: 7 * 86400,
        }
    }
}

pub struct AppState {
    pub store: Store,
    pub cfg: Config,
}

pub type S = Arc<AppState>;

pub fn router(state: S) -> Router {
    Router::new()
        .route("/", get(pages::index))
        .route("/assets/app.css", get(pages::css))
        .route("/healthz", get(pages::healthz))
        .route("/api/directory", get(pages::api_directory))
        .route("/gate/verify", get(gate::verify))
        .route("/me", get(pages::me))
        .route("/me/enrol-device", post(pages::me_enrol_device))
        .route("/me/sessions/{id}/revoke", post(pages::me_revoke_session))
        .route("/logout", post(pages::logout))
        .route(
            "/admin/users",
            get(pages::admin_users).post(pages::admin_users_create),
        )
        .route(
            "/admin/users/{name}/enabled",
            post(pages::admin_user_enabled),
        )
        .route("/admin/users/{name}/enrol", post(pages::admin_user_enrol))
        .route(
            "/admin/users/{name}/sessions",
            get(pages::admin_user_sessions),
        )
        .route(
            "/admin/users/{name}/sessions/revoke-all",
            post(pages::admin_user_sessions_revoke_all),
        )
        .route(
            "/admin/users/{name}/sessions/{id}/revoke",
            post(pages::admin_user_session_revoke),
        )
        .route("/admin/audit", get(pages::admin_audit))
        .route("/apps/{name}", get(pages::app_manage))
        .route("/apps/{name}/analytics", get(pages::app_analytics))
        .route("/apps/{name}/visits", get(pages::app_visits))
        .route(
            "/apps/{name}/grantable-users",
            get(pages::app_grantable_users),
        )
        .route("/apps/{name}/visibility", post(pages::app_visibility))
        .route("/apps/{name}/enabled", post(pages::app_enabled))
        .route("/apps/{name}/grants", post(pages::app_grant_add))
        .route(
            "/apps/{name}/grants/{user}/revoke",
            post(pages::app_grant_revoke),
        )
        .route("/apps/{name}/invites", post(pages::app_invite_create))
        .route(
            "/apps/{name}/invites/{id}/revoke",
            post(pages::app_invite_revoke),
        )
        .route(
            "/enrol/{token}",
            get(pages::enrol_get).post(pages::enrol_post),
        )
        .route(
            "/invite/{token}",
            get(pages::invite_get).post(pages::invite_post),
        )
        .route("/{name}", get(pages::launch))
        .fallback(pages::not_found)
        .with_state(state)
}

// ------------------------------------------------------------------ helpers

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    for value in headers.get_all(header::COOKIE) {
        let Ok(s) = value.to_str() else { continue };
        for pair in s.split(';') {
            let pair = pair.trim();
            if let Some((k, v)) = pair.split_once('=')
                && k.trim() == name
            {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

pub fn set_cookie(name: &str, value: &str, max_age: i64) -> String {
    format!("{name}={value}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={max_age}")
}

pub fn clear_cookie(name: &str) -> String {
    format!("{name}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn current_user(state: &AppState, headers: &HeaderMap) -> Option<(Session, User)> {
    let raw = cookie_value(headers, AUTH_COOKIE)?;
    state
        .store
        .session_lookup(SessionKind::Auth, &raw)
        .ok()
        .flatten()
}

/// CSRF guard for state-changing requests: the browser must prove the form
/// was served by this origin. Cookies are SameSite=Lax as a second layer.
pub fn same_origin(state: &AppState, headers: &HeaderMap) -> bool {
    if let Some(o) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        return o.trim_end_matches('/') == state.cfg.public_base;
    }
    if let Some(sf) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        return sf == "same-origin";
    }
    false
}

pub fn user_agent_label(headers: &HeaderMap) -> String {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let short = if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS device"
    } else if ua.contains("Android") {
        "Android device"
    } else if ua.contains("Macintosh") {
        "Mac"
    } else if ua.contains("Windows") {
        "Windows PC"
    } else if ua.contains("Linux") {
        "Linux"
    } else if ua.starts_with("curl") {
        "curl"
    } else {
        "device"
    };
    let browser = if ua.contains("Firefox") {
        "Firefox"
    } else if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Chrome") {
        "Chrome"
    } else if ua.contains("Safari") {
        "Safari"
    } else {
        ""
    };
    if browser.is_empty() {
        short.to_string()
    } else {
        format!("{browser} on {short}")
    }
}

pub fn html(status: StatusCode, body: String) -> Response {
    (status, Html(body)).into_response()
}

pub fn redirect(location: &str) -> Response {
    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, location.to_string())],
    )
        .into_response()
}

pub fn redirect_with_cookie(location: &str, cookie: String) -> Response {
    (
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, location.to_string()),
            (header::SET_COOKIE, cookie),
        ],
    )
        .into_response()
}

/// Only ever redirect to a local absolute path (never protocol-relative).
pub fn safe_next(next: Option<&str>) -> String {
    match next {
        Some(n)
            if n.starts_with('/')
                && !n.starts_with("//")
                && !n.starts_with("/\\")
                && n.len() < 2048
                && !n.chars().any(|c| c.is_control() || c.is_whitespace()) =>
        {
            n.to_string()
        }
        _ => "/".to_string(),
    }
}

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
