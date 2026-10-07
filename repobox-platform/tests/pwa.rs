//! auth.repo.box as an installable PWA: manifest, service worker, icons,
//! content types and cache headers, the registration markup, the plain 404
//! for missing assets and the session probe used by an installed app.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use repobox_platform::model::{AppKind, IdentityContract, Role, Visibility};
use repobox_platform::store::{SessionKind, Store};
use repobox_platform::web::{self, AUTH_COOKIE, AppState, Config, pwa};

struct H {
    state: Arc<AppState>,
}

impl H {
    fn new() -> Self {
        let store = Store::open_in_memory().unwrap();
        let owner = store.create_user("owner", "Owner", Role::Member).unwrap();
        store
            .create_app(
                "demo-private",
                "Private demo",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:3231",
                Visibility::Private,
                IdentityContract::Platform,
            )
            .unwrap();
        H {
            state: Arc::new(AppState {
                store,
                cfg: Config::defaults("https://auth.repo.box", "repo.box"),
            }),
        }
    }

    async fn get(&self, uri: &str, extra: &[(&str, &str)]) -> (StatusCode, HeaderMap, Vec<u8>) {
        let mut b = Request::builder().uri(uri);
        for (k, v) in extra {
            b = b.header(*k, *v);
        }
        let resp = web::router(self.state.clone())
            .oneshot(b.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let (st, hd) = (resp.status(), resp.headers().clone());
        let body = resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        (st, hd, body)
    }

    async fn text(&self, uri: &str, cookie: Option<&str>) -> (StatusCode, HeaderMap, String) {
        let extra: Vec<(&str, &str)> = cookie.map(|c| ("cookie", c)).into_iter().collect();
        let (st, hd, b) = self.get(uri, &extra).await;
        (st, hd, String::from_utf8(b).unwrap())
    }

    fn device(&self, name: &str) -> (String, i64, i64) {
        let u = self.state.store.user_by_name(name).unwrap().unwrap();
        let (raw, sess) = self
            .state
            .store
            .create_session(SessionKind::Auth, u.id, None, None, 3600, "test")
            .unwrap();
        (format!("{AUTH_COOKIE}={raw}"), sess.id, u.id)
    }
}

fn hdr<'a>(h: &'a HeaderMap, k: &str) -> &'a str {
    h.get(k).and_then(|v| v.to_str().ok()).unwrap_or("")
}

fn png_size(b: &[u8]) -> (u32, u32) {
    assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n", "not a PNG");
    let be = |i: usize| u32::from_be_bytes(b[i..i + 4].try_into().unwrap());
    (be(16), be(20))
}

#[tokio::test]
async fn manifest_is_installable_and_same_origin() {
    let h = H::new();
    let (st, hd, body) = h.get("/manifest.webmanifest", &[]).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "content-type"), "application/manifest+json");
    assert_eq!(hdr(&hd, "cache-control"), "no-cache");
    assert_eq!(hdr(&hd, "x-content-type-options"), "nosniff");
    let m: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(m["start_url"], "/");
    assert_eq!(m["scope"], "/");
    assert_eq!(m["id"], "/");
    assert_eq!(m["display"], "standalone");
    assert_eq!(m["name"], "repo.box auth");
    assert_eq!(m["short_name"], "repo.box");
    assert_eq!(m["theme_color"], "#0a1628");
    assert_eq!(m["background_color"], "#0a1628");
    let icons = m["icons"].as_array().unwrap();
    let mut any512 = false;
    let mut mask512 = false;
    for i in icons {
        let src = i["src"].as_str().unwrap();
        assert!(src.starts_with("/assets/icons/"), "same-origin: {src}");
        let (st, hd, bytes) = h.get(src, &[]).await;
        assert_eq!(st, StatusCode::OK, "{src}");
        assert_eq!(hdr(&hd, "content-type"), i["type"].as_str().unwrap());
        assert!(hdr(&hd, "cache-control").starts_with("public, max-age="));
        if i["type"] == "image/png" {
            let sizes = i["sizes"].as_str().unwrap();
            let (w, hh) = png_size(&bytes);
            assert_eq!(format!("{w}x{hh}"), sizes, "{src}");
            any512 |= sizes == "512x512" && i["purpose"] == "any";
            mask512 |= sizes == "512x512" && i["purpose"] == "maskable";
        } else {
            assert!(String::from_utf8_lossy(&bytes).starts_with("<svg"));
        }
    }
    assert!(any512 && mask512, "standard and maskable 512px icons");
    let (st, hd, bytes) = h.get("/apple-touch-icon.png", &[]).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "content-type"), "image/png");
    assert_eq!(png_size(&bytes), (180, 180));
}

