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
use super::{APP_COOKIE, S, html, set_cookie, urlencode};
use crate::model::{Visibility, validate_app_name};
use crate::render::{GATE_APP_HEADER, GATE_MARKER_HEADER};
use crate::store::{SessionKind, TokenKind};

fn hdr<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Split a request URI into (path, query pairs) and drop the `token` pair,
/// returning the clean URI to redirect to plus the token if there was one.
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
                Some(("token", v)) => token = Some(v.to_string()),
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

    // 2. Existing host-only app session?
    let session = super::cookie_value(&headers, APP_COOKIE)
        .and_then(|raw| {
            s.store
                .session_lookup(SessionKind::App, &raw)
                .ok()
                .flatten()
        })
        .filter(|(sess, _)| sess.app_id == Some(app.id))
        .filter(|(_, user)| s.store.has_access(user, &app).unwrap_or(false));

    // 3. Private apps require a session; public apps do not.
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
    use super::split_token;

    #[test]
    fn token_splitting_keeps_other_params() {
        assert_eq!(split_token("/"), ("/".into(), None));
        assert_eq!(split_token("/?token=abc"), ("/".into(), Some("abc".into())));
        assert_eq!(
            split_token("/p/q?a=1&token=abc&b=2"),
            ("/p/q?a=1&b=2".into(), Some("abc".into()))
        );
        assert_eq!(
            split_token("/p?token=abc&token=def"),
            ("/p".into(), Some("def".into()))
        );
        assert_eq!(
            split_token("//evil.com/?token=x"),
            ("/evil.com/".into(), Some("x".into()))
        );
        assert_eq!(split_token("/x?tokens=1"), ("/x?tokens=1".into(), None));
    }
}
