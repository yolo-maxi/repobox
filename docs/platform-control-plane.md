# repo.box platform control plane (`auth.repo.box`)

## Open PWA Runtime public handoffs

`auth.repo.box` also serves two fixed public Android Runtime handoffs: `https://auth.repo.box/runtime/secure-vault` and `https://auth.repo.box/runtime/hyperliquid-positions`, plus `/.well-known/assetlinks.json` for package `box.openpwa.runtime`. They are static public association/handoff responses, not registry app routes: they do not inspect platform identity, create sessions, issue or redeem `rb_launch`, or disclose platform data. Query-bearing handoff URLs return 404. Keep these routes ahead of the generic `/{name}` private-app launcher route. Private app launcher behavior remains exclusively `https://auth.repo.box/{app-name}` and continues to mint one-time `rb_launch` codes.

First slice of the platform control plane: named users, apps with owners and
per-app grants, three visibility modes, enable/disable, one-time launch codes,
single-use device links, owner-created invitation links, and the Caddy edge
contract that enforces all of it before any managed app is served.

Source: `repobox-platform/` (Rust, one static binary). Branch:
`feat/platform-auth-control-plane`.

## Architecture

```
browser ──HTTPS──▶ Caddy (repo.box host)
                    │
                    ├─ auth.repo.box ───────────────▶ 127.0.0.1:3230  repobox-platform serve
                    │     (UI: directory, launch, account, app management, admin)
                    │
                    └─ <app>.repo.box   [generated route, one per registered app]
                          1. request_header -X-RepoBox-*          strip browser-supplied identity
                          2. forward_auth 127.0.0.1:3230 /gate/verify
                                header_up X-RepoBox-Gate 1
                                header_up X-RepoBox-Gate-App <app>
                                copy_headers X-RepoBox-User X-RepoBox-User-Id X-RepoBox-Role X-RepoBox-Auth X-RepoBox-App
                          3. file_server (static app)  |  reverse_proxy 127.0.0.1:PORT (proxy app)
```

* **Registry**: SQLite (WAL) at `/var/lib/repobox-platform/platform.db`. Tables:
  `users`, `apps`, `grants`, `tokens`, `sessions`, `audit`, `access_daily`,
  `access_daily_users`, `access_totals`, `app_opens`, `app_visits`, `meta`
  (schema version 5; `apps.identity` since 5). Only
  SHA-256 hashes of tokens and session secrets are stored; raw values exist
  once, in the channel that delivers them (a 0600 file for operator-created
  links, a page shown once in the UI, a redirect for launch codes).
* **Gate** (`GET /gate/verify`): called by Caddy for every managed-app request.
  Decides per request from live registry state, so revoking a grant, disabling
  a user or an app, or changing visibility takes effect on the next request
  without touching Caddy. A 2xx lets Caddy serve and copy the identity headers;
  any other status is returned to the browser as-is (401 page, 404 for
  disabled/unknown apps, 302 redemption redirect).
* **Launch flow** (private apps):
  `auth.repo.box/<app>` (signed-in device) → mints a one-time launch code
  (90 s) bound to user+app → `302 https://<app>.repo.box/?rb_launch=<code>` →
  gate consumes the code atomically → `302` to the clean URL with
  `Set-Cookie: __Host-rb_app=…; Path=/; Secure; HttpOnly; SameSite=Lax`
  (host-only by construction: `__Host-` cookies cannot carry `Domain`) →
  subsequent requests are authorised from that cookie for 24 h.
* **Sessions and revocation**: a device session (`__Host-rb_auth`, 30 d) is
  one signed-in device; an app session (`__Host-rb_app`, 24 h, one per app
  host) records the device session that launched it (`sessions.parent_id`,
  carried by the launch code's `tokens.session_id`). `/me` lists the user's
  devices and app sessions with a revoke button on every row; "Sign out this
  device" and revoking a device also revoke the app sessions that device
  launched. Admins get `/admin/users/<name>/sessions` with the same tables,
  per-row revoke and "Sign out everywhere". Revocation is scoped by user id
  in the store, CSRF-guarded like every POST, audited (`session.revoke`,
  `session.revoke_all`), and effective on the session's next request at
  auth.repo.box and at every app's edge gate, because each lookup re-reads
  `revoked_at`; no Caddy change is involved. Sessions that expire or are
  revoked drop out of the lists.
* **Sign-in** (auth.repo.box itself): no passwords, no email. A single-use
  **device link** (`/enrol/<token>`, 7 days) is created by an admin, by the
  operator CLI, or by the user from another signed-in device. Opening it shows
  a confirmation page; the POST consumes it and sets `__Host-rb_auth` (30 d).
  GET never consumes, so link-preview bots cannot burn a link.
* **Invitations**: an app owner (or admin) creates `/invite/<token>` (7 days,
  single use). A signed-in visitor accepting it gets a grant; a new person
  picks a handle and becomes a member with the grant, signed in on that device.
* **Visibility**: `private` (default; grant required, listed only to users who
  can open it), `public_unlisted` (no auth, never in the directory),
  `public_listed` (no auth, in the directory and `/api/directory`). Admins and
  owners can always open their apps. Disabled apps answer 404 at the edge.
* **Directory** (`auth.repo.box/`): signed in, two sections in this order.
  **Your apps** is the user's access list: apps they own, apps they hold a
  grant for and, for admins, every app; visibility and enabled state do not
  filter it (a disabled app shows a badge). **Other apps** is what remains
  that is `public_listed` and enabled; it is omitted when empty. Anonymous
  visitors get a **Public directory** with only `public_listed` enabled apps.
  `public_unlisted` apps therefore appear only inside somebody's access list,
  never in the public directory or `/api/directory`. The split is
  `pages::partition_directory`.
* **Access counting** (`/apps/<name>/analytics`, owner or admin only; CLI
  `app stats <name>`): the gate increments counters *only* on the 2xx allow
  path, so 401/403/404 denials, disabled apps or users, rejected codes and the
  launch-code `302` are never counted. What is stored: one row per app per
  UTC day with a request count (`access_daily`), one all-time request counter
  per app (`access_totals`), and for **private** apps only, `(app, day,
  user_id)` presence rows (`access_daily_users`) so unique signed-in users can
  be deduplicated. Public apps get counts only: no user id is written even
  when the visitor holds a session, and no anonymous identity is ever
  invented. No URL, query string, cookie, IP, user agent, token or body is
  recorded. Daily rows (and with them the user ids) are deleted after
  `ACCESS_RETENTION_DAYS` = 90 days; the all-time counter is kept. A count is
  every allowed request including assets, not page views. The page (linked
  as **Requests** from the manage page) shows the all-time total, the 90-day
  total, unique users (private apps) and the last 14 days, and states these
  rules. Owners see only their own apps.
* **Visits / opens** (`/apps/<name>/visits`, owner or admin only; CLI
  `app visits <name> --days N`): the product question "how often do people
  open this app", answered from the platform identity rather than from
  traffic. An **open** is the first allowed HTML document navigation by a
  **signed-in** person into an app after `VISIT_WINDOW_SECS` = 30 minutes
  without a page load of that app (an inactivity window, so a two-hour
  session of clicking around is one open, and coming back after lunch is a
  second). The gate classifies each allowed request with
  `gate::is_document_navigation`: `GET` only; no `Upgrade` (WebSockets); if
  the browser sends `Sec-Fetch-Dest` it must be `document` (so scripts,
  styles, images, fonts, `fetch`/XHR (`empty`), iframes and manifests are
  out) and `Sec-Fetch-Mode`, when present, `navigate`; without `Sec-Fetch-*`
  (old browsers, curl) `Accept` must list `text/html`/`application/xhtml+xml`;
  prefetch/prerender (`Sec-Purpose`/`Purpose`) is out; a path ending in a
  plain-file extension (`.js`, `.json`, `.png`, `.pdf`, …) is out. Denied
  requests, the launch-code `302`, and anonymous visitors of public apps are
  never opens; a person who launched a public app signed in is (that is the
  only case where a public app's visitor is identified, and it is stated on
  both pages). Caddy's `forward_auth` hop forwards `Accept`, `Sec-Fetch-*`
  and `Upgrade` unchanged (verified against Caddy 2.10 locally and 2.11 on
  the host), so no route change was needed. Storage: `app_opens(app_id,
  user_id, opened_at)` one row per open, and `app_visits(app_id, user_id,
  opened_at, last_nav_at)` one row per person and app holding the current
  visit's last navigation time so the window can be applied. Nothing else:
  no URL, query, cookie, IP, user agent, token or body. Open rows are deleted
  after `OPENS_RETENTION_DAYS` = 90 days (pruned once per UTC day on the
  first open of the day, like the request counters; dead visit rows older
  than a day go with them). The page offers 7/30/90-day ranges and shows
  total opens, unique people, today's opens, a per-day table (opens, people)
  and a per-person table (display name, handle, opens in range, last opened,
  disabled badge if the account is off). Members and other owners get 403,
  exactly like the Requests page (`pages::manageable`). The Requests page is
  explicitly labelled "allowed requests (edge traffic, not visits)" and
  links to Visits.
* **Grant typeahead** (manage page, owner or admin only): the "Grant a user"
  text input is upgraded by a small inline script into an ARIA combobox
  (`role=combobox`, `aria-autocomplete=list`, `aria-expanded`,
  `aria-activedescendant`; ArrowDown/ArrowUp move, Enter selects, Escape
  closes; mouse works too) over `GET /apps/<name>/grantable-users?q=`, which
  returns `{name, display_name}` for **enabled** users who are not the
  app's owner and not already granted, matched case-insensitively on the
  handle or the display name (`store::user_matches`), at most 12,
  `Cache-Control: no-store`. The endpoint is guarded by the same
  `manageable` check as the page (401 anonymous, 403 member/grantee/other
  owner, 404 unknown app) and carries no ids, roles, sessions, devices or
  links. Suggestions are rendered with `textContent`, never as HTML.
  Selecting fills the input with the canonical handle and the form still
  posts to the unchanged `/apps/<name>/grants` endpoint; a handle typed in
  full without JavaScript works exactly as before. Invitations and access
  semantics are untouched.
* **Identity headers** the origin may trust (only ever set by the gate):
  `X-RepoBox-App`, `X-RepoBox-Auth` (`session`|`public`), and for sessions
  `X-RepoBox-User`, `X-RepoBox-User-Id`, `X-RepoBox-Role`.
* **Safety boundary**: `serve`, `demo-origin` and every proxy target must be
  loopback; the renderer refuses anything else. Static roots must sit under
  `/srv/repobox-platform/apps`. App names are validated DNS labels with a
  reserved list (`auth`, `git`, `ens`, `api`, `www`, …). CSRF on all POSTs:
  `Origin` must match `https://auth.repo.box` (or `Sec-Fetch-Site: same-origin`).

### Platform identity contract (repo.box policy, mandatory for private apps)

**Rule.** Every managed private app uses the platform identity at ingress
and receives only the centrally injected identity. No app-specific password,
login, owner setup link or app session is part of the supported private-app
contract. App-level authorisation is **record scoping by the injected
identity** (`X-RepoBox-User-Id` is the stable key; `X-RepoBox-User` is the
handle and may be renamed), never a second login.

**Where it is enforced.**

* *Edge (mechanical, every request):* the rendered route strips every
  `X-RepoBox-*` header a client sent, asks the gate, and copies only the
  gate-issued identity (`copy_headers`); an origin therefore never sees a
  browser-supplied identity. Origins bind loopback only. This part holds for
  every managed app regardless of what its code does.
* *Publish path (declarative, non-optional):* `apps.identity` is part of the
  registry manifest. `app register --visibility private` is refused unless
  `--identity platform` is given; that flag is the only accepted value and
  it is the operator's declaration that the app has no login of its own.
  An existing app carries `pending` until an operator runs
  `app attest <name> [--note …]` after the migration/preflight review
  (audited `app.attest`, one-way). A `pending` app cannot be made private,
  neither by the CLI (`app visibility`) nor by the owner's visibility form
  (the private radio is disabled and the POST is refused with a message).
  `app show`, `app list` (IDENTITY column, WARNING line for pending private
  apps, `identity` in `--json`), the owner's manage page (Identity row with
  the rule stated) and the rendered `apps.caddy` (`# identity:` line per
  app) all show the contract, so the route file doubles as the manifest
  operators review. `routes render` prints a WARNING per pending private
  app (deploy logs carry it).
