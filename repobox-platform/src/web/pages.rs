//! auth.repo.box pages: directory, launch, account, app management, admin,
//! enrolment and invitation redemption. All server-rendered.
//!
//! Guards return `Result<T, Response>` so a handler can `?`-style bail with a
//! fully rendered error page; the large `Err` variant is intentional.
#![allow(clippy::result_large_err)]

use std::collections::HashMap;

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::html::{Shell, app_card, esc, flash, fmt_rel, fmt_ts, page, status_page, vis_badge};
use super::{
    AUTH_COOKIE, AppState, S, clear_cookie, current_user, html, redirect, redirect_with_cookie,
    safe_next, same_origin, set_cookie, urlencode, user_agent_label,
};
use crate::model::{App, Role, User, Visibility, validate_display_name, validate_user_name};
use crate::store::{SessionKind, StoreError, TokenKind};

type Q = Query<HashMap<String, String>>;

fn msg_text(key: &str) -> Option<&'static str> {
    Some(match key {
        "enrolled" => "This device is now signed in.",
        "invited" => "Invitation accepted. The app is in your directory.",
        "saved" => "Saved.",
        "grant_added" => "Access granted.",
        "grant_removed" => "Access revoked.",
        "invite_revoked" => "Invitation revoked.",
        "session_revoked" => "Device signed out.",
        "signed_out" => "Signed out on this device.",
        "user_created" => "User created. Give them a device link to sign in.",
        "user_saved" => "User updated.",
        "csrf" => "That request did not come from this site. Try again.",
        "no_user" => "No user with that name.",
        "self_disable" => "You cannot disable your own account.",
        "bad_name" => "That name is not valid.",
        "exists" => "That name is already taken.",
        "bad_visibility" => "Unknown visibility value.",
        "not_allowed" => "You do not have access to that.",
        "not_found" => "Not found.",
        _ => return None,
    })
}

fn flashes(q: &HashMap<String, String>) -> String {
    flash(
        q.get("ok").and_then(|k| msg_text(k)),
        q.get("err").and_then(|k| msg_text(k)),
    )
}

fn shell<'a>(
    _s: &'a AppState,
    title: &'a str,
    user: Option<&'a User>,
    active: &'a str,
) -> Shell<'a> {
    Shell {
        title,
        user,
        active,
        standalone: None,
    }
}

fn err_page(
    s: &AppState,
    user: Option<&User>,
    status: StatusCode,
    heading: &str,
    text: &str,
) -> Response {
    let sh = shell(s, heading, user, "");
    let icon = match status {
        StatusCode::NOT_FOUND => "∅",
        StatusCode::FORBIDDEN => "⛔",
        StatusCode::UNAUTHORIZED => "🔒",
        _ => "!",
    };
    html(
        status,
        status_page(
            &sh,
            icon,
            heading,
            text,
            "<a class=\"btn\" href=\"/\">Back to directory</a>",
        ),
    )
}

fn internal(s: &AppState, e: impl std::fmt::Display) -> Response {
    tracing::error!("internal error: {e}");
    err_page(
        s,
        None,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong",
        "The control plane hit an internal error. Try again in a moment.",
    )
}

fn require_user(
    s: &AppState,
    headers: &HeaderMap,
) -> Result<(crate::store::Session, User), Response> {
    current_user(s, headers).ok_or_else(|| {
        let sh = shell(s, "Sign in", None, "");
        html(
            StatusCode::UNAUTHORIZED,
            status_page(
                &sh,
                "🔒",
                "This device is not signed in",
                "auth.repo.box has no passwords. An admin, or you from another signed-in device, creates a single-use device link; opening it signs this device in.",
                "<a class=\"btn\" href=\"/\">Public directory</a>",
            ),
        )
    })
}

fn require_admin(s: &AppState, headers: &HeaderMap) -> Result<User, Response> {
    let (_, u) = require_user(s, headers)?;
    if !u.is_admin() {
        return Err(err_page(
            s,
            Some(&u),
            StatusCode::FORBIDDEN,
            "Admins only",
            "This page is for platform admins.",
        ));
    }
    Ok(u)
}

fn require_same_origin(s: &AppState, headers: &HeaderMap, back: &str) -> Result<(), Response> {
    if same_origin(s, headers) {
        Ok(())
    } else {
        Err(redirect(&format!("{back}?err=csrf")))
    }
}

fn load_app(s: &AppState, name: &str, user: Option<&User>) -> Result<App, Response> {
    match s.store.app_by_name(name) {
        Ok(Some(a)) => Ok(a),
        Ok(None) => Err(err_page(
            s,
            user,
            StatusCode::NOT_FOUND,
            "No such app",
            "Nothing is registered under that name.",
        )),
        Err(e) => Err(internal(s, e)),
    }
}

fn manageable(s: &AppState, name: &str, headers: &HeaderMap) -> Result<(User, App), Response> {
    let (_, user) = require_user(s, headers)?;
    let app = load_app(s, name, Some(&user))?;
    if !s.store.can_manage(&user, &app) {
        return Err(err_page(
            s,
            Some(&user),
            StatusCode::FORBIDDEN,
            "Not your app",
            "Only the app owner or a platform admin can manage it.",
        ));
    }
    Ok((user, app))
}

// ------------------------------------------------------------------ basics

pub async fn css() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        super::css::CSS,
    )
        .into_response()
}

pub async fn healthz(State(s): State<S>) -> Response {
    match s.store.list_apps() {
        Ok(apps) => (StatusCode::OK, format!("ok apps={}\n", apps.len())).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, format!("db error: {e}\n")).into_response(),
    }
}

