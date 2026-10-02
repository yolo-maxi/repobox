//! Platform AI capability, end to end in-process: the control plane router
//! (what Caddy forwards `/_repo_box/ai/v1/*` to) → the real broker on a
//! loopback port → a fake ChatMock on another loopback port. Also the
//! discovery documents, the service-token API and the MCP surface.

use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use repobox_platform::ai::{self, Bridge, BrokerConfig, Secret};
use repobox_platform::model::{
    AI_MAX_BODY_BYTES, AI_PUBLIC_POLICY_SIGNED_IN_QUOTA, AiPolicy, App, AppKind, IdentityContract,
    Role, User, Visibility,
};
use repobox_platform::store::{SessionKind, Store};
use repobox_platform::web::{self, APP_COOKIE, AppState, Config};

const BASE: &str = "https://auth.repo.box";
const SECRET: &str = "test-bridge-secret-0123456789abcdefghijklmnop";

/// What the fake ChatMock saw.
#[derive(Default)]
struct Seen {
    calls: AtomicUsize,
    last_body: Mutex<Option<Value>>,
    last_auth: Mutex<Option<String>>,
}

async fn fake_chatmock(seen: Arc<Seen>) -> String {
    use axum::routing::{get, post};
    let s2 = seen.clone();
    let app = Router::new()
        .route(
            "/v1/models",
            get(|| async {
                axum::Json(json!({"object": "list", "data": [
                    {"id": "gpt-5.6-terra"}, {"id": "gpt-5.6-luna"}, {"id": "gpt-5.5"}
                ]}))
            }),
        )
        .route(
            "/v1/chat/completions",
            post(move |headers: HeaderMap, body: String| {
                let seen = s2.clone();
                async move {
                    seen.calls.fetch_add(1, Ordering::SeqCst);
                    let v: Value = serde_json::from_str(&body).unwrap();
                    *seen.last_auth.lock().unwrap() = headers
                        .get(header::AUTHORIZATION)
                        .map(|h| h.to_str().unwrap().to_string());
                    let model = v["model"].as_str().unwrap().to_string();
                    *seen.last_body.lock().unwrap() = Some(v);
                    axum::Json(json!({
                        "id": "resp_abc", "object": "chat.completion", "created": 1, "model": model,
                        "choices": [{"index": 0, "finish_reason": "stop",
                            "message": {"role": "assistant", "content": "Hello from the fake model"}}],
                        "usage": {"prompt_tokens": 5, "completion_tokens": 5, "total_tokens": 10},
                        "provider_internal": "must not leak",
                    }))
                }
            }),
        );
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    format!("http://{addr}")
}

async fn broker(chatmock: &str, models: &[&str]) -> String {
    let b = ai::Broker::new(BrokerConfig {
        upstream: chatmock.into(),
        secret: Secret::new(SECRET.into()).unwrap(),
        models: models.iter().map(|m| m.to_string()).collect(),
        max_concurrency: 4,
        timeout: std::time::Duration::from_secs(10),
    })
    .unwrap();
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(l, ai::broker_router(b)).await.unwrap() });
    format!("http://{addr}")
}

struct H {
    state: Arc<AppState>,
    seen: Arc<Seen>,
    broker: String,
    owner: User,
    bob: User,
    eve: User,
    other: User,
    private: App,
}

impl H {
    async fn new() -> Self {
        let clock = Arc::new(AtomicI64::new(1_800_000_000));
        let c = clock.clone();
        let store =
            Store::open_in_memory_with_clock(Arc::new(move || c.load(Ordering::SeqCst))).unwrap();
        store.create_user("fran", "Fran", Role::Admin).unwrap();
        let owner = store.create_user("owner", "Owner", Role::Member).unwrap();
        let bob = store.create_user("bob", "Bob", Role::Member).unwrap();
        let eve = store.create_user("eve", "Eve", Role::Member).unwrap();
        let other = store.create_user("other", "Other", Role::Member).unwrap();
        let reg = |name: &str, owner: &User, vis: Visibility| {
            store
                .create_app(
                    name,
                    name,
                    "",
                    owner.id,
                    AppKind::Proxy,
                    "127.0.0.1:3231",
                    vis,
                    IdentityContract::Platform,
                )
                .unwrap()
        };
        let private = reg("ai-private", &owner, Visibility::Private);
        let off = reg("ai-off", &owner, Visibility::Private);
        reg("ai-public", &owner, Visibility::PublicListed);
        reg("elsewhere", &other, Visibility::Private);
        store.add_grant(private.id, bob.id, None).unwrap();
        store.add_grant(off.id, bob.id, None).unwrap();
        let mut p = off.ai.clone();
        p.enabled = false;
        store.set_app_ai(off.id, &p).unwrap();

        let seen = Arc::new(Seen::default());
        let chatmock = fake_chatmock(seen.clone()).await;
        let broker = broker(&chatmock, &["gpt-5.6-terra", "gpt-5.6-luna", "gpt-5.6-sol"]).await;
        let mut cfg = Config::defaults(BASE, "repo.box");
        cfg.ai = Some(Arc::new(
            Bridge::new(&broker, Secret::new(SECRET.into()).unwrap(), 4).unwrap(),
        ));
        H {
            state: Arc::new(AppState { store, cfg }),
            seen,
            broker,
            owner,
            bob,
            eve,
            other,
            private,
        }
    }

