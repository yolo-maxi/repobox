//! Publisher API: trusted first-party agents deploy and operate their own
//! apps over HTTPS with one credential.
//!
//! * `Authorization: Bearer rbpub_…` — a **publisher token** (not a service
//!   token, not OAuth): operator-issued, bound to one publisher (immutable
//!   `pub_…` id, handle, canonical owner record), expiring (≤ 30 days),
//!   revocable, hashed at rest. Every route below refuses anything else
//!   before reading a body.
//! * A publisher sees and changes only apps it created (`apps.publisher_id`).
//!   Apps of other publishers or operators are indistinguishable from
//!   missing ones; a deploy to a name held by anyone else is refused with one
//!   generic message.
//! * This service only validates and queues. The root deploy worker imports
//!   the uploaded archive, runs, health-gates and routes (see
//!   `publisher::worker`); the
//!   service has no Docker, Git, network egress or Caddy authority.

use std::time::Duration;

use axum::extract::{Path, Query as Q, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::S;
use crate::model::{AI_CHAT_PATH, AI_MODELS_PATH, App, User};
use crate::publisher::archive;
use crate::publisher::manifest;
use crate::publisher::spool::{self, Job, Op, Query, Spool};
use crate::publisher::{
    JOB_TIMEOUT_SECS, MAX_APPS_PER_PUBLISHER, MAX_RELEASES_PER_DAY, TOKEN_PREFIX,
};
use crate::store::{Publisher, PublisherToken, Release, StoreError};

/// Where the API finds the spool and what else counts as a taken name.
#[derive(Debug, Clone)]
pub struct PublisherConfig {
    pub spool: Spool,
    /// Caddy files whose site addresses are taken names (the Caddyfile and
    /// the operator-rendered apps; never the published routes).
    pub host_files: Vec<std::path::PathBuf>,
    /// Directories whose entries are taken names (legacy static subdomains).
    pub reserved_dirs: Vec<std::path::PathBuf>,
    /// Refuse uploads while the spool filesystem has less free space
    /// (a 2 GiB upload plus `docker load` needs room).
    pub min_free_bytes: u64,
}

pub const MAX_WAIT_SECS: u64 = 600;

// ------------------------------------------------------------------ errors

pub struct PubError {
    status: StatusCode,
    code: String,
    message: String,
    extra: Option<Value>,
}

impl PubError {
    fn new(status: StatusCode, code: &str, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            extra: None,
        }
    }
    fn with(mut self, extra: Value) -> Self {
        self.extra = Some(extra);
        self
    }
    fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "internal error",
        )
    }
    fn not_found(what: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("no such {what} for this publisher"),
        )
    }
    pub fn body(&self) -> Value {
        let mut e = json!({"code": self.code, "message": self.message});
        if let Some(x) = &self.extra
            && let (Some(m), Some(x)) = (e.as_object_mut(), x.as_object())
        {
            for (k, v) in x {
                m.insert(k.clone(), v.clone());
            }
        }
        json!({ "error": e })
    }
}

impl IntoResponse for PubError {
    fn into_response(self) -> Response {
        let mut r = json_response(self.status, &self.body());
        if self.status == StatusCode::UNAUTHORIZED {
            r.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"repo.box publisher\""),
            );
        }
        r
    }
}

fn store_err(e: StoreError) -> PubError {
    match e {
        StoreError::Conflict(m) => {
            let (code, msg) = m.split_once(": ").unwrap_or(("conflict", m.as_str()));
            let code = if code.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                code
            } else {
                "conflict"
            };
            PubError::new(StatusCode::CONFLICT, code, msg.to_string())
        }
        StoreError::Invalid(m) => PubError::new(StatusCode::BAD_REQUEST, "invalid", m),
        StoreError::NotFound => PubError::not_found("app"),
        StoreError::Db(_) => PubError::internal(),
    }
}

pub fn json_response(status: StatusCode, v: &Value) -> Response {
    let mut r = (status, v.to_string()).into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn reply(r: Result<(StatusCode, Value), PubError>) -> Response {
    match r {
        Ok((st, v)) => json_response(st, &v),
        Err(e) => e.into_response(),
    }
}

// -------------------------------------------------------------------- auth

pub struct PubCaller {
    pub token: PublisherToken,
    pub publisher: Publisher,
    pub owner: User,
}

impl PubCaller {
    fn audit(&self, s: &S, action: &str, subject: &str, detail: &str) {
        s.store.audit(
            Some(self.owner.id),
            action,
            subject,
            &format!(
                "publisher={} token={} {detail}",
                self.publisher.public_id, self.token.name
            ),
        );
    }
}

/// Exactly one `Authorization: Bearer rbpub_…` header. Nothing else (no
/// cookie, no identity header, no service token) authenticates here.
pub fn caller(s: &S, headers: &HeaderMap) -> Result<PubCaller, PubError> {
    let unauth = |m: &str| PubError::new(StatusCode::UNAUTHORIZED, "unauthorized", m);
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let (Some(v), None) = (values.next(), values.next()) else {
        return Err(unauth(&format!(
            "exactly one Authorization: Bearer rbpub_… header is required; docs: {}",
            super::docs::docs_url(&s.cfg.public_base)
        )));
    };
    let raw = v
        .to_str()
        .ok()
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .ok_or_else(|| {
            unauth(&format!(
                "use Authorization: Bearer rbpub_…; docs: {}",
                super::docs::docs_url(&s.cfg.public_base)
            ))
        })?;
    if !raw.starts_with(TOKEN_PREFIX) {
        return Err(unauth(&format!(
            "this endpoint needs a publisher token (rbpub_…); service tokens (rbp_…) cannot deploy; docs: {}",
            super::docs::docs_url(&s.cfg.public_base)
        )));
    }
    match s.store.publisher_token_lookup(raw) {
        Ok(Some((token, publisher, owner))) => Ok(PubCaller {
            token,
            publisher,
            owner,
        }),
        Ok(None) => Err(unauth(&format!(
            "publisher token is invalid, expired or revoked; docs: {}",
            super::docs::docs_url(&s.cfg.public_base)
        ))),
        Err(_) => Err(PubError::internal()),
    }
}

fn cfg(s: &S) -> Result<&PublisherConfig, PubError> {
    s.cfg.publisher.as_ref().ok_or_else(|| {
        PubError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "not_configured",
            "publishing is not configured on this control plane",
        )
    })
}