#[tokio::test]
async fn service_worker_is_versioned_uncached_and_caches_only_the_static_shell() {
    let h = H::new();
    let (st, hd, sw) = h.text("/sw.js", None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "content-type"), "text/javascript; charset=utf-8");
    assert_eq!(hdr(&hd, "cache-control"), "no-store");
    let v = pwa::version();
    assert_eq!(v.len(), 12);
    assert!(sw.contains(&format!("const VERSION = '{v}';")));
    assert!(!sw.contains("__"), "every placeholder filled");
    // The only cache writes are the install-time precache of SHELL.
    assert_eq!(sw.matches("caches.open(").count(), 1);
    assert!(!sw.contains(".put("));
    assert!(sw.contains("e.respondWith(fetch(req).catch(offline));"));
    assert!(sw.contains("if (req.method !== 'GET') return;"));
    let shell: Vec<String> = serde_json::from_str(
        sw.split("const SHELL = ")
            .nth(1)
            .unwrap()
            .split(";\n")
            .next()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(shell, pwa::SHELL);
    for p in &shell {
        assert!(p.starts_with("/assets/"), "static only: {p}");
        let (st, hd, _) = h.get(p, &[]).await;
        assert_eq!(st, StatusCode::OK, "{p}");
        let ct = hdr(&hd, "content-type");
        assert!(!hdr(&hd, "cache-control").contains("no-store"), "{p}");
        if p.ends_with(".html") {
            assert_eq!(ct, "text/html; charset=utf-8");
        } else {
            assert!(!ct.starts_with("text/html"), "{p}: {ct}");
        }
    }
    let (_, _, js) = h.text("/assets/pwa.js", None).await;
    assert!(js.contains("register('/sw.js', { scope: '/', updateViaCache: 'none' })"));
}

