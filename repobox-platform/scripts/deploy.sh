#!/usr/bin/env bash
#
# Deploy the repobox-platform control plane + demo apps to the repo.box host.
#
# Runs from the Hetzner build box (repo.box is publish/serve-only, it never
# builds). Steps:
#   1. test + build a static musl binary
#   2. ship binary, demo pages, systemd units and the Caddy block to the host
#   3. install under /srv/repobox-platform, data under /var/lib/repobox-platform
#   4. back up the DB (if any), (re)start services, health-check on loopback
#   5. register the three demo apps if missing, render the managed routes,
#      apply the Caddy block (backup -> validate -> reload, auto-restore)
#   6. sweep the live HTTPS endpoints
#
# Never prints tokens. The first admin's device link is written by
# `bootstrap-admin` to a 0600 file on the host (see docs).
#
#   ./scripts/deploy.sh                 full deploy
#   SKIP_TESTS=1 ./scripts/deploy.sh    skip cargo test
#   NO_CADDY=1 ./scripts/deploy.sh      everything except the Caddy change (sweep still runs)
#   RETIRE_HOSTS="a.repo.box b.repo.box" ./scripts/deploy.sh
#                                       also remove those hosts' standalone legacy
#                                       Caddy blocks, in the same validated apply as
#                                       the rendered routes that replace them
#   CADDY_DRY_RUN=1 ./scripts/deploy.sh validate the Caddy change, install nothing
#
# STATIC_ROOTS lists the directories a registered static app may live under
# (space separated); the rendered routes are refused otherwise.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
CRATE_DIR="$PWD"
cd ..
REPO="$PWD"

HOST="${DEPLOY_HOST:-fran@204.168.190.248}"
TARGET=x86_64-unknown-linux-musl
BIN="$REPO/target/$TARGET/release/repobox-platform"
STAGE=/home/fran/repobox-platform-stage
ADMIN_NAME="${ADMIN_NAME:-fran}"
STATIC_ROOTS="${STATIC_ROOTS:-/srv/repobox-platform/apps /var/www/repo.box/subdomains}"
RETIRE_HOSTS="${RETIRE_HOSTS:-}"

log() { printf '\n=== %s ===\n' "$*"; }
remote() { ssh -o BatchMode=yes "$HOST" "$@"; }

log "gates: fmt/clippy/test/build"
cargo fmt -p repobox-platform -- --check
cargo clippy -p repobox-platform --all-targets -- -D warnings
if [[ -z "${SKIP_TESTS:-}" ]]; then
  cargo test -p repobox-platform
fi
cargo build --release -p repobox-platform --target "$TARGET"
ldd "$BIN" 2>&1 | command grep -qiE "statically linked|not a dynamic executable" || { echo "FATAL: binary is not static" >&2; exit 1; }
"$BIN" --version

log "ship to $HOST:$STAGE"
remote "mkdir -p '$STAGE'"
rsync -az --delete \
  "$BIN" \
  "$CRATE_DIR/deploy/" \
  "$HOST:$STAGE/"
rsync -az --delete "$CRATE_DIR/demo/" "$HOST:$STAGE/demo/"

log "install"
remote "set -e
  sudo -n install -d -o root -g root -m 0755 /srv/repobox-platform /srv/repobox-platform/bin /srv/repobox-platform/apps
  sudo -n install -d -o fran -g fran -m 0700 /var/lib/repobox-platform
  sudo -n install -d -o fran -g fran -m 0700 /home/fran/backups/repobox-platform
  sudo -n install -d -o root -g root -m 0755 /etc/caddy/repobox-platform
  for app in demo-unlisted demo-listed; do
    sudo -n install -d -o root -g root -m 0755 /srv/repobox-platform/apps/\$app
    sudo -n install -o root -g root -m 0644 '$STAGE/demo/'\$app/index.html /srv/repobox-platform/apps/\$app/index.html
  done
  if [ -f /var/lib/repobox-platform/platform.db ]; then
    '$STAGE/repobox-platform' backup --out /home/fran/backups/repobox-platform/platform-\$(date -u +%Y%m%dT%H%M%SZ).db
  fi
  sudo -n install -o root -g root -m 0755 '$STAGE/repobox-platform' /srv/repobox-platform/bin/repobox-platform
  sudo -n install -o root -g root -m 0644 '$STAGE/repobox-platform.service' /etc/systemd/system/repobox-platform.service
  sudo -n install -o root -g root -m 0644 '$STAGE/repobox-platform-demo-private.service' /etc/systemd/system/repobox-platform-demo-private.service
  sudo -n install -o root -g root -m 0755 '$STAGE/caddy-apply.py' /srv/repobox-platform/caddy-apply.py
  sudo -n systemctl daemon-reload
  sudo -n systemctl enable --now repobox-platform.service repobox-platform-demo-private.service
  sudo -n systemctl restart repobox-platform.service repobox-platform-demo-private.service
  sleep 1.5
  systemctl is-active repobox-platform.service repobox-platform-demo-private.service
  curl -sf http://127.0.0.1:3230/healthz
  curl -sf -o /dev/null -w 'demo origin: %{http_code}\n' http://127.0.0.1:3231/
"