// -------------------------------------------------------- progress / sync

/// Fold worker progress into the registry. Cheap; called by every
/// publisher request and by a background tick.
pub fn sync(s: &S) {
    let Some(pc) = s.cfg.publisher.as_ref() else {
        return;
    };
    if let Ok(list) = s.store.releases_in_progress() {
        for r in list {
            if let Some(res) = pc.spool.result(&r.id) {
                let _ = s.store.apply_release_progress(&res);
            }
        }
    }
    let _ = s.store.fail_stale_releases();
}

async fn wait_for(s: &S, id: &str, secs: u64) -> Option<Release> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs.min(MAX_WAIT_SECS));
    loop {
        sync(s);
        let r = s.store.publisher_release(None, id).ok().flatten()?;
        if !r.in_progress() || tokio::time::Instant::now() >= deadline {
            return Some(r);
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

/// Is `name` served or reserved outside the registry (a legacy Caddy site
/// or static subdomain directory)?
fn taken_outside_registry(s: &S, pc: &PublisherConfig, name: &str) -> bool {
    let host = format!("{name}.{}", s.cfg.domain);
    for f in &pc.host_files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for line in text.lines() {
            if line.starts_with([' ', '\t', '#']) || !line.trim_end().ends_with('{') {
                continue;
            }
            let addrs = line.trim_end().trim_end_matches('{');
            for a in addrs.split([',', ' ']).filter(|a| !a.is_empty()) {
                let a = a.split("://").last().unwrap_or(a);
                let a = a.split([':', '/']).next().unwrap_or(a);
                if a.eq_ignore_ascii_case(&host) {
                    return true;
                }
            }
        }
    }
    pc.reserved_dirs.iter().any(|d| d.join(name).exists())
}

// ------------------------------------------------------------------- JSON

pub fn launcher_url(s: &S, name: &str) -> String {
    format!("{}/{name}", s.cfg.public_base)
}

fn api(s: &S, path: &str) -> String {
    format!("{}/api/platform/v1/publisher{path}", s.cfg.public_base)
}

pub fn release_json(s: &S, p: &Publisher, r: &Release) -> Value {
    let m = r.manifest();
    let failure = if r.status == "failed" {
        json!({"code": r.failure_code, "message": r.failure})
    } else {
        Value::Null
    };
    json!({
        "id": r.id,
        "app": r.app_name,
        "version": r.version,
        "label": m.as_ref().map(|m| m.version.clone()).unwrap_or_default(),
        "op": r.op,
        "rollback_of": r.rollback_of,
        "status": r.status,
        "done": !r.in_progress(),
        "artifact": {
            "sha256": r.artifact_sha256,
            "bytes": r.artifact_bytes,
            "format": r.build_mode,
            "image_id": r.image_id,
        },
        "provenance": m.as_ref().map(|m| json!({
            "repository": m.provenance.repository,
            "commit": m.provenance.commit,
            "note": m.provenance.note,
        })),
        "runtime": m.as_ref().map(|m| json!({
            "port": m.runtime.port,
            "health_path": m.runtime.health_path,
            "memory_mb": m.runtime.memory_mb,
            "env": m.runtime.env.keys().collect::<Vec<_>>(),
        })),
        "failure": failure,
        "retained_for_rollback": r.retained,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
        "finished_at": r.finished_at,
        "publisher": {"id": p.public_id, "handle": p.handle},
        "links": {
            "self": api(s, &format!("/releases/{}", r.id)),
            "build_log": api(s, &format!("/releases/{}/log", r.id)),
            "app": api(s, &format!("/apps/{}", r.app_name)),
        },
    })
}

pub fn app_json(s: &S, c: &PubCaller, app: &App) -> Value {
    let live = s.store.live_release(&app.name).ok().flatten();
    let latest = s
        .store
        .publisher_releases(Some(c.publisher.id), Some(&app.name), 1)
        .ok()
        .and_then(|v| v.into_iter().next());
    let status = if !app.enabled {
        "disabled"
    } else if latest.as_ref().is_some_and(|r| r.in_progress()) {
        "deploying"
    } else if live.is_some() {
        "live"
    } else if latest.as_ref().is_some_and(|r| r.status == "failed") {
        "failed"
    } else {
        "not_deployed"
    };
    let host = app.host(&s.cfg.domain);
    json!({
        "name": app.name,
        "title": app.title,
        "description": app.description,
        "status": status,
        "owner": {"publisher_id": c.publisher.public_id, "publisher": c.publisher.handle, "platform_user": c.owner.name},
        "launcher_url": launcher_url(s, &app.name),
        "direct_url": format!("https://{host}/"),
        "direct_url_access": "edge-gated: private app, platform identity only; anonymous requests get 401. Share launcher_url; people sign in on auth.repo.box and are redirected with a one-time code.",
        "visibility": app.visibility.as_str(),
        "identity": app.identity.as_str(),
        "enabled": app.enabled,
        "ai": {
            "policy": app.ai.to_json(),
            "chat_completions_url": format!("https://{host}{AI_CHAT_PATH}"),
            "models_url": format!("https://{host}{AI_MODELS_PATH}"),
            "call_from": "the app's own pages (same origin, the viewer's platform session); no API key",
        },
        "current_release": live.as_ref().map(|r| release_json(s, &c.publisher, r)),
        "latest_release": latest.as_ref().map(|r| release_json(s, &c.publisher, r)),
        "links": {
            "self": api(s, &format!("/apps/{}", app.name)),
            "releases": api(s, &format!("/releases?app={}", app.name)),
            "logs": api(s, &format!("/apps/{}/logs", app.name)),
            "rollback": api(s, &format!("/apps/{}/rollback", app.name)),
            "restart": api(s, &format!("/apps/{}/restart", app.name)),
        },
    })
}

fn release_reply(s: &S, c: &PubCaller, r: &Release, app: &App) -> (StatusCode, Value) {
    let st = if r.in_progress() {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    (
        st,
        json!({
            "release": release_json(s, &c.publisher, r),
            "app": app_json(s, c, app),
            "next": if r.in_progress() {
                format!("poll GET {} (add ?wait=120 to long-poll) until release.done; build output at release.links.build_log", api(s, &format!("/releases/{}", r.id)))
            } else if r.status == "failed" {
                "read release.failure and release.links.build_log, fix, and deploy again".to_string()
            } else {
                format!("open {} in a browser (platform sign-in); runtime logs at app.links.logs", launcher_url(s, &r.app_name))
            },
        }),
    )
}

// -------------------------------------------------------------- operations

/// What a deploy request looks like; returned with every malformed request.
pub fn upload_contract(s: &S) -> Value {
    json!({
        "docs": format!("{}#publish", super::docs::docs_url(&s.cfg.public_base)),
        "method": "POST",
        "url": format!("{}/api/platform/v1/publisher/releases", s.cfg.public_base),
        "content_type": "multipart/form-data",
        "parts": [
            {"name": "manifest", "order": 1, "content": "JSON release manifest (see manifest)"},
            {"name": "image", "order": 2, "content": "the output of `docker save <image>` (tar, or gzip-compressed tar), exactly one linux/amd64 image, at most 2 GiB"},
        ],
        "manifest": {
            "minimal": {"name": "trip-planner", "title": "Trip planner"},
            "fields": {
                "name": "DNS label a-z0-9- (becomes https://<name>.repo.box); required",
                "title": "1-80 chars; required",
                "description": "optional, <= 500 chars",
                "version": "optional label of your own",
                "runtime": "{port: container port (default: the image's single EXPOSE, else 8080; also $PORT), health_path (default /), memory_mb 64-1024 (default 512), env {NAME: value} (non-secret)}",
                "ai": "boolean, default true (new apps only)",
                "provenance": "{repository, commit, note} recorded only",
            },
        },
        "example": "docker build --platform linux/amd64 -t trip-planner . && docker save trip-planner | gzip > trip-planner.tar.gz && curl -fsS -H \"Authorization: Bearer $REPOBOX_PUBLISHER_TOKEN\" -F 'manifest={\"name\":\"trip-planner\",\"title\":\"Trip planner\",\"runtime\":{\"port\":3000,\"health_path\":\"/healthz\"}};type=application/json' -F image=@trip-planner.tar.gz 'https://auth.repo.box/api/platform/v1/publisher/releases?wait=300'",
    })
}

fn free_bytes(dir: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes()).ok()?;
    // SAFETY: statvfs fills the zeroed struct for a valid NUL-terminated path.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    Some(st.f_bavail as u64 * st.f_frsize as u64)
}

/// Default for `PublisherConfig::min_free_bytes`.
pub const MIN_FREE_BYTES: u64 = 8 * 1024 * 1024 * 1024;

async fn op_upload(
    s: &S,
    c: &PubCaller,
    mp: &mut axum::extract::Multipart,
    wait: u64,
) -> Result<(StatusCode, Value), PubError> {
    let pc = cfg(s)?;
    let contract = || json!({"expected": upload_contract(s)});
    let mp_err = |e: axum::extract::multipart::MultipartError| {
        PubError::new(
            e.status(),
            "bad_request",
            format!("multipart body: {}", e.body_text()),
        )
        .with(contract())
    };
    // 1. the manifest, before a single image byte is accepted
    let field = mp.next_field().await.map_err(mp_err)?.ok_or_else(|| {
        PubError::new(
            StatusCode::BAD_REQUEST,
            "missing_manifest",
            "the first part must be 'manifest'",
        )
        .with(contract())
    })?;
    if field.name() != Some("manifest") {
        return Err(PubError::new(
            StatusCode::BAD_REQUEST,
            "missing_manifest",
            "the first part must be 'manifest' (then 'image')",
        )
        .with(contract()));
    }
    let mut raw = Vec::new();
    let mut field = field;
    while let Some(chunk) = field.chunk().await.map_err(mp_err)? {
        raw.extend_from_slice(&chunk);
        if raw.len() > manifest::MAX_MANIFEST_BYTES {
            return Err(PubError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "too_large",
                "the manifest is too large",
            ));
        }
    }
    drop(field);
    let m = manifest::parse(&raw)
        .map_err(|e| PubError::new(StatusCode::BAD_REQUEST, e.code, e.message).with(contract()))?;
    let own = s
        .store
        .publisher_app(c.publisher.id, &m.name)
        .map_err(|_| PubError::internal())?;
    let held_elsewhere = own.is_none()
        && (s
            .store
            .app_by_name(&m.name)
            .map_err(|_| PubError::internal())?
            .is_some()
            || taken_outside_registry(s, pc, &m.name));
    if held_elsewhere {
        return Err(store_err(StoreError::Conflict(
            crate::store::NAME_UNAVAILABLE.into(),
        )));
    }
    sync(s);
    if let Some(busy) = s
        .store
        .release_in_progress(&m.name)
        .map_err(|_| PubError::internal())?
    {
        return Err(PubError::new(
            StatusCode::CONFLICT,
            "release_in_progress",
            format!("release {busy} of this app is still in progress; wait for it to finish"),
        ));
    }
    if free_bytes(&pc.spool.uploads()).is_some_and(|f| f < pc.min_free_bytes) {
        return Err(PubError::new(
            StatusCode::INSUFFICIENT_STORAGE,
            "no_space",
            "the platform is low on disk; try later or ask an operator",
        ));
    }
    // 2. the image archive, streamed to the spool
    let mut field = mp.next_field().await.map_err(mp_err)?.ok_or_else(|| {
        PubError::new(
            StatusCode::BAD_REQUEST,
            "missing_image",
            "the second part must be 'image' (docker save output)",
        )
        .with(contract())
    })?;
    if field.name() != Some("image") {
        return Err(PubError::new(
            StatusCode::BAD_REQUEST,
            "missing_image",
            "the second part must be 'image' (docker save output)",
        )
        .with(contract()));
    }
    let id = spool::new_id("rel", s.store.now());
    let part = pc.spool.uploads().join(format!(".{id}.part"));
    let written = async {
        use sha2::Digest;
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&part)
            .await
            .map_err(|_| PubError::internal())?;
        let mut h = sha2::Sha256::new();
        let mut n: u64 = 0;
        let mut head: Vec<u8> = Vec::new();
        while let Some(chunk) = field.chunk().await.map_err(mp_err)? {
            n += chunk.len() as u64;
            if n > archive::MAX_UPLOAD_BYTES {
                return Err(PubError::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "too_large",
                    "the image archive is larger than 2 GiB",
                ));
            }
            if head.len() < 512 {
                head.extend_from_slice(&chunk[..chunk.len().min(512 - head.len())]);
            }
            h.update(&chunk);
            f.write_all(&chunk)
                .await
                .map_err(|_| PubError::internal())?;
        }
        f.sync_all().await.map_err(|_| PubError::internal())?;
        let gzip = archive::sniff(&head).ok_or_else(|| {
            PubError::new(
                StatusCode::BAD_REQUEST,
                "bad_archive",
                "'image' must be the output of `docker save` (a tar, optionally gzip-compressed)",
            )
            .with(contract())
        })?;
        Ok::<_, PubError>((format!("sha256:{}", archive::hex(&h.finalize())), n, gzip))
    }
    .await;
    drop(field);
    let cleanup = || {
        let _ = std::fs::remove_file(&part);
    };
    let (sha256, bytes, gzip) = match written {
        Ok(v) => v,
        Err(e) => {
            cleanup();
            return Err(e);
        }
    };
    if !matches!(mp.next_field().await, Ok(None)) {
        cleanup();
        return Err(PubError::new(
            StatusCode::BAD_REQUEST,
            "unexpected_part",
            "send exactly two parts: manifest, image",
        )
        .with(contract()));
    }
    let (app, release) =
        match s
            .store
            .begin_deploy(&c.publisher, &c.token.name, &m, &id, (&sha256, bytes))
        {
            Ok(v) => v,
            Err(e) => {
                cleanup();
                return Err(store_err(e));
            }
        };
    if std::fs::rename(&part, pc.spool.upload_path(&id)).is_err() {
        cleanup();
        fail_release(s, &id, &m.name);
        return Err(PubError::internal());
    }
    queue(
        s,
        pc,
        &Job {
            v: 1,
            op: Op::Deploy,
            app: m.name.clone(),
            id: id.clone(),
            manifest: Some(m.clone()),
            upload: Some(spool::Upload {
                sha256: sha256.clone(),
                bytes,
                gzip,
            }),
            target: String::new(),
            purge_data: false,
            created_at: s.store.now(),
        },
    )?;
    c.audit(
        s,
        if own.is_some() {
            "publisher.update"
        } else {
            "publisher.create"
        },
        &m.name,
        &format!("release={id} artifact={sha256} bytes={bytes}"),
    );
    tracing::info!(
        "publisher deploy app={} release={id} publisher={} bytes={bytes}",
        m.name,
        c.publisher.public_id
    );
    let release = if wait > 0 {
        wait_for(s, &id, wait).await.unwrap_or(release)
    } else {
        release
    };
    let app = s.store.app_by_id(app.id).unwrap_or(app);
    Ok(release_reply(s, c, &release, &app))
}