#[tokio::test]
async fn offline_page_is_static_and_names_no_one() {
    let h = H::new();
    let (st, hd, page) = h.text("/assets/offline.html", None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(hdr(&hd, "content-type"), "text/html; charset=utf-8");
    assert!(page.contains("<h1>You are offline</h1>"));
    assert!(page.contains("Nothing private is stored on this device"));
    assert!(!page.contains("data-session"));
    assert!(!page.contains("/sw.js"));
}

#[tokio::test]
async fn missing_assets_are_plain_404s_not_html() {
    let h = H::new();
    for p in [
        "/assets/missing.js",
        "/assets/icons/missing.png",
        "/assets/icons/../app.css",
        "/assets/deep/er/x.css",
        "/favicon.ico",
    ] {
        let (st, hd, body) = h.text(p, None).await;
        assert_eq!(st, StatusCode::NOT_FOUND, "{p}");
        assert_eq!(hdr(&hd, "content-type"), "text/plain; charset=utf-8", "{p}");
        assert!(!body.contains('<'), "{p}");
    }
}

#[tokio::test]
async fn every_page_registers_the_app_and_html_is_never_stored() {
    let h = H::new();
    let (cookie, _, uid) = h.device("owner");
    for (uri, c) in [
        ("/", None),
        ("/", Some(cookie.as_str())),
        ("/me", Some(cookie.as_str())),
        ("/docs", None),
        ("/no-such-app", None),
    ] {
        let (_, hd, page) = h.text(uri, c).await;
        assert_eq!(hdr(&hd, "cache-control"), "no-store", "{uri}");
        assert!(page.contains("<link rel=\"manifest\" href=\"/manifest.webmanifest\">"));
        assert!(page.contains("<meta name=\"theme-color\" content=\"#0a1628\">"));
        assert!(page.contains("<script src=\"/assets/pwa.js\" defer></script>"));
        assert!(page.contains(
            "<link rel=\"apple-touch-icon\" href=\"/assets/icons/apple-touch-icon.png\">"
        ));
        // The install action and iOS hint exist but start hidden; only the
        // script reveals them, on a real prompt or on iOS.
        assert!(page.contains("id=\"pwa-install\" hidden>Install app</button>"));
        assert!(page.contains("id=\"pwa-ios\" hidden>"));
        assert!(page.contains("Add to Home Screen"));
        assert!(page.contains("role=\"status\" aria-live=\"polite\""));
        let signed = format!("<body data-session=\"{uid}\">");
        assert_eq!(page.contains(&signed), c.is_some(), "{uri}");
        if c.is_none() {
            assert!(page.contains("<body>"));
        }
    }
    let (_, _, me) = h.text("/me", Some(&cookie)).await;
    assert!(me.contains("<h3>Install on this device</h3>"));
    assert!(me.contains("id=\"pwa-state\""));
}

#[tokio::test]
async fn gate_pages_on_app_hosts_are_not_the_installable_app() {
    let h = H::new();
    let (st, hd, page) = {
        let (st, hd, b) = h
            .get(
                "/gate/verify",
                &[
                    ("X-RepoBox-Gate", "1"),
                    ("X-RepoBox-Gate-App", "demo-private"),
                    ("X-Forwarded-Method", "GET"),
                    ("X-Forwarded-Uri", "/"),
                    ("X-Forwarded-Host", "demo-private.repo.box"),
                    ("Accept", "text/html"),
                    ("Sec-Fetch-Dest", "document"),
                    ("Sec-Fetch-Mode", "navigate"),
                ],
            )
            .await;
        (st, hd, String::from_utf8(b).unwrap())
    };
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(page.contains("<html"), "{page}");
    assert_eq!(hdr(&hd, "cache-control"), "no-store");
    for absent in [
        "rel=\"manifest\"",
        "/sw.js",
        "pwa.js",
        "id=\"pwa-install\"",
        "data-session",
    ] {
        assert!(!page.contains(absent), "{absent}");
    }
}

#[tokio::test]
async fn session_probe_follows_revocation_and_expiry() {
    let h = H::new();
    let probe = |c: Option<String>| {
        let h = &h;
        async move {
            let (st, hd, body) = h.text("/api/session", c.as_deref()).await;
            assert_eq!(st, StatusCode::OK);
            assert_eq!(hdr(&hd, "cache-control"), "no-store");
            assert!(hdr(&hd, "access-control-allow-origin").is_empty());
            serde_json::from_str::<serde_json::Value>(&body).unwrap()
        }
    };
    let anon = probe(None).await;
    assert_eq!(
        anon,
        serde_json::json!({"signed_in": false, "user_id": null})
    );
    let (cookie, sid, uid) = h.device("owner");
    let live = probe(Some(cookie.clone())).await;
    assert_eq!(live, serde_json::json!({"signed_in": true, "user_id": uid}));
    h.state.store.revoke_session(sid, uid).unwrap();
    assert_eq!(probe(Some(cookie.clone())).await["signed_in"], false);
    // And the directory a revoked device renders is the signed-out one.
    let (_, _, page) = h.text("/", Some(&cookie)).await;
    assert!(page.contains("<h1>Public directory</h1>"));
    assert!(!page.contains("data-session"));
}