pub async fn not_found(State(s): State<S>, headers: HeaderMap) -> Response {
    let user = current_user(&s, &headers).map(|(_, u)| u);
    err_page(
        &s,
        user.as_ref(),
        StatusCode::NOT_FOUND,
        "Not found",
        "There is nothing at this address.",
    )
}

// --------------------------------------------------------------- directory

pub async fn index(State(s): State<S>, headers: HeaderMap, Query(q): Q) -> Response {
    let user = current_user(&s, &headers).map(|(_, u)| u);
    let apps = match s.store.list_apps() {
        Ok(a) => a,
        Err(e) => return internal(&s, e),
    };
    let domain = &s.cfg.domain;
    let mut body = String::new();
    body.push_str(&flashes(&q));

    match &user {
        None => {
            body.push_str("<h1>App directory</h1><p class=\"lead\">Public apps on repo.box. Private apps only appear here once this device is signed in.</p>");
            let listed: Vec<&App> = apps
                .iter()
                .filter(|a| a.visibility == Visibility::PublicListed && a.enabled)
                .collect();
            if listed.is_empty() {
                body.push_str("<div class=\"empty\">No public apps are listed right now.</div>");
            } else {
                body.push_str("<div class=\"grid\">");
                for a in listed {
                    body.push_str(&app_card(a, domain, false, false));
                }
                body.push_str("</div>");
            }
            body.push_str("<div class=\"panel\" style=\"margin-top:28px\"><h3>Signing in</h3><p class=\"muted\" style=\"margin:0\">There are no passwords and no email. Someone who is already signed in (or a platform admin) creates a single-use device link for you; opening it signs this device in for 30 days. Private apps are then launched from here with a one-time code, never with a shared cookie.</p></div>");
        }
        Some(u) => {
            body.push_str(&format!("<h1>Hello, {}</h1><p class=\"lead\">Apps you can open. Launching mints a one-time code for this device, which becomes a session on the app's own host.</p>", esc(&u.display_name)));
            let mut directory: Vec<&App> = Vec::new();
            for a in &apps {
                let visible = match a.visibility {
                    Visibility::PublicListed => true,
                    Visibility::Private => s.store.has_access(u, a).unwrap_or(false),
                    Visibility::PublicUnlisted => false,
                };
                if visible {
                    directory.push(a);
                }
            }
            if directory.is_empty() {
                body.push_str("<div class=\"empty\">Nothing to show yet. Ask an app owner for an invitation.</div>");
            } else {
                body.push_str("<div class=\"grid\">");
                for a in directory {
                    body.push_str(&app_card(a, domain, true, s.store.can_manage(u, a)));
                }
                body.push_str("</div>");
            }
            let managed: Vec<&App> = apps.iter().filter(|a| s.store.can_manage(u, a)).collect();
            if !managed.is_empty() {
                body.push_str(&format!(
                    "<h2>Apps you manage <span class=\"count\">{}</span></h2><div class=\"grid\">",
                    managed.len()
                ));
                for a in managed {
                    body.push_str(&app_card(a, domain, true, true));
                }
                body.push_str("</div>");
                body.push_str("<p class=\"hint\" style=\"margin-top:10px\">Registering a new app and rendering its route is an operator step (<code>repobox-platform app register</code> + <code>routes render</code>), not something this UI does.</p>");
            }
        }
    }
    let sh = shell(&s, "Directory", user.as_ref(), "directory");
    html(StatusCode::OK, page(&sh, &body))
}

pub async fn api_directory(State(s): State<S>) -> Response {
    let apps = match s.store.list_apps() {
        Ok(a) => a,
        Err(e) => return internal(&s, e),
    };
    let items: Vec<serde_json::Value> = apps
        .iter()
        .filter(|a| a.visibility == Visibility::PublicListed && a.enabled)
        .map(|a| {
            serde_json::json!({
                "name": a.name,
                "title": a.title,
                "description": a.description,
                "url": a.url(&s.cfg.domain),
            })
        })
        .collect();
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=60"),
        ],
        serde_json::json!({ "apps": items }).to_string(),
    )
        .into_response()
}

// ------------------------------------------------------------------ launch

pub async fn launch(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(q): Q,
) -> Response {
    let user = current_user(&s, &headers).map(|(_, u)| u);
    let app = match load_app(&s, &name, user.as_ref()) {
        Ok(a) => a,
        Err(r) => return r,
    };
    let next = safe_next(q.get("next").map(String::as_str));
    if !app.enabled {
        let manage = user
            .as_ref()
            .map(|u| s.store.can_manage(u, &app))
            .unwrap_or(false);
        let actions = if manage {
            format!(
                "<a class=\"btn\" href=\"/apps/{}\">Manage app</a>",
                esc(&app.name)
            )
        } else {
            String::new()
        };
        let sh = shell(&s, &app.title, user.as_ref(), "");
        return html(
            StatusCode::NOT_FOUND,
            status_page(
                &sh,
                "⏸",
                "This app is switched off",
                "Its owner has disabled it.",
                &actions,
            ),
        );
    }
    let Some(user) = user else {
        if app.visibility.is_public() {
            return redirect(&format!("https://{}{}", app.host(&s.cfg.domain), next));
        }
        let sh = shell(&s, &app.title, None, "");
        return html(
            StatusCode::UNAUTHORIZED,
            status_page(
                &sh,
                "🔒",
                &format!("{} is private", app.title),
                "Sign this device in first. There are no passwords: a signed-in device or an admin creates a single-use device link for you.",
                "<a class=\"btn\" href=\"/\">Public directory</a>",
            ),
        );
    };
    let allowed = s.store.has_access(&user, &app).unwrap_or(false);
    if !allowed && !app.visibility.is_public() {
        return err_page(
            &s,
            Some(&user),
            StatusCode::FORBIDDEN,
            "No access",
            "You do not have a grant for this app. Ask its owner for an invitation.",
        );
    }
    if !allowed {
        // Public app, no grant: open it without an identity.
        return redirect(&format!("https://{}{}", app.host(&s.cfg.domain), next));
    }
    let (code, _) = match s.store.create_token(
        TokenKind::Launch,
        Some(user.id),
        Some(app.id),
        Some(user.id),
        s.cfg.launch_ttl,
        "",
    ) {
        Ok(v) => v,
        Err(e) => return internal(&s, e),
    };
    s.store.audit(Some(user.id), "launch.mint", &app.name, "");
    let sep = if next.contains('?') { '&' } else { '?' };
    let location = format!(
        "https://{}{}{}token={}",
        app.host(&s.cfg.domain),
        next,
        sep,
        code
    );
    let mut resp = (StatusCode::FOUND, "").into_response();
    resp.headers_mut()
        .insert(header::LOCATION, location.parse().unwrap());
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    resp
}