    fn router(&self) -> Router {
        web::router(self.state.clone())
    }

    fn app(&self, name: &str) -> App {
        self.state.store.app_by_name(name).unwrap().unwrap()
    }

    /// An app session cookie as the gate would have set after a launch.
    fn session(&self, user: &User, app: &str) -> String {
        let a = self.app(app);
        let (raw, _) = self
            .state
            .store
            .create_session(SessionKind::App, user.id, Some(a.id), None, 3600, "test")
            .unwrap();
        format!("{APP_COOKIE}={raw}")
    }

    async fn send(&self, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
        let resp = self.router().oneshot(req).await.unwrap();
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let v = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()));
        (status, headers, v)
    }

    /// Run the real gate for `app` (what Caddy's forward_auth does) and
    /// return the identity headers it issued, as `copy_headers` would.
    async fn gate_identity(
        &self,
        app: &str,
        cookie: Option<&str>,
    ) -> (StatusCode, Vec<(String, String)>) {
        let mut b = Request::builder()
            .uri("/gate/verify")
            .header("X-RepoBox-Gate", "1")
            .header("X-RepoBox-Gate-App", app)
            .header("X-Forwarded-Method", "POST")
            .header("X-Forwarded-Uri", "/_repo_box/ai/v1/chat/completions");
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        let resp = self
            .router()
            .oneshot(b.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let st = resp.status();
        let ids = resp
            .headers()
            .iter()
            .filter(|(k, _)| k.as_str().starts_with("x-repobox-"))
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect();
        (st, ids)
    }

    /// The request Caddy forwards to `/gate/ai/v1/chat/completions` after a
    /// successful gate: marker + app from the route, identity from the gate.
    async fn chat_as(
        &self,
        app: &str,
        cookie: Option<&str>,
        identity: &[(String, String)],
        body: Value,
        extra: &[(&str, &str)],
    ) -> (StatusCode, HeaderMap, Value) {
        let mut b = Request::builder()
            .method("POST")
            .uri("/gate/ai/v1/chat/completions")
            .header("X-RepoBox-Gate", "1")
            .header("X-RepoBox-Gate-App", app)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ORIGIN, format!("https://{app}.repo.box"))
            .header("Sec-Fetch-Site", "same-origin");
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        for (k, v) in identity {
            b = b.header(k.as_str(), v.as_str());
        }
        let mut req = b.body(Body::from(body.to_string())).unwrap();
        for (k, v) in extra {
            let name = axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap();
            if v.is_empty() {
                req.headers_mut().remove(&name);
            } else {
                req.headers_mut().insert(name, v.parse().unwrap());
            }
        }
        self.send(req).await
    }

    /// Full honest path: gate first, then the AI hop with its identity.
    async fn chat(&self, user: &User, app: &str, body: Value) -> (StatusCode, Value) {
        let cookie = self.session(user, app);
        let (st, ids) = self.gate_identity(app, Some(&cookie)).await;
        assert_eq!(st, StatusCode::OK, "gate allows {} on {app}", user.name);
        let (st, _, v) = self.chat_as(app, Some(&cookie), &ids, body, &[]).await;
        (st, v)
    }

    fn calls(&self) -> usize {
        self.seen.calls.load(Ordering::SeqCst)
    }
}

fn hello() -> Value {
    json!({"messages": [{"role": "user", "content": "Say hello"}], "user": "tracking-id", "metadata": {"x": 1}})
}

fn identity(user: &User) -> Vec<(String, String)> {
    vec![
        ("x-repobox-auth".into(), "session".into()),
        ("x-repobox-user-id".into(), user.id.to_string()),
        ("x-repobox-user".into(), user.name.clone()),
        ("x-repobox-app".into(), "ai-private".into()),
    ]
}

fn code(v: &Value) -> &str {
    v["error"]["code"].as_str().unwrap_or("")
}

// ------------------------------------------------------------- registry

