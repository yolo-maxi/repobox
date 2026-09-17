//! The edge gate. Caddy calls `GET /gate/verify` via `forward_auth` before
//! serving any managed app. The contract:
//!
//! * 2xx: Caddy serves the request and copies the identity headers we set
//!   (`copy_headers`) onto the request to the origin.
//! * else: Caddy returns our response to the browser verbatim, which is how
//!   the one-time launch code becomes a host-only session cookie plus a
//!   clean-URL redirect, and how anonymous users are turned away.
//!
//! Nothing here trusts a header the browser could have set: the app name comes
//! from `header_up` in the generated route, and the generated route strips
//! `X-RepoBox-*` before this hop.

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::html::{Shell, status_page};
use super::{APP_COOKIE, LAUNCH_PARAM, S, html, set_cookie, urlencode};
use crate::model::{Visibility, validate_app_name};
use crate::render::{GATE_APP_HEADER, GATE_MARKER_HEADER};
use crate::store::{SessionKind, TokenKind};

fn hdr<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Split a request URI into (path, query pairs) and drop the launch-code
/// pair (`LAUNCH_PARAM`), returning the clean URI to redirect to plus the code
/// if there was one. Every other query parameter, including an app's own
/// `token=`, is passed through untouched.
fn split_token(uri: &str) -> (String, Option<String>) {
    let (path, query) = match uri.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (uri, None),
    };
    let mut path = if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    };
    // A protocol-relative path would make the relative Location an open redirect.
    while path.starts_with("//") || path.starts_with("/\\") {
        path.remove(0);
    }
    let mut token = None;
    let mut keep = Vec::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if pair.is_empty() {
                continue;
            }
            match pair.split_once('=') {
                Some((k, v)) if k == LAUNCH_PARAM => token = Some(v.to_string()),
                _ => keep.push(pair),
            }
        }
    }
    let clean = if keep.is_empty() {
        path
    } else {
        format!("{path}?{}", keep.join("&"))
    };
    (clean, token)
}

/// File extensions that are plainly not a page even when a browser navigates
/// to them directly (an image opened in a tab, a JSON file, a download).
const NON_PAGE_EXTENSIONS: &[&str] = &[
    "js",
    "mjs",
    "cjs",
    "css",
    "map",
    "json",
    "xml",
    "txt",
    "ico",
    "png",
    "jpg",
    "jpeg",
    "gif",
    "svg",
    "webp",
    "avif",
    "bmp",
    "woff",
    "woff2",
    "ttf",
    "otf",
    "eot",
    "mp3",
    "mp4",
    "webm",
    "ogg",
    "wav",
    "wasm",
    "pdf",
    "zip",
    "gz",
    "csv",
    "md",
    "webmanifest",
    "manifest",
];

/// Is this request a person's browser navigating to an HTML document of the
/// app? This is what an *open* is built on (see `store::record_open`), so it
/// is deliberately strict; when in doubt it answers `false`:
///
/// * `GET` only: `HEAD`, `POST` and friends are never navigations;
/// * no `Upgrade` (WebSocket handshakes are `GET`s);
/// * if the browser says what it is fetching (`Sec-Fetch-Dest`), it must be
///   `document` (not `iframe`, `empty` for fetch/XHR, `script`, `image`, …)
///   and `Sec-Fetch-Mode`, when present, must be `navigate`;
/// * without `Sec-Fetch-Dest` the `Accept` header must ask for HTML;
/// * prefetch/prerender speculation is not a person opening a page;
/// * the path must not end in a non-page file extension.
///
/// Caddy's `forward_auth` hop forwards these request headers unchanged
/// (verified against Caddy 2.10/2.11), and nothing here is stored: the
/// answer is a boolean.
pub fn is_document_navigation(method: &str, uri: &str, headers: &HeaderMap) -> bool {
    if method != "GET" {
        return false;
    }
    if hdr(headers, "upgrade").is_some() {
        return false;
    }
    let purpose = hdr(headers, "sec-purpose")
        .or_else(|| hdr(headers, "purpose"))
        .unwrap_or("")
        .to_ascii_lowercase();
    if purpose.contains("prefetch") || purpose.contains("prerender") {
        return false;
    }
    match hdr(headers, "sec-fetch-dest").map(|v| v.trim().to_ascii_lowercase()) {
        Some(dest) if dest == "document" => {
            if let Some(mode) = hdr(headers, "sec-fetch-mode")
                && !mode.trim().eq_ignore_ascii_case("navigate")
            {
                return false;
            }
        }
        Some(_) => return false,
        None => {
            let accepts_html = hdr(headers, "accept").unwrap_or("").split(',').any(|part| {
                let mime = part.split(';').next().unwrap_or("").trim();
                mime.eq_ignore_ascii_case("text/html")
                    || mime.eq_ignore_ascii_case("application/xhtml+xml")
            });
            if !accepts_html {
                return false;
            }
        }
    }
    let path = uri.split('?').next().unwrap_or("/");
    let last = path.rsplit('/').next().unwrap_or("");
    if let Some((_, ext)) = last.rsplit_once('.')
        && NON_PAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
    {
        return false;
    }
    true
}