// ----------------------------------------------------------------- account

pub async fn me(State(s): State<S>, headers: HeaderMap, Query(q): Q) -> Response {
    let (sess, user) = match require_user(&s, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let sessions = s
        .store
        .list_sessions(user.id, SessionKind::Auth)
        .unwrap_or_default();
    let now = s.store.now();
    let mut body = flashes(&q);
    body.push_str(&format!(
        "<h1>{}</h1><p class=\"lead\">Signed in as <code>{}</code> · {}</p>",
        esc(&user.display_name),
        esc(&user.name),
        if user.is_admin() {
            "platform admin"
        } else {
            "member"
        }
    ));
    body.push_str("<div class=\"two\"><div class=\"panel\"><h3>Sign in another device</h3><p class=\"muted\">Creates a single-use link, valid for 7 days, that signs in whichever device opens it as you. Send it over a channel you trust; it is shown once.</p><form method=\"post\" action=\"/me/enrol-device\"><button class=\"btn primary\" type=\"submit\">Create device link</button></form></div>");
    body.push_str("<div class=\"panel\"><h3>Sign out here</h3><p class=\"muted\">Ends the session on this device only. App sessions on individual app hosts expire on their own within 24 hours.</p><form method=\"post\" action=\"/logout\"><button class=\"btn danger\" type=\"submit\">Sign out this device</button></form></div></div>");
    body.push_str(&format!("<h2>Signed-in devices <span class=\"count\">{}</span></h2><div class=\"panel\"><div class=\"table-wrap\"><table><thead><tr><th>Device</th><th>Signed in</th><th>Last seen</th><th>Expires</th><th></th></tr></thead><tbody>", sessions.len()));
    for x in &sessions {
        let this = if x.id == sess.id {
            " <span class=\"badge\">this device</span>"
        } else {
            ""
        };
        body.push_str(&format!(
            "<tr><td>{}{}</td><td>{}</td><td>{}</td><td>{}</td><td><form class=\"inline\" method=\"post\" action=\"/me/sessions/{}/revoke\"><button class=\"btn small danger\" type=\"submit\">Sign out</button></form></td></tr>",
            esc(if x.label.is_empty() { "device" } else { &x.label }),
            this,
            fmt_ts(x.created_at),
            fmt_rel(now, x.last_seen_at),
            fmt_rel(now, x.expires_at),
            x.id
        ));
    }
    body.push_str("</tbody></table></div></div>");
    let sh = shell(&s, "Account", Some(&user), "me");
    html(StatusCode::OK, page(&sh, &body))
}

fn show_link_once(
    s: &AppState,
    user: &User,
    heading: &str,
    intro: &str,
    link: &str,
    expires_at: i64,
    back: &str,
) -> Response {
    let body = format!(
        "<h1>{}</h1><p class=\"lead\">{}</p><div class=\"panel\"><div class=\"secret\">{}</div><p class=\"hint\" style=\"margin-top:10px\">Single use · expires {} · this page will not show it again.</p><div class=\"row\"><a class=\"btn\" href=\"{}\">Done</a></div></div>",
        esc(heading),
        esc(intro),
        esc(link),
        fmt_ts(expires_at),
        esc(back)
    );
    let sh = shell(s, heading, Some(user), "");
    let mut resp = html(StatusCode::OK, page(&sh, &body));
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    resp
}

pub async fn me_enrol_device(State(s): State<S>, headers: HeaderMap) -> Response {
    let (_, user) = match require_user(&s, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_same_origin(&s, &headers, "/me") {
        return r;
    }
    match s.store.create_token(
        TokenKind::Enrol,
        Some(user.id),
        None,
        Some(user.id),
        s.cfg.link_ttl,
        "self",
    ) {
        Ok((raw, tok)) => {
            s.store
                .audit(Some(user.id), "enrol.create", &user.name, "self");
            let link = format!("{}/enrol/{}", s.cfg.public_base, raw);
            show_link_once(
                &s,
                &user,
                "Device link ready",
                "Open this on the device you want to sign in. It signs that device in as you.",
                &link,
                tok.expires_at,
                "/me",
            )
        }
        Err(e) => internal(&s, e),
    }
}

pub async fn me_revoke_session(
    State(s): State<S>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let (_, user) = match require_user(&s, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    if let Err(r) = require_same_origin(&s, &headers, "/me") {
        return r;
    }
    let _ = s.store.revoke_session(id, user.id);
    s.store
        .audit(Some(user.id), "session.revoke", &user.name, "");
    redirect("/me?ok=session_revoked")
}

pub async fn logout(State(s): State<S>, headers: HeaderMap) -> Response {
    if let Some((sess, user)) = current_user(&s, &headers)
        && same_origin(&s, &headers)
    {
        let _ = s.store.revoke_session(sess.id, user.id);
        s.store
            .audit(Some(user.id), "session.logout", &user.name, "");
    }
    redirect_with_cookie("/?ok=signed_out", clear_cookie(AUTH_COOKIE))
}

// ------------------------------------------------------------------- admin

#[derive(Deserialize)]
pub struct CreateUserForm {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub role: String,
}

pub async fn admin_users(State(s): State<S>, headers: HeaderMap, Query(q): Q) -> Response {
    let admin = match require_admin(&s, &headers) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let users = s.store.list_users().unwrap_or_default();
    let mut body = flashes(&q);
    body.push_str("<h1>Users</h1><p class=\"lead\">Named identities on the platform. A user signs a device in by opening a single-use device link; nothing is emailed.</p>");
    body.push_str("<div class=\"panel\"><h3>Create a user</h3><form method=\"post\" action=\"/admin/users\" class=\"row\"><div class=\"field\"><label for=\"name\">Handle</label><input type=\"text\" id=\"name\" name=\"name\" placeholder=\"e.g. ocean\" required autocomplete=\"off\" autocapitalize=\"none\"></div><div class=\"field\"><label for=\"dn\">Display name</label><input type=\"text\" id=\"dn\" name=\"display_name\" placeholder=\"Ocean\" required autocomplete=\"off\"></div><div class=\"field\" style=\"flex:0 0 140px\"><label for=\"role\">Role</label><select id=\"role\" name=\"role\"><option value=\"member\">member</option><option value=\"admin\">admin</option></select></div><div class=\"field\" style=\"flex:0 0 auto\"><label>&nbsp;</label><button class=\"btn primary\" type=\"submit\">Create</button></div></form></div>");
    body.push_str(&format!("<h2>All users <span class=\"count\">{}</span></h2><div class=\"panel\"><div class=\"table-wrap\"><table><thead><tr><th>Handle</th><th>Name</th><th>Role</th><th>Status</th><th>Since</th><th></th></tr></thead><tbody>", users.len()));
    for u in &users {
        let status = if u.enabled {
            "<span class=\"badge public_listed\">active</span>"
        } else {
            "<span class=\"badge off\">disabled</span>"
        };
        let toggle = if u.id == admin.id {
            String::new()
        } else if u.enabled {
            format!(
                "<form class=\"inline\" method=\"post\" action=\"/admin/users/{}/enabled\"><input type=\"hidden\" name=\"enabled\" value=\"0\"><button class=\"btn small danger\" type=\"submit\">Disable</button></form>",
                esc(&u.name)
            )
        } else {
            format!(
                "<form class=\"inline\" method=\"post\" action=\"/admin/users/{}/enabled\"><input type=\"hidden\" name=\"enabled\" value=\"1\"><button class=\"btn small\" type=\"submit\">Enable</button></form>",
                esc(&u.name)
            )
        };
        let enrol = if u.enabled {
            format!(
                "<form class=\"inline\" method=\"post\" action=\"/admin/users/{}/enrol\"><button class=\"btn small\" type=\"submit\">Device link</button></form> ",
                esc(&u.name)
            )
        } else {
            String::new()
        };
        body.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"row\" style=\"justify-content:flex-end\">{}{}</td></tr>",
            esc(&u.name),
            esc(&u.display_name),
            u.role.as_str(),
            status,
            fmt_ts(u.created_at),
            enrol,
            toggle
        ));
    }
    body.push_str("</tbody></table></div></div>");
    let sh = shell(&s, "Users", Some(&admin), "users");
    html(StatusCode::OK, page(&sh, &body))
}

pub async fn admin_users_create(
    State(s): State<S>,
    headers: HeaderMap,
    Form(f): Form<CreateUserForm>,
) -> Response {
    let admin = match require_admin(&s, &headers) {
        Ok(u) => u,
        Err(r) => return r,
    };
    if let Err(r) = require_same_origin(&s, &headers, "/admin/users") {
        return r;
    }
    let name = f.name.trim().to_lowercase();
    if validate_user_name(&name).is_err() || validate_display_name(&f.display_name).is_err() {
        return redirect("/admin/users?err=bad_name");
    }
    let role = if f.role == "admin" {
        Role::Admin
    } else {
        Role::Member
    };
    match s.store.create_user(&name, f.display_name.trim(), role) {
        Ok(u) => {
            s.store
                .audit(Some(admin.id), "user.create", &u.name, role.as_str());
            redirect("/admin/users?ok=user_created")
        }
        Err(StoreError::Conflict(_)) => redirect("/admin/users?err=exists"),
        Err(StoreError::Invalid(_)) => redirect("/admin/users?err=bad_name"),
        Err(e) => internal(&s, e),
    }
}

#[derive(Deserialize)]
pub struct EnabledForm {
    pub enabled: String,
}

pub async fn admin_user_enabled(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Form(f): Form<EnabledForm>,
) -> Response {
    let admin = match require_admin(&s, &headers) {
        Ok(u) => u,
        Err(r) => return r,
    };
    if let Err(r) = require_same_origin(&s, &headers, "/admin/users") {
        return r;
    }
    let Ok(Some(target)) = s.store.user_by_name(&name) else {
        return redirect("/admin/users?err=no_user");
    };
    if target.id == admin.id {
        return redirect("/admin/users?err=self_disable");
    }
    let enabled = f.enabled == "1";
    if let Err(e) = s.store.set_user_enabled(target.id, enabled) {
        return internal(&s, e);
    }
    if !enabled {
        let _ = s.store.revoke_user_sessions(target.id);
    }
    s.store.audit(
        Some(admin.id),
        if enabled {
            "user.enable"
        } else {
            "user.disable"
        },
        &target.name,
        "",
    );
    redirect("/admin/users?ok=user_saved")
}

pub async fn admin_user_enrol(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Response {
    let admin = match require_admin(&s, &headers) {
        Ok(u) => u,
        Err(r) => return r,
    };
    if let Err(r) = require_same_origin(&s, &headers, "/admin/users") {
        return r;
    }
    let Ok(Some(target)) = s.store.user_by_name(&name) else {
        return redirect("/admin/users?err=no_user");
    };
    if !target.enabled {
        return redirect("/admin/users?err=not_allowed");
    }
    match s.store.create_token(
        TokenKind::Enrol,
        Some(target.id),
        None,
        Some(admin.id),
        s.cfg.link_ttl,
        "admin",
    ) {
        Ok((raw, tok)) => {
            s.store
                .audit(Some(admin.id), "enrol.create", &target.name, "admin");
            let link = format!("{}/enrol/{}", s.cfg.public_base, raw);
            show_link_once(
                &s,
                &admin,
                &format!("Device link for {}", target.display_name),
                "Send this to them over a channel you trust. Whoever opens it is signed in as this user, once.",
                &link,
                tok.expires_at,
                "/admin/users",
            )
        }
        Err(e) => internal(&s, e),
    }
}

pub async fn admin_audit(State(s): State<S>, headers: HeaderMap) -> Response {
    let admin = match require_admin(&s, &headers) {
        Ok(u) => u,
        Err(r) => return r,
    };
    let entries = s.store.list_audit(200).unwrap_or_default();
    let mut body = String::from(
        "<h1>Audit log</h1><p class=\"lead\">Latest 200 control-plane events. Tokens and secrets are never recorded.</p><div class=\"panel\"><div class=\"table-wrap\"><table><thead><tr><th>When</th><th>Actor</th><th>Action</th><th>Subject</th><th>Detail</th></tr></thead><tbody>",
    );
    for e in &entries {
        body.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td><code>{}</code></td><td>{}</td><td class=\"muted\">{}</td></tr>",
            fmt_ts(e.at),
            esc(e.actor.as_deref().unwrap_or("—")),
            esc(&e.action),
            esc(&e.subject),
            esc(&e.detail)
        ));
    }
    body.push_str("</tbody></table></div></div>");
    let sh = shell(&s, "Audit", Some(&admin), "audit");
    html(StatusCode::OK, page(&sh, &body))
}

