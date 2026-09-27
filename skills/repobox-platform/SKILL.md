---
name: repobox-platform
description: Use when building, deploying or operating an app under <name>.repo.box through the repo.box platform (auth.repo.box) — especially deploying your own app with a publisher token by uploading a `docker save` archive, checking release status/logs, rolling back, calling the same-origin AI endpoint /_repo_box/ai/v1/chat/completions, or using the platform API/MCP.
---

# repo.box platform — agent skill

The repo.box platform (`auth.repo.box`, binary `repobox-platform`) publishes
managed apps at `https://<name>.repo.box`. Caddy asks the platform gate on
every request, strips browser-supplied `X-RepoBox-*` headers and injects only
the gate-issued identity. Private apps have **no login of their own**.

Every enabled managed app also gets a platform-owned, same-origin,
OpenAI-compatible AI endpoint backed by ChatMock. Apps never hold a model
credential.

Discovery (public, versioned, no secrets):

| What | Where |
|------|-------|
| Docs (HTML, start here) | `https://auth.repo.box/docs` (publisher quickstart at `/docs#publish`) |
| Capabilities document | `GET https://auth.repo.box/api/platform/v1` (also `/.well-known/repobox-platform.json`) |
| OpenAPI 3.1 | `GET https://auth.repo.box/api/platform/v1/openapi.json` |
| This skill | `GET https://auth.repo.box/api/platform/v1/skill.md`, or `repobox-platform skill` |
| MCP | `POST https://auth.repo.box/api/platform/v1/mcp` (bearer service or publisher token) |
| CLI | `repobox-platform --help` and `<subcommand> --help` (operator host) |

## 0. Deploy your own app (publisher token)

A **publisher** is a trusted external agent with one credential,
`Authorization: Bearer rbpub_…` (a *publisher token*: operator-issued,
written once to a 0600 file, expires within 30 days, revocable; not a
service token, not a browser session, not OAuth). It deploys apps to
`https://<name>.repo.box` by uploading a Docker image archive. It needs no
registry, Git host, SSH or Docker access on repo.box. It owns, sees and can
change **only the apps it created**; everything else looks like `404`, and a
name held by anyone else is refused with `409 name_unavailable`.

**One request deploys (and the same request with the same name updates):**

```bash
docker build --platform linux/amd64 -t myapp .
docker save myapp | gzip > myapp.tar.gz
curl -fsS -H "Authorization: Bearer $REPOBOX_PUBLISHER_TOKEN" \
  -F 'manifest={"name":"myapp","title":"My app","runtime":{"port":8080,"health_path":"/healthz"}};type=application/json' \
  -F image=@myapp.tar.gz \
  'https://auth.repo.box/api/platform/v1/publisher/releases?wait=300'
```

- `multipart/form-data`, parts in this order: `manifest` (JSON), then
  `image`.
- Supported archives: `docker save` output (tar, optionally gzip);
  `podman save --format docker-archive|oci-archive`; an OCI layout tar
  (`docker buildx build --platform linux/amd64 --provenance=false --output type=oci,dest=myapp.tar .`).
  Exactly one `linux/amd64` image, at most 2 GiB.
- Manifest: `name` (DNS label) and `title` are required. Optional fields:
  `description`, `version` (your label), `ai` (default `true`), and
  `provenance` (`repository`/`commit`/`note`, recorded only). `runtime`
  takes `port` (the container port; default is the image's single
  `EXPOSE`, else 8080; also passed as `$PORT`), `health_path` (default `/`),
  `memory_mb` (64–1024, default 512) and `env` (non-secret settings).
  Unknown fields are refused.
- Whatever names/tags the archive carries are ignored. The platform imports
  it only as `repobox-pub/<app>:<release>`.

**Answer:** `202` while queued/in progress, `200` once done (`?wait=N`
long-polls up to 600 s). `release`: `id`, `version`, `status`
(`queued → building → starting → live | failed`; older live releases become
`superseded`; a restart ends `done`), `artifact`
(`sha256`, `bytes`, `format`, `image_id`), `failure {code, message}`,
`links.self`, `links.build_log`. `app`: `launcher_url`
(`https://auth.repo.box/<name>`, the URL to share), `direct_url` (edge-gated:
anonymous requests get 401), `status`, `ai`, `current_release`. `next` says
what to do.