fn fail_release(s: &S, id: &str, app: &str) {
    let _ = s.store.apply_release_progress(&spool::JobResult {
        id: id.to_string(),
        app: app.to_string(),
        state: "failed".into(),
        code: "internal".into(),
        failure: "the release could not be queued; try again".into(),
        updated_at: s.store.now(),
        finished_at: Some(s.store.now()),
        ..Default::default()
    });
}

fn queue(s: &S, pc: &PublisherConfig, job: &Job) -> Result<(), PubError> {
    if let Err(e) = pc.spool.enqueue(job) {
        tracing::error!("publisher spool write failed for {}: {e}", job.id);
        let _ = std::fs::remove_file(pc.spool.upload_path(&job.id));
        fail_release(s, &job.id, &job.app);
        return Err(PubError::internal());
    }
    Ok(())
}

fn own_app(s: &S, c: &PubCaller, name: &str) -> Result<App, PubError> {
    s.store
        .publisher_app(c.publisher.id, name)
        .map_err(|_| PubError::internal())?
        .ok_or_else(|| PubError::not_found("app"))
}

async fn op_rollback(
    s: &S,
    c: &PubCaller,
    name: &str,
    body: &Value,
    wait: u64,
) -> Result<(StatusCode, Value), PubError> {
    let pc = cfg(s)?;
    let app = own_app(s, c, name)?;
    sync(s);
    let wanted = body.get("release").and_then(Value::as_str);
    if let Some(k) = body
        .as_object()
        .and_then(|o| o.keys().find(|k| *k != "release"))
    {
        return Err(PubError::new(
            StatusCode::BAD_REQUEST,
            "invalid",
            format!("unknown field '{k}' (allowed: release)"),
        ));
    }
    let releases = s
        .store
        .publisher_releases(Some(c.publisher.id), Some(&app.name), 100)
        .map_err(|_| PubError::internal())?;
    let target = match wanted {
        Some(id) => releases.into_iter().find(|r| r.id == id),
        None => releases
            .into_iter()
            .find(|r| r.retained && r.status == "superseded"),
    }
    .ok_or_else(|| PubError::not_found("retained release"))?;
    if !target.retained || !matches!(target.status.as_str(), "superseded" | "live") {
        return Err(PubError::new(
            StatusCode::CONFLICT,
            "release_not_retained",
            "only retained, previously live releases can be rolled back to",
        ));
    }
    let id = spool::new_id("rel", s.store.now());
    let r = s
        .store
        .begin_app_op(
            &c.publisher,
            &c.token.name,
            &app,
            "rollback",
            Some(&target),
            &id,
        )
        .map_err(store_err)?;
    queue(
        s,
        pc,
        &Job {
            v: 1,
            op: Op::Rollback,
            app: app.name.clone(),
            id: id.clone(),
            manifest: None,
            upload: None,
            target: target.id.clone(),
            purge_data: false,
            created_at: s.store.now(),
        },
    )?;
    c.audit(
        s,
        "publisher.rollback",
        &app.name,
        &format!("release={id} to={}", target.id),
    );
    let r = if wait > 0 {
        wait_for(s, &id, wait).await.unwrap_or(r)
    } else {
        r
    };
    Ok(release_reply(s, c, &r, &app))
}