// ------------------------------------------------------------- app manage

pub async fn app_manage(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(q): Q,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let owner = s.store.user_by_id(app.owner_id).ok();
    let grants = s.store.list_grants(app.id).unwrap_or_default();
    let invites = s
        .store
        .list_app_tokens(TokenKind::Invite, app.id)
        .unwrap_or_default();
    let now = s.store.now();
    let base = format!("/apps/{}", esc(&app.name));
    let mut body = flashes(&q);
    body.push_str(&format!(
        "<div class=\"row\"><h1 style=\"margin:0\">{}</h1>{}{}</div><p class=\"lead\"><span class=\"mono\">{}</span>{}</p>",
        esc(&app.title),
        vis_badge(app.visibility),
        if app.enabled { "" } else { "<span class=\"badge off\">disabled</span>" },
        esc(&app.host(&s.cfg.domain)),
        if app.description.is_empty() { String::new() } else { format!(" · {}", esc(&app.description)) }
    ));
    body.push_str(&format!(
        "<div class=\"row\" style=\"margin-bottom:18px\"><a class=\"btn primary\" href=\"/{}\">Launch</a><a class=\"btn\" href=\"/\">Directory</a></div>",
        esc(&app.name)
    ));
    body.push_str(&format!(
        "<div class=\"panel\"><dl class=\"kv\"><dt>Route</dt><dd><code>{}</code> → <code>{}</code></dd><dt>Owner</dt><dd>{}</dd><dt>Registered</dt><dd>{}</dd></dl><p class=\"hint\" style=\"margin:10px 0 0\">Route shape and origin are operator-managed (CLI + rendered Caddy config). This page controls who may open the app.</p></div>",
        app.kind.as_str(),
        esc(&app.target),
        owner.map(|o| format!("{} (<code>{}</code>)", esc(&o.display_name), esc(&o.name))).unwrap_or_else(|| "—".into()),
        fmt_ts(app.created_at)
    ));

    body.push_str("<div class=\"two\" style=\"margin-top:14px\">");
    body.push_str(&format!("<div class=\"panel\"><h3>Visibility</h3><form method=\"post\" action=\"{base}/visibility\" class=\"stack\"><div class=\"radios\">"));
    for v in Visibility::ALL {
        body.push_str(&format!(
            "<label><input type=\"radio\" name=\"visibility\" value=\"{}\"{}><span><strong>{}</strong><span class=\"hint\">{}</span></span></label>",
            v.as_str(),
            if v == app.visibility { " checked" } else { "" },
            esc(v.label()),
            esc(v.help())
        ));
    }
    body.push_str("</div><div><button class=\"btn primary\" type=\"submit\">Save visibility</button></div></form></div>");
    body.push_str(&format!(
        "<div class=\"panel\"><h3>Availability</h3><p class=\"muted\">{}</p><form method=\"post\" action=\"{base}/enabled\"><input type=\"hidden\" name=\"enabled\" value=\"{}\"><button class=\"btn {}\" type=\"submit\">{}</button></form></div>",
        if app.enabled { "The app is on. Disabling it makes the edge answer 404 for everyone immediately, including holders of a live session." } else { "The app is off. Nobody can reach it until you enable it again." },
        if app.enabled { "0" } else { "1" },
        if app.enabled { "danger" } else { "primary" },
        if app.enabled { "Disable app" } else { "Enable app" }
    ));
    body.push_str("</div>");

    body.push_str(&format!(
        "<h2>Access <span class=\"count\">{}</span></h2><div class=\"panel\">",
        grants.len()
    ));
    body.push_str(&format!("<form method=\"post\" action=\"{base}/grants\" class=\"row\" style=\"margin-bottom:14px\"><div class=\"field\"><label for=\"grant-user\">Grant a user by handle</label><input type=\"text\" id=\"grant-user\" name=\"user\" placeholder=\"handle\" required autocomplete=\"off\" autocapitalize=\"none\"></div><div class=\"field\" style=\"flex:0 0 auto\"><label>&nbsp;</label><button class=\"btn primary\" type=\"submit\">Grant</button></div></form>"));
    if grants.is_empty() {
        body.push_str("<div class=\"empty\">No grants yet. Admins and the owner can always open the app.</div>");
    } else {
        body.push_str("<div class=\"table-wrap\"><table><thead><tr><th>User</th><th>Granted</th><th>By</th><th></th></tr></thead><tbody>");
        for g in &grants {
            body.push_str(&format!(
                "<tr><td>{} <code>{}</code>{}</td><td>{}</td><td>{}</td><td style=\"text-align:right\"><form class=\"inline\" method=\"post\" action=\"{base}/grants/{}/revoke\"><button class=\"btn small danger\" type=\"submit\">Revoke</button></form></td></tr>",
                esc(&g.user.display_name),
                esc(&g.user.name),
                if g.user.enabled { "" } else { " <span class=\"badge off\">disabled</span>" },
                fmt_ts(g.created_at),
                esc(g.granted_by.as_deref().unwrap_or("—")),
                esc(&g.user.name)
            ));
        }
        body.push_str("</tbody></table></div>");
    }
    body.push_str("</div>");

    body.push_str(&format!("<h2>Invitations <span class=\"count\">{}</span></h2><div class=\"panel\"><p class=\"muted\">An invitation link grants access to whoever opens it, once. A person who is not on the platform yet picks a handle and becomes a user at the same time.</p><form method=\"post\" action=\"{base}/invites\" style=\"margin-bottom:14px\"><button class=\"btn primary\" type=\"submit\">Create invitation link</button></form>", invites.iter().filter(|t| t.status(now) == "active").count()));
    if invites.is_empty() {
        body.push_str("<div class=\"empty\">No invitations created.</div>");
    } else {
        body.push_str("<div class=\"table-wrap\"><table><thead><tr><th>Created</th><th>Status</th><th>Expires</th><th>Used by</th><th></th></tr></thead><tbody>");
        for t in &invites {
            let status = t.status(now);
            let used_by = t
                .used_by
                .and_then(|id| s.store.user_by_id(id).ok())
                .map(|u| u.name)
                .unwrap_or_else(|| "—".into());
            let action = if status == "active" {
                format!(
                    "<form class=\"inline\" method=\"post\" action=\"{base}/invites/{}/revoke\"><button class=\"btn small danger\" type=\"submit\">Revoke</button></form>",
                    t.id
                )
            } else {
                String::new()
            };
            body.push_str(&format!(
                "<tr><td>{}</td><td><span class=\"badge {}\">{}</span></td><td>{}</td><td>{}</td><td style=\"text-align:right\">{}</td></tr>",
                fmt_ts(t.created_at),
                if status == "active" { "public_listed" } else { "" },
                status,
                fmt_rel(now, t.expires_at),
                esc(&used_by),
                action
            ));
        }
        body.push_str("</tbody></table></div>");
    }
    body.push_str("</div>");

    let sh = shell(&s, &app.title, Some(&user), "");
    html(StatusCode::OK, page(&sh, &body))
}