**Runtime contract.** The container is reached only by the platform edge on
the host loopback; listen on `0.0.0.0:$PORT` inside it. A release goes live only
after `GET health_path` answers (2xx/3xx; any non-5xx for the default `/`)
within 120 s. Until then the previous release keeps serving (blue/green).
Every request carries the edge-injected identity
(`X-RepoBox-User-Id`, `X-RepoBox-User`, `X-RepoBox-Role`, `X-RepoBox-Auth:
session`). The app has **no login of its own**; key records on
`X-RepoBox-User-Id`. The AI endpoint (§1) is on by default. `/data`
(`$REPOBOX_DATA_DIR`) persists across updates and rollbacks. The platform
also sets `PORT`, `REPOBOX_APP`, `REPOBOX_APP_URL`,
`REPOBOX_LAUNCHER_URL`, `REPOBOX_AI_CHAT_PATH` and `REPOBOX_RELEASE`.
Limits: 1 CPU, `memory_mb`, 512 processes. There is no privileged mode, host
network, host port or host mount. The app is private: an operator grants
people access; a publisher cannot change visibility or grants.

**Operate your apps** (same bearer):

| | |
|---|---|
| `GET /api/platform/v1/publisher/whoami` | your publisher id, apps, limits |
| `GET /api/platform/v1/publisher/apps[/{name}]` | status, launcher URL, current release |
| `GET /api/platform/v1/publisher/releases[?app=]`, `/releases/{id}[?wait=]` | release records |
| `GET /api/platform/v1/publisher/releases/{id}/log` | deploy log (text): import, start, health, route |
| `GET /api/platform/v1/publisher/apps/{name}/logs?tail=200` | container state + recent stdout/stderr |
| `POST /api/platform/v1/publisher/apps/{name}/rollback` | run a retained earlier release again (body optional: `{"release":"rel-…"}`; 3 releases retained) |
| `POST /api/platform/v1/publisher/apps/{name}/restart` | restart the live container |

Errors are `{"error":{"code","message"}}`. Upload mistakes add `expected`,
which is the exact request to send. MCP (`POST /api/platform/v1/mcp` with
the publisher bearer) offers `publisher_whoami`, `how_to_deploy`,
`list_my_apps`, `get_my_app`, `list_releases`, `get_release`,
`get_build_log`, `get_app_logs`, `rollback_app` and `restart_app`. Image
bytes travel only over the HTTPS upload.

`runtime.env` is plain, non-secret configuration: values are stored as sent
with the release record and passed as ordinary container environment.

Not in v1: registry pulls or Git builds (upload the image), secrets
management (do not bake secrets into images or `runtime.env`), a database
backup/export API (back up `/data` from inside the app), custom domains,
public visibility. The limits are 10 apps per publisher and 100 releases per 24 h.

Operators: `repobox-platform publisher create --handle NAME`,
`publisher token create --publisher NAME --name NAME-1 --out FILE`,
`publisher token revoke`, `publisher disable`, `publisher releases`,
`publisher release ID`, `publisher remove-app NAME --yes`.

## 1. The app AI endpoint

```
POST https://<app>.repo.box/_repo_box/ai/v1/chat/completions
GET  https://<app>.repo.box/_repo_box/ai/v1/models
```