log "registry: admin + demo apps"
remote "set -e
  P=/srv/repobox-platform/bin/repobox-platform
  if ! \$P user list | command grep -q '^$ADMIN_NAME '; then
    \$P bootstrap-admin --name '$ADMIN_NAME' --display-name 'Fran' --out /home/fran/secrets/repobox-platform-$ADMIN_NAME-\$(date -u +%Y%m%dT%H%M%SZ).url
  fi
  reg() { \$P app show \"\$1\" >/dev/null 2>&1 || \$P app register \"\$@\"; }
  reg demo-private  --title 'Private demo'  --description 'Private by default: needs a grant, shows the edge-injected identity.' --owner '$ADMIN_NAME' --kind proxy  --target 127.0.0.1:3231 --visibility private
  reg demo-unlisted --title 'Unlisted demo' --description 'Public, but absent from the directory.' --owner '$ADMIN_NAME' --kind static --target /srv/repobox-platform/apps/demo-unlisted --visibility public_unlisted
  reg demo-listed   --title 'Listed demo'   --description 'Public and listed in the directory.'     --owner '$ADMIN_NAME' --kind static --target /srv/repobox-platform/apps/demo-listed   --visibility public_listed
  \$P app list
"

if [[ -n "${NO_CADDY:-}" ]]; then
  log "NO_CADDY set: skipping route render + Caddy apply (routes unchanged)"
else
  log "caddy: render routes, apply managed block + routes${RETIRE_HOSTS:+, retire: $RETIRE_HOSTS}"
  roots_args=""; for r in $STATIC_ROOTS; do roots_args="$roots_args --apps-root '$r'"; done
  retire_args=""; for h in $RETIRE_HOSTS; do retire_args="$retire_args --retire '$h'"; done
  remote "set -e
    P=/srv/repobox-platform/bin/repobox-platform
    \$P routes render --check-roots $roots_args --out '$STAGE/apps.caddy'
    sudo -n python3 /srv/repobox-platform/caddy-apply.py apply '$STAGE/auth.repo.box.caddy' \
      --apps '$STAGE/apps.caddy' $retire_args ${CADDY_DRY_RUN:+--dry-run}
  "
  if [[ -n "${CADDY_DRY_RUN:-}" ]]; then log "CADDY_DRY_RUN set: stopping before the live sweep"; exit 0; fi
fi

log "live sweep"
fail=0
check() { # url expected-code  (retries while the TLS cert is still being issued)
  local code i
  for i in $(seq 1 24); do
    code=$(curl -s -o /dev/null -w '%{http_code}' "$1" || true)
    [[ "$code" != "000" && -n "$code" ]] && break
    sleep 5
  done
  printf '  %-45s %s (want %s)\n' "$1" "$code" "$2"
  [[ "$code" == "$2" ]] || fail=1
}
check https://auth.repo.box/healthz 200
check https://auth.repo.box/ 200
check https://auth.repo.box/gate/verify 404
# Every registered app, expectation derived from the registry: anonymous gets
# 401 on private, 200 on public, 404 on disabled; a spoofed identity header
# never changes a private answer.
apps_json=$(remote "/srv/repobox-platform/bin/repobox-platform app list --json")
while read -r name vis enabled; do
  if [[ "$enabled" != "true" ]]; then want=404
  elif [[ "$vis" == "private" ]]; then want=401
  else want=200; fi
  check "https://$name.repo.box/" "$want"
  if [[ "$want" == "401" ]]; then
    code=$(curl -s -o /dev/null -w '%{http_code}' -H 'X-RepoBox-User: fran' -H 'X-RepoBox-Role: admin' -H 'X-RepoBox-Auth: session' "https://$name.repo.box/" || true)
    printf '  %-45s %s (want 401, spoofed identity)\n' "https://$name.repo.box/" "$code"
    [[ "$code" == "401" ]] || fail=1
  fi
done < <(printf '%s' "$apps_json" | python3 -c 'import json,sys; [print(a["name"], a["visibility"], str(a["enabled"]).lower()) for a in json.load(sys.stdin)]')
dir=$(curl -s https://auth.repo.box/api/directory)
echo "  directory: $dir"
while read -r name vis enabled; do
  n=$(printf '%s' "$dir" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(sum(1 for a in d.get("apps", d if isinstance(d, list) else []) if a.get("name")==sys.argv[1]))' "$name")
  if [[ "$vis" == "public_listed" && "$enabled" == "true" ]]; then want=1; else want=0; fi
  printf '  %-45s listed %s time(s) (want %s)\n' "$name" "$n" "$want"
  [[ "$n" == "$want" ]] || fail=1
done < <(printf '%s' "$apps_json" | python3 -c 'import json,sys; [print(a["name"], a["visibility"], str(a["enabled"]).lower()) for a in json.load(sys.stdin)]')
anon=$(curl -s https://auth.repo.box/)
echo "$anon" | command grep -q 'id="public-apps"' || { echo "  anonymous directory is not the public directory" >&2; fail=1; }
echo "$anon" | command grep -qE 'demo-unlisted|demo-private' && { echo "  anonymous directory leaks a non-listed app" >&2; fail=1; }
[[ "$fail" -eq 0 ]] || { echo "SWEEP FAILED (Caddy backup path printed above; see docs for rollback)" >&2; exit 1; }
log "deploy verified"
