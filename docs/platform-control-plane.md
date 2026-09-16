# repo.box platform control plane (`auth.repo.box`)

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
  `users`, `apps`, `grants`, `tokens`, `sessions`, `audit`, `meta`. Only
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
  (90 s) bound to user+app → `302 https://<app>.repo.box/?token=<code>` →
  gate consumes the code atomically → `302` to the clean URL with
  `Set-Cookie: __Host-rb_app=…; Path=/; Secure; HttpOnly; SameSite=Lax`
  (host-only by construction: `__Host-` cookies cannot carry `Domain`) →
  subsequent requests are authorised from that cookie for 24 h.
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
* **Identity headers** the origin may trust (only ever set by the gate):
  `X-RepoBox-App`, `X-RepoBox-Auth` (`session`|`public`), and for sessions
  `X-RepoBox-User`, `X-RepoBox-User-Id`, `X-RepoBox-Role`.
* **Safety boundary**: `serve`, `demo-origin` and every proxy target must be
  loopback; the renderer refuses anything else. Static roots must sit under
  `/srv/repobox-platform/apps`. App names are validated DNS labels with a
  reserved list (`auth`, `git`, `ens`, `api`, `www`, …). CSRF on all POSTs:
  `Origin` must match `https://auth.repo.box` (or `Sec-Fetch-Site: same-origin`).

### Route model

`repobox-platform routes render` is a pure function of the sorted registry: the
same registry always yields byte-identical `apps.caddy`. It is imported by one
line inside the managed block of the main Caddyfile. Registering an app and
rendering routes is **operator-only** (CLI on the host); the web UI has no
create/deploy/route controls.

## Local development

```bash
cargo test -p repobox-platform                       # 27 unit + integration tests
cargo clippy -p repobox-platform --all-targets -- -D warnings
cargo fmt -p repobox-platform -- --check
repobox-platform/scripts/edge-e2e.sh                 # real Caddy on loopback, 33 live checks
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
| `/srv/repobox-platform/bin/repobox-platform` | static musl binary (CLI + services) |
| `/srv/repobox-platform/apps/<name>/` | static app roots (root-owned, world-readable) |
| `/srv/repobox-platform/caddy-apply.py` | Caddyfile apply/rollback helper |
| `/var/lib/repobox-platform/platform.db` | registry (owner `fran`, 0700 dir) |
| `/etc/caddy/repobox-platform/apps.caddy` | rendered managed routes |
| `/etc/caddy/Caddyfile` | contains the `# BEGIN/END repobox-platform managed` block |
| `/etc/caddy/backups/Caddyfile.pre-repobox-platform-<stamp>` | pre-change backups |
| `/home/fran/backups/repobox-platform/platform-<stamp>.db` | DB backups taken by every deploy |
| `/home/fran/secrets/repobox-platform-<user>-<stamp>.url` | operator-created device links (0600) |

Services: `repobox-platform.service` (127.0.0.1:3230) and
`repobox-platform-demo-private.service` (127.0.0.1:3231). Ports are claimed in
`~/clawd/PORT-REGISTRY.md`.

```bash
repobox-platform/scripts/deploy.sh          # gates → build → ship → install → registry → Caddy → sweep
NO_CADDY=1 repobox-platform/scripts/deploy.sh   # everything except the Caddy change
```

The Caddy step (`caddy-apply.py apply`) backs up the Caddyfile, swaps either
the existing managed block or the exact legacy `auth.repo.box → 127.0.0.1:3005`
block, runs `caddy validate` on the candidate, and only then installs and
reloads; a failed reload restores the backup automatically. If neither block
matches exactly it aborts and changes nothing.

Legacy note: the old auth-proxy on port 3005 (a domain-wide `media_auth`
cookie) is not running; `auth.repo.box` returned 502 before this change.
`uniswap.repo.box` still forward-auths to `127.0.0.1:3005/auth` and is not
modified; `/set-token*` and `/login*` on `auth.repo.box` are kept pointing at
3005 so that contract is untouched.

### Operator tasks

```bash
P=/srv/repobox-platform/bin/repobox-platform            # runs as fran, uses /var/lib/repobox-platform/platform.db
$P bootstrap-admin --name fran --out /home/fran/secrets/repobox-platform-fran-$(date -u +%s).url
$P user create ocean --display-name Ocean               # then: $P user enrol ocean --out <0600 file>
$P app register myapp --title "My app" --owner fran --kind proxy --target 127.0.0.1:3299
$P app grant myapp --user ocean
$P routes render --check-roots --out /tmp/apps.caddy \
  && sudo install -m 0644 /tmp/apps.caddy /etc/caddy/repobox-platform/apps.caddy \
  && sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile \
  && sudo systemctl reload caddy
$P audit --limit 50
```

## Backup and restore

* **Backup** (online, consistent, safe while serving):
  `$P backup --out /home/fran/backups/repobox-platform/platform-$(date -u +%Y%m%dT%H%M%SZ).db`
  (refuses to overwrite; file is 0600). `deploy.sh` does this before every
  install.
* **Restore**: `sudo systemctl stop repobox-platform && cp <backup> /var/lib/repobox-platform/platform.db && rm -f /var/lib/repobox-platform/platform.db-wal /var/lib/repobox-platform/platform.db-shm && sudo systemctl start repobox-platform`,
  then `routes render` + Caddy reload if the app set differs.

## Test procedure

1. `cargo test -p repobox-platform` — covers: code redemption → host-only cookie
   + clean redirect (other query params preserved), replay rejection, expiry,
   app binding of codes, grant revocation killing live sessions, disabled
   app/user denial, the three visibility modes at the gate and in the
   directory, spoofed `X-RepoBox-*` headers never influencing a decision,
   app-session binding to one app, launch requiring sign-in and access,
   open-redirect hardening of `next`, single-use POST-only device links,
   invitation flow (new and existing users), owner/admin-only management, CSRF.
2. `repobox-platform/scripts/edge-e2e.sh` — same contract through a real Caddy
   with the generated routes (proves the strip + forward_auth + copy_headers
   mechanics, host-only/HttpOnly cookies, no secrets in logs).
3. Live (after deploy):
   ```bash
   curl -sI https://demo-private.repo.box/ | head -1                # 401
   curl -sI -H 'X-RepoBox-User: x' https://demo-private.repo.box/   # 401 (spoof stripped)
   curl -sI https://demo-unlisted.repo.box/ | head -1               # 200
   curl -sI https://demo-listed.repo.box/ | head -1                 # 200
   curl -s https://auth.repo.box/api/directory                       # demo-listed only
   # launch: sign a device in (device link), then
   curl -s -b jar -o /dev/null -w '%{redirect_url}' https://auth.repo.box/demo-private   # ...?token=
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

## Current limitations

* One control plane process, one SQLite file, no HA; fine for this scale.
* Sessions: 30 d device sessions, 24 h app sessions, no sliding renewal; no
  per-app session list in the UI (revoking a grant or disabling the user cuts
  them anyway).
* Role changes (member ↔ admin) are CLI-only; the UI creates users, toggles
  enabled, issues device links.
* The directory lives on `auth.repo.box` (+ `/api/directory`); the main
  repo.box site does not consume it yet.
* Rate limiting relies on Caddy/fail2ban; tokens are 256-bit so brute force is
  not practical, but there is no dedicated throttle on `/enrol/*`/`/invite/*`.
* No app deletion from the UI (`app remove` via CLI, then re-render).
* Existing apps on repo.box are not migrated; only the three demo apps use the
  managed route model.