**Who can call it.** A browser (or the app's own backend relaying the
browser's request) that holds the app's platform session — i.e. a signed-in
platform user with access to the app who opened it from
`https://auth.repo.box/<app>`. No API key, no per-app credential, nothing to
configure in the app. Anonymous visitors, other origins and anything with
forged `X-RepoBox-*` headers get `401`/`403`.

**Request (OpenAI chat.completions subset):**

```json
{
  "model": "gpt-5.6-terra",
  "messages": [
    {"role": "system", "content": "You are concise."},
    {"role": "user", "content": "Summarise my note in one line."}
  ],
  "max_tokens": 400,
  "temperature": 0.3
}
```

- Required: `messages` (1–64; roles `system`, `developer`, `user`,
  `assistant`; content is a string or `[{"type":"text","text":…}]`).
- Optional: `model` (default: the app's default model), `max_tokens` /
  `max_completion_tokens` (≤ app limit; default = app limit), `temperature`
  0–2, `top_p` 0–1, `presence_penalty`/`frequency_penalty` −2–2, `stop`
  (≤4 strings), `response_format` `{"type":"json_object"}`, `n: 1`.
- Ignored/dropped: `user`, `metadata`, `store`, anything unrecognised — the
  upstream body is rebuilt from recognised fields only.
- **Refused in v1:** `stream: true` (v1 is **non-streaming**; use a normal
  request and show a spinner), `tools`, `tool_choice`, `functions`,
  `function_call`, tool messages, images/audio.
- `Content-Type: application/json` is required; `Origin`, when sent, must be
  `https://<app>.repo.box`.

**Response:** `{"id","object":"chat.completion","created","model","choices":[{"index":0,"message":{"role":"assistant","content":…},"finish_reason"}],"usage":{…}}`.
Output text is capped at `max_tokens × 8` characters (`finish_reason:
"length"` when cut), because the platform does not rely on ChatMock honouring `max_tokens`.

**Errors** use the OpenAI shape `{"error":{"message","type","code"}}`:

| Status | `code` | Meaning |
|--------|--------|---------|
| 401 | `unauthenticated` | no valid app session / identity for this app |
| 403 | `ai_disabled` | the app's AI policy is off |
| 403 | `cross_origin` | request not from the app's own origin |
| 404 | `app_disabled`, `not_found` | app switched off / unknown path |
| 400 | `model_not_allowed`, `model_unavailable`, `max_tokens_too_large`, `streaming_unsupported`, `unsupported_parameter`, `invalid_parameter`, `too_many_messages`, `invalid_json` | request outside policy |
| 413 | `input_too_large`, `request_too_large` | message characters over the app limit / body over 256 KiB |
| 415 | `content_type` | not `application/json` |
| 429 | `quota_exceeded`, `busy` | daily quota spent (UTC day) / broker at capacity — retry later |
| 502/503/504 | `upstream_unavailable`, `upstream_invalid`, `ai_unavailable`, `upstream_timeout` | provider trouble; retry with backoff |

A request counts against the daily quotas once it passes validation, even if
the provider then fails.

**Frontend example (same origin, no key):**

```js
async function ask(messages) {
  const r = await fetch("/_repo_box/ai/v1/chat/completions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "same-origin",
    body: JSON.stringify({ messages, max_tokens: 400 }),
  });
  const data = await r.json();
  if (!r.ok) throw new Error(data.error?.code || r.status);
  return data.choices[0].message.content;
}
```

OpenAI SDKs work with `baseURL: "/_repo_box/ai/v1"` (browser) and any
placeholder `apiKey` — the key is ignored; the platform session is the
credential. Do not ship a real key.

**Backend example (proxy app relaying the signed-in user's request):** the
backend receives the browser's `Cookie` header; forward it unchanged to the
same origin. Never store or log it.

```python
r = requests.post(f"https://{APP}.repo.box/_repo_box/ai/v1/chat/completions",
                  headers={"Content-Type": "application/json",
                           "Cookie": incoming_request.headers["Cookie"]},
                  json={"messages": msgs, "max_tokens": 300}, timeout=95)
```

Treat model output as untrusted text: escape before rendering HTML, validate
JSON you asked for, never execute it.

## 2. AI policy (per app, in the registry)

| Field | Default (private platform app) | Ceiling |
|-------|------|---------|
| `enabled` | **on** for new private `--identity platform` apps; off otherwise (and for every app registered before this feature) | — |
| `provider` | `chatmock` (only provider in v1) | — |
| `default_model` / `models` | `gpt-5.6-terra` / `gpt-5.6-terra, gpt-5.6-luna` | known: `gpt-5.6-terra`, `gpt-5.6-luna`, `gpt-5.6-sol`, `gpt-5.5`; the broker additionally routes only models ChatMock currently exposes |
| `max_input_chars` (all messages) | 32 000 | 64 000 |
| `max_output_tokens` | 2 048 | 4 096 |
| `user_daily_requests` | 200 | 2 000 |
| `app_daily_requests` | 2 000 | 20 000 |
| `public_policy` | none | only `signed-in-quota` |

**Public apps:** AI stays off unless the app declares `public_policy:
signed-in-quota` *and* explicit per-user and per-app quotas. Even then only
signed-in platform users with access to the app can call it — never
anonymous visitors; the platform is not a public free model proxy. Switching
an AI-enabled private app to public without that policy switches AI off.
AI requires the platform identity contract (`identity: platform`).

## 3. Access model for agents

- **CLI is canonical** (operators on the repo.box host):
  `app register … --identity platform [--no-ai]`, `app ai show|enable|disable|set`,
  `app requests list|approve|reject`, `service-token create|list|revoke`,
  `routes render`, `skill`.
- **Machine API** for agents without host access, with a **service token**
  (`Authorization: Bearer rbp_…`). Service tokens are *not OAuth*: an
  operator issues one with the CLI, it is bound to one owner, reaches only
  apps that owner owns (optionally narrowed with `--app`), carries explicit
  scopes, expires (≤ 90 days), is revocable and stored hashed. A scoped OAuth
  client-credentials issuer is the remaining dependency.

| Scope | Allows |
|-------|--------|
| `apps:read` | `GET /api/platform/v1/apps`, `/apps/{name}` |
| `ai:read` | `GET /apps/{name}/ai` (policy + today's counters) |
| `ai:write` | `PATCH /apps/{name}/ai` within ceilings (unknown fields refused) |
| `routes:read` | `GET /apps/{name}/route` — read-only Caddy preview |
| `apps:request` | `GET|POST /app-requests` — a registration *request*; an operator approves |
| `release:read` | `GET /release` — version, commit, schema, broker reachability |

Never available to agents: SSH, Caddy apply/reload, the database, users,
grants, sessions, device links, or ChatMock/ChatGPT credentials.

```bash
T=$(head -1 /path/to/token-file)          # 0600 file written by the operator
curl -s https://auth.repo.box/api/platform/v1/whoami -H "Authorization: Bearer $T"
curl -s https://auth.repo.box/api/platform/v1/apps/my-app/ai -H "Authorization: Bearer $T"
curl -s -X PATCH https://auth.repo.box/api/platform/v1/apps/my-app/ai \
  -H "Authorization: Bearer $T" -H 'Content-Type: application/json' \
  -d '{"max_output_tokens": 1024, "models": ["gpt-5.6-terra"], "default_model": "gpt-5.6-terra"}'
curl -s -X POST https://auth.repo.box/api/platform/v1/app-requests \
  -H "Authorization: Bearer $T" -H 'Content-Type: application/json' \
  -d '{"name":"my-new-app","title":"My app","kind":"proxy","target":"127.0.0.1:4555","visibility":"private","identity":"platform"}'
```

**MCP:** `POST /api/platform/v1/mcp`, JSON-RPC 2.0, protocol `2025-06-18`,
JSON responses (no SSE), same bearer. Tools: `whoami`, `list_apps`,
`get_app`, `get_ai_policy`, `update_ai_policy`, `preview_route`,
`request_app_registration`, `list_app_requests`, `release_status` — each
enforces the same scope as the REST call.

Operator examples:

```bash
repobox-platform app register my-app --title "My app" --owner fran \
  --kind proxy --target 127.0.0.1:4555 --identity platform      # AI on by default
repobox-platform app ai show my-app
repobox-platform app ai set my-app --models gpt-5.6-terra,gpt-5.6-luna --max-output-tokens 1024 --user-daily 100
repobox-platform app ai set my-public-app --public-policy signed-in-quota --user-daily 20 --app-daily 200 --enable
repobox-platform service-token create --name my-agent --owner fran \
  --scope apps:read --scope ai:read --app my-app --ttl-days 14 --out /home/fran/secrets/my-agent.token
repobox-platform service-token revoke my-agent
```

## 4. Security invariants

1. `/_repo_box/*` is reserved on every managed host and handled **before**
   the app origin; only `/_repo_box/ai/v1/*` is served, the rest is 404. An
   app origin never sees these requests.
2. The route strips all browser `X-RepoBox-*`, runs the normal gate, copies
   only gate-issued identity, then proxies to the control plane on loopback.
3. The control plane re-validates the app's `__Host-rb_app` session for that
   app and user; headers alone (e.g. a direct loopback call) never suffice.
4. The control plane reaches the broker only via a repo.box-loopback reverse
   SSH tunnel, with a bridge secret loaded from a root-owned 0600 systemd
   credential. The broker binds Hetzner loopback, checks the secret in
   constant time, re-validates the request against platform ceilings and its
   model allowlist ∩ ChatMock's live models, and calls ChatMock
   (`127.0.0.1:8111`, which accepts any compatibility key and must never be
   exposed). No `ai.repo.box` exists.
5. Nothing logs or stores prompts, completions, cookies, launch codes, the
   bridge secret, service-token values or OAuth material. Log lines carry
   app, user id, model, status, character counts and latency; the database
   keeps daily request counters only (90-day retention).
6. No model tool execution and no arbitrary app actions in v1.
7. The control plane runs as its own non-login system user
   (`repobox-platform`); the registry and the bridge credential are not
   readable by app services. Host-level limit: the `fran` account (which runs
   many app services) is root-equivalent through the `docker` group and
   passwordless sudo, so a compromised `fran` service could still reach them
   through root; the broker's allowlist, ceilings and concurrency bound what
   any caller holding the secret can do.
