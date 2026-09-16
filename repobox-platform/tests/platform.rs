//! End-to-end tests against the in-process router: launch-code redemption,
//! replay, expiry, revocation, disabled app/user, visibility modes, spoofed
//! identity headers, clean redirects, enrolment and invitation links, CSRF.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

use repobox_platform::model::{AppKind, Role, User, Visibility};
use repobox_platform::store::{SessionKind, Store};
use repobox_platform::web::{self, APP_COOKIE, AUTH_COOKIE, AppState, Config};

const BASE: &str = "https://auth.repo.box";

struct H {
    state: Arc<AppState>,
    clock: Arc<AtomicI64>,
    fran: User,
    owner: User,
    bob: User,
    eve: User,
}

impl H {
    fn new() -> Self {
        let clock = Arc::new(AtomicI64::new(1_800_000_000));
        let c = clock.clone();
        let store =
            Store::open_in_memory_with_clock(Arc::new(move || c.load(Ordering::SeqCst))).unwrap();
        let fran = store.create_user("fran", "Fran", Role::Admin).unwrap();
        let owner = store.create_user("owner", "Owner", Role::Member).unwrap();
        let bob = store.create_user("bob", "Bob", Role::Member).unwrap();
        let eve = store.create_user("eve", "Eve", Role::Member).unwrap();
        let private = store
            .create_app(
                "demo-private",
                "Private demo",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:3231",
                Visibility::Private,
            )
            .unwrap();
        store
            .create_app(
                "demo-unlisted",
                "Unlisted demo",
                "",
                owner.id,
                AppKind::Static,
                "/srv/repobox-platform/apps/demo-unlisted",
                Visibility::PublicUnlisted,
            )
            .unwrap();
        store
            .create_app(
                "demo-listed",
                "Listed demo",
                "",
                owner.id,
                AppKind::Static,
                "/srv/repobox-platform/apps/demo-listed",
                Visibility::PublicListed,
            )
            .unwrap();
        store
            .create_app(
                "other-private",
                "Other private",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:3299",
                Visibility::Private,
            )
            .unwrap();
        store.add_grant(private.id, bob.id, Some(owner.id)).unwrap();
        let state = Arc::new(AppState {
            store,
            cfg: Config::defaults(BASE, "repo.box"),
        });
        H {
            state,
            clock,
            fran,
            owner,
            bob,
            eve,
        }
    }

    fn router(&self) -> Router {
        web::router(self.state.clone())
    }

    fn auth_cookie(&self, user: &User) -> String {
        let (raw, _) = self
            .state
            .store
            .create_session(SessionKind::Auth, user.id, None, None, 86400, "test")
            .unwrap();
        format!("{AUTH_COOKIE}={raw}")
    }

    /// A device session with its id, so tests can revoke it or find its children.
    fn device(&self, user: &User, label: &str) -> (String, i64) {
        let (raw, sess) = self
            .state
            .store
            .create_session(SessionKind::Auth, user.id, None, None, 86400, label)
            .unwrap();
        (format!("{AUTH_COOKIE}={raw}"), sess.id)
    }

    /// Launch `app` from an existing device cookie and redeem the code at the
    /// gate; returns the app-session cookie.
    async fn launch_from(&self, device_cookie: &str, app: &str) -> String {
        let (st, h, _) = self.get(&format!("/{app}"), Some(device_cookie)).await;
        assert_eq!(st, StatusCode::FOUND);
        let loc = h.get(header::LOCATION).unwrap().to_str().unwrap();
        let code = loc.split("token=").nth(1).unwrap().to_string();
        let (st, hd, _) = self.gate(app, &format!("/?token={code}"), None, &[]).await;
        assert_eq!(st, StatusCode::FOUND);
        app_cookie_from(&hd)
    }

    async fn send(&self, req: Request<Body>) -> (StatusCode, HeaderMap, String) {
        let resp = self.router().oneshot(req).await.unwrap();
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        (status, headers, String::from_utf8_lossy(&body).to_string())
    }

    async fn get(&self, uri: &str, cookie: Option<&str>) -> (StatusCode, HeaderMap, String) {
        let mut b = Request::builder().uri(uri);
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        self.send(b.body(Body::empty()).unwrap()).await
    }

    async fn post(
        &self,
        uri: &str,
        cookie: Option<&str>,
        form: &str,
        origin: bool,
    ) -> (StatusCode, HeaderMap, String) {
        let mut b = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        if origin {
            b = b.header(header::ORIGIN, BASE);
        }
        self.send(b.body(Body::from(form.to_string())).unwrap())
            .await
    }

    /// Simulate Caddy's forward_auth hop for `app` with the original request URI.
    async fn gate(
        &self,
        app: &str,
        uri: &str,
        cookie: Option<&str>,
        extra: &[(&str, &str)],
    ) -> (StatusCode, HeaderMap, String) {
        let mut b = Request::builder()
            .uri("/gate/verify")
            .header("X-RepoBox-Gate", "1")
            .header("X-RepoBox-Gate-App", app)
            .header("X-Forwarded-Method", "GET")
            .header("X-Forwarded-Uri", uri)
            .header("X-Forwarded-Host", format!("{app}.repo.box"));
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        for (k, v) in extra {
            b = b.header(*k, *v);
        }
        self.send(b.body(Body::empty()).unwrap()).await
    }

    /// Mint a launch code for `user` on `app` through the real launch route.
    async fn mint(&self, user: &User, app: &str) -> String {
        let cookie = self.auth_cookie(user);
        let (st, h, _) = self.get(&format!("/{app}"), Some(&cookie)).await;
        assert_eq!(st, StatusCode::FOUND);
        let loc = h
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(
            loc.starts_with(&format!("https://{app}.repo.box/?token=")),
            "{loc}"
        );
        loc.split("token=").nth(1).unwrap().to_string()
    }
}

fn hdr<'a>(h: &'a HeaderMap, k: &str) -> Option<&'a str> {
    h.get(k).and_then(|v| v.to_str().ok())
}