#[tokio::test]
async fn registry_defaults_and_public_policy() {
    let h = H::new().await;
    let s = &h.state.store;
    let private = h.app("ai-private");
    assert_eq!(
        private.ai,
        AiPolicy::private_default(),
        "private platform apps get AI by default"
    );
    assert!(private.ai.enabled);
    assert_eq!(private.ai.provider, "chatmock");
    let public = h.app("ai-public");
    assert!(!public.ai.enabled, "public apps start with AI off");
    // Enabling AI on a public app needs the explicit abuse/quota policy.
    let mut p = public.ai.clone();
    p.enabled = true;
    let err = s.set_app_ai(public.id, &p).unwrap_err().to_string();
    assert!(err.contains("abuse/quota policy"), "{err}");
    assert!(!h.app("ai-public").ai.enabled);
    p.public_policy = Some(AI_PUBLIC_POLICY_SIGNED_IN_QUOTA.into());
    p.user_daily_requests = 5;
    p.app_daily_requests = 50;
    assert!(s.set_app_ai(public.id, &p).unwrap().ai.enabled);
    // A private AI app made public without a policy loses AI in the same step.
    assert!(
        s.set_app_visibility(private.id, Visibility::PublicUnlisted)
            .unwrap()
    );
    let now = h.app("ai-private");
    assert_eq!(now.visibility, Visibility::PublicUnlisted);
    assert!(!now.ai.enabled);
    // Beyond platform ceilings is refused.
    let mut too_much = AiPolicy::private_default();
    too_much.max_output_tokens = 1_000_000;
    assert!(s.set_app_ai(h.app("ai-off").id, &too_much).is_err());
}

// ------------------------------------------------------ happy path

#[tokio::test]
async fn authenticated_request_reaches_the_model_through_the_broker() {
    let h = H::new().await;
    let (st, v) = h.chat(&h.bob, "ai-private", hello()).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["object"], "chat.completion");
    assert_eq!(v["model"], "gpt-5.6-terra", "policy default model");
    assert_eq!(
        v["choices"][0]["message"]["content"],
        "Hello from the fake model"
    );
    assert!(!v.to_string().contains("must not leak"));
    assert_eq!(h.calls(), 1);
    // ChatMock received only the rebuilt body and no credential at all.
    let body = h.seen.last_body.lock().unwrap().clone().unwrap();
    assert_eq!(
        body,
        json!({"model": "gpt-5.6-terra", "messages": [{"role": "user", "content": "Say hello"}],
               "max_tokens": 2048, "stream": false})
    );
    assert_eq!(*h.seen.last_auth.lock().unwrap(), None);
    // Usage is counted per app and user (counters only).
    assert_eq!(h.state.store.ai_usage_today(h.private.id).unwrap(), (1, 1));
    // Owner and an explicit allowed model work too.
    let (st, v) = h
        .chat(&h.owner, "ai-private", json!({"model": "gpt-5.6-luna", "messages": [{"role": "user", "content": "hi"}], "max_tokens": 10}))
        .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["model"], "gpt-5.6-luna");
    assert_eq!(h.state.store.ai_usage_today(h.private.id).unwrap(), (2, 2));
}

// ------------------------------------------------------------ auth

#[tokio::test]
async fn unauthenticated_and_forged_requests_are_refused() {
    let h = H::new().await;
    let bob_cookie = h.session(&h.bob, "ai-private");

    // The gate itself turns an anonymous AI call away with JSON.
    let (st, ids) = h.gate_identity("ai-private", None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(ids.iter().all(|(k, _)| k != "x-repobox-user-id"));

    // Forged gate identity without the app session cookie.
    let (st, _, v) = h
        .chat_as("ai-private", None, &identity(&h.bob), hello(), &[])
        .await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::UNAUTHORIZED, "unauthenticated")
    );
    // Cookie present but no gate identity (the AI hop was reached directly).
    let (st, _, _) = h
        .chat_as("ai-private", Some(&bob_cookie), &[], hello(), &[])
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    // Bob's cookie with a forged identity naming somebody else.
    let mut forged = identity(&h.bob);
    forged[1].1 = h.owner.id.to_string();
    let (st, _, _) = h
        .chat_as("ai-private", Some(&bob_cookie), &forged, hello(), &[])
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    // A public-mode identity is never enough.
    let mut public = identity(&h.bob);
    public[0].1 = "public".into();
    let (st, _, _) = h
        .chat_as("ai-private", Some(&bob_cookie), &public, hello(), &[])
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    // A session for a different app does not transfer.
    let off_cookie = h.session(&h.bob, "ai-off");
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&off_cookie),
            &identity(&h.bob),
            hello(),
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    // Without the route marker the path does not exist.
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&bob_cookie),
            &identity(&h.bob),
            hello(),
            &[("x-repobox-gate", "")],
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    // Revoked grant: the old session no longer authorises.
    let eve_cookie = h.session(&h.eve, "ai-private");
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&eve_cookie),
            &identity(&h.eve),
            hello(),
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED, "eve has no grant");
    h.state.store.remove_grant(h.private.id, h.bob.id).unwrap();
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&bob_cookie),
            &identity(&h.bob),
            hello(),
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    // Disabled user.
    h.state
        .store
        .add_grant(h.private.id, h.bob.id, None)
        .unwrap();
    h.state.store.set_user_enabled(h.bob.id, false).unwrap();
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&bob_cookie),
            &identity(&h.bob),
            hello(),
            &[],
        )
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert_eq!(h.calls(), 0, "nothing reached the model");
}