#[derive(Deserialize)]
pub struct VisibilityForm {
    pub visibility: String,
}

pub async fn app_visibility(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Form(f): Form<VisibilityForm>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    let Some(v) = Visibility::parse(&f.visibility) else {
        return redirect(&format!("{back}?err=bad_visibility"));
    };
    if let Err(e) = s.store.set_app_visibility(app.id, v) {
        return internal(&s, e);
    }
    s.store
        .audit(Some(user.id), "app.visibility", &app.name, v.as_str());
    redirect(&format!("{back}?ok=saved"))
}

pub async fn app_enabled(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Form(f): Form<EnabledForm>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    let enabled = f.enabled == "1";
    if let Err(e) = s.store.set_app_enabled(app.id, enabled) {
        return internal(&s, e);
    }
    s.store.audit(
        Some(user.id),
        if enabled { "app.enable" } else { "app.disable" },
        &app.name,
        "",
    );
    redirect(&format!("{back}?ok=saved"))
}

#[derive(Deserialize)]
pub struct GrantForm {
    pub user: String,
}

pub async fn app_grant_add(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Form(f): Form<GrantForm>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    let handle = f.user.trim().to_lowercase();
    let Ok(Some(target)) = s.store.user_by_name(&handle) else {
        return redirect(&format!("{back}?err=no_user"));
    };
    match s.store.add_grant(app.id, target.id, Some(user.id)) {
        Ok(_) => {
            s.store
                .audit(Some(user.id), "grant.add", &app.name, &target.name);
            redirect(&format!("{back}?ok=grant_added"))
        }
        Err(e) => internal(&s, e),
    }
}