fn app_cookie_from(h: &HeaderMap) -> String {
    let sc = hdr(h, "set-cookie").expect("set-cookie");
    assert!(sc.starts_with(&format!("{APP_COOKIE}=")), "{sc}");
    assert!(sc.contains("; Path=/"), "{sc}");
    assert!(sc.contains("; Secure"), "{sc}");
    assert!(sc.contains("; HttpOnly"), "{sc}");
    assert!(sc.contains("; SameSite=Lax"), "{sc}");
    assert!(
        !sc.to_lowercase().contains("domain="),
        "host-only cookie must not carry Domain: {sc}"
    );
    sc.split(';').next().unwrap().to_string()
}

#[tokio::test]
async fn launch_code_redeems_into_host_only_session_and_clean_redirect() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;

    // Anonymous hit with the code: gate must set the cookie and redirect to the clean URL.
    let (st, hd, _) = h
        .gate(
            "demo-private",
            &format!("/dash?x=1&token={code}&y=2"),
            None,
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::FOUND);
    assert_eq!(hdr(&hd, "location"), Some("/dash?x=1&y=2"));
    assert_eq!(hdr(&hd, "cache-control"), Some("no-store"));
    let cookie = app_cookie_from(&hd);
    assert!(
        !cookie.contains(&code),
        "session secret must not be the launch code"
    );

    // Follow-up with the cookie: allowed, identity injected.
    let (st, hd, _) = h
        .gate("demo-private", "/dash?x=1&y=2", Some(&cookie), &[])
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "x-repobox-user"), Some("bob"));
    assert_eq!(
        hdr(&hd, "x-repobox-user-id"),
        Some(h.bob.id.to_string().as_str())
    );
    assert_eq!(hdr(&hd, "x-repobox-role"), Some("member"));
    assert_eq!(hdr(&hd, "x-repobox-auth"), Some("session"));
    assert_eq!(hdr(&hd, "x-repobox-app"), Some("demo-private"));
}

#[tokio::test]
async fn launch_code_replay_is_rejected() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    let (st, _, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FOUND);
    let (st, hd, body) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    assert!(hdr(&hd, "set-cookie").is_none());
    assert!(body.contains("already been used"), "{body}");
}

#[tokio::test]
async fn launch_code_expires() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    h.clock
        .fetch_add(h.state.cfg.launch_ttl + 1, Ordering::SeqCst);
    let (st, hd, body) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    assert!(hdr(&hd, "set-cookie").is_none());
    assert!(body.contains("expired"), "{body}");
}