async fn op_restart(
    s: &S,
    c: &PubCaller,
    name: &str,
    wait: u64,
) -> Result<(StatusCode, Value), PubError> {
    let pc = cfg(s)?;
    let app = own_app(s, c, name)?;
    sync(s);
    let id = spool::new_id("rel", s.store.now());
    let r = s
        .store
        .begin_app_op(&c.publisher, &c.token.name, &app, "restart", None, &id)
        .map_err(store_err)?;
    queue(
        s,
        pc,
        &Job {
            v: 1,
            op: Op::Restart,
            app: app.name.clone(),
            id: id.clone(),
            manifest: None,
            upload: None,
            target: String::new(),
            purge_data: false,
            created_at: s.store.now(),
        },
    )?;
    c.audit(s, "publisher.restart", &app.name, &format!("release={id}"));
    let r = if wait > 0 {
        wait_for(s, &id, wait).await.unwrap_or(r)
    } else {
        r
    };
    Ok(release_reply(s, c, &r, &app))
}

async fn op_logs(
    s: &S,
    c: &PubCaller,
    name: &str,
    tail: u32,
) -> Result<(StatusCode, Value), PubError> {
    let pc = cfg(s)?;
    let app = own_app(s, c, name)?;
    let id = spool::new_id("q", s.store.now());
    pc.spool
        .enqueue_query(&Query {
            v: 1,
            id: id.clone(),
            app: app.name.clone(),
            tail: tail.clamp(1, 2000),
            created_at: s.store.now(),
        })
        .map_err(|_| PubError::internal())?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(r) = pc.spool.query_result(&id) {
            return Ok((
                StatusCode::OK,
                json!({
                    "app": app.name,
                    "release": r.release,
                    "container": {"state": r.state, "started_at": r.started_at, "restarts": r.restarts},
                    "logs": r.logs,
                }),
            ));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(PubError::new(
                StatusCode::GATEWAY_TIMEOUT,
                "busy",
                "the runtime did not answer in time; retry",
            ));
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

fn op_whoami(s: &S, c: &PubCaller) -> Value {
    let apps = s.store.publisher_apps(c.publisher.id).unwrap_or_default();
    json!({
        "publisher": {"id": c.publisher.public_id, "handle": c.publisher.handle, "display_name": c.publisher.display_name},
        "owner_record": c.owner.name,
        "token": {"name": c.token.name, "expires_at": c.token.expires_at},
        "apps": apps.iter().map(|a| a.name.clone()).collect::<Vec<_>>(),
        "limits": {"apps": MAX_APPS_PER_PUBLISHER, "releases_per_24h": MAX_RELEASES_PER_DAY, "release_timeout_secs": JOB_TIMEOUT_SECS},
        "can": ["deploy new apps (docker save upload)", "update/rollback/restart its own apps", "read its own releases, deploy logs and runtime logs"],
        "cannot": ["see or change apps it did not create", "grant access", "change visibility", "SSH, Docker daemon, Caddy or database access", "provider credentials"],
    })
}

fn op_apps(s: &S, c: &PubCaller) -> Result<(StatusCode, Value), PubError> {
    sync(s);
    let apps = s
        .store
        .publisher_apps(c.publisher.id)
        .map_err(|_| PubError::internal())?;
    Ok((
        StatusCode::OK,
        json!({"apps": apps.iter().map(|a| app_json(s, c, a)).collect::<Vec<_>>()}),
    ))
}

fn op_app(s: &S, c: &PubCaller, name: &str) -> Result<(StatusCode, Value), PubError> {
    sync(s);
    let app = own_app(s, c, name)?;
    Ok((StatusCode::OK, app_json(s, c, &app)))
}

fn op_releases(
    s: &S,
    c: &PubCaller,
    app: Option<&str>,
    limit: i64,
) -> Result<(StatusCode, Value), PubError> {
    sync(s);
    let list = s
        .store
        .publisher_releases(Some(c.publisher.id), app, limit)
        .map_err(|_| PubError::internal())?;
    Ok((
        StatusCode::OK,
        json!({"releases": list.iter().map(|r| release_json(s, &c.publisher, r)).collect::<Vec<_>>()}),
    ))
}

async fn op_release(
    s: &S,
    c: &PubCaller,
    id: &str,
    wait: u64,
) -> Result<(StatusCode, Value), PubError> {
    sync(s);
    let r = s
        .store
        .publisher_release(Some(c.publisher.id), id)
        .map_err(|_| PubError::internal())?
        .ok_or_else(|| PubError::not_found("release"))?;
    let r = if wait > 0 && r.in_progress() {
        wait_for(s, id, wait).await.unwrap_or(r)
    } else {
        r
    };
    let app = own_app(s, c, &r.app_name)?;
    Ok(release_reply(s, c, &r, &app))
}

// ---------------------------------------------------------------- handlers

#[derive(serde::Deserialize, Default)]
pub struct WaitQ {
    #[serde(default)]
    wait: Option<u64>,
    #[serde(default)]
    app: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    tail: Option<u32>,
}

/// Strict JSON body: `application/json` and an object. An empty body is
/// `{}` only where `optional`.
fn json_body(headers: &HeaderMap, body: &bytes::Bytes, optional: bool) -> Result<Value, PubError> {
    if body.is_empty() && optional {
        return Ok(json!({}));
    }
    if body.len() > manifest::MAX_MANIFEST_BYTES {
        return Err(PubError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_large",
            format!(
                "the manifest must be at most {} bytes",
                manifest::MAX_MANIFEST_BYTES
            ),
        ));
    }
    let ct = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if ct.split(';').next().unwrap_or("").trim() != "application/json" {
        return Err(PubError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "send Content-Type: application/json (or no body)",
        )
        .with(json!({"expected": {"content_type": "application/json", "example": {"release": "rel-20260927120000-0a1b2c3d"}}})));
    }
    let v: Value = serde_json::from_slice(body).map_err(|e| {
        PubError::new(StatusCode::BAD_REQUEST, "invalid_json", format!("body is not valid JSON ({e})"))
            .with(json!({"expected": {"content_type": "application/json", "example": {"release": "rel-20260927120000-0a1b2c3d"}}}))
    })?;
    if !v.is_object() {
        return Err(PubError::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "body must be a JSON object",
        ));
    }
    Ok(v)
}

