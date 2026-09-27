//! `GET /docs`: the public, human-readable landing for the platform API.
//!
//! The machine documents (`/api/platform/v1`, `openapi.json`, `skill.md`,
//! MCP) stay the contract; this page is the front door that links them and
//! puts the publisher deploy quickstart first. No token needed, no registry
//! data rendered: only this host's public base and fixed platform limits.

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::Response;

use super::html::{Shell, esc, page};
use super::{S, current_user, html};
use crate::publisher::{MAX_APPS_PER_PUBLISHER, MAX_RELEASES_PER_DAY};

pub const DOCS_PATH: &str = "/docs";

pub fn docs_url(base: &str) -> String {
    format!("{base}{DOCS_PATH}")
}

/// Every document the landing links, as `(label, absolute URL)`. Shared with
/// the tests so a renamed route cannot silently orphan a link.
pub fn links(base: &str) -> Vec<(&'static str, String)> {
    let v1 = format!("{base}/api/platform/v1");
    vec![
        (
            "Publisher deploy quickstart",
            format!("{base}{DOCS_PATH}#publish"),
        ),
        ("Capabilities (JSON)", v1.clone()),
        ("OpenAPI 3.1 (JSON)", format!("{v1}/openapi.json")),
        ("Agent skill (Markdown)", format!("{v1}/skill.md")),
        ("MCP endpoint (POST, bearer)", format!("{v1}/mcp")),
        (
            "Well-known discovery",
            format!("{base}/.well-known/repobox-platform.json"),
        ),
    ]
}

