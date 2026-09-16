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
  `users`, `apps`, `grants`, `tokens`, `sessions`, `audit`, `access_daily`,
  `access_daily_users`, `access_totals`, `meta` (schema version 3). Only
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
  every allowed request including assets, not page views. The page shows the
  all-time total, the 90-day total, unique users (private apps) and the last
  14 days, and states these rules. Owners see only their own apps.
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
cargo test -p repobox-platform                       # 38 unit + integration tests
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
$P app stats myapp --days 14                              # access counters, no identity
```

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
   403), own-session listing and revocation (device revoke cascades to its
   app sessions at the gate, app-session revoke leaves the device signed in,
   logout cascades), member cannot revoke another user's session even with
   its id, admin per-user page and per-row revoke, admin "sign out
   everywhere", admin-only and CSRF guards on all of it.
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
   curl -s https://auth.repo.box/ | grep -c 'id="public-apps"'      # anonymous: public directory only
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

## Current limitations

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
