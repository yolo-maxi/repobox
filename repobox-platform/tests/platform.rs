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
            .create_session(SessionKind::Auth, user.id, None, 86400, "test")
            .unwrap();
        format!("{AUTH_COOKIE}={raw}")
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
    // Owner sees their unlisted app only in the manage section.
    let (_, _, body) = h.get("/", Some(&h.auth_cookie(&h.owner))).await;
    assert!(body.contains("Apps you manage"));
    assert!(body.contains("demo-unlisted.repo.box"));
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
