//! HTTP surface: the auth.repo.box UI and the loopback gate that Caddy calls
//! through `forward_auth` for every managed app request.

pub mod ai;
pub mod api;
pub mod css;
pub mod docs;
pub mod gate;
pub mod html;
pub mod pages;
pub mod publisher;
pub mod pwa;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{any, get, post};

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
/// Holds a raw onboarding-link token between `GET /invite/<app>/<token>` (which
/// only moves it here and redirects to the clean `/invite`) and the POST that
/// consumes it, so the token never stays in the address bar, in a rendered
/// page or in a request URI that answers with a body.
pub const INVITE_COOKIE: &str = "__Host-rb_invite";

/// Default lifetime of an onboarding link (UI and CLI): 72 hours.
pub const INVITE_TTL_DEFAULT: i64 = 72 * 3600;
/// Longest onboarding link the CLI will mint.
pub const INVITE_TTL_MAX: i64 = 7 * 86400;

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
    /// Lifetime of an onboarding (invitation) link.
    pub invite_ttl: i64,
    /// Bridge to the ChatMock broker (None: the AI endpoint answers 503).
    pub ai: Option<std::sync::Arc<crate::ai::Bridge>>,
    /// How routes are rendered on this host (for read-only route previews).
    pub routes: crate::render::RenderConfig,
    /// External publisher (None: the publisher API answers 503).
    pub publisher: Option<publisher::PublisherConfig>,
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
            invite_ttl: INVITE_TTL_DEFAULT,
            ai: None,
            routes: crate::render::RenderConfig {
                domain: domain.to_string(),
                gate: "127.0.0.1:3230".into(),
                apps_roots: vec![
                    "/srv/repobox-platform/apps".into(),
                    "/var/www/repo.box/subdomains".into(),
                ],
            },
            publisher: None,
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
        // Installable-app shell (see `pwa`): manifest, worker, script,
        // offline page and icons. Unknown `/assets/*` is a plain 404.
        .route(pwa::MANIFEST_PATH, get(pwa::manifest))
        .route(pwa::SW_PATH, get(pwa::sw))
        .route(pwa::SCRIPT_PATH, get(pwa::script))
        .route(pwa::OFFLINE_PATH, get(pwa::offline))
        .route("/assets/icons/{file}", get(pwa::icon_file))
        .route("/assets/{*rest}", any(pwa::asset_missing))
        .route("/apple-touch-icon.png", get(pwa::apple_touch_icon))
        .route("/favicon.ico", any(pwa::asset_missing))
        .route(pwa::SESSION_PATH, get(pwa::session))
        .route("/healthz", get(pages::healthz))
        // These public Android endpoints are intentionally outside the
        // identity and private-app launch-code surfaces. Keep them before the
        // generic `/{name}` launcher route below.
        .route(
            "/.well-known/assetlinks.json",
            get(pages::android_assetlinks),
        )
        .route("/runtime/secure-vault", get(pages::runtime_secure_vault))
        .route(
            "/runtime/hyperliquid-positions",
            get(pages::runtime_hyperliquid_positions),
        )
        .route("/api/directory", get(pages::api_directory))
        .route("/gate/verify", get(gate::verify))
        .route("/gate/agent-handoff-v1", get(gate::agent_handoff_v1))
        // Platform AI endpoint, reached only through an app route
        // (`/_repo_box/ai/v1/*` rewritten by Caddy); `/gate/*` is 404 on
        // auth.repo.box itself.
        .route("/gate/ai/v1/chat/completions", post(ai::chat))
        .route("/gate/ai/v1/models", get(ai::models))
        .route("/gate/ai/{*rest}", any(ai::not_found))
        // Agent/operator discovery and the scoped machine API (+ MCP).
        .route(docs::DOCS_PATH, get(docs::docs))
        .route("/docs/", get(|| async { redirect(docs::DOCS_PATH) }))
        .route("/.well-known/repobox-platform.json", get(api::discovery))
        .route("/api/platform/v1", get(api::discovery))
        .route("/api/platform/v1/openapi.json", get(api::openapi))
        .route("/api/platform/v1/skill.md", get(api::skill))
        .route("/api/platform/v1/whoami", get(api::whoami))
        .route("/api/platform/v1/apps", get(api::apps))
        .route("/api/platform/v1/apps/{name}", get(api::app))
        .route(
            "/api/platform/v1/apps/{name}/ai",
            get(api::app_ai).patch(api::app_ai_patch),
        )
        .route("/api/platform/v1/apps/{name}/route", get(api::app_route))
        .route(
            "/api/platform/v1/app-requests",
            get(api::app_requests).post(api::app_request_create),
        )
        .route("/api/platform/v1/release", get(api::release))
        // Publisher API (Bearer rbpub_…): deploy and operate own apps.
        .route("/api/platform/v1/publisher/whoami", get(publisher::whoami))
        .route(
            "/api/platform/v1/publisher/releases",
            get(publisher::releases)
                .post(publisher::deploy)
                // Image archives stream to disk; the handler enforces 2 GiB.
                .layer(axum::extract::DefaultBodyLimit::max(
                    crate::publisher::archive::MAX_UPLOAD_BYTES as usize + 64 * 1024,
                )),
        )
        .route(
            "/api/platform/v1/publisher/releases/{id}",
            get(publisher::release),
        )
        .route(
            "/api/platform/v1/publisher/releases/{id}/log",
            get(publisher::release_log),
        )
        .route("/api/platform/v1/publisher/apps", get(publisher::apps))
        .route(
            "/api/platform/v1/publisher/apps/{name}",
            get(publisher::app),
        )
        .route(
            "/api/platform/v1/publisher/apps/{name}/logs",
            get(publisher::logs),
        )
        .route(
            "/api/platform/v1/publisher/apps/{name}/rollback",
            post(publisher::rollback),
        )
        .route(
            "/api/platform/v1/publisher/apps/{name}/restart",
            post(publisher::restart),
        )
        .route("/api/platform/v1/mcp", post(api::mcp).get(api::mcp_get))
        .route("/api/platform/v1/{*rest}", any(api::api_not_found))
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
        .route("/apps/{name}/onboard", post(pages::app_onboard_create))
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
            "/invite",
            get(pages::invite_page).post(pages::invite_submit),
        )
        .route(
            "/invite/{token}",
            get(pages::invite_get).post(pages::invite_post),
        )
        .route(
            "/invite/{app}/{token}",
            get(pages::invite_get_named).post(pages::invite_post_named),
        )
        .route("/{name}", get(pages::launch))
        .fallback(pages::not_found)
        .layer(axum::middleware::map_response(html_no_store))
        .with_state(state)
}

/// Rendered pages carry identity and one-time links, so no HTML from the
/// control plane is ever stored by a browser or proxy cache (and so the back
/// button or a resumed installed app cannot replay a signed-in page). Pages
/// that already chose a policy (the static offline page) keep it.
async fn html_no_store(mut resp: Response) -> Response {
    let is_html = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/html"));
    if is_html && !resp.headers().contains_key(header::CACHE_CONTROL) {
        resp.headers_mut()
            .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    }
    resp
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