* *Migration/preflight review (human):* before attesting, the app's own
  password/login/setup/session code is removed and the app is verified to
  refuse requests without the gate identity and to scope records by it (the
  Study Diary conversion below is the reference).

**What the proxy cannot prove.** Caddy and the gate cannot mechanically
verify that arbitrary application code never renders a password screen or
keeps its own session; only the declaration at the publish path plus the
review can. That is why the contract is a registry field with an audited
attestation, not a header the route could enforce. Pre-policy private apps
keep serving while `pending` so that nothing goes dark; the manifest and the
deploy log call them out until they are converted and attested.

**Status at the 2026-09-17 deploy.** `platform`: demo-private,
demo-unlisted, demo-listed (platform-owned demos; the private one shows the
injected identity and has no login), study-diary (converted, below).
`pending`: ellies-japanese, academicweapon, uni-kitchen, fieldwork (each has
its own login/setup flow to remove: follow-ups per app), puzzlenest and
circuit (public static, undeclared; irrelevant unless made private).

### Route model

`repobox-platform routes render` is a pure function of the sorted registry: the
same registry always yields byte-identical `apps.caddy`. It is imported by one
line inside the managed block of the main Caddyfile. Registering an app and
rendering routes is **operator-only** (CLI on the host); the web UI has no
create/deploy/route controls.

### Platform AI capability (same-origin, schema 6)

Every enabled managed app host reserves `/_repo_box/*` **before** its
static/proxy origin (rendered route: strip `X-RepoBox-*` → `forward_auth`
gate → `handle /_repo_box/ai/v1/*` → `handle /_repo_box/*` 404 → `handle`
origin). `POST https://<app>.repo.box/_repo_box/ai/v1/chat/completions` and
`GET …/v1/models` are rewritten to `/gate/ai/v1/*` on the loopback control
plane (`/gate/*` is 404 on auth.repo.box), which:

* requires the Caddy marker, a gate identity of `session`, and a live
  `__Host-rb_app` session *for that app* whose user matches the gate's user
  id and still has access (forged headers without the cookie → 401);
* requires same-origin (`Content-Type: application/json`, `Origin` host and
  `Sec-Fetch-Site` checked when present);
* applies the app's AI policy (enabled, allowed/default model, input
  characters, output tokens, per-user and per-app daily quotas);
* forwards a rebuilt, non-streaming request to the broker through
  `127.0.0.1:3232` (reverse SSH tunnel) with the bridge secret, and rebuilds
  the response into plain `chat.completion`.

The broker (`repobox-platform ai-broker`, Hetzner `127.0.0.1:8127`,
`DynamicUser`, `IPAddressAllow=localhost`) checks the secret in constant
time, re-applies platform ceilings, routes only its model allowlist ∩ the
models ChatMock currently exposes, and calls ChatMock on `127.0.0.1:8111`
without any credential. Both hops log only app, user id, model, status,
character counts and latency. `ai_usage_daily` stores request counters per
app/user/UTC day (90-day retention). **v1 is non-streaming** (`stream: true`
→ 400) and has **no tools/functions/images**.

Registry policy (schema 6 columns on `apps`): new `--visibility private
--identity platform` registrations get AI **on** with defaults (terra
default, terra+luna allowed, 32 000 input chars, 2 048 output tokens,
200/user/day, 2 000/app/day; `--no-ai` opts out). Every pre-existing app and
every public/pending registration starts with AI **off** (no mass
migration). A public app can enable AI only with `public_policy
signed-in-quota` plus explicit quotas, and even then only signed-in users
with access can call it. Making an AI-enabled private app public without
that policy switches AI off in the same statement (audited). CLI: `app ai
show|enable|disable|set`.

### Agent/operator discovery and machine access

* `GET https://auth.repo.box/api/platform/v1` (= `/.well-known/repobox-platform.json`),
  `/api/platform/v1/openapi.json`, `/api/platform/v1/skill.md`: public,
  versioned, secret-free. `repobox-platform skill` prints the same skill.
* **Service tokens** (`service-token create|list|revoke`): `rbp_…` bearer,
  one owner, only that owner's apps (optionally `--app`-narrowed), scopes
  `apps:read apps:request ai:read ai:write routes:read release:read`,
  ≤ 90 days, hashed at rest, written once to a 0600 file. They are **not
  OAuth**; a scoped OAuth client-credentials issuer is the remaining
  dependency. No scope reaches users, grants, sessions, Caddy apply, SSH,
  the DB or provider credentials.
* REST: `whoami`, `apps`, `apps/{name}`, `apps/{name}/ai` (GET/PATCH within
  ceilings), `apps/{name}/route` (read-only preview), `app-requests`
  (GET/POST — a request the operator approves with `app requests approve`),
  `release`. MCP (`POST /api/platform/v1/mcp`, JSON-RPC, JSON responses only)
  exposes the same operations as tools with the same scopes. API mutations
  are audited with the token name.
* The full agent contract: `skills/repobox-platform/SKILL.md`. Rationale:
  `docs/platform-ai-plan.md`.

### External publisher: deploy by `docker save` upload (schema 7)

A **publisher** is a trusted first-party agent outside the repo.box
perimeter (the first one is `muse`). It deploys its own apps with one
credential, a **publisher token** `rbpub_…`, and the discovery document
`https://auth.repo.box/api/platform/v1` (`publishing` section, OpenAPI tag
`publisher`, `skill.md` §0, MCP publisher tools). It gets no SSH, Docker,
Caddy, database, ChatMock or browser access, and no view of any app it did
not create.

* **Principal.** Table `publishers`: an immutable `pub_<16 hex>` id, a
  handle, a display name and an **owner record** (a member user, by default
  a new one named after the handle). That user owns every app the publisher
  creates. `apps.publisher_id` marks those apps, and every publisher query
  is keyed on it. Tokens (`publisher_tokens`) are hashed, last at most 30
  days, are revocable, and are written once to a 0600 file. Disabling the
  publisher stops all of its tokens. Service tokens (`rbp_`) and publisher
  tokens are rejected on each other's API; cookies and `X-RepoBox-*` never
  authenticate here. Exactly one `Authorization` header is required. Not
  OAuth: an OAuth client-credentials issuer could replace token issuance
  later without changing the principal model.
* **Contract.** One HTTPS request, `POST /api/platform/v1/publisher/releases`,
  with `multipart/form-data` parts in this order:
  1. `manifest`: JSON with `name`, `title`, and optional `description`,
     `version`, `runtime` (`port`, `health_path`, `memory_mb`, `env`), `ai`
     and `provenance`.
  2. `image`: `docker save` output (tar or gzip), `podman save`
     (docker-archive or oci-archive), or an OCI layout tar. It must hold
     exactly one linux/amd64 image and be at most 2 GiB.

  The manifest is validated, and the name is checked against the registry,
  the Caddyfile's site addresses, the rendered apps and
  `/var/www/repo.box/subdomains`, *before* any image byte is accepted. Every
  collision answers `409 name_unavailable`. On a refusal the rest of the
  upload is read and discarded, because Caddy would otherwise turn the
  answer into a 502. Every malformed request is answered with `expected`,
  which is the exact request to send. `?wait=N` long-polls.