#[tokio::test]
async fn launch_code_is_bound_to_its_app() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    // Using a private-app code on a public app: harmless, code stripped, no cookie.
    let (st, hd, _) = h
        .gate("demo-listed", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FOUND);
    assert_eq!(hdr(&hd, "location"), Some("/"));
    assert!(hdr(&hd, "set-cookie").is_none());
    // The code was consumed by that attempt, so it cannot be used afterwards either.
    let (st, _, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    // A public-app code presented to a private app is refused.
    let code2 = h.mint(&h.owner, "demo-listed").await;
    let (st, hd, _) = h
        .gate("demo-private", &format!("/?token={code2}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    assert!(hdr(&hd, "set-cookie").is_none());
}

#[tokio::test]
async fn revoking_a_grant_kills_live_sessions_and_launch() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    let cookie = app_cookie_from(&hd);
    assert_eq!(
        h.gate("demo-private", "/", Some(&cookie), &[]).await.0,
        StatusCode::OK
    );
    let app = h.state.store.app_by_name("demo-private").unwrap().unwrap();
    h.state.store.remove_grant(app.id, h.bob.id).unwrap();
    assert_eq!(
        h.gate("demo-private", "/", Some(&cookie), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    let auth = h.auth_cookie(&h.bob);
    assert_eq!(
        h.get("/demo-private", Some(&auth)).await.0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn disabled_app_is_denied_for_everyone() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    let cookie = app_cookie_from(&hd);
    for name in ["demo-private", "demo-listed", "demo-unlisted"] {
        let app = h.state.store.app_by_name(name).unwrap().unwrap();
        h.state.store.set_app_enabled(app.id, false).unwrap();
    }
    assert_eq!(
        h.gate("demo-private", "/", Some(&cookie), &[]).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        h.gate("demo-listed", "/", None, &[]).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        h.gate("demo-unlisted", "/", None, &[]).await.0,
        StatusCode::NOT_FOUND
    );
    // A fresh code cannot be redeemed on a disabled app either.
    let auth = h.auth_cookie(&h.bob);
    assert_eq!(
        h.get("/demo-private", Some(&auth)).await.0,
        StatusCode::NOT_FOUND
    );
    let (st, hd, _) = h
        .gate(
            "demo-private",
            "/?token=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            None,
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert!(hdr(&hd, "set-cookie").is_none());
    // Disabled apps drop out of the public directory.
    let (_, _, body) = h.get("/api/directory", None).await;
    assert!(!body.contains("demo-listed"));
}

#[tokio::test]
async fn disabled_user_is_denied_everywhere() {
    let h = H::new();
    let auth = h.auth_cookie(&h.bob);
    let code = h.mint(&h.bob, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    let cookie = app_cookie_from(&hd);
    let code2 = h.mint(&h.bob, "demo-private").await;
    h.state.store.set_user_enabled(h.bob.id, false).unwrap();
    assert_eq!(
        h.gate("demo-private", "/", Some(&cookie), &[]).await.0,
        StatusCode::UNAUTHORIZED,
        "live app session"
    );
    assert_eq!(
        h.gate("demo-private", &format!("/?token={code2}"), None, &[])
            .await
            .0,
        StatusCode::FORBIDDEN,
        "unredeemed code"
    );
    assert_eq!(
        h.get("/demo-private", Some(&auth)).await.0,
        StatusCode::UNAUTHORIZED,
        "auth session is dead"
    );
    assert_eq!(h.get("/me", Some(&auth)).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn visibility_modes_at_the_gate_and_in_the_directory() {
    let h = H::new();
    // Private: anonymous denied with a sign-in link, never 200.
    let (st, hd, body) = h.gate("demo-private", "/secret", None, &[]).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(hdr(&hd, "x-repobox-user").is_none());
    assert!(
        body.contains("https://auth.repo.box/demo-private?next=/secret"),
        "{body}"
    );
    // Public unlisted and listed: 200, no identity, marked public.
    for name in ["demo-unlisted", "demo-listed"] {
        let (st, hd, _) = h.gate(name, "/", None, &[]).await;
        assert_eq!(st, StatusCode::OK, "{name}");
        assert_eq!(hdr(&hd, "x-repobox-auth"), Some("public"));
        assert_eq!(hdr(&hd, "x-repobox-app"), Some(name));
        assert!(hdr(&hd, "x-repobox-user").is_none());
        assert!(hdr(&hd, "x-repobox-role").is_none());
    }
    // Directory, anonymous: only listed.
    let (st, _, body) = h.get("/", None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.contains("demo-listed.repo.box"));
    assert!(!body.contains("demo-unlisted"));
    assert!(!body.contains("demo-private"));
    let (_, _, json) = h.get("/api/directory", None).await;
    assert!(json.contains("\"demo-listed\""));
    assert!(!json.contains("demo-unlisted"));
    assert!(!json.contains("demo-private"));
    // Directory, bob (granted): private visible, unlisted still absent.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.bob))).await;
    assert!(body.contains("demo-private.repo.box"));
    assert!(body.contains("demo-listed.repo.box"));
    assert!(!body.contains("demo-unlisted"));
    assert!(
        !body.contains("other-private"),
        "no grant, must not be listed"
    );
    // Directory, eve (no grants): no private apps.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.eve))).await;
    assert!(!body.contains("demo-private"));
    assert!(body.contains("demo-listed.repo.box"));
    // Owner sees their unlisted app, because it is in their access list.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.owner))).await;
    assert!(body.contains("id=\"your-apps\""));
    assert!(body.contains("demo-unlisted.repo.box"));
}

/// Split a directory page into its "Your apps" and "Other apps" sections.
fn sections(body: &str) -> (Option<&str>, Option<&str>) {
    let y = body.find("id=\"your-apps\"");
    let o = body.find("id=\"other-apps\"");
    let end = body.find("</main>").unwrap_or(body.len());
    let yours = y.map(|i| &body[i..o.unwrap_or(end)]);
    let others = o.map(|i| &body[i..end]);
    (yours, others)
}

#[tokio::test]
async fn directory_puts_your_apps_before_other_apps() {
    let h = H::new();
    // bob: one grant. Private app under "Your apps", listed public under "Other apps".
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.bob))).await;
    let (yours, others) = sections(&body);
    let (yours, others) = (yours.expect("your apps"), others.expect("other apps"));
    assert!(body.find("id=\"your-apps\"") < body.find("id=\"other-apps\""));
    assert!(yours.contains("demo-private.repo.box"));
    assert!(
        !yours.contains("demo-listed.repo.box"),
        "not in bob's access list"
    );
    assert!(others.contains("demo-listed.repo.box"));
    assert!(!others.contains("demo-private"));
    assert!(!body.contains("other-private"), "no grant");
    assert!(!body.contains("demo-unlisted"));
    assert!(!body.contains("Apps you manage"), "old framing removed");
    assert!(!body.contains("Manage</a>"), "bob manages nothing");
    // eve: nothing granted. Empty "Your apps" state, public apps below.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.eve))).await;
    let (yours, others) = sections(&body);
    assert!(yours.unwrap().contains("Nothing yet"));
    assert!(others.unwrap().contains("demo-listed.repo.box"));
    // owner: every app they own is theirs (all visibilities), so no "Other apps".
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.owner))).await;
    let (yours, others) = sections(&body);
    let yours = yours.unwrap();
    assert!(
        others.is_none(),
        "everything listed is already in the access list"
    );
    for host in [
        "demo-private.repo.box",
        "demo-unlisted.repo.box",
        "demo-listed.repo.box",
        "other-private.repo.box",
    ] {
        assert!(yours.contains(host), "{host}");
    }
    assert_eq!(yours.matches("Manage</a>").count(), 4);
    // admin: access to everything, told so.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.fran))).await;
    let (yours, others) = sections(&body);
    assert!(others.is_none());
    assert!(yours.unwrap().contains("platform admin"));
    // Anonymous: a public directory, no personal sections.
    let (_, _, body) = h.get("/", None).await;
    assert!(body.contains("<h1>Public directory</h1>"));
    assert!(body.contains("id=\"public-apps\""));
    let (yours, others) = sections(&body);
    assert!(yours.is_none() && others.is_none());
    assert!(body.contains("demo-listed.repo.box"));
    assert!(!body.contains("demo-private"));
    // A disabled listed app drops out of "Other apps" but stays in its owner's list.
    let listed = h.state.store.app_by_name("demo-listed").unwrap().unwrap();
    h.state.store.set_app_enabled(listed.id, false).unwrap();
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.eve))).await;
    let (_, others) = sections(&body);
    assert!(others.is_none());
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.owner))).await;
    let (yours, _) = sections(&body);
    assert!(yours.unwrap().contains("demo-listed.repo.box"));
    assert!(body.contains("disabled</span>"));
}

#[tokio::test]
async fn public_unlisted_stays_absent_unless_in_the_access_list() {
    let h = H::new();
    for cookie in [
        None,
        Some(h.auth_cookie(&h.eve)),
        Some(h.auth_cookie(&h.bob)),
    ] {
        let (_, _, body) = h.get("/", cookie.as_deref()).await;
        assert!(!body.contains("demo-unlisted"), "{body}");
    }
    let (_, _, json) = h.get("/api/directory", None).await;
    assert!(!json.contains("demo-unlisted"));
    // Owner and admin hold it in their access list.
    for u in [&h.owner, &h.fran] {
        let (_, _, body) = h.get("/", Some(&h.auth_cookie(u))).await;
        let (yours, _) = sections(&body);
        assert!(yours.unwrap().contains("demo-unlisted.repo.box"));
    }
    // A grant puts it in eve's "Your apps" (and nowhere else).
    let unlisted = h.state.store.app_by_name("demo-unlisted").unwrap().unwrap();
    h.state
        .store
        .add_grant(unlisted.id, h.eve.id, None)
        .unwrap();
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.eve))).await;
    let (yours, others) = sections(&body);
    assert!(yours.unwrap().contains("demo-unlisted.repo.box"));
    assert!(!others.unwrap().contains("demo-unlisted"));
    let (_, _, json) = h.get("/api/directory", None).await;
    assert!(!json.contains("demo-unlisted"), "still not public");
}

