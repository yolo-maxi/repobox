# repo.box platform AI capability — implementation plan (v1)

Status: plan of record for the `feat/platform-auth-control-plane` slice that
adds a same-origin, platform-owned AI endpoint to managed apps. The shipped
contract lives in `skills/repobox-platform/SKILL.md` and
`docs/platform-control-plane.md`; this file records why it is shaped this way.

## Topology found (2026-09-26)

- repo.box host (204.168.190.248): Caddy 2.11, `repobox-platform serve` on
  `127.0.0.1:3230` (auth UI + `/gate/verify`), SQLite registry.
- Hetzner build box: ChatMock on `127.0.0.1:8111` (no caller auth; accepts
  any compatibility key). Models live: gpt-5.6-sol/terra/luna, gpt-5.5.
- Existing cross-host pattern (Ellie's Japanese, Mise, AcademicWeapon): a
  narrow bridge on Hetzner loopback that requires a bearer secret, reached
  from repo.box through a reverse SSH tunnel (`ssh -R 127.0.0.1:<port>:…`)
  bound to repo.box loopback. v1 reuses this pattern with one new
  platform-owned broker instead of another app-specific bridge. (The
  pre-existing `uni-kitchen-chatmock-tunnel` exposes raw ChatMock on
  repo.box's docker bridge; it is legacy and out of scope here.)

## Request path

```
browser (same origin)                         repo.box                                  Hetzner
POST https://<app>.repo.box/_repo_box/ai/v1/chat/completions
  └─ Caddy <app> route: strip X-RepoBox-* → forward_auth /gate/verify (normal gate)
       └─ handle /_repo_box/ai/v1/* → rewrite /gate/ai/v1/* → reverse_proxy 127.0.0.1:3230
            └─ platform ai handler: re-validates the app session cookie for <app>,
               checks gate-issued identity matches, app AI policy, limits, quota
               └─ http://127.0.0.1:3232 (reverse tunnel, repo.box loopback only)
                    └─ repobox-platform ai-broker 127.0.0.1:8127 (bearer secret,
                       model allowlist ∩ live models, hard ceilings, no logging of bodies)
                         └─ ChatMock 127.0.0.1:8111
```

- `/_repo_box/*` is reserved on every managed host before the static/proxy
  origin; unknown reserved paths are 404 and never reach the origin.
- The platform handler does not trust headers alone: the Caddy marker and
  the gate-issued identity must be present *and* the `__Host-rb_app` cookie
  must resolve to a live session for that app whose user matches. A forged
  header on a direct loopback call without the cookie fails.
- Cross-site / sibling-subdomain abuse: JSON content type required (forces a
  CORS preflight, which is never answered), `Origin`/`Sec-Fetch-Site` must be
  same-origin when present.
- v1 is **non-streaming**: `stream: true` is rejected with a clear 400. Tools,
  functions and tool_choice are rejected (no model tool execution in v1).

## Registry policy

Schema v6 adds per-app AI policy columns: `ai_enabled`, `ai_provider`
(`chatmock`), `ai_default_model`, `ai_models`, `ai_max_input_chars`,
`ai_max_output_tokens`, `ai_user_daily_requests`, `ai_app_daily_requests`,
`ai_public_policy` (NULL or `signed-in-quota`). Existing rows default to
disabled (no mass migration). `app register` gives private platform-identity
apps AI on with defaults; public apps get it only with an explicit
`--ai-public-policy signed-in-quota` plus explicit quotas, and even then only
signed-in platform users with access can call it (never anonymous). Making
an AI-enabled private app public without that policy switches AI off in the
same statement (audited), so the owner UI keeps working and no public free
proxy can appear. Usage table
`ai_usage_daily(app, user, day, requests)` holds counters only.

## Agent/operator surface

- CLI stays canonical: `app ai show|enable|disable|set`, `service-token
  create|list|revoke`, `app requests list|approve|reject`, `ai-broker`.
- `GET https://auth.repo.box/api/platform/v1` (+ `/.well-known/repobox-platform.json`,
  `/api/platform/v1/openapi.json`, `/api/platform/v1/skill.md`): public,
  versioned capabilities document, no secrets.
- Bearer **service tokens** (not OAuth): owner-bound, scoped
  (`apps:read ai:read ai:write routes:read apps:request release:read`),
  optional app allowlist, expiry, revocable, hashed at rest. Operator-issued
  via CLI to a 0600 file. A full OAuth client-credentials issuer is the
  remaining dependency.
- `POST /api/platform/v1/mcp`: MCP (JSON-RPC over streamable HTTP, JSON
  responses only) with tools mapping 1:1 to the API; same bearer + scopes.
- Nothing grants SSH, Caddy apply, DB, user admin or ChatMock credentials.

## Deploy / rollback

- Hetzner: `repobox-ai-broker.service` + `repobox-ai-tunnel.service`
  (installed by `scripts/deploy-ai-bridge.sh`); secret in root-owned 0600
  files on both hosts, loaded via systemd `LoadCredential=`.
- repo.box: normal `scripts/deploy.sh` (routes re-rendered to add the
  reserved path; Caddy backup → validate → reload → rollback retained).
- Rollback: `caddy-apply.py rollback <backup>`, previous binary from the
  stage backup, `systemctl disable --now repobox-ai-broker repobox-ai-tunnel`.