pub async fn verify(State(s): State<S>, headers: HeaderMap) -> Response {
    let shell = Shell {
        title: "Gate",
        user: None,
        active: "",
        standalone: Some(&s.cfg.public_base),
    };

    // Only Caddy's forward_auth hop sets the marker; anything else is not a gate call.
    if hdr(&headers, GATE_MARKER_HEADER) != Some("1") {
        return html(
            StatusCode::NOT_FOUND,
            status_page(&shell, "∅", "Not found", "", ""),
        );
    }
    let Some(app_name) = hdr(&headers, GATE_APP_HEADER) else {
        return html(
            StatusCode::BAD_REQUEST,
            status_page(
                &shell,
                "!",
                "Gate misconfigured",
                "The route did not name an app.",
                "",
            ),
        );
    };
    if validate_app_name(app_name).is_err() {
        return html(
            StatusCode::BAD_REQUEST,
            status_page(
                &shell,
                "!",
                "Gate misconfigured",
                "Invalid app name in route.",
                "",
            ),
        );
    }
    let Ok(Some(app)) = s.store.app_by_name(app_name) else {
        return html(
            StatusCode::NOT_FOUND,
            status_page(
                &shell,
                "∅",
                "Unknown app",
                "This host is not registered with the platform.",
                "",
            ),
        );
    };

    let method = hdr(&headers, "x-forwarded-method").unwrap_or("GET");
    let uri = hdr(&headers, "x-forwarded-uri").unwrap_or("/").to_string();
    let (clean_uri, token) = split_token(&uri);
    let launch_url = format!("{}/{}", s.cfg.public_base, app.name);
    let sign_in = |text: &str| {
        format!(
            "<a class=\"btn primary\" href=\"{}?next={}\">{}</a>",
            launch_url,
            urlencode(&clean_uri),
            text
        )
    };

    // The host-only app session this browser already holds, if any.
    let session = super::cookie_value(&headers, APP_COOKIE)
        .and_then(|raw| {
            s.store
                .session_lookup(SessionKind::App, &raw)
                .ok()
                .flatten()
        })
        .filter(|(sess, _)| sess.app_id == Some(app.id))
        .filter(|(_, user)| s.store.has_access(user, &app).unwrap_or(false));

    // 1. One-time launch code redemption (only on navigations).
    if let Some(raw) = token.filter(|_| method == "GET" || method == "HEAD") {
        if !app.enabled {
            return disabled_page(&shell);
        }
        let redeemed = s.store.consume_token(TokenKind::Launch, &raw, None);
        let outcome = match redeemed {
            Ok(tok) if tok.app_id == Some(app.id) => match tok
                .user_id
                .and_then(|id| s.store.user_by_id(id).ok())
            {
                Some(user) if user.enabled && s.store.has_access(&user, &app).unwrap_or(false) => {
                    Ok((user, tok))
                }
                _ => Err("This launch code no longer grants access."),
            },
            Ok(_) => Err("This launch code belongs to a different app."),
            Err(e) => Err(e.message()),
        };
        return match outcome {
            Ok((user, tok)) => {
                let Ok((secret, _)) = s.store.create_session(
                    SessionKind::App,
                    user.id,
                    Some(app.id),
                    tok.session_id,
                    s.cfg.app_session_ttl,
                    &super::user_agent_label(&headers),
                ) else {
                    return html(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        status_page(&shell, "!", "Gate error", "Could not create a session.", ""),
                    );
                };
                s.store.audit(Some(user.id), "launch.redeem", &app.name, "");
                let mut resp = (StatusCode::FOUND, "").into_response();
                let h = resp.headers_mut();
                h.insert(
                    header::LOCATION,
                    HeaderValue::from_str(&clean_uri).unwrap_or(HeaderValue::from_static("/")),
                );
                if let Ok(v) =
                    HeaderValue::from_str(&set_cookie(APP_COOKIE, &secret, s.cfg.app_session_ttl))
                {
                    h.insert(header::SET_COOKIE, v);
                }
                h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
                resp
            }
            Err(msg) if app.visibility.is_public() => {
                // Public app: a bad code is harmless, just drop it from the URL.
                let _ = msg;
                let mut resp = (StatusCode::FOUND, "").into_response();
                resp.headers_mut().insert(
                    header::LOCATION,
                    HeaderValue::from_str(&clean_uri).unwrap_or(HeaderValue::from_static("/")),
                );
                resp
            }
            Err(msg) if session.is_some() => {
                // A stale or replayed code on a browser that is already signed
                // in to this app (back button, duplicated tab): keep the
                // session, drop the code from the URL, carry on.
                s.store.audit(
                    None,
                    "launch.reject",
                    &app.name,
                    &format!("{msg} (session kept)"),
                );
                let mut resp = (StatusCode::FOUND, "").into_response();
                resp.headers_mut().insert(
                    header::LOCATION,
                    HeaderValue::from_str(&clean_uri).unwrap_or(HeaderValue::from_static("/")),
                );
                resp.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
                resp
            }
            Err(msg) => {
                s.store.audit(None, "launch.reject", &app.name, msg);
                html(
                    StatusCode::FORBIDDEN,
                    status_page(
                        &shell,
                        "⏱",
                        "Launch code not accepted",
                        &format!(
                            "{msg} Launch codes are single-use and expire after {} seconds.",
                            s.cfg.launch_ttl
                        ),
                        &sign_in("Launch again"),
                    ),
                )
            }
        };
    }

    if !app.enabled {
        return disabled_page(&shell);
    }

    // 2./3. Private apps require a session; public apps do not.
    if app.visibility == Visibility::Private && session.is_none() {
        return html(
            StatusCode::UNAUTHORIZED,
            status_page(
                &shell,
                "🔒",
                &format!("{} is private", app.title),
                "Sign in on auth.repo.box, then launch the app from there. Your device needs an enrolment link if it is not signed in yet.",
                &sign_in("Sign in and launch"),
            ),
        );
    }

    // 4. Allow. This is the only place an access is counted: nothing above
    //    (denials, disabled apps, code redemption redirects) touches the
    //    counters. Private apps keep the signed-in user id for unique-user
    //    dedup; public traffic is counted without any identity.
    let counted_user = match (&session, app.visibility) {
        (Some((_, user)), Visibility::Private) => Some(user.id),
        _ => None,
    };
    if let Err(e) = s.store.record_access(app.id, counted_user) {
        tracing::warn!("access count for {} failed: {e}", app.name);
    }
    // An *open* is narrower than a counted request: a signed-in person's
    // browser navigating to an HTML document, deduplicated per visit by the
    // store. Anonymous traffic (public apps) is never an open.
    if let Some((_, user)) = &session
        && is_document_navigation(method, &clean_uri, &headers)
        && let Err(e) = s.store.record_open(app.id, user.id)
    {
        tracing::warn!("open count for {} failed: {e}", app.name);
    }
    let mut resp = StatusCode::OK.into_response();
    let h = resp.headers_mut();
    let put = |h: &mut HeaderMap, k: &'static str, v: &str| {
        if let Ok(hv) = HeaderValue::from_str(v) {
            h.insert(k, hv);
        }
    };
    put(h, "X-RepoBox-App", &app.name);
    match session {
        Some((_, user)) => {
            put(h, "X-RepoBox-Auth", "session");
            put(h, "X-RepoBox-User", &user.name);
            put(h, "X-RepoBox-User-Id", &user.id.to_string());
            put(h, "X-RepoBox-Role", user.role.as_str());
        }
        None => {
            put(h, "X-RepoBox-Auth", "public");
        }
    }
    resp
}