#[tokio::test]
async fn access_is_counted_only_when_the_gate_allows() {
    let h = H::new();
    let stats = |name: &str| {
        let app = h.state.store.app_by_name(name).unwrap().unwrap();
        h.state.store.app_analytics(app.id, 2).unwrap()
    };
    // Denials: anonymous, spoofed, garbage code, replayed code, revoked grant.
    assert_eq!(
        h.gate("demo-private", "/", None, &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.gate("demo-private", "/", None, &[("X-RepoBox-User", "bob")])
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.gate(
            "demo-private",
            "/?token=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            None,
            &[]
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(stats("demo-private").total_requests, 0);
    // Redemption itself is a redirect, not a served request: not counted.
    let code = h.mint(&h.bob, "demo-private").await;
    let (st, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FOUND);
    let bob_cookie = app_cookie_from(&hd);
    assert_eq!(
        h.gate("demo-private", &format!("/?token={code}"), None, &[])
            .await
            .0,
        StatusCode::FORBIDDEN,
        "replay"
    );
    assert_eq!(stats("demo-private").total_requests, 0);
    // Allowed requests count, one per request, and bob is one user however often he comes.
    for uri in ["/", "/assets/app.css", "/api/x"] {
        assert_eq!(
            h.gate("demo-private", uri, Some(&bob_cookie), &[]).await.0,
            StatusCode::OK
        );
    }
    let a = stats("demo-private");
    assert_eq!(a.total_requests, 3);
    assert_eq!(a.window_requests, 3);
    assert_eq!(a.window_users, 1);
    assert_eq!(a.recent[0].requests, 3);
    assert_eq!(a.recent[0].users, 1);
    // A second signed-in user is a second unique.
    let code = h.mint(&h.owner, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    let owner_cookie = app_cookie_from(&hd);
    h.gate("demo-private", "/", Some(&owner_cookie), &[]).await;
    let a = stats("demo-private");
    assert_eq!((a.total_requests, a.window_users), (4, 2));
    // Revoked grant and disabled app: denied, counters frozen.
    let app = h.state.store.app_by_name("demo-private").unwrap().unwrap();
    h.state.store.remove_grant(app.id, h.bob.id).unwrap();
    assert_eq!(
        h.gate("demo-private", "/", Some(&bob_cookie), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    h.state.store.set_app_enabled(app.id, false).unwrap();
    assert_eq!(
        h.gate("demo-private", "/", Some(&owner_cookie), &[])
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    // A code minted before the app was switched off is refused at the edge too.
    let (code, _) = h
        .state
        .store
        .create_token(
            repobox_platform::store::TokenKind::Launch,
            Some(h.owner.id),
            Some(app.id),
            Some(h.owner.id),
            90,
            "",
        )
        .unwrap();
    let (st, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert!(hdr(&hd, "set-cookie").is_none());
    assert_eq!(stats("demo-private").total_requests, 4);
    // Disabled user: denied, not counted.
    h.state.store.set_app_enabled(app.id, true).unwrap();
    h.state.store.set_user_enabled(h.owner.id, false).unwrap();
    assert_eq!(
        h.gate("demo-private", "/", Some(&owner_cookie), &[])
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(stats("demo-private").total_requests, 4);
    h.state.store.set_user_enabled(h.owner.id, true).unwrap();

    // Public apps: every allowed request counts, but nobody is identified,
    // not even a visitor who launched with a session.
    assert_eq!(stats("demo-listed").total_requests, 0);
    assert_eq!(
        h.gate("demo-listed", "/", None, &[]).await.0,
        StatusCode::OK
    );
    let code = h.mint(&h.fran, "demo-listed").await;
    let (st, hd, _) = h
        .gate("demo-listed", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FOUND);
    let fran_cookie = app_cookie_from(&hd);
    let (st, hd, _) = h.gate("demo-listed", "/", Some(&fran_cookie), &[]).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        hdr(&hd, "x-repobox-user"),
        Some("fran"),
        "identity still injected"
    );
    let a = stats("demo-listed");
    assert_eq!(a.total_requests, 2);
    assert_eq!(a.window_users, 0, "public app keeps no identity");
    assert_eq!(a.recent[0].users, 0);
    // A stray code on a public app is a harmless redirect and not a count.
    let (st, _, _) = h
        .gate("demo-listed", &format!("/?token={code}"), None, &[])
        .await;
    assert_eq!(st, StatusCode::FOUND);
    assert_eq!(stats("demo-listed").total_requests, 2);
    // Unlisted is counted like any public app.
    h.gate("demo-unlisted", "/", None, &[]).await;
    assert_eq!(stats("demo-unlisted").total_requests, 1);
    assert_eq!(stats("other-private").total_requests, 0, "untouched app");
}

#[tokio::test]
async fn analytics_page_is_owner_or_admin_only() {
    let h = H::new();
    let bobs = h
        .state
        .store
        .create_app(
            "bobs-app",
            "Bob's app",
            "",
            h.bob.id,
            AppKind::Proxy,
            "127.0.0.1:3298",
            Visibility::Private,
        )
        .unwrap();
    h.state
        .store
        .record_access(bobs.id, Some(h.bob.id))
        .unwrap();
    let app = h.state.store.app_by_name("demo-private").unwrap().unwrap();
    for _ in 0..5 {
        h.state.store.record_access(app.id, Some(h.bob.id)).unwrap();
    }
    h.state.store.record_access(app.id, Some(h.eve.id)).unwrap();

    assert_eq!(
        h.get("/apps/demo-private/analytics", None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.get("/apps/demo-private/analytics", Some(&h.auth_cookie(&h.eve)))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/apps/demo-private/analytics", Some(&h.auth_cookie(&h.bob)))
            .await
            .0,
        StatusCode::FORBIDDEN,
        "a grant is not ownership"
    );
    assert_eq!(
        h.get("/apps/bobs-app/analytics", Some(&h.auth_cookie(&h.owner)))
            .await
            .0,
        StatusCode::FORBIDDEN,
        "another owner"
    );
    assert_eq!(
        h.get("/apps/nope/analytics", Some(&h.auth_cookie(&h.fran)))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    // Owner and admin see the numbers; the page never mentions other apps.
    for u in [&h.owner, &h.fran] {
        let (st, _, body) = h
            .get("/apps/demo-private/analytics", Some(&h.auth_cookie(u)))
            .await;
        assert_eq!(st, StatusCode::OK);
        assert!(body.contains("<div class=\"value\">6</div>"), "{body}");
        assert!(
            body.contains("<div class=\"value\">2</div>"),
            "two unique users"
        );
        assert!(!body.contains("bobs-app"));
        assert!(!body.contains("Bob&#39;s app"));
        assert!(body.contains("Daily rows are deleted after 90 days"));
    }
    let (st, _, body) = h
        .get("/apps/bobs-app/analytics", Some(&h.auth_cookie(&h.bob)))
        .await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.contains("<div class=\"value\">1</div>"));
    // Public apps say plainly that visitors are not identified.
    let (st, _, body) = h
        .get(
            "/apps/demo-listed/analytics",
            Some(&h.auth_cookie(&h.owner)),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.contains("not kept for public apps"));
    assert!(body.contains("visitors are not identified"));
    // The manage page links to it.
    let (_, _, body) = h
        .get("/apps/demo-private", Some(&h.auth_cookie(&h.owner)))
        .await;
    assert!(body.contains("href=\"/apps/demo-private/analytics\""));
}

#[tokio::test]
async fn spoofed_identity_headers_never_reach_the_decision() {
    let h = H::new();
    let spoof = [
        ("X-RepoBox-User", "mallory"),
        ("X-RepoBox-Role", "admin"),
        ("X-RepoBox-Auth", "session"),
    ];
    // Anonymous + spoof on private: still denied, and nothing echoed back.
    let (st, hd, _) = h.gate("demo-private", "/", None, &spoof).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(hdr(&hd, "x-repobox-user").is_none());
    // Real session + spoof: gate answers with the real identity.
    let code = h.mint(&h.bob, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &spoof)
        .await;
    let cookie = app_cookie_from(&hd);
    let (st, hd, _) = h.gate("demo-private", "/", Some(&cookie), &spoof).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "x-repobox-user"), Some("bob"));
    assert_eq!(hdr(&hd, "x-repobox-role"), Some("member"));
    // Public app + spoof: no identity headers at all.
    let (st, hd, _) = h.gate("demo-listed", "/", None, &spoof).await;
    assert_eq!(st, StatusCode::OK);
    assert!(hdr(&hd, "x-repobox-user").is_none());
    assert_eq!(hdr(&hd, "x-repobox-auth"), Some("public"));
    // Without Caddy's marker the gate endpoint does not exist.
    let (st, _, _) = h.get("/gate/verify", None).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    // A browser-supplied app name is only accepted from the route's header_up.
    let mut b = Request::builder()
        .uri("/gate/verify")
        .header("X-RepoBox-Gate", "1")
        .header("X-RepoBox-Gate-App", "../etc");
    b = b.header("X-Forwarded-Uri", "/");
    let (st, _, _) = h.send(b.body(Body::empty()).unwrap()).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn app_session_is_bound_to_one_app() {
    let h = H::new();
    let code = h.mint(&h.bob, "demo-private").await;
    let (_, hd, _) = h
        .gate("demo-private", &format!("/?token={code}"), None, &[])
        .await;
    let cookie = app_cookie_from(&hd);
    let app = h.state.store.app_by_name("other-private").unwrap().unwrap();
    h.state.store.add_grant(app.id, h.bob.id, None).unwrap();
    // Same secret presented on another private host is anonymous there.
    assert_eq!(
        h.gate("other-private", "/", Some(&cookie), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    // ...and on a public host it yields no identity.
    let (st, hd, _) = h.gate("demo-listed", "/", Some(&cookie), &[]).await;
    assert_eq!(st, StatusCode::OK);
    assert!(hdr(&hd, "x-repobox-user").is_none());
    // The auth-site cookie is never an app session.
    let auth = h.auth_cookie(&h.bob).replace(AUTH_COOKIE, APP_COOKIE);
    assert_eq!(
        h.gate("demo-private", "/", Some(&auth), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn launch_requires_sign_in_and_access() {
    let h = H::new();
    let (st, hd, _) = h.get("/demo-private", None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(hdr(&hd, "location").is_none());
    assert_eq!(
        h.get("/demo-private", Some(&h.auth_cookie(&h.eve))).await.0,
        StatusCode::FORBIDDEN
    );
    // Admin and owner can always launch.
    h.mint(&h.fran, "demo-private").await;
    h.mint(&h.owner, "demo-private").await;
    // Public app, anonymous: plain redirect, no code.
    let (st, hd, _) = h.get("/demo-listed?next=/page", None).await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert_eq!(
        hdr(&hd, "location"),
        Some("https://demo-listed.repo.box/page")
    );
    // `next` cannot become an open redirect.
    let (_, hd, _) = h.get("/demo-listed?next=//evil.com", None).await;
    assert_eq!(hdr(&hd, "location"), Some("https://demo-listed.repo.box/"));
    let (_, hd, _) = h.get("/demo-listed?next=https://evil.com", None).await;
    assert_eq!(hdr(&hd, "location"), Some("https://demo-listed.repo.box/"));
    // Unknown app.
    assert_eq!(h.get("/nope", None).await.0, StatusCode::NOT_FOUND);
}

fn extract_link(body: &str, prefix: &str) -> String {
    let i = body
        .find(prefix)
        .unwrap_or_else(|| panic!("no {prefix} in page"));
    body[i + prefix.len()..i + prefix.len() + 43].to_string()
}

#[tokio::test]
async fn device_enrolment_link_is_single_use_and_post_only() {
    let h = H::new();
    let admin = h.auth_cookie(&h.fran);
    // Non-admin cannot mint links for others.
    assert_eq!(
        h.post(
            "/admin/users/bob/enrol",
            Some(&h.auth_cookie(&h.bob)),
            "",
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    // CSRF: no Origin -> bounced.
    let (st, hd, _) = h
        .post("/admin/users/bob/enrol", Some(&admin), "", false)
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "location").unwrap().contains("err=csrf"));
    let (st, hd, body) = h
        .post("/admin/users/bob/enrol", Some(&admin), "", true)
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "cache-control"), Some("no-store"));
    let raw = extract_link(&body, "https://auth.repo.box/enrol/");
    // GET shows a confirmation page and does not consume (link-preview bots must not burn it).
    let (st, hd, page) = h.get(&format!("/enrol/{raw}"), None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(hdr(&hd, "set-cookie").is_none());
    assert!(page.contains("Continue as Bob"));
    let (st, _, _) = h.get(&format!("/enrol/{raw}"), None).await;
    assert_eq!(st, StatusCode::OK, "still valid after GET");
    // POST redeems: cookie is host-only, the device is signed in as bob.
    let (st, hd, _) = h.post(&format!("/enrol/{raw}"), None, "", true).await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    let sc = hdr(&hd, "set-cookie").unwrap();
    assert!(sc.starts_with(&format!("{AUTH_COOKIE}=")));
    assert!(!sc.to_lowercase().contains("domain="));
    let cookie = sc.split(';').next().unwrap().to_string();
    let (st, _, me) = h.get("/me", Some(&cookie)).await;
    assert_eq!(st, StatusCode::OK);
    assert!(me.contains("<code>bob</code>"));
    // Second use fails.
    assert_eq!(
        h.post(&format!("/enrol/{raw}"), None, "", true).await.0,
        StatusCode::GONE
    );
    assert_eq!(
        h.get(&format!("/enrol/{raw}"), None).await.0,
        StatusCode::GONE
    );
    // Garbage is rejected before any lookup.
    assert_eq!(h.get("/enrol/not-a-token", None).await.0, StatusCode::GONE);
    // Sign out kills the session.
    let (st, hd, _) = h.post("/logout", Some(&cookie), "", true).await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "set-cookie").unwrap().contains("Max-Age=0"));
    assert_eq!(
        h.get("/me", Some(&cookie)).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn owner_invite_creates_user_and_grant_once() {
    let h = H::new();
    let owner = h.auth_cookie(&h.owner);
    // bob cannot create invites for an app he does not own.
    assert_eq!(
        h.post(
            "/apps/demo-private/invites",
            Some(&h.auth_cookie(&h.bob)),
            "",
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (st, _, body) = h
        .post("/apps/demo-private/invites", Some(&owner), "", true)
        .await;
    assert_eq!(st, StatusCode::OK);
    let raw = extract_link(&body, "https://auth.repo.box/invite/");
    let (st, _, page) = h.get(&format!("/invite/{raw}"), None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(page.contains("Invitation to Private demo"));
    // Bad handle does not burn the invitation.
    let (st, _, _) = h
        .post(
            &format!("/invite/{raw}"),
            None,
            "name=Bad%20Name&display_name=X",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    // Existing handle is refused too.
    let (st, _, _) = h
        .post(
            &format!("/invite/{raw}"),
            None,
            "name=bob&display_name=Bob",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::CONFLICT);
    // New person joins.
    let (st, hd, _) = h
        .post(
            &format!("/invite/{raw}"),
            None,
            "name=newbie&display_name=New+Person",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    let cookie = hdr(&hd, "set-cookie")
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let newbie = h
        .state
        .store
        .user_by_name("newbie")
        .unwrap()
        .expect("user created");
    assert_eq!(newbie.display_name, "New Person");
    assert_eq!(newbie.role, Role::Member);
    let app = h.state.store.app_by_name("demo-private").unwrap().unwrap();
    assert!(h.state.store.has_grant(app.id, newbie.id).unwrap());
    // ...and can launch straight away.
    let (st, _, _) = h.get("/demo-private", Some(&cookie)).await;
    assert_eq!(st, StatusCode::FOUND);
    // Invitation is spent.
    assert_eq!(
        h.get(&format!("/invite/{raw}"), None).await.0,
        StatusCode::GONE
    );
    assert_eq!(
        h.post(
            &format!("/invite/{raw}"),
            Some(&h.auth_cookie(&h.eve)),
            "",
            true
        )
        .await
        .0,
        StatusCode::GONE
    );
    // A signed-in user accepting an invitation gets a grant on their existing account.
    let (_, _, body) = h
        .post("/apps/demo-private/invites", Some(&owner), "", true)
        .await;
    let raw2 = extract_link(&body, "https://auth.repo.box/invite/");
    let (st, hd, _) = h
        .post(
            &format!("/invite/{raw2}"),
            Some(&h.auth_cookie(&h.eve)),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "set-cookie").is_none(), "existing session kept");
    assert!(h.state.store.has_grant(app.id, h.eve.id).unwrap());
}

#[tokio::test]
async fn management_is_owner_or_admin_only_and_csrf_protected() {
    let h = H::new();
    assert_eq!(
        h.get("/apps/demo-private", Some(&h.auth_cookie(&h.bob)))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/apps/demo-private", Some(&h.auth_cookie(&h.owner)))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        h.get("/apps/demo-private", Some(&h.auth_cookie(&h.fran)))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        h.get("/apps/demo-private", None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.get("/admin/users", Some(&h.auth_cookie(&h.bob))).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/admin/users", Some(&h.auth_cookie(&h.fran))).await.0,
        StatusCode::OK
    );
    let owner = h.auth_cookie(&h.owner);
    // CSRF: cross-site POST cannot change visibility.
    let (st, hd, _) = h
        .post(
            "/apps/demo-private/visibility",
            Some(&owner),
            "visibility=public_listed",
            false,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "location").unwrap().contains("err=csrf"));
    assert_eq!(
        h.state
            .store
            .app_by_name("demo-private")
            .unwrap()
            .unwrap()
            .visibility,
        Visibility::Private
    );
    // Same-origin POST works and takes effect at the gate immediately.
    let (st, _, _) = h
        .post(
            "/apps/demo-private/visibility",
            Some(&owner),
            "visibility=public_listed",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert_eq!(
        h.gate("demo-private", "/", None, &[]).await.0,
        StatusCode::OK
    );
    let (_, _, json) = h.get("/api/directory", None).await;
    assert!(json.contains("demo-private"));
    // Back to private: anonymous denied again.
    h.post(
        "/apps/demo-private/visibility",
        Some(&owner),
        "visibility=private",
        true,
    )
    .await;
    assert_eq!(
        h.gate("demo-private", "/", None, &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    // Grant management via UI.
    let (st, hd, _) = h
        .post("/apps/demo-private/grants", Some(&owner), "user=eve", true)
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "location").unwrap().contains("ok=grant_added"));
    h.mint(&h.eve, "demo-private").await;
    let (st, _, _) = h
        .post(
            "/apps/demo-private/grants/eve/revoke",
            Some(&owner),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert_eq!(
        h.get("/demo-private", Some(&h.auth_cookie(&h.eve))).await.0,
        StatusCode::FORBIDDEN
    );
    // Disable/enable via UI.
    h.post(
        "/apps/demo-private/enabled",
        Some(&owner),
        "enabled=0",
        true,
    )
    .await;
    assert_eq!(
        h.gate("demo-private", "/", None, &[]).await.0,
        StatusCode::NOT_FOUND
    );
    h.post(
        "/apps/demo-private/enabled",
        Some(&owner),
        "enabled=1",
        true,
    )
    .await;
    assert_eq!(
        h.gate("demo-private", "/", None, &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn own_sessions_are_listed_and_revoking_a_device_ends_its_app_sessions() {
    let h = H::new();
    let (dev_a, id_a) = h.device(&h.bob, "laptop");
    let (dev_b, id_b) = h.device(&h.bob, "phone");
    let app_a = h.launch_from(&dev_a, "demo-private").await;
    let app_b = h.launch_from(&dev_b, "demo-private").await;
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a), &[]).await.0,
        StatusCode::OK
    );
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_b), &[]).await.0,
        StatusCode::OK
    );
    // The account page lists both devices and both app sessions, marks this device.
    let (st, _, me) = h.get("/me", Some(&dev_a)).await;
    assert_eq!(st, StatusCode::OK);
    assert!(me.contains("id=\"devices\"") && me.contains("id=\"app-sessions\""));
    assert_eq!(
        me.matches("this device</span>").count(),
        2,
        "device row + its app session"
    );
    assert!(me.contains("laptop") && me.contains("phone"));
    assert!(me.contains("Private demo"));
    assert!(me.contains(&format!("action=\"/me/sessions/{id_b}/revoke\"")));
    let app_sessions = h
        .state
        .store
        .list_sessions(h.bob.id, SessionKind::App)
        .unwrap();
    assert_eq!(app_sessions.len(), 2);
    let app_a_id = app_sessions
        .iter()
        .find(|x| x.parent_id == Some(id_a))
        .unwrap()
        .id;
    let app_b_id = app_sessions
        .iter()
        .find(|x| x.parent_id == Some(id_b))
        .unwrap()
        .id;
    assert!(me.contains(&format!("action=\"/me/sessions/{app_b_id}/revoke\"")));
    // Sign the phone out from the laptop: phone and its app session die at once.
    let (st, hd, _) = h
        .post(
            &format!("/me/sessions/{id_b}/revoke"),
            Some(&dev_a),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "location").unwrap().contains("ok=session_revoked"));
    assert_eq!(h.get("/me", Some(&dev_b)).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_b), &[]).await.0,
        StatusCode::UNAUTHORIZED,
        "app session launched from the phone is gone at the gate"
    );
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a), &[]).await.0,
        StatusCode::OK
    );
    let (_, _, me) = h.get("/me", Some(&dev_a)).await;
    assert!(!me.contains("phone"));
    assert!(!me.contains(&format!("/me/sessions/{app_b_id}/revoke")));
    // End the laptop's app session only: the device stays signed in.
    let (st, hd, _) = h
        .post(
            &format!("/me/sessions/{app_a_id}/revoke"),
            Some(&dev_a),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(
        hdr(&hd, "location")
            .unwrap()
            .contains("ok=app_session_revoked")
    );
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(h.get("/me", Some(&dev_a)).await.0, StatusCode::OK);
    // Relaunching works and produces a fresh app session tied to the laptop.
    let app_a2 = h.launch_from(&dev_a, "demo-private").await;
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a2), &[]).await.0,
        StatusCode::OK
    );
    // Signing out here (logout) ends that app session too.
    let (st, _, _) = h.post("/logout", Some(&dev_a), "", true).await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a2), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn sessions_of_other_users_cannot_be_revoked_by_members() {
    let h = H::new();
    let (owner_dev, owner_id) = h.device(&h.owner, "owner-laptop");
    let owner_app = h.launch_from(&owner_dev, "demo-private").await;
    let (bob_dev, _) = h.device(&h.bob, "bob-laptop");
    // Right id, wrong user: refused, nothing changes.
    let (st, hd, _) = h
        .post(
            &format!("/me/sessions/{owner_id}/revoke"),
            Some(&bob_dev),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(hdr(&hd, "location").unwrap().contains("err=not_found"));
    assert_eq!(h.get("/me", Some(&owner_dev)).await.0, StatusCode::OK);
    assert_eq!(
        h.gate("demo-private", "/", Some(&owner_app), &[]).await.0,
        StatusCode::OK
    );
    // Cross-site POST is bounced even for one's own session.
    let (_, hd, _) = h
        .post(
            &format!("/me/sessions/{owner_id}/revoke"),
            Some(&owner_dev),
            "",
            false,
        )
        .await;
    assert!(hdr(&hd, "location").unwrap().contains("err=csrf"));
    assert_eq!(h.get("/me", Some(&owner_dev)).await.0, StatusCode::OK);
    // The admin page is admins only.
    assert_eq!(
        h.get("/admin/users/bob/sessions", Some(&h.auth_cookie(&h.eve)))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        h.get("/admin/users/bob/sessions", None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.post(
            "/admin/users/bob/sessions/revoke-all",
            Some(&bob_dev),
            "",
            true
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn admin_can_revoke_any_users_sessions() {
    let h = H::new();
    let fran = h.auth_cookie(&h.fran);
    let (dev_a, id_a) = h.device(&h.bob, "bob-laptop");
    let (dev_b, id_b) = h.device(&h.bob, "bob-phone");
    let app_a = h.launch_from(&dev_a, "demo-private").await;
    let app_b = h.launch_from(&dev_b, "demo-private").await;
    let (owner_dev, owner_id) = h.device(&h.owner, "owner-laptop");
    let app_b_id = h
        .state
        .store
        .list_sessions(h.bob.id, SessionKind::App)
        .unwrap()
        .into_iter()
        .find(|x| x.parent_id == Some(id_b))
        .unwrap()
        .id;
    // Users list links to the per-user page; the page lists everything.
    let (_, _, users) = h.get("/admin/users", Some(&fran)).await;
    assert!(users.contains("href=\"/admin/users/bob/sessions\""));
    let (st, _, page) = h.get("/admin/users/bob/sessions", Some(&fran)).await;
    assert_eq!(st, StatusCode::OK);
    assert!(page.contains("Sign out everywhere"));
    assert!(page.contains("bob-laptop") && page.contains("bob-phone"));
    assert!(page.contains(&format!(
        "action=\"/admin/users/bob/sessions/{id_a}/revoke\""
    )));
    assert!(page.contains(&format!(
        "action=\"/admin/users/bob/sessions/{app_b_id}/revoke\""
    )));
    assert!(!page.contains("this device"));
    assert_eq!(
        h.get("/admin/users/nope/sessions", Some(&fran)).await.0,
        StatusCode::NOT_FOUND
    );
    // Revoke one app session: gate refuses it at once, the device stays.
    let (st, hd, _) = h
        .post(
            &format!("/admin/users/bob/sessions/{app_b_id}/revoke"),
            Some(&fran),
            "",
            true,
        )
        .await;
    assert_eq!(st, StatusCode::SEE_OTHER);
    assert!(
        hdr(&hd, "location")
            .unwrap()
            .contains("ok=app_session_revoked")
    );
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_b), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a), &[]).await.0,
        StatusCode::OK
    );
    assert_eq!(h.get("/me", Some(&dev_b)).await.0, StatusCode::OK);
    // Revoke a device: it and its app session die.
    let (_, hd, _) = h
        .post(
            &format!("/admin/users/bob/sessions/{id_a}/revoke"),
            Some(&fran),
            "",
            true,
        )
        .await;
    assert!(hdr(&hd, "location").unwrap().contains("ok=session_revoked"));
    assert_eq!(h.get("/me", Some(&dev_a)).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        h.gate("demo-private", "/", Some(&app_a), &[]).await.0,
        StatusCode::UNAUTHORIZED
    );
    // A session id of another user under bob's path is a no-op.
    let (_, hd, _) = h
        .post(
            &format!("/admin/users/bob/sessions/{owner_id}/revoke"),
            Some(&fran),
            "",
            true,
        )
        .await;
    assert!(hdr(&hd, "location").unwrap().contains("err=not_found"));
    assert_eq!(h.get("/me", Some(&owner_dev)).await.0, StatusCode::OK);
    // Sign out everywhere: CSRF-guarded, then total.
    let (_, hd, _) = h
        .post(
            "/admin/users/bob/sessions/revoke-all",
            Some(&fran),
            "",
            false,
        )
        .await;
    assert!(hdr(&hd, "location").unwrap().contains("err=csrf"));
    assert_eq!(h.get("/me", Some(&dev_b)).await.0, StatusCode::OK);
    let (_, hd, _) = h
        .post(
            "/admin/users/bob/sessions/revoke-all",
            Some(&fran),
            "",
            true,
        )
        .await;
    assert!(
        hdr(&hd, "location")
            .unwrap()
            .contains("ok=sessions_revoked_all")
    );
    assert_eq!(h.get("/me", Some(&dev_b)).await.0, StatusCode::UNAUTHORIZED);
    assert!(
        h.state
            .store
            .list_sessions(h.bob.id, SessionKind::Auth)
            .unwrap()
            .is_empty()
    );
    assert!(
        h.state
            .store
            .list_sessions(h.bob.id, SessionKind::App)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        h.get("/me", Some(&owner_dev)).await.0,
        StatusCode::OK,
        "other users untouched"
    );
    let audit = h.state.store.list_audit(10).unwrap();
    assert!(
        audit
            .iter()
            .any(|e| e.action == "session.revoke_all" && e.subject == "bob")
    );
    assert!(
        audit
            .iter()
            .any(|e| e.action == "session.revoke" && e.detail == "admin app")
    );
}

#[tokio::test]
async fn pages_render_for_every_role_and_never_leak_secrets() {
    let h = H::new();
    let fran = h.auth_cookie(&h.fran);
    let (_, hd, body) = h.post("/me/enrol-device", Some(&fran), "", true).await;
    assert_eq!(hdr(&hd, "cache-control"), Some("no-store"));
    let raw = extract_link(&body, "https://auth.repo.box/enrol/");
    for path in [
        "/",
        "/me",
        "/admin/users",
        "/admin/audit",
        "/apps/demo-private",
        "/apps/demo-listed",
        "/apps/demo-private/analytics",
        "/apps/demo-listed/analytics",
        "/admin/users/bob/sessions",
    ] {
        let (st, _, page) = h.get(path, Some(&fran)).await;
        assert_eq!(st, StatusCode::OK, "{path}");
        assert!(
            page.contains("<meta name=\"viewport\""),
            "{path} must be responsive"
        );
        assert!(!page.contains(&raw), "{path} leaked a raw token");
        assert!(
            !page.contains(fran.split('=').nth(1).unwrap()),
            "{path} leaked the session secret"
        );
    }
    let (st, hd, css) = h.get("/assets/app.css", None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(hdr(&hd, "content-type").unwrap().starts_with("text/css"));
    assert!(css.contains("@media (max-width:600px)"));
    let (st, _, hz) = h.get("/healthz", None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(hz.starts_with("ok apps="));
}