pub async fn deploy(State(s): State<S>, Q(q): Q<WaitQ>, req: axum::extract::Request) -> Response {
    use axum::extract::FromRequest;
    let c = match caller(&s, req.headers()) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let is_multipart = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("multipart/form-data"));
    if !is_multipart {
        return PubError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "send multipart/form-data with a 'manifest' part and an 'image' part (docker save output)",
        )
        .with(json!({"expected": upload_contract(&s)}))
        .into_response();
    }
    let mut mp = match axum::extract::Multipart::from_request(req, &s).await {
        Ok(mp) => mp,
        Err(e) => {
            return PubError::new(StatusCode::BAD_REQUEST, "bad_request", e.body_text())
                .into_response();
        }
    };
    let r = op_upload(&s, &c, &mut mp, q.wait.unwrap_or(0)).await;
    if r.is_err() {
        // Read the rest of the (authenticated) upload before answering: a
        // reverse proxy still streaming the image would otherwise turn the
        // refusal into a 502.
        while let Ok(Some(mut f)) = mp.next_field().await {
            while let Ok(Some(_)) = f.chunk().await {}
        }
    }
    reply(r)
}

pub async fn releases(State(s): State<S>, headers: HeaderMap, Q(q): Q<WaitQ>) -> Response {
    reply(
        caller(&s, &headers)
            .and_then(|c| op_releases(&s, &c, q.app.as_deref(), q.limit.unwrap_or(50))),
    )
}