* **Split of authority.**
  * `repobox-platform.service` (the non-root `repobox-platform` user)
    streams the upload to `/var/spool/repobox-publisher/uploads`
    (sha256 computed on the way), records a `queued` release
    (`publisher_releases`) and writes a job into `jobs/`. That is all it can
    do: its write access covers only `jobs/`, `uploads/` and `queries/`.
  * `repobox-publisher-worker.path` → `repobox-publisher-worker.service`
    (root oneshot, systemd-sandboxed; never opens the registry) handles the
    job:
    1. Re-hashes the upload and streams a copy whose `manifest.json` /
       `index.json` name records are replaced by
       `repobox-pub/<app>:<release>` (`repositories` dropped). The tags the
       archive carried are therefore never applied on the host; this was
       verified on the live containerd image store before the build.
    2. Runs `docker load`, then checks the image is linux/amd64.
    3. Starts the container from a fixed argv template: network
       `repobox-published` (bridge `rbpub0`, 172.31.240.0/24, no
       inter-container traffic; iptables drops new connections from the
       bridge to the host and to private/link-local ranges); a
       `127.0.0.1:<4600-4999>` port; `--memory`, 1 CPU, pids 512;
       `no-new-privileges`; NET_RAW/MKNOD dropped; restart
       `unless-stopped`; volume `rbpub-<app>-data:/data`.
    4. Health-gates the new container while the old release keeps serving.
    5. Renders `/etc/caddy/repobox-platform/published.caddy` from its own
       state (validated labels and loopback ports only, with the standard
       gated route incl. `/_repo_box/ai`) and applies it with
       `caddy-apply.py apply-published`: refuse hosts defined elsewhere,
       back up, validate the whole config, reload, restore on failure.
    6. Retires the old container and keeps 3 release images for rollback.

    Progress, the verdict and a deploy log land in `results/` (root 0755)
    and are folded into the registry by the service (every request plus a
    2 s tick; a release unanswered for 1 h becomes `failed/timeout`).
  * `repobox-publisher-query.path/.service` answers runtime log/status
    queries (`docker logs`/`inspect` of the app's current container only),
    so logs stay available during a long deploy.
* **Lifecycle.** A new app is private, identity `platform`, AI on (policy
  default) and owned by the publisher's owner record. An upload with the same
  name updates the app: title/description change; identity, visibility,
  grants and AI policy are preserved. Release states: `queued → building →
  starting → live | failed`; the previous live release becomes `superseded`,
  and a restart ends as `done`. `rollback` runs a retained release again
  (new release record, same image, `/data` kept); `restart` restarts the
  live container. Limits: 10 apps per publisher, 100 releases per 24 h, one
  release in progress per app, 3 GiB total memory across published apps
  (worker flag).
* **Operator CLI.** `publisher create|list|show|disable|enable`,
  `publisher token create|list|revoke`, `publisher releases`,
  `publisher release ID` (with the deploy log),
  `publisher remove-app NAME [--purge-data] --yes`. `routes render`
  excludes publisher apps, and `app remove` refuses them.

## Local development

```bash
cargo test -p repobox-platform                       # unit + integration tests (tests/ai.rs: AI path)
cargo clippy -p repobox-platform --all-targets -- -D warnings
cargo fmt -p repobox-platform -- --check
repobox-platform/scripts/edge-e2e.sh                 # real Caddy + real broker + fake ChatMock on loopback
repobox-platform/scripts/publisher-e2e.sh            # real Caddy + real Docker: upload -> live -> launch -> AI -> update/rollback/remove
```

Run it by hand:

```bash
P=target/debug/repobox-platform; DB=/tmp/platform.db
$P --db $DB user create fran --display-name Fran --admin
$P --db $DB app register demo --title Demo --owner fran --kind proxy --target 127.0.0.1:3231
$P --db $DB serve --bind 127.0.0.1:3230 --public-base https://auth.repo.box
$P --db $DB routes render --gate 127.0.0.1:3230           # prints the Caddy snippet
```

`edge-e2e.sh` is the reference for exercising the full contract locally (it
uses Caddy's internal CA on port 8443 and `curl --resolve`).

## Deployment (repo.box host, from Hetzner)

Layout on the host:

| Path | Purpose |
|---|---|
| `/srv/repobox-platform/bin/repobox-platform` | static musl binary (CLI + services); `.prev-<stamp>` copies kept by each deploy |
| `/usr/local/bin/repobox-platform` | operator CLI wrapper (root via sudo; runs the binary as `repobox-platform`, hands `--out` files back to the caller 0600) |
| `/srv/repobox-platform/apps/<name>/` | static app roots (root-owned, world-readable) |
| `/srv/repobox-platform/caddy-apply.py` | Caddyfile apply/rollback helper |
| `/var/lib/repobox-platform/platform.db` | registry (owner `repobox-platform`, dir 0700, files 0600; `spool/` is the wrapper's private hand-off dir) |
| `/etc/repobox-platform/ai-bridge.secret` | AI bridge secret (root 0600), loaded via `LoadCredential=` |
| `/etc/caddy/repobox-platform/apps.caddy` | rendered managed routes |
| `/etc/caddy/Caddyfile` | contains the `# BEGIN/END repobox-platform managed` block |
| `/etc/caddy/backups/Caddyfile.pre-repobox-platform-<stamp>` | pre-change backups |
| `/home/fran/backups/repobox-platform/platform-<stamp>.db` | DB backups taken by every deploy |
| `/home/fran/secrets/repobox-platform-<user>-<stamp>.url` | operator-created device links (0600) |

Services: `repobox-platform.service` (127.0.0.1:3230, user
`repobox-platform`, non-login system user; `IPAddressAllow=localhost`) and
`repobox-platform-demo-private.service` (127.0.0.1:3231, `DynamicUser`). Ports are claimed in
`~/clawd/PORT-REGISTRY.md`.

```bash
repobox-platform/scripts/deploy.sh          # gates → build → ship → install → registry → Caddy → sweep
NO_CADDY=1 repobox-platform/scripts/deploy.sh   # everything except the Caddy change
```

The Caddy step (`caddy-apply.py apply --apps <rendered>`) backs up the
Caddyfile and `apps.caddy`, swaps either the existing managed block or the
exact legacy `auth.repo.box → 127.0.0.1:3005` block, installs the rendered
routes, runs `caddy validate` on the candidate, and only then installs and
reloads; a failed validation or reload restores both files automatically. If
neither block matches exactly, or the block is a route-only fragment, or a
hostname would be defined both by the Caddyfile and by the rendered routes, it
aborts and changes nothing.

Legacy note: the old auth-proxy on port 3005 (a domain-wide `media_auth`
cookie) is not running; `auth.repo.box` returned 502 before this change.
`uniswap.repo.box` still forward-auths to `127.0.0.1:3005/auth` and is not
modified; `/set-token*` and `/login*` on `auth.repo.box` are kept pointing at
3005 so that contract is untouched.

### Operator tasks

```bash
P=/usr/local/bin/repobox-platform   # sudo + runuser as repobox-platform; --out files land 0600, owned by you
$P bootstrap-admin --name fran --out /home/fran/secrets/repobox-platform-fran-$(date -u +%s).url
$P user create ocean --display-name Ocean               # then: $P user enrol ocean --out <0600 file>
$P app register myapp --title "My app" --owner fran --kind proxy --target 127.0.0.1:3299
$P app grant myapp --user ocean
$P routes render --check-roots --out /tmp/apps.caddy \
  && sudo install -m 0644 /tmp/apps.caddy /etc/caddy/repobox-platform/apps.caddy \
  && sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile \
  && sudo systemctl reload caddy
$P audit --limit 50
$P app stats myapp --days 14                              # access counters, no identity
$P app visits myapp --days 30                             # opens by signed-in people (see Visits)
$P app register myapp --title T --owner u --kind proxy --target 127.0.0.1:PORT \
   --visibility private --identity platform                # private needs the declaration
$P app attest myapp --note "login removed, verified …"    # pre-policy app, after review
$P app list                                               # IDENTITY column + WARNING for pending private apps
$P app transfer-owner myapp --owner ocean                # audited; route/visibility/enabled/grants kept
$P user sessions ocean                                    # live device + app sessions, no secrets
$P user consolidate --keep ocean-2 --retire ocean --name ocean
#   folds `ocean` into `ocean-2` (which keeps its id and therefore its sessions),
#   moves apps + grants, revokes the retired user's sessions and unused links,
#   renames the retired row `retired-<id>-ocean`, renames the kept user `ocean`;
#   one transaction, one `user.consolidate` audit row
```

Static app roots must live under one of the directories given to
`routes render --apps-root` (repeatable; `deploy.sh` passes `STATIC_ROOTS`,
default `/srv/repobox-platform/apps /var/www/repo.box/subdomains`).

Retiring a legacy standalone site block once its generated route exists:

```bash
RETIRE_HOSTS="myapp.repo.box" repobox-platform/scripts/deploy.sh
# = caddy-apply.py apply <block> --apps <rendered> --retire myapp.repo.box
```

`--retire` removes exactly one top-level `myapp.repo.box { ... }` block (plus
the comment lines glued directly above it). It refuses when the host is
defined by zero or several blocks, when the block also serves a host that is
not being retired, or when the rendered routes do not define the host exactly
once; the Caddyfile and `apps.caddy` are backed up, validated and reloaded as
one change and restored together on failure. `CADDY_DRY_RUN=1` validates the
whole change and installs nothing.

## Backup and restore

* **Backup** (online, consistent, safe while serving):
  `$P backup --out /home/fran/backups/repobox-platform/platform-$(date -u +%Y%m%dT%H%M%SZ).db`
  (refuses to overwrite; file is 0600). `deploy.sh` does this before every
  install.
* **Restore**: `sudo systemctl stop repobox-platform && cp <backup> /var/lib/repobox-platform/platform.db && rm -f /var/lib/repobox-platform/platform.db-wal /var/lib/repobox-platform/platform.db-shm && sudo systemctl start repobox-platform`,
  then `routes render` + Caddy reload if the app set differs.
* **Schema migration**: the schema is applied with `CREATE TABLE IF NOT EXISTS`
  on every open, so upgrading is just installing the new binary and
  restarting. Version 2 (2026-09-16) added the three `access_*` tables; a
  version-1 backup restores fine and simply starts with empty counters.
  Version 3 added `sessions.parent_id` and `tokens.session_id` through a
  guarded `ALTER TABLE ... ADD COLUMN` on open (`ADDED_COLUMNS`); pre-existing
  app sessions have no parent and are revoked individually or by "Sign out
  everywhere". The
  `backup` step of `deploy.sh` runs with the *new* binary against the live
  DB, which creates the new tables before the service restarts; that is safe
  because the old binary ignores tables it does not know.

## Test procedure

1. `cargo test -p repobox-platform` — covers: code redemption → host-only cookie
   + clean redirect (other query params preserved), replay rejection, expiry,
   app binding of codes, grant revocation killing live sessions, disabled
   app/user denial, the three visibility modes at the gate and in the
   directory, spoofed `X-RepoBox-*` headers never influencing a decision,
   app-session binding to one app, launch requiring sign-in and access,
   open-redirect hardening of `next`, single-use POST-only device links,
   invitation flow (new and existing users), owner/admin-only management, CSRF,
   directory partitioning (Your apps before Other apps, admin/owner/grantee/
   anonymous views), public-unlisted absent unless in the access list, access
   counting only on allowed requests (denials, disabled, replay and the
   redemption redirect do not count), unique-user dedup for private apps and
   no identity for public apps, 90-day pruning, counter-only access tables,
   and owner/admin-only analytics (a grant is not ownership; other owners get
   403), opens: the document-navigation classifier (unit tests over
   `Sec-Fetch-Dest`/`Sec-Fetch-Mode`/`Accept`/`Upgrade`/`Sec-Purpose`/methods/
   file extensions), end to end at the gate (redemption redirect, assets,
   fetches, WebSocket, iframe, HEAD/POST and `.json` navigations are counted
   as requests but never as opens; page loads inside the 30-minute window
   are one open, a return after it a second; per app, per person; revoked
   grant denied; anonymous public page loads never, signed-in public launches
   yes), 30-minute inactivity dedup, range clamping and 90-day pruning in
   the store, schema 3 → 4 migration (opens tables appear, version bumps),
   opens tables hold only ids and timestamps, the visits page
   (401/403/404/200 matrix, handle + display name, other apps absent, range
   picker fallback, cross-links and the "not visits" label on Requests), the
   grant typeahead endpoint (401/403/404/200 matrix, `{name, display_name}`
   only, owner and granted users excluded, disabled users never suggested,
   handle and display-name matching, empty result for no match, the combobox
   markup and key handlers present on the manage page, suggestions never
   injected as HTML, a picked handle posts to the unchanged grant endpoint),
   own-session listing and revocation (device revoke cascades to its
   app sessions at the gate, app-session revoke leaves the device signed in,
   logout cascades), member cannot revoke another user's session even with
   its id, admin per-user page and per-row revoke, admin "sign out
   everywhere", admin-only and CSRF guards on all of it.
2. `repobox-platform/scripts/edge-e2e.sh` — same contract through a real Caddy
   with the generated routes (proves the strip + forward_auth + copy_headers
   mechanics, host-only/HttpOnly cookies, no secrets in logs), plus opens:
   curl's `*/*` requests through the launch flow leave zero opens, one
   browser-style page load then a second one in the same visit plus an
   asset and an API fetch leave exactly one open by `bob` in
   `app visits demo-private`, an anonymous browser-style load of the public
   app leaves zero, and the visits page / suggestion endpoint answer 401
   anonymous and 403 to a member.
3. Live (after deploy):
   ```bash
   curl -sI https://demo-private.repo.box/ | head -1                # 401
   curl -sI -H 'X-RepoBox-User: x' https://demo-private.repo.box/   # 401 (spoof stripped)
   curl -sI https://demo-unlisted.repo.box/ | head -1               # 200
   curl -sI https://demo-listed.repo.box/ | head -1                 # 200
   curl -s https://auth.repo.box/api/directory                       # demo-listed only
   curl -s https://auth.repo.box/ | grep -c 'id="public-apps"'      # anonymous: public directory only
   # launch: sign a device in (device link), then
   curl -s -b jar -o /dev/null -w '%{redirect_url}' https://auth.repo.box/demo-private   # ...?rb_launch=
   curl -s -c appjar -o /dev/null -w '%{http_code} %{redirect_url}' "<that url>"         # 302 https://demo-private.repo.box/
   curl -s -b appjar https://demo-private.repo.box/whoami.json                            # x-repobox-user
   ```
4. Origins are not public: `ss -ltn` on the host shows 3230/3231 bound to
   127.0.0.1 only; from outside `curl http://204.168.190.248:3230/` fails.

## Deployment record (2026-09-16)

* Host: `204.168.190.248` (repo.box). Services `repobox-platform` (127.0.0.1:3230) and
  `repobox-platform-demo-private` (127.0.0.1:3231) active; `ss -ltn` shows both bound to
  loopback only and external probes to those ports fail.
* Caddy: legacy `auth.repo.box → 127.0.0.1:3005` block replaced by the managed block;
  `caddy validate` OK, `systemctl reload caddy` OK. Backup:
  `/etc/caddy/backups/Caddyfile.pre-repobox-platform-20260916T003645Z` (plus a manual
  `Caddyfile.pre-repobox-platform-manual-20260916T003639Z`). Let's Encrypt certificates for
  the three demo hosts were issued ~7 s after the reload.
* Live URLs: https://auth.repo.box/ · https://demo-private.repo.box/ (401 anonymous) ·
  https://demo-unlisted.repo.box/ (200, not in directory) · https://demo-listed.repo.box/
  (200, in directory and `/api/directory`).
* Verified live with curl (53 checks) and in a real browser at 1280×900 and 390×844:
  device link → directory → launch → clean URL on the app host, cookies host-only
  (`__Host-` prefix, no `Domain`), HttpOnly, SameSite=Lax; no browser storage used;
  spoofed `X-RepoBox-*` headers ignored; replay/expiry/revocation/disable all enforced;
  journal contains no launch code.
* Admin bootstrap: user `fran` (admin). Their single-use device link (7 days) was written
  to a 0600 file on the host (`/home/fran/secrets/repobox-platform-fran-20260916T003644Z.url`)
  and mirrored to `~/clawd/secrets/repobox-platform-fran-enrol-20260916.url` on the build
  box. It has not been printed anywhere. A fresh one can be made with `user enrol fran`.
* Throwaway users `review-bot` and `review-admin` were used for verification and are
  disabled (their unused device links are therefore dead).

## Rollback

```bash
# Caddy (auth.repo.box back to the legacy 3005 proxy, managed app routes gone):
sudo python3 /srv/repobox-platform/caddy-apply.py rollback /etc/caddy/backups/Caddyfile.pre-repobox-platform-<stamp>
# Services:
sudo systemctl disable --now repobox-platform repobox-platform-demo-private
```
The registry and backups stay on disk; nothing else on the host was changed.

Exercised live on 2026-09-16: rollback restored the legacy config (auth.repo.box
502 again, demo hosts unreachable), re-apply brought everything back with
`caddy validate` + reload OK. The current pre-change backup is
`/etc/caddy/backups/Caddyfile.pre-repobox-platform-20260916T004359Z`. The live
Caddyfile must stay 0644 (the `caddy` user reads it on reload); `caddy-apply.py`
enforces this on both apply and rollback.

## Deployment record (2026-09-16, directory + access counting)

* Commit `1be2af84` deployed with `NO_CADDY=1 repobox-platform/scripts/deploy.sh`
  at 10:38 UTC. No route or host changed, so Caddy was neither re-applied nor
  reloaded: the Caddyfile still carries its 00:43 UTC mtime and the caddy unit
  has been up since 2026-08-05 with no reload event in its journal. The deploy
  restarted only the two platform units, which came back active with `/healthz`
  OK and both listeners bound to 127.0.0.1 only.
* Pre-deploy registry backup: `/home/fran/backups/repobox-platform/platform-20260916T103827Z.db`
  (schema 1). After the restart `meta.schema_version` is `2` and the three
  `access_*` tables exist.
* Live evidence: the deploy sweep itself counted one allowed request each on
  `demo-listed` and `demo-unlisted` while `demo-private` (401 in the sweep)
  stayed at zero. Two short-lived throwaway users (`review-member` with a grant
  on `demo-private`, `review-admin2`) were enrolled through their device links
  in a real browser at 1280×900 and 390×844: the member's directory shows
  *Your apps* (demo-private) above *Other apps* (demo-listed) with no unlisted
  app and no Manage button; the admin's directory shows all three under *Your
  apps* only; launching `demo-private` landed on the clean app URL with
  `x-repobox-user=review-member` at the origin and a host-only HttpOnly
  `__Host-rb_app` cookie; the member got 403 on the analytics page; the
  admin's analytics page showed 3 allowed requests / 1 signed-in user for
  `demo-private` and "not kept for public apps" for `demo-listed`; no page had
  horizontal overflow at 390 px. Anonymous curl: *Public directory* with the
  listed demo only, `/api/directory` unchanged, private 401 with and without
  spoofed `X-RepoBox-*` headers, analytics 401, gate 404 via the public host.
  After disabling both throwaway users, three anonymous 401s on `demo-private`
  left its counter at 3. The platform journal contains no token or URL. The
  consumed device-link files were deleted.

## Deployment record (2026-09-16, session revocation)

* Commit `ef63ce9b` deployed at 12:31 UTC with `NO_CADDY=1 deploy.sh`; Caddy
  again untouched (same Caddyfile checksum and mtime, no reload, caddy unit up
  since 2026-08-05). Pre-deploy backup
  `/home/fran/backups/repobox-platform/platform-20260916T123151Z.db` (schema 2).
  After restart `schema_version` is 3 and `sessions.parent_id` /
  `tokens.session_id` exist; the access counters from the morning survived.
* Live browser run (Playwright, 1280×900 and 390×844) with throwaway users
  `review-member3` (grant on demo-private), `review-member4`, `review-admin3`,
  35 checks, all passed: device A signed in through its CLI link and minted a
  second device link from `/me`; device B (iOS user agent) signed in with it;
  both launched demo-private and were identified at the origin; `/me` on A
  listed 2 devices and 2 app sessions with "this device" on A's rows and no
  horizontal overflow at 390 px; A signed B out from `/me`, after which B got
  401 on `/me` and 401 at the demo-private gate on its very next request while
  A stayed 200 on both; the admin page for the member showed 1 device + 1 app
  session, "End session" made A's app session 401 at the gate while A's device
  stayed signed in, and "Sign out everywhere" emptied the page and made A's
  `/me` 401; a plain member got 403 on the admin page. Audit shows
  `session.revoke` (`auth`, and `admin app`) and `session.revoke_all` (`admin 1`)
  with the right actors and subjects. The platform journal has no token or
  URL. The throwaway users were disabled and their link files shredded.
* A follow-up commit only changes the relative-time label for very recent
  timestamps ("just now" instead of "in 0s") and this record; deployed the
  same way.

## Deployment record (2026-09-17, Ellie Beaumont migration)

Scope, authorised by Fran: bring Ellie's existing repo.box apps under the
platform with a single Ellie identity, fix the launch handoff that failed on
her first live launch, and retire the standalone legacy Caddy blocks for
exactly those hostnames. Fieldwork (Fran's) was not touched.

* **Launch handoff bug.** The first live launch (13:43 UTC) minted and
  redeemed a code correctly, then the app's very next request was answered
  403 "This link is not valid" by the gate. Cause: the gate treated *every*
  `?token=` query parameter as a launch code, and all three of Ellie's apps
  use `?token=` for their own setup/invite links (`/api/setup/check?token=`,
  `/invite?token=`). Fix: the code now travels as `?rb_launch=` (shared
  constant `LAUNCH_PARAM`), `token` is passed through untouched, and a stale
  or replayed code presented by a browser that already holds the app session
  redirects to the clean URL instead of 403 (audited as `launch.reject …
  (session kept)`). Regression tests: `apps_own_token_parameter_passes_through_the_gate`,
  `stale_code_on_a_signed_in_browser_is_dropped_not_rejected`, plus two new
  checks in `edge-e2e.sh`.
* **Identity.** Live inspection (read-only) showed the enrolled device
  belonged to user id 10 `ellie-beaumont-study-diary` (device sessions 24 and
  28, "Chrome on Windows PC", three study-diary app sessions 25–27), while id 9
  `ellie-beaumont` had no session and one unredeemed enrolment link. Per
  Fran's decision the enrolled row was kept: new CLI
  `user consolidate --keep ellie-beaumont-study-diary --retire ellie-beaumont --name ellie-beaumont`
  ran on the host at 14:26 UTC in one transaction: id 10 is now named
  `ellie-beaumont` (sessions untouched and verified live with
  `user sessions`), id 9 became `retired-9-ellie-beaumont`, disabled, its
  unused link revoked (`tokens_revoked=1`), audit row `user.consolidate`.
  No device link was generated or sent.
* **New CLI**: `app transfer-owner <app> --owner <user>` (atomic owner +
  `updated_at`, audited `app.transfer_owner`, refuses unknown/disabled users
  and stale snapshots; route/visibility/enabled/grants untouched),
  `user consolidate`, `user sessions` (no secrets). `routes render
  --apps-root` is repeatable so registry static roots may live under
  `/var/www/repo.box/subdomains` as well as `/srv/repobox-platform/apps`.
  45 tests (21 unit, 24 integration), clippy `-D warnings`, rustfmt clean.
* **Caddy helper**: `caddy-apply.py apply --apps <rendered> --retire <host>…`
  installs the rendered routes and removes exact legacy site blocks in the
  same backed-up, validated, reloaded change (restores both files on
  failure); the route-fragment guard from 137f7ff3 is kept; `--dry-run`.
  `deploy.sh` gained `STATIC_ROOTS`, `RETIRE_HOSTS`, `CADDY_DRY_RUN` and a
  registry-driven sweep (every registered app: 401/200/404 by visibility and
  state, spoofed identity headers still 401, directory count per app).
* **Origins.** `ellies-japanese` ran as a host-network container binding
  `0.0.0.0:3417` (the app hardcodes `0.0.0.0` in production; ufw already
  blocked it externally, confirmed by timeout from Hetzner). Service
  configuration only, originals in `/root/backups/ellies-japanese-20260917/`:
  the unit now runs the container on the bridge with
  `-p 127.0.0.1:3417:3417 --add-host=host.docker.internal:host-gateway`, and
  `AI_BRIDGE_URL` points at `host.docker.internal:3429` (the pattern
  uni-kitchen already uses). The restart exposed an unrelated latent fault:
  the app refuses to boot when `SETUP_TOKEN_EXPIRES_AT` is in the past, and
  it had expired on 2026-09-16 (any restart since then would have failed).
  The owner account is already set up, so that token is inert; the expiry
  was moved to `2030-01-01T00:00:00Z` to let the service start. Follow-up for
  the app itself: accept an expired setup token once an owner exists. After
  the change: unit active, `127.0.0.1:3417` via docker-proxy only, `200`
  through Caddy, AI bridge reachable from inside the container. All six
  origins now bind loopback (3417, 3022, 4184, 3025 checked with `ss`); none
  answers from outside (3417/3022/4184/3025 time out from Hetzner).
* **Registry** (all owner `ellie-beaumont`): `ellies-japanese` proxy 3417
  private; `academicweapon` proxy 3022 private; `uni-kitchen` proxy 4184
  private; `study-diary` proxy 3025 private (already registered, owner
  unchanged by the consolidation); `puzzlenest` static
  `/var/www/repo.box/subdomains/circuit` public_listed; `circuit` same root,
  public_unlisted (reachable alias, not a second directory entry).
* **Deploy.** 14:23 UTC `NO_CADDY=1` (binary + helper; DB backup
  `platform-20260917T142328Z.db`), then 14:26 UTC full deploy with
  `RETIRE_HOSTS="ellies-japanese.repo.box academicweapon.repo.box uni-kitchen.repo.box circuit.repo.box puzzlenest.repo.box"`
  after a `CADDY_DRY_RUN=1` pass (DB backup `platform-20260917T142643Z.db`).
  Caddy backups `/etc/caddy/backups/Caddyfile.pre-repobox-platform-20260917T142646Z`
  and `apps.caddy.pre-repobox-platform-20260917T142646Z`. Retired blocks
  (old line numbers): ellies-japanese 1258–1267, academicweapon 1268–1275
  (with its two glued comment lines), uni-kitchen 1302–1304,
  `circuit.repo.box, puzzlenest.repo.box` 1305–1317. Afterwards the Caddyfile
  defines none of the five hosts, `apps.caddy` defines all ten managed apps
  once, `caddy validate` is clean, the live file is 0644.
* **Live results.** Anonymous: ellies-japanese, academicweapon, uni-kitchen,
  study-diary → 401 with the platform "is private / Sign in and launch" page
  (origins untouched), also 401 with spoofed `X-RepoBox-*` headers;
  puzzlenest and circuit → 200 with byte-identical HTML, CSS asset 200;
  `/api/directory` lists Puzzle Nest once and not circuit. Handoff retest
  on production with throwaway `review-handoff-142911` (granted on
  demo-private and ellies-japanese, then revoked, disabled and its link file
  shredded): enrol via curl 200/303, launcher redirects with `rb_launch`,
  gate 302 to the clean URL, origin sees the throwaway identity, an
  app-owned `?token=` reaches the origin with identity intact, replay without
  a session 403, and on the real ellies-japanese app `/api/setup/check?token=probe`
  is answered by the app (`200 {"valid":false}`), not by the gate. The
  stale-code-with-session fallback was verified locally against a real
  Caddy and by the integration tests (the live script's copy of that check
  had a shell-quoting bug and is not counted). Platform journal since the
  deploy contains no launch code.
* **Behaviour differences vs. the legacy blocks, accepted:** Puzzle Nest no
  longer has the `try_files … /index.html` fallback, so unknown paths return
  404 instead of the index (its bundle has no history routing);
  `academicweapon.repo.box/healthz` is now gated (401) instead of the legacy
  404 rule; `encode zstd gzip` is not emitted by the renderer.
* **Deferred:** none of the six legacy routes remain; the app-side fix for
  the ellies-japanese startup check is a follow-up in that app's repository.

## Deployment record (2026-09-17, visits/opens, grant typeahead, identity policy)

Scope, per Fran: owner/admin stats for how often people *open* an app,
built on the platform identity and privacy-conscious; a typeahead for the
Grant control; then the platform identity contract as mandatory policy
with Study Diary converted (recorded in the next section). Three
`NO_CADDY=1` deploys, no Caddy change, routes untouched (live `apps.caddy`
still dated 14:26:46 UTC).

* **Deploys.** `56f16f86` at 15:33 UTC (visits + typeahead; DB backup
  `platform-20260917T153348Z.db`, schema 3 → 4 on start); `9cc0a161` at 15:44
  UTC (typeahead selection fixes found in a real browser; backup
  `platform-20260917T154403Z.db`); `03051fdb` at 15:56 UTC (identity
  policy; backup `platform-20260917T155609Z.db`, schema 4 → 5 on start:
  `apps.identity` added with default `pending`). Each deploy: fmt, clippy
  `-D warnings`, 27 unit + 28 integration tests, static musl build, host
  binary SHA-256 matched the local build (`a5eae4d8…`, then `4e5c3d00…`),
  service restart, registry-driven live sweep green (all private hosts 401
  incl. spoofed identity, public 200, disabled 404, directory counts).
* **Local proof.** `edge-e2e.sh` through a real Caddy 2.10.2: curl's `*/*`
  requests through the whole launch flow leave zero opens; one browser-style
  page load, a second one in the same visit, an asset and an API fetch leave
  exactly one open by `bob`; an anonymous browser-style load of the public
  app leaves none; visits page and suggestion endpoint answer 401/403 to
  anonymous/member; the CLI refuses an undeclared private registration; the
  manifest lines appear; attest → private works. Playwright (Chromium,
  behind a local `local_certs` Caddy so `__Host-` cookies apply): 28 checks
  on the combobox: `role=combobox`, `aria-expanded` toggling, ArrowDown sets
  `aria-activedescendant` and `aria-selected`, Enter fills the canonical
  handle without submitting, the pick survives a suggestion response that
  lands after the key press, no-match row, disabled account never suggested,
  Escape closes, ArrowUp from nothing goes to the last option, mouse pick,
  a second Enter posts to the unchanged grant endpoint and the grants table
  shows exactly the picked person, a granted person disappears from
  suggestions, and with JavaScript off the typed handle still grants.
* **Live evidence, opens** (throwaway `vprobe-a-153442`, granted on
  demo-private, enrolled via curl; plus `vprobe-b-…` as a typeahead target
  and throwaway-owned `vprobe-app-…` registered without a route): the
  launch redirect left no open; one browser-style page load through the
  real edge, then a second load, an API fetch and a `*/*` curl left exactly
  **1 open** in `app visits demo-private --days 1`; visits page 401
  anonymous / 403 for the member (also 403 on suggestions and Requests); the
  throwaway owner's own visits page rendered with the empty state and named
  no other app; suggestions for `?q=<probe-b>` returned exactly
  `{"display_name":"Visits Probe B","name":"vprobe-b-…"}`, the owner itself
  and a nonsense query returned `{"users":[]}`, the payload carried no ids,
  sessions or tokens, `Cache-Control: no-store`; granting through the
  unchanged endpoint (303 `ok=grant_added`) removed the person from the
  suggestions. Journal since the deploy: 1 line, no `rb_launch=`. Teardown:
  grant revoked, app removed (its opens cascade), both users disabled, link
  shredded. The probe's single open on demo-private remains as a 90-day row
  (`app_opens`: 1 row, `app_visits`: 1 row at the time).
* **Live evidence, policy** (15:56 UTC): after the deploy `app list`
  warned about 6 pending private apps; `app attest` run for study-diary
  (converted), demo-private, demo-unlisted, demo-listed (platform demos
  without any login) → 4 audit rows `app.attest`; `app list` now shows
  IDENTITY `platform` for those four and warns about the remaining four
  (`academicweapon, ellies-japanese, fieldwork, uni-kitchen`); a private
  registration without `--identity platform` is refused and leaves no app;
  `app visibility ellies-japanese private` is refused with the policy
  message; `routes render` to the staging file prints one WARNING per
  pending private app and the file carries `# identity:` for all 10 apps
  (4 platform, 4 PENDING REVIEW, 2 pending public) while differing from the
  live `apps.caddy` by nothing but those comment lines. Live Caddy was not
  touched.

## Deployment record (2026-09-26, platform AI capability + machine API)

Commit `f87beb0a` (+ this docs follow-up) on `feat/platform-auth-control-plane`.

* **Hetzner** (`scripts/deploy-ai-bridge.sh`): `repobox-ai-broker`
  (`/opt/repobox-platform/bin`, `DynamicUser`, `127.0.0.1:8127`) and
  `repobox-ai-tunnel` (`ssh -R 127.0.0.1:3232:127.0.0.1:8127` to repo.box).
  Bridge secret created once at `/etc/repobox-platform/ai-bridge.secret`
  (root 0600) and copied to the same path on repo.box over ssh stdin; never
  printed. Verified: broker and ChatMock bound to loopback only, broker not
  reachable on the public IP, secret required locally and through the
  tunnel, tunnel end on repo.box bound to `127.0.0.1:3232` only.
* **repo.box** (`scripts/deploy.sh`, full): DB backup
  `/home/fran/backups/repobox-platform/platform-20260926T231116Z.db`;
  previous binary kept as `/srv/repobox-platform/bin/repobox-platform.prev-20260926T231116Z`;
  schema 5 → 6 in place (every existing app: AI **off**); routes re-rendered
  for all 22 apps (only change: the reserved `/_repo_box/*` handles; dry run
  on a registry copy first), `caddy validate` OK, reload OK. Caddy backups
  `/etc/caddy/backups/Caddyfile.pre-repobox-platform-20260926T231119Z` and
  `apps.caddy.pre-repobox-platform-20260926T231119Z`. Rollback:
  `sudo python3 /srv/repobox-platform/caddy-apply.py rollback /etc/caddy/backups/Caddyfile.pre-repobox-platform-20260926T231119Z --apps /etc/caddy/backups/apps.caddy.pre-repobox-platform-20260926T231119Z`,
  then reinstall the `.prev-…` binary and the previous unit (no
  `LoadCredential`/`--ai-upstream`), and `deploy-ai-bridge.sh rollback` on
  Hetzner.
* **Sweep:** every host's page answer unchanged; anonymous + forged AI call
  401 on all 21 enabled hosts, 404 on the disabled one; discovery 200,
  `/api/platform/v1/apps` 401, `auth.repo.box/gate/ai/…` 404. One pre-existing
  failure: `fieldwork-write` (registry says public) answers 404, the same
  drift recorded on 2026-09-25; not touched.
* **Live E2E with a disposable identity** (`aiprobe-231148`, granted on
  demo-private, AI enabled there for the test): anonymous app/launcher/AI
  401; forged identity + gate headers 401; enrol → launcher 302 → launch
  code → app 200; auth session alone does not unlock AI (401); same-origin
  POST returned a real `gpt-5.6-terra` completion (≈4 s first call); models =
  terra, luna; the session with forged admin headers was served and logged as
  user 27 (the probe), never as fran; cross-origin 403, `stream` 400,
  `gpt-5.6-sol` 400, 40 000 chars 413, `tools` 400, `/_repo_box/whoami.json`
  404, the session on another host 401. Direct from repo.box loopback: tunnel
  without/with a fake secret 401, control-plane hop with forged headers and no
  cookie 401. Discovery, OpenAPI, skill, service-token whoami/apps/ai/release
  (broker reachable, schema 6, commit f87beb0a), `ai:write` missing 403,
  other app 404, MCP `tools/list` (9) and `get_ai_policy` all as specified.
  A prompt canary appeared in **no** journal on either host nor in ChatMock
  state; the bridge secret, `rbp_` tokens, `__Host-rb_*` cookies and
  `rb_launch=` codes appeared in no journal. Cleanup: probe user disabled
  (sessions revoked), grant and service token revoked, link/token/cookie
  files shredded, demo-private AI switched back **off**.
* **Stated limit found in deployment:** systemd exposes the credential to the
  service user (`/run/credentials/repobox-platform.service/ai-bridge-secret`,
  `fran` 0400). Any other process running as `fran` on repo.box could read it
  and call the broker directly through `127.0.0.1:3232`, bypassing per-app
  policy and quotas (still bounded by the broker's allowlist, ceilings and
  concurrency of 4, and never reaching beyond ChatMock). Closing this needs
  the control plane under its own system user (DB ownership move) — follow-up.

## Deployment record (2026-09-26 evening: launch-code log leak, service user)

Commit `46dd48c1` (+ docs), pushed and deployed.

* **Release blocker fixed — launch codes in edge logs.** Independent
  verification saw `edge-e2e.sh` fail "control plane log has no launch
  token" (1 hit). Reproduced with `RUST_LOG=trace` (timing only): the hit was
  in **Caddy's** log, `http.handlers.reverse_proxy` "aborting with incomplete
  response" / `writing: client disconnected`, which records `request.uri`
  (`/gate/verify?rb_launch=<code>`) and `X-Forwarded-Uri`. Trigger: a client
  hanging up while Caddy copied the gate's 403 HTML body for a replayed code.
  Fix: every gate answer to a code-bearing request is body-less (redeem 302,
  public/stale 302, rejection/disabled **303** to
  `https://auth.repo.box/<app>?launch_error=<kind>&next=<path>` — no code —
  where the launcher explains and offers *Launch again*, never
  auto-relaunching). HTTP/transport crates are capped at `info` regardless of
  `RUST_LOG`. `edge-e2e.sh` now runs a deterministic abrupt-client stress
  (TLS, hang up after headers) on code-bearing URLs and checks every log for
  `rb_launch=`, tokens, cookies, prompts and the bridge secret. The previous
  binary (`E2E_BIN=…`) fails it deterministically (40/40 bodies, 27 log
  hits); the fix passes 83/83, also under `RUST_LOG=trace`. Production: a bad
  code answers `303`, 0 bytes; 25 abrupt TLS clients got 0 bodies; no
  `rb_launch=` in any repo.box journal line since.
* **Control plane under its own system user.** `repobox-platform` (uid 995,
  nologin, no home). `deploy.sh` created it, stopped the service, chowned the
  registry (dir 0700, files 0600; were `fran` 0644), backed up
  (`platform-20260926T233014Z.db`), installed the hardened unit
  (`ProtectHome=true`, `IPAddressDeny=any` + `IPAddressAllow=localhost`,
  `UMask=0077`, kernel/namespace/SUID restrictions) and asserts the running
  uid on every deploy. Demo origin: `DynamicUser`. Operators:
  `/usr/local/bin/repobox-platform` (sudo + runuser; `--out` files written
  back as the caller 0600 with O_EXCL; the raw binary now prints a hint).
  Previous units: `/root/backups/repobox-platform-units-20260926T232929Z/`.
* **Proven on the host:** main PID owned by `repobox-platform`; 3230/3231
  bound 127.0.0.1 (3232 = the tunnel's sshd, 127.0.0.1); as `fran` — and as a
  `fran` service with NoNewPrivileges — the DB, the state dir, the credential
  (`/run/credentials/repobox-platform.service/`, `repobox-platform` 0400) and
  the service's `/proc/…/environ` are unreadable, and sudo is blocked in the
  NoNewPrivileges context; a transient unit with the same IP policy as the
  service user cannot reach the internet but reaches the gate. **Host-level
  limit (not changed here):** every `fran` service inherits the `docker`
  group (verified: the Docker socket is reachable even under
  NoNewPrivileges) and `fran` has NOPASSWD sudo, so a compromised `fran`
  service is still root-equivalent. Closing that needs apps off the `fran`
  account/docker group — a host-wide change.
* **Live E2E (disposable `aiprobe-233106`)**: all anonymous/forged/direct
  refusals, real `gpt-5.6-terra` completion, policy refusals, rejected-code
  checks, discovery/OpenAPI/skill/service token/MCP, release reports commit
  `46dd48c1`. Log search since the probe start (103 repo.box / 21 Hetzner
  journal lines): 0 hits for `rb_launch=`, prompt canary, bridge secret,
  `rbp_`, `__Host-rb_`; Caddy logged nothing during the stress. Revocation
  proven before shredding (token, app session, AI, device session all 401);
  user disabled, grant and token revoked, files shredded, demo-private AI off.
  Registry sweep: every host as expected except the pre-existing
  `fieldwork-write` 404.
* **Rollback (user migration):** `sudo systemctl stop repobox-platform`;
  `sudo chown fran:fran /var/lib/repobox-platform /var/lib/repobox-platform/platform.db*`;
  restore the unit files from `/root/backups/repobox-platform-units-20260926T232929Z/`
  and the `.prev-…` binary; `daemon-reload`; start.

## Deployment record (2026-09-27, external publisher: docker save upload)

Commits 9a6e89a8 and b5e528cf, deployed with `scripts/deploy.sh` from
Hetzner. The live binary sha256 prefix `00f7272d…` equals the local build of
b5e528cf.

* **Gates.** fmt; clippy `-D warnings`; 92 tests (38 unit, 16 AI, 31
  platform, 7 publisher; the deploy re-ran them); `edge-e2e.sh` ALL PASS;
  `publisher-e2e.sh` ALL PASS (real Caddy + real Docker: upload, live,
  launcher, same-origin AI, update, failed update keeps the old release
  serving, rollback, restart, logs, isolation, revocation, removal, no
  secrets in logs).
* **Spike before building.** On the live containerd image store,
  `docker load` of an archive whose names were rewritten
  (docker-save and OCI-only variants) produced exactly the rewritten name;
  an unnamed archive loaded dangling. The spike images were removed.
* **Rollout.**
  1. `CADDY_DRY_RUN=1`: binary + units installed, registry migrated to
     schema 7 (DB backups `platform-20260927T090929Z.db`, `…090953Z.db`,
     `…091438Z.db` in `/home/fran/backups/repobox-platform`); the managed
     block with `import …/published.caddy` validated.
  2. Guarded apply: Caddy backup
     `/etc/caddy/backups/Caddyfile.pre-repobox-platform-20260927T090957Z`,
     validate + reload OK.

  The sweep's only mismatch is the pre-existing `fieldwork-write` 404 drift
  (left untouched).
* **Boundary, proven on the host.**
  * The API runs as `repobox-platform` (uid 995), which gets
    `permission denied` on the Docker socket, has no sudo and no
    supplementary groups, and listens only on 127.0.0.1:3230. It can write
    only `/var/lib/repobox-platform` and the spool, and permissions confine
    it to `jobs/`, `uploads/` and `queries/`.
  * The worker is a root oneshot (ProtectSystem=strict, ProtectHome,
    NoNewPrivileges) with write access to the spool,
    `/srv/repobox-platform/published`, `/etc/caddy` and
    `/run/xtables.lock` only.
* **Bug found by the live run.** The first live release failed cleanly
  (`failed/internal`): systemd bind-mounts every `ReadWritePaths` entry
  separately, so the `uploads/ → jobs/` rename returned EXDEV. Fixed in
  b5e528cf: one spool path, orphan cleanup on queue failure, worker prunes
  unclaimed uploads. Redeployed with `NO_CADDY=1`.
* **Live external-style test** (disposable publisher `pubprobe-09270910`,
  `pub_7b6fd49a15701621`; token written 0600, copied to the agent side and
  shredded on the host; HTTPS only from Hetzner). The run, in order:
  1. Discovery `publishing` section read.
  2. Upload of a 56 MB `docker save | gzip` archive → `200 live` in ≈5 s
     (release record carried digest, format `docker-save+gzip`, image id,
     launcher URL and the edge-gated direct URL).
  3. Container checks: image `repobox-pub/<app>:<release>`, user `node`,
     not privileged, network `repobox-published` (172.31.240.0/24,
     icc=false, bridge rbpub0), port `127.0.0.1:4600` only,
     MKNOD/NET_RAW dropped, no-new-privileges, 128 MiB, pids 512, volume
     `/data`. Firewall rules present in DOCKER-USER and INPUT. The route
     rendered with the gated shape.
  4. Anonymous direct host 401, spoofed identity 401, anonymous AI 401.
  5. Operator-granted disposable viewer, driven in Chromium through
     `auth.repo.box/<app>`: clean redirect (no code in the URL), page v1,
     gate identity visible, same-origin AI button → 200 with the real
     model's answer.
  6. Update with a second, different image → v3 live, `/data` kept,
     access model unchanged. Runtime logs endpoint OK.
  7. Isolation and unauthorized requests:
     * Publisher lists only its own app.
     * `study-diary`, `demo-private` and `couchclub` answer 404 to
       get/restart/rollback/logs.
     * Taking the name `study-diary` → 409 `name_unavailable`.
     * The service API refuses the publisher token (401).
     * Anonymous requests, a bad token, two Authorization headers and an
       anonymous upload all get 401.
     * MCP `list_my_apps` shows only its own app.
  8. Live rollback → the v1 image again.
  9. Revocation → whoami, upload and MCP all 401.
  10. 0 hits for the token, `rbpub_`, `rb_launch=` or the enrol token in
      1,233 journal lines or the Caddy logs.
* **Cleanup.** `publisher remove-app --purge-data`: container, images,
  volume and route removed; `published.caddy` has 0 apps; Caddy validates;
  the host no longer resolves to a site. The publisher, its owner record
  and the viewer are disabled, the token revoked, spool results deleted,
  and local artifacts shredded or removed. Release rows remain as audit
  history (app link cleared).
* **Issuing the real credential** (not done; Fran's decision):
  `sudo /usr/local/bin/repobox-platform publisher create --handle muse`,
  then
  `sudo /usr/local/bin/repobox-platform publisher token create --publisher muse --name muse-1 --ttl-days 30 --out /home/fran/secrets/muse-publisher.token`.
  The agent starts at `https://auth.repo.box/api/platform/v1` and deploys
  with `POST https://auth.repo.box/api/platform/v1/publisher/releases`.

## Deployment record (2026-09-27, public docs landing)

Feedback: "the plumbing is solid, the surface is a mystery". An agent with a
publisher token found the releases API but no human-readable docs;
`/docs` was 404, because it fell through to the `/{name}` launcher.

* `GET https://auth.repo.box/docs`: a public, server-rendered page (no token,
  no registry data) linking the publisher quickstart (`/docs#publish`),
  capabilities JSON, OpenAPI, `skill.md`, the MCP endpoint and the
  well-known document. It shows the exact deploy endpoint, the multipart
  parts in order (`manifest`, then `image`), the `docker save` recipe, the
  minimal manifest, and the release status, logs, rollback and restart URLs.
  It also states the contract: apps are private, `/data` survives updates and
  rollbacks, `runtime.env` is plain non-secret config, and v1 has no secrets
  management and no DB backup/export API. `/docs/` answers 303 to `/docs`,
  and every auth page has a "Docs" nav item. `docs` was already a reserved
  app name.
* Discovery (`/api/platform/v1`, well-known) has a top-level `docs`, plus
  `documentation.docs`, `documentation.publisher_quickstart` and
  `publishing.docs`. OpenAPI has `externalDocs` and a `/docs` path. The upload
  contract (`how_to_deploy`, malformed-upload answers) carries `docs`.
* Every service-token and publisher-token 401, and the API 404, names
  `https://auth.repo.box/docs`.
* Stale publisher MCP `initialize` instructions ("a Git repository (repo.box
  clones, builds and runs it)") now describe the direct `docker save` upload.
* Gates: fmt, clippy `-D warnings`, 93 tests (new
  `docs_landing_is_public_and_linked_from_discovery_and_401s`),
  `publisher-e2e.sh` ALL PASS (57 checks, 4 new `/docs` checks through real
  Caddy), `edge-e2e.sh` ALL PASS. The deploy sweep now checks `/docs`.
* Deployed e02f458a, then 9b748268 (long URLs wrap: page width 390 at a
  390 px viewport, was 463) with `NO_CADDY=1`. No Caddy, route, runtime or
  token change. DB backups `platform-20260927T093558Z.db` and
  `-093804Z.db`. The live binary sha256 (`f26def18…`) equals the local build.
  The sweep had 89 checks; the only mismatch was the pre-existing
  `fieldwork-write` 404. That 404 comes from the origin on
  127.0.0.1:4110 (JSON body), not the edge.
* External HTTPS checks: `/docs` 200 `text/html`, all links present,
  `/docs/` 303. Discovery and well-known `docs` =
  `https://auth.repo.box/docs`. OpenAPI `externalDocs` is present, and
  `skill.md` has the docs row. Anonymous and invalid-token 401s name the docs
  URL. `/`, `/me`, `/demo-private` and `/admin/users` answer as before.

## Study Diary conversion (2026-09-17, reference for the identity policy)

Scope, per Fran's decision: Study Diary (`study-diary.repo.box`, proxy
`127.0.0.1:3025`, container `study-diary`, owner `ellie-beaumont`) drops its
app-specific password, owner setup link and internal login/session flow and
trusts only the identity the repo.box edge injects, using it to scope
records. No link was generated or sent; no email flow exists.

* **Code** (source of record on the host: `/srv/study-diary`, staged from
  `/home/fran/study-diary-stage`; there is no git repository for it on the
  host or on Hetzner). `server/index.mjs`: `platformIdentity(headers)`
  accepts exactly `X-RepoBox-Auth: session` + numeric `X-RepoBox-User-Id` +
  a valid handle in `X-RepoBox-User`, else `401 {"error":"Open Study Diary
  from auth.repo.box."}` (also for `X-RepoBox-Auth: public`, a handle without
  the marker, malformed ids, or the old `sd_session` cookie). Accounts are
  found or created by platform user id; handles follow renames. Removed:
  `/api/auth/{status,setup,setup/exchange,login,logout}` (now 404),
  `scripts/create-owner-setup.mjs`, scrypt/session/setup-token code, the
  `STUDY_DIARY_OWNER_EMAIL` requirement, the `sessions`, `setup_tokens` and
  `setup_credentials` tables. Same-origin checks on PUT/POST and the
  TimeEdit preview guards are unchanged. Frontend: the password/setup screens
  are replaced by an "open from auth.repo.box" page shown only when the
  identity is absent; Settings names the signed-in handle and links to
  `auth.repo.box/me` for devices/sign-out. `README.md` states the contract.
* **Data.** `openDatabase` migrates a password-era database in place
  (`migrateLegacyAuth`, foreign keys off during the table swap and
  `foreign_key_check` afterwards): account rows and `planner_data` kept,
  credential/session tables dropped, each old account becomes `legacy-<id>`
  with no platform identity, so its records are unreachable until
  `scripts/bind-legacy-owner.mjs --legacy-id N --platform-user-id N --handle H`
  binds it (one-way; refuses to merge if that identity already holds
  records here; replaces an empty interim account). Tests (8, `node --test`):
  absent/partial/forged/public identity refused on every route without
  creating an account, identity selects the record scope and nobody else's
  (incl. renamed handle and same-handle-different-id), old endpoints and
  tables gone, migration + bind, timetable guards.
* **Host change** (15:51–15:52 UTC). Backup
  `/home/fran/backups/study-diary/study-diary-pre-identity-20260917T155108Z.db`
  (`quick_check` ok, 1 user, 1 planner row, payload 2610 bytes) and the old
  unit at `study-diary.service.pre-identity`. Image `study-diary:identity-20260917T155017Z`
  built on the host from the staged source (frontend built on Hetzner with
  pnpm; lint clean). Unit: image tag updated, `STUDY_DIARY_OWNER_EMAIL`
  removed; restart at 15:51 UTC, container on `127.0.0.1:3025` only. After
  start: tables `planner_data`, `users` only; account 1 `legacy-1` with the
  2610-byte payload intact; bind → account 1 = platform user 10
  `ellie-beaumont` (bound_at 15:52:03 UTC). Ellie's existing platform app
  sessions stay valid; her old `sd_session` cookie is ignored.
* **Boundary evidence.** Loopback origin: no headers, `X-RepoBox-Auth:
  public`, handle-only, old cookie → all `401` with the gate message;
  `POST /api/auth/login` and `GET /api/auth/status` → 404; `/` → the static
  shell (its UI shows the gate page without identity). Through the edge:
  anonymous → 401 (gate page), spoofed `X-RepoBox-*` → 401. Isolation with
  throwaway platform user `sdprobe-155235` (id 14; granted, enrolled via
  curl, launched, redeemed 302): `/api/auth/me` returned that handle and id,
  `/api/planner` was empty (not Ellie's), the same request with forged
  Ellie headers stayed empty (stripped at the edge), its own PUT was stored
  under its own account, Ellie's payload length stayed 2610, after `app
  revoke` the edge answered 401. Cleanup: grant revoked, user disabled, link
  shredded, the probe's account and its orphaned planner row deleted (the
  sqlite CLI does not cascade; `foreign_key_check` clean), leaving account 1
  only. A process on the host can still speak to loopback with forged
  headers: that is the host trust boundary (same as the SQLite file), stated
  in the README.
* **Follow-ups (not done):** ellies-japanese, academicweapon, uni-kitchen and
  fieldwork keep their own login/setup flows and are `pending` in the
  manifest; each needs the same conversion and `app attest`. Study Diary has
  no git repository; consider importing `/srv/study-diary` into one.

## Current limitations

* Platform AI v1: non-streaming only; text only; no tools. A request counts
  against the daily quotas once it passes validation, even if the provider
  then fails. The quota is per UTC day, not a sliding window. Output is cut
  at `max_tokens × 8` characters because the platform does not rely on
  ChatMock honouring `max_tokens`. An app backend can call the endpoint only
  by relaying the signed-in user's cookie (no server-to-server credential in
  v1). Service tokens are not OAuth, and the machine API has no dedicated
  rate limit beyond Caddy. Since 2026-09-26 (evening) the control plane runs
  as its own user; the remaining host-level limit (fran is root-equivalent via
  docker/sudo) is in that record.

* External publisher v1 is Docker image upload only (no registry pulls or
  Git builds). There is no secrets management, so env values are plain
  settings in the manifest. Only linux/amd64 is accepted. Apps are always
  private; an operator grants people access. Containers share the host
  kernel: isolation is runc + seccomp/AppArmor defaults + the fixed run
  template, with no gVisor. The deploy worker is root with Docker authority
  and is therefore the trusted computing base; its inputs are only
  validated jobs, and it runs argv without a shell. The worker's iptables
  rules are re-asserted on every worker run and once at boot (the worker unit
  is enabled for `multi-user.target`). Uploads are at most 2 GiB and go through
  Caddy without a dedicated throttle.
* One control plane process, one SQLite file, no HA; fine for this scale.
* Sessions: 30 d device sessions, 24 h app sessions, no sliding renewal.
  App sessions created before schema 3 carry no device link, so revoking a
  device does not cascade to them; they still expire within 24 h and can be
  ended individually.
* A revoked session is refused on its next request; a page already rendered
  in the browser stays on screen until it reloads or fetches.
* Role changes (member ↔ admin) are CLI-only; the UI creates users, toggles
  enabled, issues device links.
* The directory lives on `auth.repo.box` (+ `/api/directory`); the main
  repo.box site does not consume it yet.
* Rate limiting relies on Caddy/fail2ban; tokens are 256-bit so brute force is
  not practical, but there is no dedicated throttle on `/enrol/*`/`/invite/*`.
* No app deletion from the UI (`app remove` via CLI, then re-render).
* Existing apps on repo.box are not migrated; only the three demo apps use the
  managed route model.
* Access counting is deliberately coarse: every allowed request (assets
  included) counts one, days are UTC, there is no per-path or per-user
  breakdown, unique users exist only for private apps and only inside the
  90-day window, and an app that flips from private to public keeps its
  earlier per-day user rows until they age out. Counters are best-effort: a
  failed increment is logged and never blocks serving.
* The identity contract is declarative beyond the edge: the route strips
  and injects mechanically, but whether an origin still renders its own
  login can only be established by review and attestation. Four pre-policy
  private apps are `pending` (see the policy section) and keep serving.
* Opens are a browser-side notion read from request headers: a client that
  sends `Accept: text/html` without `Sec-Fetch-*` (scripted HTML fetches,
  very old browsers) looks like a navigation, and a browser that navigates
  to an HTML path with an unlisted file extension counts. Opens are
  best-effort like the counters; a person is one identity per platform
  user, so two people sharing a device look like one. Opens exist only
  inside the 90-day window (no all-time total). Since only signed-in people
  are recorded, a public app's opens describe launches from auth.repo.box,
  not its audience.