pub async fn app_grant_revoke(
    State(s): State<S>,
    headers: HeaderMap,
    Path((name, handle)): Path<(String, String)>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    let Ok(Some(target)) = s.store.user_by_name(&handle) else {
        return redirect(&format!("{back}?err=no_user"));
    };
    let _ = s.store.remove_grant(app.id, target.id);
    s.store
        .audit(Some(user.id), "grant.remove", &app.name, &target.name);
    redirect(&format!("{back}?ok=grant_removed"))
}

pub async fn app_invite_create(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    match s.store.create_token(
        TokenKind::Invite,
        None,
        Some(app.id),
        Some(user.id),
        s.cfg.link_ttl,
        "",
    ) {
        Ok((raw, tok)) => {
            s.store.audit(Some(user.id), "invite.create", &app.name, "");
            let link = format!("{}/invite/{}", s.cfg.public_base, raw);
            show_link_once(
                &s,
                &user,
                &format!("Invitation to {}", app.title),
                "Whoever opens this link gets access to the app, once. New people choose a handle on the way in.",
                &link,
                tok.expires_at,
                &back,
            )
        }
        Err(e) => internal(&s, e),
    }
}

pub async fn app_invite_revoke(
    State(s): State<S>,
    headers: HeaderMap,
    Path((name, id)): Path<(String, i64)>,
) -> Response {
    let (user, app) = match manageable(&s, &name, &headers) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let back = format!("/apps/{}", app.name);
    if let Err(r) = require_same_origin(&s, &headers, &back) {
        return r;
    }
    if let Ok(tok) = s.store.token_by_id(id)
        && tok.app_id == Some(app.id)
        && tok.kind == TokenKind::Invite
    {
        let _ = s.store.revoke_token(id);
        s.store.audit(Some(user.id), "invite.revoke", &app.name, "");
    }
    redirect(&format!("{back}?ok=invite_revoked"))
}