#[tokio::test]
async fn cross_origin_and_non_json_requests_are_refused() {
    let h = H::new().await;
    let c = h.session(&h.bob, "ai-private");
    let id = identity(&h.bob);
    let (st, _, v) = h
        .chat_as(
            "ai-private",
            Some(&c),
            &id,
            hello(),
            &[("origin", "https://evil.repo.box")],
        )
        .await;
    assert_eq!((st, code(&v)), (StatusCode::FORBIDDEN, "cross_origin"));
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&c),
            &id,
            hello(),
            &[("sec-fetch-site", "same-site")],
        )
        .await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    let (st, _, v) = h
        .chat_as(
            "ai-private",
            Some(&c),
            &id,
            hello(),
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::UNSUPPORTED_MEDIA_TYPE, "content_type")
    );
    // A server-side caller (no Origin / Sec-Fetch-Site) relaying the user's
    // cookie is fine.
    let (st, _, _) = h
        .chat_as(
            "ai-private",
            Some(&c),
            &id,
            hello(),
            &[("origin", ""), ("sec-fetch-site", "")],
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h.calls(), 1);
}

// --------------------------------------------------------- policy/limits

#[tokio::test]
async fn disabled_ai_and_disabled_apps_are_refused() {
    let h = H::new().await;
    let (st, v) = h.chat(&h.bob, "ai-off", hello()).await;
    assert_eq!((st, code(&v)), (StatusCode::FORBIDDEN, "ai_disabled"));
    h.state.store.set_app_enabled(h.private.id, false).unwrap();
    let cookie = h.session(&h.bob, "ai-private");
    let (st, _) = h.gate_identity("ai-private", Some(&cookie)).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "gate refuses a disabled app");
    let (st, _, v) = h
        .chat_as("ai-private", Some(&cookie), &identity(&h.bob), hello(), &[])
        .await;
    assert_eq!((st, code(&v)), (StatusCode::NOT_FOUND, "app_disabled"));
    assert_eq!(h.calls(), 0);
}

#[tokio::test]
async fn unsupported_models_and_oversized_requests_are_refused() {
    let h = H::new().await;
    let m = json!([{"role": "user", "content": "hi"}]);
    for (body, want_status, want_code) in [
        (
            json!({"model": "gpt-5.6-sol", "messages": m}),
            StatusCode::BAD_REQUEST,
            "model_not_allowed",
        ),
        (
            json!({"model": "gpt-4o", "messages": m}),
            StatusCode::BAD_REQUEST,
            "model_not_allowed",
        ),
        (
            json!({"messages": m, "max_tokens": 5000}),
            StatusCode::BAD_REQUEST,
            "max_tokens_too_large",
        ),
        (
            json!({"messages": m, "stream": true}),
            StatusCode::BAD_REQUEST,
            "streaming_unsupported",
        ),
        (
            json!({"messages": m, "tools": [{"type": "function"}]}),
            StatusCode::BAD_REQUEST,
            "unsupported_parameter",
        ),
        (
            json!({"messages": [{"role": "user", "content": "x".repeat(32_001)}]}),
            StatusCode::PAYLOAD_TOO_LARGE,
            "input_too_large",
        ),
    ] {
        let (st, v) = h.chat(&h.bob, "ai-private", body).await;
        assert_eq!((st, code(&v)), (want_status, want_code), "{v}");
    }
    // A body over the byte ceiling is cut off before parsing.
    let cookie = h.session(&h.bob, "ai-private");
    let huge = json!({"messages": [{"role": "user", "content": "y".repeat(AI_MAX_BODY_BYTES)}]});
    let (st, _, v) = h
        .chat_as("ai-private", Some(&cookie), &identity(&h.bob), huge, &[])
        .await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::PAYLOAD_TOO_LARGE, "request_too_large")
    );
    // A model the app allows but the broker does not currently route.
    let mut p = h.private.ai.clone();
    p.models.push("gpt-5.5".into());
    h.state.store.set_app_ai(h.private.id, &p).unwrap();
    let (st, v) = h
        .chat(
            &h.bob,
            "ai-private",
            json!({"model": "gpt-5.5", "messages": m}),
        )
        .await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::BAD_REQUEST, "model_not_allowed"),
        "{v}"
    );
    assert_eq!(h.calls(), 0, "no rejected request reached the model");
    // No quota was spent on requests refused before the upstream call.
    assert_eq!(
        h.state.store.ai_usage_today(h.private.id).unwrap().0,
        1,
        "only the broker-refused one"
    );
}

