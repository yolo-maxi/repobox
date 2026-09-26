---
name: repobox-platform
description: Use when building, publishing or operating an app under <name>.repo.box through the repo.box platform control plane (auth.repo.box) — especially calling the same-origin AI endpoint /_repo_box/ai/v1/chat/completions, inspecting or changing an app's AI policy, discovering the platform API/MCP, requesting an app registration, previewing routes or checking release status.
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
| Capabilities document | `GET https://auth.repo.box/api/platform/v1` (also `/.well-known/repobox-platform.json`) |
| OpenAPI 3.1 | `GET https://auth.repo.box/api/platform/v1/openapi.json` |
| This skill | `GET https://auth.repo.box/api/platform/v1/skill.md`, or `repobox-platform skill` |
| MCP | `POST https://auth.repo.box/api/platform/v1/mcp` (bearer service token) |
| CLI | `repobox-platform --help` and `<subcommand> --help` (operator host) |

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
7. Known limit: on repo.box the bridge secret is readable by processes of the
   control plane's service user; the broker's allowlist, ceilings and
   concurrency still bound what such a process could do.

## 5. Test, deploy, rollback (operators)

- Tests: `cargo fmt -p repobox-platform -- --check`,
  `cargo clippy -p repobox-platform --all-targets -- -D warnings`,
  `cargo test -p repobox-platform` (AI path: `tests/ai.rs`).
- Broker + tunnel on Hetzner: `repobox-platform/scripts/deploy-ai-bridge.sh`
  (installs `repobox-ai-broker.service` on `127.0.0.1:8127` and
  `repobox-ai-tunnel.service` forwarding repo.box `127.0.0.1:3232`; creates
  the shared secret once on both hosts, never prints it).
- Control plane + routes: `repobox-platform/scripts/deploy.sh` (gates, DB
  backup, service restart, `routes render`, guarded Caddy apply: backup →
  validate → reload → auto-restore on failure; `CADDY_DRY_RUN=1` first).
- Rollback: `sudo python3 /srv/repobox-platform/caddy-apply.py rollback
  <backup>`; restore the previous binary from `/srv/repobox-platform/bin/`
  backups; `sudo systemctl disable --now repobox-ai-tunnel repobox-ai-broker`
  (Hetzner) turns the AI endpoint into a clean 502/503 without touching app
  routing. Per app: `repobox-platform app ai disable <name>`.