// ------------------------------------------------------------- enrolment

fn link_error(s: &AppState, e: crate::store::RedeemError) -> Response {
    let sh = shell(s, "Link not valid", None, "");
    html(
        StatusCode::GONE,
        status_page(
            &sh,
            "⏱",
            "Link not valid",
            e.message(),
            "<a class=\"btn\" href=\"/\">Directory</a>",
        ),
    )
}

pub async fn enrol_get(
    State(s): State<S>,
    headers: HeaderMap,
    Path(raw): Path<String>,
) -> Response {
    let tok = match s.store.peek_token(TokenKind::Enrol, &raw) {
        Ok(t) => t,
        Err(e) => return link_error(&s, e),
    };
    let Some(target) = tok
        .user_id
        .and_then(|id| s.store.user_by_id(id).ok())
        .filter(|u| u.enabled)
    else {
        return link_error(&s, crate::store::RedeemError::Unknown);
    };
    let current = current_user(&s, &headers).map(|(_, u)| u);
    let switching = current.as_ref().map(|c| c.id != target.id).unwrap_or(false);
    let note = if switching {
        format!(
            "<p class=\"flash err\">This device is currently signed in as <code>{}</code>. Continuing switches it to <code>{}</code>.</p>",
            esc(&current.as_ref().unwrap().name),
            esc(&target.name)
        )
    } else {
        String::new()
    };
    let body = format!(
        "<div class=\"status-page\"><div class=\"icon\">🔑</div><h1>Sign this device in</h1><p>This link signs this device in as <strong>{}</strong> (<code>{}</code>) for 30 days. It works once.</p>{}<form method=\"post\" action=\"/enrol/{}\"><button class=\"btn primary\" type=\"submit\">Continue as {}</button></form></div>",
        esc(&target.display_name),
        esc(&target.name),
        note,
        esc(&raw),
        esc(&target.display_name)
    );
    let sh = shell(&s, "Sign in", None, "");
    let mut resp = html(StatusCode::OK, page(&sh, &body));
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    resp
}

pub async fn enrol_post(
    State(s): State<S>,
    headers: HeaderMap,
    Path(raw): Path<String>,
) -> Response {
    if !same_origin(&s, &headers) {
        return redirect(&format!("/enrol/{}", urlencode(&raw)));
    }
    let tok = match s.store.consume_token(TokenKind::Enrol, &raw, None) {
        Ok(t) => t,
        Err(e) => return link_error(&s, e),
    };
    let Some(target) = tok
        .user_id
        .and_then(|id| s.store.user_by_id(id).ok())
        .filter(|u| u.enabled)
    else {
        return link_error(&s, crate::store::RedeemError::Unknown);
    };
    match s.store.create_session(
        SessionKind::Auth,
        target.id,
        None,
        s.cfg.auth_session_ttl,
        &user_agent_label(&headers),
    ) {
        Ok((secret, _)) => {
            s.store
                .audit(Some(target.id), "enrol.redeem", &target.name, "");
            redirect_with_cookie(
                "/?ok=enrolled",
                set_cookie(AUTH_COOKIE, &secret, s.cfg.auth_session_ttl),
            )
        }
        Err(e) => internal(&s, e),
    }
}

// ------------------------------------------------------------ invitations

#[derive(Deserialize, Default)]
pub struct InviteForm {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub display_name: String,
}