fn disabled_page(shell: &Shell<'_>) -> Response {
    html(
        StatusCode::NOT_FOUND,
        status_page(
            shell,
            "⏸",
            "This app is switched off",
            "Its owner has disabled it on auth.repo.box.",
            "",
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{is_document_navigation, split_token};
    use axum::http::HeaderMap;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        h
    }

    #[test]
    fn document_navigations_are_recognised() {
        let nav = headers(&[
            ("accept", "text/html,application/xhtml+xml,*/*;q=0.8"),
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "navigate"),
        ]);
        assert!(is_document_navigation("GET", "/", &nav));
        assert!(is_document_navigation("GET", "/dash?x=1", &nav));
        assert!(is_document_navigation("GET", "/page.html", &nav));
        assert!(is_document_navigation("GET", "/user/john.doe", &nav));
        assert!(
            is_document_navigation("GET", "/api/setup", &nav),
            "no path guessing"
        );
        // Old browser or curl: no Sec-Fetch headers, Accept decides.
        assert!(is_document_navigation(
            "GET",
            "/",
            &headers(&[("accept", "text/html;q=0.9, */*;q=0.1")])
        ));
        assert!(is_document_navigation(
            "GET",
            "/",
            &headers(&[("accept", "application/xhtml+xml")])
        ));
    }

    #[test]
    fn everything_else_is_not_an_open() {
        let nav = headers(&[
            ("accept", "text/html"),
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "navigate"),
        ]);
        for m in ["HEAD", "POST", "PUT", "OPTIONS"] {
            assert!(!is_document_navigation(m, "/", &nav), "{m}");
        }
        // Assets and fetches announce themselves.
        for dest in [
            "script",
            "style",
            "image",
            "font",
            "empty",
            "iframe",
            "manifest",
            "websocket",
        ] {
            let h = headers(&[("accept", "*/*"), ("sec-fetch-dest", dest)]);
            assert!(!is_document_navigation("GET", "/", &h), "{dest}");
        }
        // A fetch that happens to accept HTML is still not a navigation.
        let h = headers(&[
            ("accept", "text/html"),
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
        ]);
        assert!(!is_document_navigation("GET", "/", &h));
        let h = headers(&[
            ("accept", "text/html"),
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "cors"),
        ]);
        assert!(!is_document_navigation("GET", "/", &h));
        // WebSocket handshake.
        let h = headers(&[("upgrade", "websocket"), ("connection", "Upgrade")]);
        assert!(!is_document_navigation("GET", "/ws", &h));
        // No Sec-Fetch and a non-HTML Accept (curl default, JSON clients).
        assert!(!is_document_navigation(
            "GET",
            "/",
            &headers(&[("accept", "*/*")])
        ));
        assert!(!is_document_navigation(
            "GET",
            "/",
            &headers(&[("accept", "application/json")])
        ));
        assert!(!is_document_navigation("GET", "/", &headers(&[])));
        // Speculative loads.
        let h = headers(&[
            ("accept", "text/html"),
            ("sec-fetch-dest", "document"),
            ("sec-purpose", "prefetch;prerender"),
        ]);
        assert!(!is_document_navigation("GET", "/", &h));
        let h = headers(&[("accept", "text/html"), ("purpose", "prefetch")]);
        assert!(!is_document_navigation("GET", "/", &h));
        // Direct routes to plain files.
        for p in [
            "/app.js",
            "/data.json",
            "/logo.PNG",
            "/x/y.css?v=2",
            "/report.pdf",
        ] {
            assert!(!is_document_navigation("GET", p, &nav), "{p}");
        }
    }

    #[test]
    fn token_splitting_keeps_other_params() {
        assert_eq!(split_token("/"), ("/".into(), None));
        assert_eq!(
            split_token("/?rb_launch=abc"),
            ("/".into(), Some("abc".into()))
        );
        assert_eq!(
            split_token("/p/q?a=1&rb_launch=abc&b=2"),
            ("/p/q?a=1&b=2".into(), Some("abc".into()))
        );
        assert_eq!(
            split_token("/p?rb_launch=abc&rb_launch=def"),
            ("/p".into(), Some("def".into()))
        );
        assert_eq!(
            split_token("//evil.com/?rb_launch=x"),
            ("/evil.com/".into(), Some("x".into()))
        );
        assert_eq!(
            split_token("/x?rb_launchs=1"),
            ("/x?rb_launchs=1".into(), None)
        );
    }

    #[test]
    fn apps_own_token_parameter_is_never_treated_as_a_launch_code() {
        // Ellie's apps use `?token=` for their own setup/invite links; the
        // gate must pass those through untouched.
        assert_eq!(
            split_token("/api/setup/check?token=abc"),
            ("/api/setup/check?token=abc".into(), None)
        );
        assert_eq!(
            split_token("/invite?token=abc&rb_launch=code"),
            ("/invite?token=abc".into(), Some("code".into()))
        );
    }
}
