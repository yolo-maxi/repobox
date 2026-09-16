//! A tiny loopback origin used by the private demo app. It has no auth code
//! of its own: it simply renders whatever identity the edge injected, which is
//! the point of the demo. Because the generated route strips browser-supplied
//! `X-RepoBox-*` headers before the gate runs, anything shown here came from
//! the control plane.

use std::net::SocketAddr;

use axum::Router;
use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

use crate::web::css::CSS;
use crate::web::html::esc;

#[derive(Clone)]
struct Demo {
    app: String,
    title: String,
    bind: String,
}

pub async fn serve(
    bind: SocketAddr,
    app: String,
    title: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if !bind.ip().is_loopback() {
        return Err(format!("demo origin must bind to loopback, got {bind}").into());
    }
    let state = Demo {
        app,
        title,
        bind: bind.to_string(),
    };
    let router = Router::new()
        .route("/whoami.json", get(whoami))
        .route(
            "/assets/app.css",
            get(|| async { ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], CSS) }),
        )
        .fallback(get(index))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("demo origin listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}

fn h<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

async fn whoami(headers: HeaderMap) -> Response {
    let mut identity = serde_json::Map::new();
    for (k, v) in headers.iter() {
        let key = k.as_str();
        if key.starts_with("x-repobox-") || key.starts_with("x-forwarded-") {
            identity.insert(
                key.to_string(),
                serde_json::Value::String(v.to_str().unwrap_or("").to_string()),
            );
        }
    }
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        serde_json::Value::Object(identity).to_string(),
    )
        .into_response()
}

async fn index(
    State(d): State<Demo>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    let user = h(&headers, "x-repobox-user");
    let auth = h(&headers, "x-repobox-auth").unwrap_or("none");
    let role = h(&headers, "x-repobox-role").unwrap_or("—");
    let uid = h(&headers, "x-repobox-user-id").unwrap_or("—");
    let app = h(&headers, "x-repobox-app").unwrap_or("—");
    let xff = h(&headers, "x-forwarded-for").unwrap_or("—");
    let xfh = h(&headers, "x-forwarded-host").unwrap_or("—");
    let xfp = h(&headers, "x-forwarded-proto").unwrap_or("—");

    let identity_card = match user {
        Some(u) => format!(
            "<div class=\"panel\" style=\"border-color:rgba(110,231,168,.5)\"><h3>Authenticated identity (injected by the edge)</h3><dl class=\"kv\"><dt>X-RepoBox-User</dt><dd><strong>{}</strong></dd><dt>X-RepoBox-User-Id</dt><dd>{}</dd><dt>X-RepoBox-Role</dt><dd>{}</dd><dt>X-RepoBox-Auth</dt><dd>{}</dd><dt>X-RepoBox-App</dt><dd>{}</dd></dl></div>",
            esc(u),
            esc(uid),
            esc(role),
            esc(auth),
            esc(app)
        ),
        None => format!(
            "<div class=\"panel\" style=\"border-color:rgba(255,123,138,.5)\"><h3>No identity injected</h3><p class=\"muted\" style=\"margin:0\">The edge did not attach a user. For a private app this should never render: the gate returns 401 before the request reaches this origin. (X-RepoBox-Auth = <code>{}</code>)</p></div>",
            esc(auth)
        ),
    };

    let mut raw = String::new();
    let mut names: Vec<&str> = headers.keys().map(|k| k.as_str()).collect();
    names.sort_unstable();
    for k in names {
        if k.starts_with("x-repobox-") || k.starts_with("x-forwarded-") {
            raw.push_str(&format!(
                "{}: {}\n",
                esc(k),
                esc(h(&headers, k).unwrap_or(""))
            ));
        }
    }

    let body = format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex"><title>{title}</title><link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Ccircle cx='16' cy='16' r='14' fill='%230a1628'/%3E%3Ccircle cx='16' cy='16' r='7' fill='%234fc3f7'/%3E%3C/svg%3E"><link rel="stylesheet" href="/assets/app.css"></head>
<body><header class="top"><div class="wrap"><span class="brand"><span class="dot"></span>{app_name} <small>/ demo app</small></span><span class="who"><span class="badge private">private</span></span></div></header>
<main><div class="wrap">
<h1>{title}</h1>
<p class="lead">Mode: <strong>private</strong> (default). This page is served by a loopback-only origin on <code>{bind}</code>. Caddy asked the control plane before proxying; the identity below arrived as request headers injected at the edge.</p>
{identity_card}
<div class="two" style="margin-top:14px">
<div class="panel"><h3>Why this proves the edge did it</h3><ul style="margin:0;padding-left:18px"><li>The origin has no login code and reads only request headers.</li><li>The generated route runs <code>request_header -X-RepoBox-*</code> before the gate, so a browser cannot smuggle these headers in. Try: <code>curl -H 'X-RepoBox-User: mallory' https://{host}/</code> — anonymous gets 401, and with a valid session the page still shows the real user.</li><li>The origin listens on loopback only; it is unreachable from the internet except through this Caddy route.</li></ul></div>
<div class="panel"><h3>Request as seen by the origin</h3><dl class="kv"><dt>Path</dt><dd><code>{path}</code></dd><dt>X-Forwarded-For</dt><dd>{xff}</dd><dt>X-Forwarded-Host</dt><dd>{xfh}</dd><dt>X-Forwarded-Proto</dt><dd>{xfp}</dd></dl><pre class="mono" style="white-space:pre-wrap;color:var(--dim);margin:12px 0 0;font-size:.8rem">{raw}</pre><p class="hint" style="margin:8px 0 0">Machine-readable: <a href="/whoami.json">/whoami.json</a></p></div>
</div>
<p class="hint" style="margin-top:18px">Launch flow: auth.repo.box/{app_name} → https://{host}/?token=… → gate redeems the one-time code → host-only session cookie → redirect to the clean URL you are on now.</p>
</div></main>
<footer><div class="wrap"><span>repo.box platform · demo origin</span><span class="muted">no tokens in this page, no browser storage</span></div></footer></body></html>"#,
        title = esc(&d.title),
        app_name = esc(&d.app),
        bind = esc(&d.bind),
        host = esc(&format!("{}.repo.box", d.app)),
        path = esc(&uri.to_string()),
        xff = esc(xff),
        xfh = esc(xfh),
        xfp = esc(xfp),
    );
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "no-store")],
        Html(body),
    )
        .into_response()
}