pub async fn invite_get(
    State(s): State<S>,
    headers: HeaderMap,
    Path(raw): Path<String>,
) -> Response {
    let tok = match s.store.peek_token(TokenKind::Invite, &raw) {
        Ok(t) => t,
        Err(e) => return link_error(&s, e),
    };
    let Some(app) = tok.app_id.and_then(|id| s.store.app_by_id(id).ok()) else {
        return link_error(&s, crate::store::RedeemError::Unknown);
    };
    let inviter = tok
        .created_by
        .and_then(|id| s.store.user_by_id(id).ok())
        .map(|u| u.display_name)
        .unwrap_or_else(|| "the owner".into());
    let current = current_user(&s, &headers).map(|(_, u)| u);
    let form = match &current {
        Some(u) => format!(
            "<p>You are signed in as <strong>{}</strong>. Accepting adds this app to your directory.</p><form method=\"post\" action=\"/invite/{}\"><button class=\"btn primary\" type=\"submit\">Accept invitation</button></form>",
            esc(&u.display_name),
            esc(&raw)
        ),
        None => format!(
            "<p>Pick a handle to join the platform and get access. This device will be signed in as the new user.</p><form method=\"post\" action=\"/invite/{}\" class=\"stack\" style=\"text-align:left;max-width:360px;margin:0 auto\"><div class=\"field\"><label for=\"name\">Handle</label><input type=\"text\" id=\"name\" name=\"name\" placeholder=\"e.g. ocean\" required autocomplete=\"off\" autocapitalize=\"none\"><span class=\"hint\">lowercase letters, digits, . _ -</span></div><div class=\"field\"><label for=\"dn\">Display name</label><input type=\"text\" id=\"dn\" name=\"display_name\" placeholder=\"Ocean\" required autocomplete=\"off\"></div><div><button class=\"btn primary\" type=\"submit\">Join and open</button></div></form>",
            esc(&raw)
        ),
    };
    let body = format!(
        "<div class=\"status-page\"><div class=\"icon\">✉️</div><h1>Invitation to {}</h1><p class=\"muted\">{} invited you · <span class=\"mono\">{}</span></p>{}</div>",
        esc(&app.title),
        esc(&inviter),
        esc(&app.host(&s.cfg.domain)),
        form
    );
    let sh = shell(&s, "Invitation", current.as_ref(), "");
    let mut resp = html(StatusCode::OK, page(&sh, &body));
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    resp
}

pub async fn invite_post(
    State(s): State<S>,
    headers: HeaderMap,
    Path(raw): Path<String>,
    Form(f): Form<InviteForm>,
) -> Response {
    if !same_origin(&s, &headers) {
        return redirect(&format!("/invite/{}", urlencode(&raw)));
    }
    // Validate before consuming so a typo does not burn the invitation.
    let tok = match s.store.peek_token(TokenKind::Invite, &raw) {
        Ok(t) => t,
        Err(e) => return link_error(&s, e),
    };
    let Some(app) = tok.app_id.and_then(|id| s.store.app_by_id(id).ok()) else {
        return link_error(&s, crate::store::RedeemError::Unknown);
    };
    let current = current_user(&s, &headers).map(|(_, u)| u);
    let (user, new_cookie) = match current {
        Some(u) => (u, None),
        None => {
            let name = f.name.trim().to_lowercase();
            if validate_user_name(&name).is_err() || validate_display_name(&f.display_name).is_err()
            {
                let sh = shell(&s, "Invitation", None, "");
                return html(
                    StatusCode::BAD_REQUEST,
                    status_page(
                        &sh,
                        "!",
                        "Pick another handle",
                        "Handles are 1-32 lowercase letters, digits, '.', '_' or '-'. Display names are 1-64 characters.",
                        &format!("<a class=\"btn\" href=\"/invite/{}\">Back</a>", esc(&raw)),
                    ),
                );
            }
            if matches!(s.store.user_by_name(&name), Ok(Some(_))) {
                let sh = shell(&s, "Invitation", None, "");
                return html(
                    StatusCode::CONFLICT,
                    status_page(
                        &sh,
                        "!",
                        "Handle already taken",
                        "Choose a different handle, or sign in on this device first if that handle is yours.",
                        &format!("<a class=\"btn\" href=\"/invite/{}\">Back</a>", esc(&raw)),
                    ),
                );
            }
            // Consume first so two people cannot both join on one invitation.
            if let Err(e) = s.store.consume_token(TokenKind::Invite, &raw, None) {
                return link_error(&s, e);
            }
            let u = match s
                .store
                .create_user(&name, f.display_name.trim(), Role::Member)
            {
                Ok(u) => u,
                Err(e) => return internal(&s, e),
            };
            s.store.audit(Some(u.id), "user.join", &u.name, &app.name);
            let secret = match s.store.create_session(
                SessionKind::Auth,
                u.id,
                None,
                s.cfg.auth_session_ttl,
                &user_agent_label(&headers),
            ) {
                Ok((secret, _)) => secret,
                Err(e) => return internal(&s, e),
            };
            (
                u,
                Some(set_cookie(AUTH_COOKIE, &secret, s.cfg.auth_session_ttl)),
            )
        }
    };
    if new_cookie.is_none()
        && let Err(e) = s
            .store
            .consume_token(TokenKind::Invite, &raw, Some(user.id))
    {
        return link_error(&s, e);
    }
    let _ = s.store.add_grant(app.id, user.id, tok.created_by);
    s.store.audit(Some(user.id), "invite.redeem", &app.name, "");
    match new_cookie {
        Some(c) => redirect_with_cookie("/?ok=invited", c),
        None => redirect("/?ok=invited"),
    }
}