#[tokio::test]
async fn daily_quotas_apply_per_user_and_per_app() {
    let h = H::new().await;
    let mut p = h.private.ai.clone();
    p.user_daily_requests = 2;
    p.app_daily_requests = 3;
    h.state.store.set_app_ai(h.private.id, &p).unwrap();
    for _ in 0..2 {
        assert_eq!(
            h.chat(&h.bob, "ai-private", hello()).await.0,
            StatusCode::OK
        );
    }
    let (st, v) = h.chat(&h.bob, "ai-private", hello()).await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::TOO_MANY_REQUESTS, "quota_exceeded")
    );
    assert_eq!(
        h.chat(&h.owner, "ai-private", hello()).await.0,
        StatusCode::OK
    );
    let (st, v) = h.chat(&h.owner, "ai-private", hello()).await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::TOO_MANY_REQUESTS, "quota_exceeded"),
        "app quota"
    );
    assert_eq!(h.calls(), 3);
}

#[tokio::test]
async fn public_ai_needs_policy_and_a_signed_in_user() {
    let h = H::new().await;
    let public = h.app("ai-public");
    let mut p = public.ai.clone();
    p.enabled = true;
    p.public_policy = Some(AI_PUBLIC_POLICY_SIGNED_IN_QUOTA.into());
    p.user_daily_requests = 1;
    p.app_daily_requests = 10;
    h.state.store.set_app_ai(public.id, &p).unwrap();
    // Anonymous visitor of a public app: the gate allows the page but the AI
    // hop refuses without a platform session.
    let (st, ids) = h.gate_identity("ai-public", None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(ids.contains(&("x-repobox-auth".into(), "public".into())));
    let (st, _, v) = h.chat_as("ai-public", None, &ids, hello(), &[]).await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::UNAUTHORIZED, "unauthenticated")
    );
    // The owner (signed in, has access) can, within the explicit quota.
    assert_eq!(
        h.chat(&h.owner, "ai-public", hello()).await.0,
        StatusCode::OK
    );
    assert_eq!(
        h.chat(&h.owner, "ai-public", hello()).await.0,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn not_configured_bridge_answers_503() {
    let h = H::new().await;
    let mut cfg = h.state.cfg.clone();
    cfg.ai = None;
    let state = Arc::new(AppState {
        store: Store::open_in_memory().unwrap(),
        cfg,
    });
    let owner = state.store.create_user("o", "O", Role::Member).unwrap();
    let app = state
        .store
        .create_app(
            "x",
            "x",
            "",
            owner.id,
            AppKind::Proxy,
            "127.0.0.1:1",
            Visibility::Private,
            IdentityContract::Platform,
        )
        .unwrap();
    let (raw, _) = state
        .store
        .create_session(SessionKind::App, owner.id, Some(app.id), None, 60, "")
        .unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/gate/ai/v1/chat/completions")
        .header("X-RepoBox-Gate", "1")
        .header("X-RepoBox-Gate-App", "x")
        .header("X-RepoBox-Auth", "session")
        .header("X-RepoBox-User-Id", owner.id.to_string())
        .header(header::COOKIE, format!("{APP_COOKIE}={raw}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(hello().to_string()))
        .unwrap();
    let resp = web::router(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ---------------------------------------------------------- broker

#[tokio::test]
async fn broker_requires_the_bridge_secret_and_live_models() {
    let h = H::new().await;
    let client = ai::Upstream::new(&h.broker).unwrap();
    let body = json!({"model": "gpt-5.6-terra", "messages": [{"role": "user", "content": "hi"}]});
    let send = |auth: Option<String>, body: Value| {
        let client = client.clone();
        async move {
            let hs: Vec<(&str, String)> =
                auth.map(|a| vec![("authorization", a)]).unwrap_or_default();
            client
                .send(
                    axum::http::Method::POST,
                    "/v1/chat/completions",
                    &hs,
                    Some(body.to_string().into_bytes()),
                    std::time::Duration::from_secs(5),
                )
                .await
                .unwrap()
        }
    };
    assert_eq!(
        send(None, body.clone()).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(
            Some("Bearer wrong-secret-wrong-secret-wrong-secret".into()),
            body.clone()
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(Some(format!("Bearer {SECRET}x")), body.clone())
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    // gpt-5.6-sol is on the broker allowlist but not exposed by (fake) ChatMock.
    let r = send(
        Some(format!("Bearer {SECRET}")),
        json!({"model": "gpt-5.6-sol", "messages": [{"role": "user", "content": "hi"}]}),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert!(String::from_utf8_lossy(&r.body).contains("model_unavailable"));
    // gpt-5.5 is live but not on the broker allowlist.
    let r = send(
        Some(format!("Bearer {SECRET}")),
        json!({"model": "gpt-5.5", "messages": [{"role": "user", "content": "hi"}]}),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(h.calls(), 0);
    let r = send(Some(format!("Bearer {SECRET}")), body).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(h.calls(), 1);
    // Models list is the allowlist ∩ live, and also needs the secret.
    let r = client
        .send(
            axum::http::Method::GET,
            "/v1/models",
            &[],
            None,
            std::time::Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = client
        .send(
            axum::http::Method::GET,
            "/v1/models",
            &[("authorization", format!("Bearer {SECRET}"))],
            None,
            std::time::Duration::from_secs(5),
        )
        .await
        .unwrap();
    let ids: Vec<String> = serde_json::from_slice::<Value>(&r.body).unwrap()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids, ["gpt-5.6-terra", "gpt-5.6-luna"]);
    // Unknown broker paths are 404.
    let r = client
        .send(
            axum::http::Method::POST,
            "/v1/embeddings",
            &[("authorization", format!("Bearer {SECRET}"))],
            Some(b"{}".to_vec()),
            std::time::Duration::from_secs(5),
        )
        .await
        .unwrap();
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn app_models_endpoint_lists_allowed_live_models() {
    let h = H::new().await;
    let cookie = h.session(&h.bob, "ai-private");
    let req = Request::builder()
        .uri("/gate/ai/v1/models")
        .header("X-RepoBox-Gate", "1")
        .header("X-RepoBox-Gate-App", "ai-private")
        .header("X-RepoBox-Auth", "session")
        .header("X-RepoBox-User-Id", h.bob.id.to_string())
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    let (st, _, v) = h.send(req).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["data"].as_array().unwrap().len(), 2);
    let req = Request::builder()
        .uri("/gate/ai/v1/embeddings")
        .body(Body::empty())
        .unwrap();
    assert_eq!(h.send(req).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gate_never_redeems_launch_codes_on_reserved_paths() {
    let h = H::new().await;
    let req = Request::builder()
        .uri("/gate/verify")
        .header("X-RepoBox-Gate", "1")
        .header("X-RepoBox-Gate-App", "ai-private")
        .header("X-Forwarded-Method", "GET")
        .header(
            "X-Forwarded-Uri",
            "/_repo_box/ai/v1/models?rb_launch=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .body(Body::empty())
        .unwrap();
    let (st, hd, v) = h.send(req).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(hd.get(header::SET_COOKIE).is_none());
    assert_eq!(code(&v), "unauthenticated", "JSON, not the HTML page");
}

// ----------------------------------------------------- discovery + API

fn token(h: &H, owner: &User, scopes: &[&str], apps: &[&str]) -> String {
    let (raw, _) = h
        .state
        .store
        .create_service_token(
            &format!("t{}", h.state.store.list_service_tokens().unwrap().len()),
            owner,
            &scopes.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            &apps.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            86400,
        )
        .unwrap();
    raw
}

async fn api(
    h: &H,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut b = Request::builder().method(method).uri(uri);
    if let Some(t) = bearer {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let req = match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    h.send(req).await
}

#[tokio::test]
async fn discovery_documents_are_public_versioned_and_secret_free() {
    let h = H::new().await;
    for path in ["/api/platform/v1", "/.well-known/repobox-platform.json"] {
        let (st, _, v) = api(&h, "GET", path, None, None).await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(v["api_version"], "v1");
        assert_eq!(v["authentication"]["type"], "service_token");
        assert!(
            v["authentication"]["oauth"]
                .as_str()
                .unwrap()
                .contains("not implemented")
        );
        assert_eq!(v["app_ai_endpoint"]["streaming"], false);
        assert!(
            v["app_ai_endpoint"]["url"]
                .as_str()
                .unwrap()
                .ends_with("/_repo_box/ai/v1/chat/completions")
        );
        assert!(v["mcp"]["tools"].as_array().unwrap().len() >= 8);
        let text = v.to_string();
        for leak in [SECRET, "127.0.0.1", "8111", "ai-private", "bob"] {
            assert!(!text.contains(leak), "discovery leaks {leak}");
        }
    }
    let (st, _, v) = api(&h, "GET", "/api/platform/v1/openapi.json", None, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["openapi"], "3.1.0");
    assert!(v["paths"]["/api/platform/v1/apps/{name}/ai"]["patch"].is_object());
    let (st, hd, v) = api(&h, "GET", "/api/platform/v1/skill.md", None, None).await;
    assert_eq!(st, StatusCode::OK);
    assert!(
        hd[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("text/markdown")
    );
    assert!(
        v.as_str()
            .unwrap()
            .contains("/_repo_box/ai/v1/chat/completions")
    );
}

#[tokio::test]
async fn service_tokens_are_owner_scoped_and_least_privilege() {
    let h = H::new().await;
    let (st, hd, _) = api(&h, "GET", "/api/platform/v1/apps", None, None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert!(hd.get(header::WWW_AUTHENTICATE).is_some());
    let (st, _, _) = api(
        &h,
        "GET",
        "/api/platform/v1/apps",
        Some("rbp_not-a-token"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);

    let read = token(&h, &h.owner, &["apps:read", "ai:read"], &[]);
    let (st, _, v) = api(&h, "GET", "/api/platform/v1/apps", Some(&read), None).await;
    assert_eq!(st, StatusCode::OK);
    let names: Vec<&str> = v["apps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["ai-off", "ai-private", "ai-public"],
        "only the owner's apps"
    );
    assert!(
        !v.to_string().contains("127.0.0.1"),
        "no origin targets in the app view"
    );
    let (st, _, _) = api(
        &h,
        "GET",
        "/api/platform/v1/apps/elsewhere",
        Some(&read),
        None,
    )
    .await;
    assert_eq!(
        st,
        StatusCode::NOT_FOUND,
        "another owner's app does not exist for this token"
    );
    let (st, _, v) = api(
        &h,
        "GET",
        "/api/platform/v1/apps/ai-private/ai",
        Some(&read),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["policy"]["enabled"], true);
    // Missing scopes.
    let (st, _, v) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-private/ai",
        Some(&read),
        Some(json!({"enabled": false})),
    )
    .await;
    assert_eq!(
        (st, code(&v)),
        (StatusCode::FORBIDDEN, "insufficient_scope")
    );
    for path in [
        "/api/platform/v1/apps/ai-private/route",
        "/api/platform/v1/release",
        "/api/platform/v1/app-requests",
    ] {
        assert_eq!(
            api(&h, "GET", path, Some(&read), None).await.0,
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }

    // ai:write within bounds, narrowed to one app.
    let write = token(&h, &h.owner, &["ai:write", "ai:read"], &["ai-private"]);
    let (st, _, v) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-private/ai",
        Some(&write),
        Some(json!({"max_output_tokens": 512, "default_model": "gpt-5.6-luna"})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(h.app("ai-private").ai.max_output_tokens, 512);
    let (st, _, _) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-private/ai",
        Some(&write),
        Some(json!({"max_output_tokens": 99999})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    let (st, _, _) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-private/ai",
        Some(&write),
        Some(json!({"provider": "openai"})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    let (st, _, _) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-public/ai",
        Some(&write),
        Some(json!({"enabled": true})),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "outside the token's app list");
    // Public app: enabling without the explicit policy is refused.
    let all = token(&h, &h.owner, &["ai:write"], &[]);
    let (st, _, v) = api(
        &h,
        "PATCH",
        "/api/platform/v1/apps/ai-public/ai",
        Some(&all),
        Some(json!({"enabled": true})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("abuse/quota policy")
    );
    let (st, _, _) = api(&h, "PATCH", "/api/platform/v1/apps/ai-public/ai", Some(&all), Some(json!({"enabled": true, "public_policy": "signed-in-quota", "user_daily_requests": 5, "app_daily_requests": 20}))).await;
    assert_eq!(st, StatusCode::OK);
    // Audited with the token name.
    let audit = h.state.store.list_audit(5).unwrap();
    assert!(
        audit
            .iter()
            .any(|e| e.action == "app.ai" && e.detail.contains("service-token="))
    );

    // Revoked and expired tokens stop working immediately.
    let name = h.state.store.list_service_tokens().unwrap()[0].name.clone();
    h.state.store.revoke_service_token(&name).unwrap();
    assert_eq!(
        api(&h, "GET", "/api/platform/v1/apps", Some(&read), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    // Disabled owner.
    h.state.store.set_user_enabled(h.owner.id, false).unwrap();
    assert_eq!(
        api(&h, "GET", "/api/platform/v1/whoami", Some(&write), None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    // Tokens cannot be minted for apps the owner does not own.
    assert!(
        h.state
            .store
            .create_service_token(
                "x",
                &h.other,
                &["apps:read".into()],
                &["ai-private".into()],
                86400
            )
            .is_err()
    );
    assert!(
        h.state
            .store
            .create_service_token("y", &h.other, &["admin".into()], &[], 86400)
            .is_err()
    );
}

#[tokio::test]
async fn route_preview_release_and_registration_requests() {
    let h = H::new().await;
    let t = token(
        &h,
        &h.owner,
        &["routes:read", "release:read", "apps:request"],
        &[],
    );
    let (st, _, v) = api(
        &h,
        "GET",
        "/api/platform/v1/apps/ai-private/route",
        Some(&t),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let caddy = v["caddy"].as_str().unwrap();
    assert!(caddy.contains("handle /_repo_box/ai/v1/* {"));
    assert!(
        caddy.find("request_header -X-RepoBox-*").unwrap()
            < caddy.find("reverse_proxy 127.0.0.1:3231").unwrap()
    );
    let (st, _, v) = api(&h, "GET", "/api/platform/v1/release", Some(&t), None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["schema_version"], "8");
    assert_eq!(v["ai_broker"]["reachable"], true);
    assert!(!v.to_string().contains(SECRET));
    // Registration is only a request.
    let (st, _, v) = api(&h, "POST", "/api/platform/v1/app-requests", Some(&t), Some(json!({"name": "new-app", "title": "New", "kind": "proxy", "target": "127.0.0.1:4555", "visibility": "private"}))).await;
    assert_eq!(
        st,
        StatusCode::BAD_REQUEST,
        "private needs the identity declaration: {v}"
    );
    let (st, _, _) = api(&h, "POST", "/api/platform/v1/app-requests", Some(&t), Some(json!({"name": "new-app", "title": "New", "kind": "proxy", "target": "0.0.0.0:4555", "visibility": "private", "identity": "platform"}))).await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "non-loopback origin");
    let (st, _, v) = api(&h, "POST", "/api/platform/v1/app-requests", Some(&t), Some(json!({"name": "new-app", "title": "New", "kind": "proxy", "target": "127.0.0.1:4555", "visibility": "private", "identity": "platform"}))).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["request"]["status"], "pending");
    assert!(
        h.state.store.app_by_name("new-app").unwrap().is_none(),
        "nothing registered yet"
    );
    let (_, _, v) = api(&h, "GET", "/api/platform/v1/app-requests", Some(&t), None).await;
    assert_eq!(v["requests"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn mcp_surface_mirrors_the_api() {
    let h = H::new().await;
    let t = token(&h, &h.owner, &["apps:read", "ai:read"], &[]);
    let rpc = |body: Value| api(&h, "POST", "/api/platform/v1/mcp", Some(&t), Some(body));
    assert_eq!(
        api(
            &h,
            "POST",
            "/api/platform/v1/mcp",
            None,
            Some(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}))
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let (st, _, v) = rpc(json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}})).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["result"]["serverInfo"]["name"], "repobox-platform");
    assert_eq!(
        rpc(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
            .await
            .0,
        StatusCode::ACCEPTED
    );
    let (_, _, v) = rpc(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).await;
    let tools: Vec<&str> = v["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    for want in [
        "list_apps",
        "get_ai_policy",
        "update_ai_policy",
        "preview_route",
        "request_app_registration",
        "release_status",
    ] {
        assert!(tools.contains(&want), "{want}");
    }
    let (_, _, v) = rpc(json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "get_ai_policy", "arguments": {"name": "ai-private"}}})).await;
    assert_eq!(v["result"]["isError"], false);
    assert_eq!(
        v["result"]["structuredContent"]["policy"]["provider"],
        "chatmock"
    );
    let (_, _, v) = rpc(json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "update_ai_policy", "arguments": {"name": "ai-private", "enabled": false}}})).await;
    assert_eq!(
        v["result"]["isError"], true,
        "scope enforced through MCP too"
    );
    assert!(h.app("ai-private").ai.enabled);
    let (_, _, v) = rpc(json!({"jsonrpc": "2.0", "id": 5, "method": "nope"})).await;
    assert_eq!(v["error"]["code"], -32601);
}