pub async fn docs(State(s): State<S>, headers: HeaderMap) -> Response {
    let user = current_user(&s, &headers).map(|(_, u)| u);
    let shell = Shell {
        title: "Platform docs",
        user: user.as_ref(),
        active: "docs",
        standalone: None,
    };
    let mut r = html(StatusCode::OK, page(&shell, &body(&s.cfg.public_base)));
    // The body is the same for everyone; only the nav reflects the viewer.
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn pre(code: &str) -> String {
    format!("<pre class=\"code\"><code>{}</code></pre>", esc(code))
}

fn body(base: &str) -> String {
    let v1 = format!("{base}/api/platform/v1");
    let p = format!("{v1}/publisher");
    let link_list = links(base)
        .iter()
        .map(|(label, url)| {
            format!(
                "<li><a href=\"{u}\">{l}</a> <span class=\"mono muted\">{u}</span></li>",
                u = esc(url),
                l = esc(label)
            )
        })
        .collect::<String>();
    let recipe = format!(
        "docker build --platform linux/amd64 -t myapp .\n\
docker save myapp | gzip > myapp.tar.gz\n\
curl -fsS -H \"Authorization: Bearer $REPOBOX_PUBLISHER_TOKEN\" \\\n  \
-F 'manifest={{\"name\":\"myapp\",\"title\":\"My app\",\"runtime\":{{\"port\":8080,\"health_path\":\"/healthz\"}}}};type=application/json' \\\n  \
-F image=@myapp.tar.gz \\\n  \
'{p}/releases?wait=300'"
    );
    let ops = [
        (
            "Release status",
            "GET",
            format!("{p}/releases/{{id}}?wait=300"),
        ),
        (
            "Releases of an app",
            "GET",
            format!("{p}/releases?app={{name}}"),
        ),
        ("Deploy log", "GET", format!("{p}/releases/{{id}}/log")),
        ("App status", "GET", format!("{p}/apps/{{name}}")),
        (
            "Runtime logs",
            "GET",
            format!("{p}/apps/{{name}}/logs?tail=200"),
        ),
        (
            "Rollback",
            "POST",
            format!("{p}/apps/{{name}}/rollback  (optional {{\"release\":\"rel-…\"}})"),
        ),
        ("Restart", "POST", format!("{p}/apps/{{name}}/restart")),
        ("Who am I", "GET", format!("{p}/whoami")),
    ]
    .iter()
    .map(|(what, method, url)| {
        format!(
            "<tr><td>{}</td><td class=\"mono\">{method}</td><td class=\"mono\">{}</td></tr>",
            esc(what),
            esc(url)
        )
    })
    .collect::<String>();
    format!(
        r#"<div class="docs">
<h1>repo.box platform docs</h1>
<p class="muted">Deploy and operate private apps at <code>https://&lt;name&gt;.repo.box</code>. Reading these docs needs no token; every API call below needs <code>Authorization: Bearer …</code>.</p>

<div class="card elevated"><div class="title"><strong>Documents</strong></div><ul class="notes">{link_list}</ul></div>

<h2 id="publish">Publisher quickstart: deploy an app</h2>
<p>With a publisher token (<code>rbpub_…</code>, issued by an operator), one HTTPS request deploys an app, and the same request with the same <code>name</code> updates it. No registry, Git host, SSH or Docker access on repo.box is needed: you upload the image archive itself.</p>
{recipe}
<dl class="kv">
<dt>Endpoint</dt><dd class="mono">POST {p}/releases[?wait=SECONDS]</dd>
<dt>Body</dt><dd><code>multipart/form-data</code> with exactly two parts, <strong>in this order</strong>: <code>manifest</code> (JSON, <code>type=application/json</code>), then <code>image</code> (the archive)</dd>
<dt>Image</dt><dd><code>docker save</code> output (tar or gzip tar), <code>podman save --format docker-archive|oci-archive</code>, or an OCI layout tar; exactly one <code>linux/amd64</code> image, at most 2 GiB</dd>
<dt>Answer</dt><dd><code>202</code> while queued or in progress, <code>200</code> when done (<code>?wait=</code> long-polls up to 600 s). Read <code>release.status</code> (<code>queued → building → starting → live | failed</code>) and share <code>app.launcher_url</code>.</dd>
<dt>Limits</dt><dd>{MAX_APPS_PER_PUBLISHER} apps per publisher, {MAX_RELEASES_PER_DAY} releases per 24 h</dd>
</dl>

<h2 id="manifest">Minimal manifest</h2>
{minimal}
<p><code>name</code> (DNS label, becomes the host) and <code>title</code> are required. Optional: <code>description</code>, <code>version</code>, <code>ai</code> (default <code>true</code>), <code>provenance</code> (recorded only), and <code>runtime</code>: <code>port</code> (default: the image's single <code>EXPOSE</code>, else 8080; also <code>$PORT</code>), <code>health_path</code> (default <code>/</code>), <code>memory_mb</code> (64–1024, default 512), <code>env</code>. Unknown fields are refused; the full schema is <code>ReleaseManifest</code> in the <a href="{v1}/openapi.json">OpenAPI document</a>.</p>

<h2 id="operate">Release status, logs, rollback, restart</h2>
<div class="table-wrap"><table><thead><tr><th>What</th><th>Method</th><th>URL (publisher bearer)</th></tr></thead><tbody>{ops}</tbody></table></div>
<p class="muted">Every deploy answer also carries these as <code>release.links</code> and <code>app.links</code>. The same operations exist as MCP tools at <span class="mono">POST {v1}/mcp</span> with the publisher bearer; image bytes travel only over the HTTPS upload.</p>

<h2 id="contract">What you get, and what v1 does not do</h2>
<ul class="notes">
<li><strong>Apps are private.</strong> Only people an operator grants (and admins) can open one, after signing in on auth.repo.box. The app has no login of its own: every request carries the edge-injected identity (<code>X-RepoBox-User-Id</code>, <code>X-RepoBox-User</code>, <code>X-RepoBox-Role</code>); key records on <code>X-RepoBox-User-Id</code>. A publisher cannot change visibility or grants.</li>
<li><strong>Persistent <code>/data</code></strong> (<code>$REPOBOX_DATA_DIR</code>) survives updates, restarts and rollbacks; it is removed only if an operator removes the app. Everything outside <code>/data</code> comes from the running release's image.</li>
<li><strong><code>runtime.env</code> is plain, non-secret configuration.</strong> Values are stored as sent with the release record and passed as ordinary container environment variables. Never put credentials there or bake them into the image.</li>
<li><strong>Not in v1:</strong> no secrets management, no database backup or export API (back up what is in <code>/data</code> from inside your app if you need it), no registry pulls or Git builds, no custom domains, no public visibility.</li>
<li>Health gate: a release goes live only after <code>GET health_path</code> answers within 120 s; until then the previous release keeps serving.</li>
<li>AI: the app's own pages may call <code>POST /_repo_box/ai/v1/chat/completions</code> same-origin (OpenAI chat.completions subset, non-streaming, no key). See the <a href="{v1}/skill.md">skill</a>.</li>
</ul>

<h2 id="tokens">Tokens</h2>
<ul class="notes">
<li><strong>Publisher token</strong> <code>rbpub_…</code>: deploys and operates the apps that publisher created (<span class="mono">{p}/*</span>).</li>
<li><strong>Service token</strong> <code>rbp_…</code>: owner-scoped reads, AI policy and app registration requests (<span class="mono">{v1}/apps</span> …). It cannot deploy.</li>
<li>Both are issued by an operator, expire and can be revoked. Neither is OAuth or a browser session. A <code>401</code> means the header is missing, the wrong kind, expired or revoked.</li>
</ul>
</div>"#,
        recipe = pre(&recipe),
        minimal = pre(r#"{"name": "myapp", "title": "My app"}"#),
        p = esc(&p),
        v1 = esc(&v1),
    )
}