pub async fn release(
    State(s): State<S>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Q(q): Q<WaitQ>,
) -> Response {
    match caller(&s, &headers) {
        Ok(c) => reply(op_release(&s, &c, &id, q.wait.unwrap_or(0)).await),
        Err(e) => e.into_response(),
    }
}

pub async fn release_log(
    State(s): State<S>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let c = match caller(&s, &headers) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    let found = s
        .store
        .publisher_release(Some(c.publisher.id), &id)
        .ok()
        .flatten();
    let (Some(_), Some(pc)) = (found, s.cfg.publisher.as_ref()) else {
        return PubError::not_found("release").into_response();
    };
    let text = pc
        .spool
        .build_log(&id)
        .unwrap_or_else(|| "(no build output yet)\n".into());
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        text,
    )
        .into_response()
}

pub async fn apps(State(s): State<S>, headers: HeaderMap) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_apps(&s, &c)))
}

pub async fn app(State(s): State<S>, headers: HeaderMap, Path(name): Path<String>) -> Response {
    reply(caller(&s, &headers).and_then(|c| op_app(&s, &c, &name)))
}

pub async fn logs(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Q(q): Q<WaitQ>,
) -> Response {
    match caller(&s, &headers) {
        Ok(c) => reply(op_logs(&s, &c, &name, q.tail.unwrap_or(200)).await),
        Err(e) => e.into_response(),
    }
}

pub async fn rollback(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Q(q): Q<WaitQ>,
    body: bytes::Bytes,
) -> Response {
    let c = match caller(&s, &headers) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };
    match json_body(&headers, &body, true) {
        Ok(v) => reply(op_rollback(&s, &c, &name, &v, q.wait.unwrap_or(0)).await),
        Err(e) => e.into_response(),
    }
}

pub async fn restart(
    State(s): State<S>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Q(q): Q<WaitQ>,
) -> Response {
    match caller(&s, &headers) {
        Ok(c) => reply(op_restart(&s, &c, &name, q.wait.unwrap_or(0)).await),
        Err(e) => e.into_response(),
    }
}

pub async fn whoami(State(s): State<S>, headers: HeaderMap) -> Response {
    reply(caller(&s, &headers).map(|c| (StatusCode::OK, op_whoami(&s, &c))))
}

// --------------------------------------------------------------------- MCP

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": props, "required": required, "additionalProperties": false},
    })
}

pub fn mcp_tools() -> Vec<Value> {
    let name = json!({"name": {"type": "string", "description": "app name (DNS label)"}});
    let wait = json!({"type": "integer", "minimum": 0, "maximum": MAX_WAIT_SECS, "description": "seconds to wait for the release to finish (default 0)"});
    vec![
        tool(
            "publisher_whoami",
            "Show this publisher, its apps, limits and what it can/cannot do",
            json!({}),
            &[],
        ),
        tool(
            "how_to_deploy",
            "How to deploy or update an app: an HTTPS multipart upload of `docker save` output plus a small JSON manifest (MCP carries no image bytes). Returns the exact request.",
            json!({}),
            &[],
        ),
        tool(
            "list_my_apps",
            "List apps this publisher created, with status, launcher URL and current release",
            json!({}),
            &[],
        ),
        tool(
            "get_my_app",
            "One of this publisher's apps",
            name.clone(),
            &["name"],
        ),
        tool(
            "list_releases",
            "Releases of this publisher (optionally one app), newest first",
            json!({"app": {"type": "string"}, "limit": {"type": "integer", "minimum": 1, "maximum": 500}}),
            &[],
        ),
        tool(
            "get_release",
            "One release; wait_seconds long-polls until it is done",
            json!({"id": {"type": "string"}, "wait_seconds": wait}),
            &["id"],
        ),
        tool(
            "get_build_log",
            "Build/deploy log of a release (text)",
            json!({"id": {"type": "string"}}),
            &["id"],
        ),
        tool(
            "get_app_logs",
            "Runtime logs and container state of the app's live release",
            json!({"name": {"type": "string"}, "tail": {"type": "integer", "minimum": 1, "maximum": 2000}}),
            &["name"],
        ),
        tool(
            "rollback_app",
            "Run a retained earlier release again (default: the most recent previous one)",
            json!({"name": {"type": "string"}, "release": {"type": "string"}, "wait_seconds": wait}),
            &["name"],
        ),
        tool(
            "restart_app",
            "Restart the live release's container",
            json!({"name": {"type": "string"}, "wait_seconds": wait}),
            &["name"],
        ),
    ]
}

