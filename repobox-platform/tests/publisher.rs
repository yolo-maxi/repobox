//! External publisher, in-process: the publisher API against a real store
//! and spool, with the deploy worker's verdicts written into the spool the
//! way the worker writes them (the Docker side is covered by
//! scripts/edge-e2e.sh). Covers token-kind isolation, ownership, name
//! collisions, the upload contract, the release lifecycle, rollback,
//! restart, revocation, discovery, OpenAPI and MCP.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use repobox_platform::model::{AppKind, IdentityContract, Role, Visibility};
use repobox_platform::publisher::spool::{Job, JobResult, Op, Spool};
use repobox_platform::store::Store;
use repobox_platform::web::publisher::PublisherConfig;
use repobox_platform::web::{self, AppState, Config};

const BASE: &str = "https://auth.repo.box";

struct H {
    state: Arc<AppState>,
    clock: Arc<AtomicI64>,
    spool: Spool,
    dir: std::path::PathBuf,
    muse: String,
    other: String,
    service: String,
}

impl Drop for H {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn tmpdir(tag: &str) -> std::path::PathBuf {
    use rand::RngCore;
    let mut b = [0u8; 6];
    rand::rngs::OsRng.fill_bytes(&mut b);
    let d = std::env::temp_dir().join(format!(
        "rbp-pub-{tag}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5]
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

impl H {
    fn new() -> Self {
        let clock = Arc::new(AtomicI64::new(1_800_000_000));
        let c = clock.clone();
        let store =
            Store::open_in_memory_with_clock(Arc::new(move || c.load(Ordering::SeqCst))).unwrap();
        let fran = store.create_user("fran", "Fran", Role::Admin).unwrap();
        store
            .create_app(
                "study-diary",
                "Study Diary",
                "",
                fran.id,
                AppKind::Proxy,
                "127.0.0.1:3025",
                Visibility::Private,
                IdentityContract::Platform,
            )
            .unwrap();
        let (muse, _) = store.create_publisher("muse", "Muse", None).unwrap();
        let (other, _) = store
            .create_publisher("other-agent", "Other agent", None)
            .unwrap();
        let (muse_tok, _) = store
            .create_publisher_token("muse-1", &muse, 7 * 86400)
            .unwrap();
        let (other_tok, _) = store
            .create_publisher_token("other-1", &other, 7 * 86400)
            .unwrap();
        let (service, _) = store
            .create_service_token("fran-agent", &fran, &["apps:read".into()], &[], 86400)
            .unwrap();
        let dir = tmpdir("h");
        let spool = Spool::new(dir.join("spool"));
        spool.create_dirs().unwrap();
        std::fs::write(dir.join("Caddyfile"), "legacy-site.repo.box {\n\trespond 200\n}\nauth.repo.box, www.legacy-two.repo.box {\n}\n").unwrap();
        std::fs::create_dir_all(dir.join("subdomains/old-static")).unwrap();
        let mut cfg = Config::defaults(BASE, "repo.box");
        cfg.publisher = Some(PublisherConfig {
            spool: spool.clone(),
            host_files: vec![dir.join("Caddyfile")],
            reserved_dirs: vec![dir.join("subdomains")],
            min_free_bytes: 0,
        });
        H {
            state: Arc::new(AppState { store, cfg }),
            clock,
            spool,
            dir,
            muse: muse_tok,
            other: other_tok,
            service,
        }
    }

    async fn send(&self, req: Request<Body>) -> (StatusCode, Value) {
        let r = web::router(self.state.clone()).oneshot(req).await.unwrap();
        let st = r.status();
        let b = r.into_body().collect().await.unwrap().to_bytes();
        (
            st,
            serde_json::from_slice(&b).unwrap_or(Value::String(String::from_utf8_lossy(&b).into())),
        )
    }

    async fn get(&self, path: &str, tok: Option<&str>) -> (StatusCode, Value) {
        let mut b = Request::get(path);
        if let Some(t) = tok {
            b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        self.send(b.body(Body::empty()).unwrap()).await
    }

    async fn post_json(&self, path: &str, tok: &str, body: Value) -> (StatusCode, Value) {
        self.send(
            Request::post(path)
                .header(header::AUTHORIZATION, format!("Bearer {tok}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
    }

    async fn deploy(&self, tok: &str, manifest: &Value, image: &[u8]) -> (StatusCode, Value) {
        self.send(multipart(
            tok,
            &[
                (
                    "manifest",
                    "application/json",
                    manifest.to_string().as_bytes(),
                ),
                ("image", "application/x-tar", image),
            ],
        ))
        .await
    }

    /// Answer a queued job the way the deploy worker does.
    fn worker_answers(&self, id: &str, state: &str, retained: &[&str]) {
        let job: Job = serde_json::from_slice(
            &std::fs::read(self.spool.jobs().join(format!("{id}.json"))).unwrap(),
        )
        .unwrap();
        std::fs::remove_file(self.spool.jobs().join(format!("{id}.json"))).unwrap();
        let _ = std::fs::remove_file(self.spool.upload_path(id));
        let now = self.clock.load(Ordering::SeqCst);
        let r = JobResult {
            id: id.into(),
            app: job.app,
            state: state.into(),
            code: if state == "failed" {
                "unhealthy".into()
            } else {
                String::new()
            },
            failure: if state == "failed" {
                "the app did not become healthy".into()
            } else {
                String::new()
            },
            build_mode: "docker-save".into(),
            image_id: "sha256:feed".into(),
            retained: retained.iter().map(|s| s.to_string()).collect(),
            current: retained.first().map(|s| s.to_string()).unwrap_or_default(),
            updated_at: now,
            finished_at: (state == "live" || state == "failed").then_some(now),
            ..Default::default()
        };
        std::fs::write(
            self.spool.results().join(format!("{id}.json")),
            serde_json::to_vec(&r).unwrap(),
        )
        .unwrap();
    }
}

/// (part name, content type, bytes)
type Part<'a> = (&'a str, &'a str, &'a [u8]);

fn multipart(tok: &str, parts: &[Part]) -> Request<Body> {
    let boundary = "XrepoboxBoundary7";
    let mut body = Vec::new();
    for (name, ct, data) in parts {
        body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{name}\"\r\nContent-Type: {ct}\r\n\r\n").as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    let mut b = Request::post("/api/platform/v1/publisher/releases").header(
        header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={boundary}"),
    );
    if !tok.is_empty() {
        b = b.header(header::AUTHORIZATION, format!("Bearer {tok}"));
    }
    b.body(Body::from(body)).unwrap()
}

/// A minimal `docker save`-shaped tar (the API only checks the tar magic;
/// the worker does the real import).
fn image_tar() -> Vec<u8> {
    let mut b = tar::Builder::new(Vec::new());
    for (p, c) in [
        (
            "manifest.json",
            r#"[{"Config":"blobs/sha256/cc","RepoTags":["anything:latest"],"Layers":[]}]"#,
        ),
        (
            "index.json",
            r#"{"schemaVersion":2,"manifests":[{"digest":"sha256:bb"}]}"#,
        ),
    ] {
        let mut h = tar::Header::new_ustar();
        h.set_size(c.len() as u64);
        h.set_mode(0o644);
        b.append_data(&mut h, p, c.as_bytes()).unwrap();
    }
    b.into_inner().unwrap()
}

fn manifest(name: &str) -> Value {
    json!({"name": name, "title": "Trip planner", "runtime": {"port": 3000, "health_path": "/healthz"},
           "provenance": {"repository": "https://example.com/trip", "commit": "abc1234"}})
}

#[tokio::test]
async fn deploy_queues_a_private_platform_app_owned_by_the_publisher() {
    let h = H::new();
    let img = image_tar();
    let (st, v) = h.deploy(&h.muse, &manifest("trip"), &img).await;
    assert_eq!(st, StatusCode::ACCEPTED, "{v}");
    let r = &v["release"];
    assert_eq!(r["status"], "queued");
    assert_eq!(r["version"], 1);
    assert_eq!(r["publisher"]["handle"], "muse");
    assert!(r["publisher"]["id"].as_str().unwrap().starts_with("pub_"));
    assert_eq!(r["artifact"]["bytes"], img.len());
    use sha2::Digest;
    let want = format!(
        "sha256:{}",
        repobox_platform::publisher::archive::hex(&sha2::Sha256::digest(&img))
    );
    assert_eq!(r["artifact"]["sha256"], want);
    assert_eq!(r["provenance"]["commit"], "abc1234");
    let a = &v["app"];
    assert_eq!(a["launcher_url"], "https://auth.repo.box/trip");
    assert_eq!(a["direct_url"], "https://trip.repo.box/");
    assert!(
        a["direct_url_access"]
            .as_str()
            .unwrap()
            .starts_with("edge-gated")
    );
    assert_eq!(a["visibility"], "private");
    assert_eq!(a["identity"], "platform");
    assert_eq!(a["ai"]["policy"]["enabled"], true);
    assert_eq!(a["status"], "deploying");
    // Registry: owned by the publisher's own owner record, routed by the worker.
    let app = h.state.store.app_by_name("trip").unwrap().unwrap();
    let muse = h.state.store.publisher_by_handle("muse").unwrap().unwrap();
    assert_eq!(app.owner_id, muse.owner_id);
    assert_eq!(app.publisher_id, Some(muse.id));
    assert!(app.ai.enabled);
    // Spool: the job and the exact uploaded bytes.
    let id = r["id"].as_str().unwrap();
    let job: Job =
        serde_json::from_slice(&std::fs::read(h.spool.jobs().join(format!("{id}.json"))).unwrap())
            .unwrap();
    assert_eq!(job.op, Op::Deploy);
    assert_eq!(job.upload.as_ref().unwrap().sha256, want);
    assert!(!job.upload.as_ref().unwrap().gzip);
    assert_eq!(std::fs::read(h.spool.upload_path(id)).unwrap(), img);
    // A second release while this one runs is refused.
    let (st, v) = h.deploy(&h.muse, &manifest("trip"), &img).await;
    assert_eq!(st, StatusCode::CONFLICT);
    assert_eq!(v["error"]["code"], "release_in_progress");
    // The worker's verdict makes it live.
    h.worker_answers(id, "live", &[id]);
    let (st, v) = h
        .get(
            &format!("/api/platform/v1/publisher/releases/{id}"),
            Some(&h.muse),
        )
        .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["release"]["status"], "live");
    assert_eq!(v["release"]["done"], true);
    assert_eq!(v["release"]["retained_for_rollback"], true);
    assert_eq!(v["app"]["status"], "live");
    assert!(
        v["next"]
            .as_str()
            .unwrap()
            .contains("https://auth.repo.box/trip")
    );
}

#[tokio::test]
async fn updates_rollback_and_restart_stay_within_the_app() {
    let h = H::new();
    let img = image_tar();
    let (_, v) = h.deploy(&h.muse, &manifest("trip"), &img).await;
    let r1 = v["release"]["id"].as_str().unwrap().to_string();
    h.worker_answers(&r1, "live", &[&r1]);
    h.clock.fetch_add(10, Ordering::SeqCst);
    let mut m2 = manifest("trip");
    m2["title"] = json!("Trip planner v2");
    m2["version"] = json!("2.0.0");
    let (st, v) = h.deploy(&h.muse, &m2, &img).await;
    assert_eq!(st, StatusCode::ACCEPTED, "{v}");
    assert_eq!(v["release"]["version"], 2);
    assert_eq!(v["app"]["title"], "Trip planner v2");
    let r2 = v["release"]["id"].as_str().unwrap().to_string();
    h.worker_answers(&r2, "live", &[&r2, &r1]);
    let (_, v) = h
        .get(
            "/api/platform/v1/publisher/releases?app=trip",
            Some(&h.muse),
        )
        .await;
    let st: Vec<&str> = v["releases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["status"].as_str().unwrap())
        .collect();
    assert_eq!(st, vec!["live", "superseded"]);
    // Identity/access model preserved across the update.
    let app = h.state.store.app_by_name("trip").unwrap().unwrap();
    assert_eq!(
        (app.visibility, app.identity),
        (Visibility::Private, IdentityContract::Platform)
    );

    // A failed update leaves the live release live.
    h.clock.fetch_add(10, Ordering::SeqCst);
    let (_, v) = h.deploy(&h.muse, &manifest("trip"), &img).await;
    let r3 = v["release"]["id"].as_str().unwrap().to_string();
    h.worker_answers(&r3, "failed", &[&r2, &r1]);
    let (_, v) = h
        .get(
            &format!("/api/platform/v1/publisher/releases/{r3}"),
            Some(&h.muse),
        )
        .await;
    assert_eq!(v["release"]["status"], "failed");
    assert_eq!(v["release"]["failure"]["code"], "unhealthy");
    assert_eq!(v["app"]["current_release"]["id"], r2.as_str());

    // Rollback (default: the most recent previous retained release).
    let (st, v) = h
        .send(
            Request::post("/api/platform/v1/publisher/apps/trip/rollback")
                .header(header::AUTHORIZATION, format!("Bearer {}", h.muse))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{v}");
    assert_eq!(v["release"]["op"], "rollback");
    assert_eq!(v["release"]["rollback_of"], r1.as_str());
    let rb = v["release"]["id"].as_str().unwrap().to_string();
    let job: Job =
        serde_json::from_slice(&std::fs::read(h.spool.jobs().join(format!("{rb}.json"))).unwrap())
            .unwrap();
    assert_eq!((job.op, job.target.as_str()), (Op::Rollback, r1.as_str()));
    h.worker_answers(&rb, "live", &[&rb, &r2, &r1]);
    let (_, v) = h
        .get("/api/platform/v1/publisher/apps/trip", Some(&h.muse))
        .await;
    assert_eq!(v["current_release"]["id"], rb.as_str());
    // Unknown/unretained target.
    let (st, _) = h
        .post_json(
            "/api/platform/v1/publisher/apps/trip/rollback",
            &h.muse,
            json!({"release": "rel-20000101000000-00000000"}),
        )
        .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (st, _) = h
        .post_json(
            "/api/platform/v1/publisher/apps/trip/rollback",
            &h.muse,
            json!({"bogus": 1}),
        )
        .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);

    // Restart: a 'done' record, the live release stays live.
    let (st, v) = h
        .send(
            Request::post("/api/platform/v1/publisher/apps/trip/restart")
                .header(header::AUTHORIZATION, format!("Bearer {}", h.muse))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{v}");
    let rs = v["release"]["id"].as_str().unwrap().to_string();
    h.worker_answers(&rs, "live", &[&rb, &r2, &r1]);
    let (_, v) = h
        .get(
            &format!("/api/platform/v1/publisher/releases/{rs}"),
            Some(&h.muse),
        )
        .await;
    assert_eq!(v["release"]["status"], "done");
    assert_eq!(v["app"]["current_release"]["id"], rb.as_str());
}

#[tokio::test]
async fn publishers_only_reach_their_own_apps_and_names_fail_closed() {
    let h = H::new();
    let img = image_tar();
    let (_, v) = h.deploy(&h.muse, &manifest("trip"), &img).await;
    let rid = v["release"]["id"].as_str().unwrap().to_string();

    // The other publisher sees nothing of muse's, and cannot touch it.
    let (_, v) = h
        .get("/api/platform/v1/publisher/apps", Some(&h.other))
        .await;
    assert_eq!(v["apps"], json!([]));
    for path in [
        "/api/platform/v1/publisher/apps/trip",
        "/api/platform/v1/publisher/apps/trip/logs",
        &format!("/api/platform/v1/publisher/releases/{rid}"),
        &format!("/api/platform/v1/publisher/releases/{rid}/log"),
    ] {
        assert_eq!(
            h.get(path, Some(&h.other)).await.0,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    let (_, v) = h
        .get("/api/platform/v1/publisher/releases", Some(&h.other))
        .await;
    assert_eq!(v["releases"], json!([]));
    for op in ["rollback", "restart"] {
        let (st, _) = h
            .post_json(
                &format!("/api/platform/v1/publisher/apps/trip/{op}"),
                &h.other,
                json!({}),
            )
            .await;
        assert_eq!(st, StatusCode::NOT_FOUND, "{op}");
    }
    // Existing operator apps are neither listed nor reachable.
    let (_, v) = h
        .get("/api/platform/v1/publisher/apps", Some(&h.muse))
        .await;
    let names: Vec<&str> = v["apps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["trip"]);
    assert_eq!(
        h.get("/api/platform/v1/publisher/apps/study-diary", Some(&h.muse))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        h.post_json(
            "/api/platform/v1/publisher/apps/study-diary/restart",
            &h.muse,
            json!({})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    // Every kind of name collision gets the same answer, before any upload.
    for name in ["study-diary", "trip", "legacy-site", "old-static"] {
        let tok = if name == "trip" { &h.other } else { &h.muse };
        let (st, v) = h.deploy(tok, &manifest(name), &img).await;
        assert_eq!(st, StatusCode::CONFLICT, "{name}: {v}");
        assert_eq!(v["error"]["code"], "name_unavailable", "{name}");
    }
    assert_eq!(
        h.deploy(&h.muse, &manifest("auth"), &img).await.0,
        StatusCode::BAD_REQUEST
    );
    // www.legacy-two.repo.box is a different host from legacy-two.repo.box.
    assert_eq!(
        h.deploy(&h.muse, &manifest("legacy-two"), &img).await.0,
        StatusCode::ACCEPTED
    );
    // Nothing was left in the spool by refused uploads.
    let uploads = std::fs::read_dir(h.spool.uploads()).unwrap().count();
    assert_eq!(uploads, 2);
}

#[tokio::test]
async fn only_a_single_publisher_bearer_opens_the_api() {
    let h = H::new();
    let img = image_tar();
    // Anonymous, service token, cookies, junk, two headers: all 401, nothing written.
    for tok in ["", "rbpub_nope", &h.service.clone()] {
        let (st, v) = h.deploy(tok, &manifest("trip"), &img).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{tok}: {v}");
    }
    let (_, v) = h.deploy(&h.service, &manifest("trip"), &img).await;
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("service tokens")
    );
    let two = Request::get("/api/platform/v1/publisher/apps")
        .header(header::AUTHORIZATION, format!("Bearer {}", h.muse))
        .header(header::AUTHORIZATION, format!("Bearer {}", h.other))
        .body(Body::empty())
        .unwrap();
    assert_eq!(h.send(two).await.0, StatusCode::UNAUTHORIZED);
    let cookie = Request::get("/api/platform/v1/publisher/apps")
        .header(header::COOKIE, "__Host-rb_auth=whatever")
        .header("X-RepoBox-User", "fran")
        .body(Body::empty())
        .unwrap();
    assert_eq!(h.send(cookie).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(std::fs::read_dir(h.spool.uploads()).unwrap().count(), 0);
    assert!(h.state.store.app_by_name("trip").unwrap().is_none());
    // A publisher token opens nothing on the service-token API.
    assert_eq!(
        h.get("/api/platform/v1/apps", Some(&h.muse)).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.get("/api/platform/v1/whoami", Some(&h.muse)).await.0,
        StatusCode::UNAUTHORIZED
    );

    // Revocation, publisher disable and expiry take effect on the next request.
    assert_eq!(
        h.get("/api/platform/v1/publisher/whoami", Some(&h.muse))
            .await
            .0,
        StatusCode::OK
    );
    h.state.store.revoke_publisher_token("muse-1").unwrap();
    assert_eq!(
        h.get("/api/platform/v1/publisher/whoami", Some(&h.muse))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let other = h
        .state
        .store
        .publisher_by_handle("other-agent")
        .unwrap()
        .unwrap();
    h.state
        .store
        .set_publisher_enabled(other.id, false)
        .unwrap();
    assert_eq!(
        h.get("/api/platform/v1/publisher/whoami", Some(&h.other))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    h.state.store.set_publisher_enabled(other.id, true).unwrap();
    assert_eq!(
        h.get("/api/platform/v1/publisher/whoami", Some(&h.other))
            .await
            .0,
        StatusCode::OK
    );
    h.clock.fetch_add(8 * 86400, Ordering::SeqCst);
    assert_eq!(
        h.get("/api/platform/v1/publisher/whoami", Some(&h.other))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    // Token lifetime bounds.
    assert!(
        h.state
            .store
            .create_publisher_token("long", &other, 31 * 86400)
            .is_err()
    );
}

#[tokio::test]
async fn malformed_uploads_say_exactly_what_to_send() {
    let h = H::new();
    let img = image_tar();
    // Not multipart.
    let (st, v) = h
        .post_json(
            "/api/platform/v1/publisher/releases",
            &h.muse,
            manifest("trip"),
        )
        .await;
    assert_eq!(st, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(v["error"]["expected"]["parts"][1]["name"], "image");
    assert!(
        v["error"]["expected"]["example"]
            .as_str()
            .unwrap()
            .contains("docker save")
    );
    // Image before manifest, missing image, extra part, bad manifest, not an archive.
    let m = manifest("trip").to_string();
    let cases: Vec<(Vec<Part>, &str)> = vec![
        (
            vec![
                ("image", "application/x-tar", &img),
                ("manifest", "application/json", m.as_bytes()),
            ],
            "missing_manifest",
        ),
        (
            vec![("manifest", "application/json", m.as_bytes())],
            "missing_image",
        ),
        (
            vec![("manifest", "application/json", b"{nope")],
            "invalid_manifest",
        ),
        (
            vec![(
                "manifest",
                "application/json",
                br#"{"name":"trip","title":"T","runtime":{"privileged":true}}"#,
            )],
            "invalid_manifest",
        ),
        (
            vec![
                ("manifest", "application/json", m.as_bytes()),
                ("image", "application/zip", b"PK\x03\x04zip"),
            ],
            "bad_archive",
        ),
        (
            vec![
                ("manifest", "application/json", m.as_bytes()),
                ("image", "application/x-tar", &img),
                ("extra", "text/plain", b"x"),
            ],
            "unexpected_part",
        ),
    ];
    for (parts, code) in cases {
        let (st, v) = h.send(multipart(&h.muse, &parts)).await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{code}: {v}");
        assert_eq!(v["error"]["code"], code);
        assert!(v["error"]["expected"].is_object(), "{code}");
    }
    // Nothing registered, nothing left behind.
    assert!(h.state.store.app_by_name("trip").unwrap().is_none());
    assert_eq!(std::fs::read_dir(h.spool.uploads()).unwrap().count(), 0);
    assert_eq!(std::fs::read_dir(h.spool.jobs()).unwrap().count(), 0);
    // gzip is accepted.
    use std::io::Write;
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gz.write_all(&img).unwrap();
    let (st, v) = h
        .deploy(&h.muse, &manifest("trip"), &gz.finish().unwrap())
        .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{v}");
    let id = v["release"]["id"].as_str().unwrap();
    let job: Job =
        serde_json::from_slice(&std::fs::read(h.spool.jobs().join(format!("{id}.json"))).unwrap())
            .unwrap();
    assert!(job.upload.unwrap().gzip);
}

#[tokio::test]
async fn unanswered_releases_time_out() {
    let h = H::new();
    let (_, v) = h.deploy(&h.muse, &manifest("trip"), &image_tar()).await;
    let id = v["release"]["id"].as_str().unwrap().to_string();
    h.clock.fetch_add(
        repobox_platform::publisher::JOB_TIMEOUT_SECS + 1,
        Ordering::SeqCst,
    );
    let (_, v) = h
        .get(
            &format!("/api/platform/v1/publisher/releases/{id}"),
            Some(&h.muse),
        )
        .await;
    assert_eq!(v["release"]["status"], "failed");
    assert_eq!(v["release"]["failure"]["code"], "timeout");
    // The app is free for a new release.
    assert_eq!(
        h.deploy(&h.muse, &manifest("trip"), &image_tar()).await.0,
        StatusCode::ACCEPTED
    );
}

#[tokio::test]
async fn discovery_openapi_skill_and_mcp_describe_the_publisher() {
    let h = H::new();
    let (st, v) = h.get("/api/platform/v1", None).await;
    assert_eq!(st, StatusCode::OK);
    let p = &v["publishing"];
    assert_eq!(p["available"], true);
    assert_eq!(p["credential"]["type"], "publisher_token");
    assert!(p["recipe"][1].as_str().unwrap().contains("docker save"));
    assert_eq!(p["upload"]["parts"][0]["name"], "manifest");
    assert!(p["archive_formats"].as_array().unwrap().len() >= 3);
    assert!(
        p["runtime_contract"]["ai"]
            .as_str()
            .unwrap()
            .contains("/_repo_box/ai/v1/chat/completions")
    );
    let (_, o) = h.get("/api/platform/v1/openapi.json", None).await;
    assert!(o["paths"]["/api/platform/v1/publisher/releases"]["post"]["requestBody"]["content"]["multipart/form-data"].is_object());
    assert_eq!(
        o["components"]["securitySchemes"]["publisherToken"]["bearerFormat"],
        "rbpub_ publisher token"
    );
    assert!(o["components"]["schemas"]["ReleaseManifest"]["properties"]["runtime"].is_object());
    let skill = web::api::SKILL_MD;
    for needle in [
        "docker save",
        "rbpub_",
        "/api/platform/v1/publisher/releases",
        "rollback",
    ] {
        assert!(skill.contains(needle), "skill.md lacks {needle}");
    }
    // MCP: a publisher token gets the publisher tool set.
    let rpc = |method: &str, params: Value| json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    let (_, v) = h
        .post_json(
            "/api/platform/v1/mcp",
            &h.muse,
            rpc("tools/list", json!({})),
        )
        .await;
    let tools: Vec<&str> = v["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    for t in [
        "how_to_deploy",
        "list_my_apps",
        "get_release",
        "get_app_logs",
        "rollback_app",
        "restart_app",
    ] {
        assert!(tools.contains(&t), "{t}");
    }
    assert!(!tools.contains(&"list_apps"));
    h.deploy(&h.muse, &manifest("trip"), &image_tar()).await;
    let (_, v) = h
        .post_json(
            "/api/platform/v1/mcp",
            &h.muse,
            rpc(
                "tools/call",
                json!({"name": "list_my_apps", "arguments": {}}),
            ),
        )
        .await;
    assert_eq!(v["result"]["structuredContent"]["apps"][0]["name"], "trip");
    let (_, v) = h
        .post_json(
            "/api/platform/v1/mcp",
            &h.other,
            rpc(
                "tools/call",
                json!({"name": "get_my_app", "arguments": {"name": "trip"}}),
            ),
        )
        .await;
    assert_eq!(v["result"]["isError"], true);
    let (_, v) = h
        .post_json(
            "/api/platform/v1/mcp",
            &h.muse,
            rpc(
                "tools/call",
                json!({"name": "how_to_deploy", "arguments": {}}),
            ),
        )
        .await;
    assert_eq!(
        v["result"]["structuredContent"]["content_type"],
        "multipart/form-data"
    );
    let (st, _) = h
        .post_json(
            "/api/platform/v1/mcp",
            "rbpub_x",
            rpc("tools/list", json!({})),
        )
        .await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
}