/// Dispatch one MCP tool call for a publisher caller.
pub async fn mcp_call(
    s: &S,
    c: &PubCaller,
    tool: &str,
    args: &Value,
) -> Option<Result<Value, Value>> {
    let str_arg = |k: &str| {
        args.get(k)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let wait = args
        .get("wait_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let r: Result<(StatusCode, Value), PubError> = match tool {
        "publisher_whoami" => Ok((StatusCode::OK, op_whoami(s, c))),
        "how_to_deploy" => Ok((StatusCode::OK, upload_contract(s))),
        "list_my_apps" => op_apps(s, c),
        "get_my_app" => op_app(s, c, &str_arg("name")),
        "list_releases" => {
            let app = str_arg("app");
            op_releases(
                s,
                c,
                (!app.is_empty()).then_some(app.as_str()),
                args.get("limit").and_then(Value::as_i64).unwrap_or(50),
            )
        }
        "get_release" => op_release(s, c, &str_arg("id"), wait).await,
        "get_build_log" => {
            let id = str_arg("id");
            match (
                s.store
                    .publisher_release(Some(c.publisher.id), &id)
                    .ok()
                    .flatten(),
                s.cfg.publisher.as_ref(),
            ) {
                (Some(_), Some(pc)) => Ok((
                    StatusCode::OK,
                    json!({"id": id, "log": pc.spool.build_log(&id).unwrap_or_default()}),
                )),
                _ => Err(PubError::not_found("release")),
            }
        }
        "get_app_logs" => {
            op_logs(
                s,
                c,
                &str_arg("name"),
                args.get("tail").and_then(Value::as_u64).unwrap_or(200) as u32,
            )
            .await
        }
        "rollback_app" => {
            let b = match args.get("release") {
                Some(v) => json!({"release": v}),
                None => json!({}),
            };
            op_rollback(s, c, &str_arg("name"), &b, wait).await
        }
        "restart_app" => op_restart(s, c, &str_arg("name"), wait).await,
        _ => return None,
    };
    Some(r.map(|(_, v)| v).map_err(|e| e.body()))
}

// --------------------------------------------------------------- discovery

/// The `publishing` section of the discovery document: everything an
/// external agent needs besides its token.
pub fn discovery(s: &S) -> Value {
    let base = &s.cfg.public_base;
    let p = |path: &str| format!("{base}/api/platform/v1/publisher{path}");
    json!({
        "docs": format!("{}#publish", super::docs::docs_url(base)),
        "summary": "Deploy your own private app to https://<name>.repo.box by uploading a Docker image archive (`docker save`) with a small JSON manifest in one HTTPS request. No registry, Git host, SSH or Docker access needed.",
        "available": s.cfg.publisher.is_some(),
        "credential": {
            "type": "publisher_token",
            "scheme": "Authorization: Bearer rbpub_<43 base64url chars>",
            "issued_by": "an operator (`repobox-platform publisher token create`), written once to a 0600 file; expires within 30 days; revocable",
            "binding": "one publisher principal (immutable pub_… id + handle); it owns and can see only the apps it creates",
            "not": "not a service token (rbp_…), not a browser session, not OAuth. OAuth client-credentials issuance is a possible future replacement.",
        },
        "recipe": [
            "docker build --platform linux/amd64 -t myapp .   # any Dockerfile; the app listens on 0.0.0.0:$PORT (or the port you declare)",
            "docker save myapp | gzip > myapp.tar.gz          # or: docker save -o myapp.tar myapp / podman save --format docker-archive -o myapp.tar myapp",
            format!("curl -fsS -H \"Authorization: Bearer $REPOBOX_PUBLISHER_TOKEN\" -F 'manifest={{\"name\":\"myapp\",\"title\":\"My app\",\"runtime\":{{\"port\":8080,\"health_path\":\"/healthz\"}}}};type=application/json' -F image=@myapp.tar.gz \"{base}/api/platform/v1/publisher/releases?wait=300\""),
            "read release.status (live/failed) and app.launcher_url from the answer; if still queued, GET release.links.self?wait=300",
            "open app.launcher_url in a browser (platform sign-in); updates: repeat the upload with the same name; rollback/restart/logs via app.links",
        ],
        "upload": upload_contract(s),
        "archive_formats": [
            "docker save output (tar), optionally gzip-compressed (Docker 25+ writes Docker + OCI index metadata; both are read)",
            "podman save --format docker-archive or --format oci-archive (tar)",
            "OCI image layout tar, e.g. docker buildx build --platform linux/amd64 --provenance=false --output type=oci,dest=myapp.tar .",
            "exactly one image, linux/amd64, at most 2 GiB uploaded",
        ],
        "runtime_contract": {
            "network": "the container is reachable only by the platform edge over the host loopback; listen on 0.0.0.0:$PORT inside the container",
            "port": "runtime.port, else the image's single EXPOSEd TCP port, else 8080; passed as $PORT",
            "health": "a release goes live only after GET runtime.health_path answers (2xx/3xx; any non-5xx when it is the default /) within 120 s; the previous release keeps serving until then",
            "identity": "every request carries X-RepoBox-User-Id / X-RepoBox-User / X-RepoBox-Role / X-RepoBox-Auth injected by the edge; the app has no login of its own; key records on X-RepoBox-User-Id",
            "ai": "same-origin POST /_repo_box/ai/v1/chat/completions from the app's pages (OpenAI chat.completions subset, non-streaming); no key; on by default",
            "data": "a persistent volume at /data (env REPOBOX_DATA_DIR) survives updates and rollbacks",
            "env": "PORT, REPOBOX_APP, REPOBOX_APP_URL, REPOBOX_LAUNCHER_URL, REPOBOX_AI_CHAT_PATH, REPOBOX_RELEASE, REPOBOX_DATA_DIR, plus runtime.env",
            "limits": "memory_mb (64-1024, default 512), 1 CPU, 512 processes; no privileged mode, host network, host ports or host mounts",
            "visibility": "private: only people an operator grants (and admins) can open it; the publisher cannot change visibility or grants",
        },
        "endpoints": {
            "deploy": format!("POST {} (multipart: manifest, image)", p("/releases")),
            "releases": format!("GET {}?app=NAME", p("/releases")),
            "release": format!("GET {}?wait=SECONDS", p("/releases/{id}")),
            "deploy_log": format!("GET {}", p("/releases/{id}/log")),
            "apps": format!("GET {}", p("/apps")),
            "app": format!("GET {}", p("/apps/{name}")),
            "runtime_logs": format!("GET {}?tail=200", p("/apps/{name}/logs")),
            "rollback": format!("POST {} (optional JSON {{\"release\": \"rel-…\"}})", p("/apps/{name}/rollback")),
            "restart": format!("POST {}", p("/apps/{name}/restart")),
            "whoami": format!("GET {}", p("/whoami")),
            "mcp": format!("POST {base}/api/platform/v1/mcp (same bearer; publisher tools)"),
        },
        "release_statuses": ["queued", "building (importing the image)", "starting (health check)", "live", "superseded", "failed", "done (restart)"],
        "limits": {
            "apps_per_publisher": MAX_APPS_PER_PUBLISHER,
            "releases_per_24h": MAX_RELEASES_PER_DAY,
            "upload_max_bytes": archive::MAX_UPLOAD_BYTES,
            "retained_releases_per_app": crate::publisher::worker::DEFAULT_KEEP,
        },
        "mcp_tools": mcp_tools().iter().map(|t| t["name"].clone()).collect::<Vec<_>>(),
        "not_in_v1": ["registry pulls or Git builds (upload the image instead)", "secrets management (do not bake secrets into images)", "custom domains", "public visibility (operator decision)"],
    })
}

/// OpenAPI path items of the publisher API.
pub fn openapi_paths() -> Value {
    let err = json!({"$ref": "#/components/schemas/Error"});
    let std_errors = json!({
        "401": {"description": "missing, wrong-kind, expired or revoked publisher token", "content": {"application/json": {"schema": err}}},
        "404": {"description": "no such app/release for this publisher", "content": {"application/json": {"schema": err}}},
    });
    let op = |summary: &str, extra: Value| {
        let mut o = json!({
            "summary": summary,
            "tags": ["publisher"],
            "security": [{"publisherToken": []}],
            "responses": {"200": {"description": "OK", "content": {"application/json": {}}}},
        });
        for (k, v) in std_errors.as_object().unwrap() {
            o["responses"][k] = v.clone();
        }
        if let Value::Object(m) = extra {
            for (k, v) in m {
                o[k] = v;
            }
        }
        o
    };
    let name =
        json!({"name": "name", "in": "path", "required": true, "schema": {"type": "string"}});
    let id = json!({"name": "id", "in": "path", "required": true, "schema": {"type": "string"}});
    let wait = json!({"name": "wait", "in": "query", "schema": {"type": "integer", "minimum": 0, "maximum": MAX_WAIT_SECS}, "description": "long-poll until the release is done"});
    json!({
        "/api/platform/v1/publisher/whoami": {"get": op("This publisher, its apps and limits", json!({}))},
        "/api/platform/v1/publisher/releases": {
            "get": op("Releases of this publisher", json!({"parameters": [{"name": "app", "in": "query", "schema": {"type": "string"}}, {"name": "limit", "in": "query", "schema": {"type": "integer"}}]})),
            "post": op("Deploy or update an app from an uploaded docker save archive", json!({
                "parameters": [wait.clone()],
                "requestBody": {"required": true, "content": {"multipart/form-data": {"schema": {
                    "type": "object", "required": ["manifest", "image"],
                    "properties": {
                        "manifest": {"$ref": "#/components/schemas/ReleaseManifest"},
                        "image": {"type": "string", "format": "binary", "description": "docker save output (tar or tar.gz), one linux/amd64 image, <= 2 GiB; send after manifest"},
                    }},
                    "encoding": {"manifest": {"contentType": "application/json"}}}}},
                "responses": {"200": {"description": "release finished (live or failed)"}, "202": {"description": "release queued/in progress"}, "409": {"description": "name unavailable, or a release of this app is in progress"}, "413": {"description": "too large"}, "415": {"description": "not multipart/form-data"}},
            })),
        },
        "/api/platform/v1/publisher/releases/{id}": {"get": op("One release (status, artifact, failure, links)", json!({"parameters": [id.clone(), wait.clone()]}))},
        "/api/platform/v1/publisher/releases/{id}/log": {"get": op("Deploy log (text/plain)", json!({"parameters": [id]}))},
        "/api/platform/v1/publisher/apps": {"get": op("Apps this publisher created", json!({}))},
        "/api/platform/v1/publisher/apps/{name}": {"get": op("One app: status, launcher URL, current release, AI policy", json!({"parameters": [name.clone()]}))},
        "/api/platform/v1/publisher/apps/{name}/logs": {"get": op("Runtime logs and container state", json!({"parameters": [name.clone(), {"name": "tail", "in": "query", "schema": {"type": "integer", "maximum": 2000}}]}))},
        "/api/platform/v1/publisher/apps/{name}/rollback": {"post": op("Run a retained earlier release again", json!({"parameters": [name.clone(), wait.clone()], "requestBody": {"required": false, "content": {"application/json": {"schema": {"type": "object", "properties": {"release": {"type": "string"}}, "additionalProperties": false}}}}}))},
        "/api/platform/v1/publisher/apps/{name}/restart": {"post": op("Restart the live container", json!({"parameters": [name, wait]}))},
    })
}

pub fn manifest_schema() -> Value {
    json!({
        "type": "object", "required": ["name", "title"], "additionalProperties": false,
        "properties": {
            "name": {"type": "string", "pattern": "^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$"},
            "title": {"type": "string", "maxLength": 80},
            "description": {"type": "string", "maxLength": 500},
            "version": {"type": "string", "maxLength": 64},
            "ai": {"type": "boolean", "default": true},
            "runtime": {"type": "object", "additionalProperties": false, "properties": {
                "port": {"type": "integer", "minimum": 1, "maximum": 65535},
                "health_path": {"type": "string", "default": "/"},
                "memory_mb": {"type": "integer", "minimum": manifest::MIN_MEMORY_MB, "maximum": manifest::MAX_MEMORY_MB, "default": manifest::DEFAULT_MEMORY_MB},
                "env": {"type": "object", "additionalProperties": {"type": ["string", "number", "boolean"]}},
            }},
            "provenance": {"type": "object", "additionalProperties": false, "properties": {
                "repository": {"type": "string"}, "commit": {"type": "string"}, "note": {"type": "string"},
            }},
        },
    })
}
