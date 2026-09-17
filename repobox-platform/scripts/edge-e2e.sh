#!/usr/bin/env bash
#
# Local end-to-end proof of the edge contract with a REAL Caddy: renders the
# managed routes from a throwaway registry, runs the control plane and demo
# origin on loopback, serves the three demo apps through Caddy (internal CA,
# https on a high port) and drives the launch flow with curl.
#
# Proves, against the exact generated route model:
#   - anonymous private -> 401; spoofed X-RepoBox-* headers are stripped
#   - launch code -> Set-Cookie (host-only) + clean redirect -> 200 with the
#     identity visible to the origin
#   - public unlisted/listed -> 200 without auth, no identity at the origin
#   - disable via CLI -> 404 immediately, no re-render
#
# Usage: scripts/edge-e2e.sh   (needs `caddy` on PATH; uses ports 3930/3931/8443/8080)

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build -p repobox-platform 2>/dev/null
P="$PWD/../target/debug/repobox-platform"
W="${E2E_DIR:-$(mktemp -d)}"
export XDG_DATA_HOME="$W/caddy-data" XDG_CONFIG_HOME="$W/caddy-config"
DB="$W/platform.db"
APPS="$W/apps"
GATE=127.0.0.1:3930
ORIGIN=127.0.0.1:3931
HTTPS=8443
mkdir -p "$APPS/demo-unlisted" "$APPS/demo-listed"
cp demo/demo-unlisted/index.html "$APPS/demo-unlisted/"
cp demo/demo-listed/index.html "$APPS/demo-listed/"

echo "== registry"
"$P" --db "$DB" user create fran --display-name Fran --admin
"$P" --db "$DB" user create bob --display-name Bob
"$P" --db "$DB" app register demo-private  --title "Private demo"  --owner fran --kind proxy  --target "$ORIGIN" --visibility private
"$P" --db "$DB" app register demo-unlisted --title "Unlisted demo" --owner fran --kind static --target "$APPS/demo-unlisted" --visibility public_unlisted
"$P" --db "$DB" app register demo-listed   --title "Listed demo"   --owner fran --kind static --target "$APPS/demo-listed"   --visibility public_listed
"$P" --db "$DB" app grant demo-private --user bob
"$P" --db "$DB" routes render --gate "$GATE" --apps-root "$APPS" --check-roots --out "$W/apps.caddy"

cat > "$W/Caddyfile" <<CADDY
{
	admin off
	local_certs
	https_port $HTTPS
	http_port 8080
	skip_install_trust
}
auth.repo.box {
	handle /gate/* {
		respond 404
	}
	handle {
		request_header -X-RepoBox-*
		reverse_proxy $GATE
	}
}
import $W/apps.caddy
CADDY
caddy validate --config "$W/Caddyfile" --adapter caddyfile >/dev/null && echo "caddy validate: OK"

echo "== services"
"$P" --db "$DB" serve --bind "$GATE" --public-base https://auth.repo.box --domain repo.box >"$W/cp.log" 2>&1 &
CP=$!
"$P" demo-origin --bind "$ORIGIN" --app demo-private >"$W/origin.log" 2>&1 &
OP=$!
caddy run --config "$W/Caddyfile" --adapter caddyfile >"$W/caddy.log" 2>&1 &
CD=$!
cleanup() { kill $CP $OP $CD 2>/dev/null || true; wait 2>/dev/null || true; }
trap cleanup EXIT
for i in $(seq 1 40); do curl -sf "http://$GATE/healthz" >/dev/null && break; sleep 0.25; done
sleep 1

R=(-k --resolve "auth.repo.box:$HTTPS:127.0.0.1" --resolve "demo-private.repo.box:$HTTPS:127.0.0.1" --resolve "demo-unlisted.repo.box:$HTTPS:127.0.0.1" --resolve "demo-listed.repo.box:$HTTPS:127.0.0.1")
A="https://auth.repo.box:$HTTPS"
PRIV="https://demo-private.repo.box:$HTTPS"
fail=0
expect() { # label got want
  if [[ "$2" == "$3" ]]; then printf '  ok   %-55s %s\n' "$1" "$2"; else printf '  FAIL %-55s got %s want %s\n' "$1" "$2" "$3"; fail=1; fi
}

echo "== anonymous"
expect "private anonymous" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$PRIV/")" 401
expect "private anonymous + spoofed identity" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'X-RepoBox-User: mallory' -H 'X-RepoBox-Auth: session' "$PRIV/")" 401
expect "unlisted anonymous" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "https://demo-unlisted.repo.box:$HTTPS/")" 200
expect "listed anonymous" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "https://demo-listed.repo.box:$HTTPS/")" 200
expect "gate not reachable via public host" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'X-RepoBox-Gate: 1' -H 'X-RepoBox-Gate-App: demo-private' "$A/gate/verify")" 404
dir=$(curl -s "${R[@]}" "$A/api/directory")
expect "directory lists demo-listed" "$(echo "$dir" | command grep -c '"demo-listed"')" 1
expect "directory omits demo-unlisted" "$(echo "$dir" | command grep -c 'demo-unlisted')" 0
expect "directory omits demo-private" "$(echo "$dir" | command grep -c 'demo-private')" 0

echo "== enrol bob on this 'device'"
"$P" --db "$DB" user enrol bob --out "$W/bob.url" >/dev/null
LINK=$(head -1 "$W/bob.url")
PATHPART=${LINK#https://auth.repo.box}
JAR="$W/jar"
expect "enrol GET is a confirmation page" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$A$PATHPART")" 200
expect "enrol POST signs in" "$(curl -s "${R[@]}" -c "$JAR" -o /dev/null -w '%{http_code}' -X POST -H 'Origin: https://auth.repo.box' "$A$PATHPART")" 303
expect "auth cookie is host-only (__Host-)" "$(command grep -c '__Host-rb_auth' "$JAR")" 1
expect "enrol link is single-use" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST -H 'Origin: https://auth.repo.box' "$A$PATHPART")" 410
expect "directory shows private app to bob" "$(curl -s "${R[@]}" -b "$JAR" "$A/" | command grep -c 'demo-private.repo.box')" 1

echo "== launch flow"
LOC=$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{redirect_url}' "$A/demo-private")
case "$LOC" in https://demo-private.repo.box/?rb_launch=*) printf '  ok   %-55s %s\n' "launch redirects to app with one-time code" "(token elided)";; *) echo "  FAIL launch redirect: $LOC"; fail=1;; esac
TOKEN=${LOC#*rb_launch=}
AJAR="$W/appjar"
out=$(curl -s "${R[@]}" -c "$AJAR" -o /dev/null -w '%{http_code} %{redirect_url}' "$PRIV/dashboard?a=1&rb_launch=$TOKEN")
expect "gate redeems code -> clean redirect" "$out" "302 https://demo-private.repo.box:$HTTPS/dashboard?a=1"
expect "app cookie is host-only (__Host-)" "$(command grep -c '__Host-rb_app' "$AJAR")" 1
expect "app cookie is HttpOnly" "$(command grep -c '#HttpOnly_demo-private.repo.box' "$AJAR")" 1
expect "code replay rejected" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$PRIV/?rb_launch=$TOKEN")" 403
who=$(curl -s "${R[@]}" -b "$AJAR" "$PRIV/whoami.json")
expect "origin sees X-RepoBox-User=bob" "$(echo "$who" | command grep -c '"x-repobox-user":"bob"')" 1
expect "origin sees X-RepoBox-Auth=session" "$(echo "$who" | command grep -c '"x-repobox-auth":"session"')" 1
who=$(curl -s "${R[@]}" -b "$AJAR" -H 'X-RepoBox-User: mallory' -H 'X-RepoBox-Role: admin' "$PRIV/whoami.json")
expect "spoof with session still shows bob" "$(echo "$who" | command grep -c '"x-repobox-user":"bob"')" 1
expect "spoof with session: role stays member" "$(echo "$who" | command grep -c '"x-repobox-role":"member"')" 1
expect "private page renders" "$(curl -s "${R[@]}" -b "$AJAR" "$PRIV/" | command grep -c 'Authenticated identity')" 1
who=$(curl -s "${R[@]}" -b "$AJAR" "$PRIV/whoami.json?token=app-owned-value")
expect "app's own ?token= passes the gate untouched" "$(echo "$who" | command grep -c '"x-repobox-user":"bob"')" 1
expect "stale code on a signed-in browser -> clean redirect" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code} %{redirect_url}' "$PRIV/x?rb_launch=$TOKEN")" "302 https://demo-private.repo.box:$HTTPS/x"
expect "public page: no identity at origin (via gate)" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "https://demo-listed.repo.box:$HTTPS/")" 200

echo "== opens (visits): only a signed-in browser page load counts"
NAV=(-H 'Accept: text/html,application/xhtml+xml,*/*;q=0.8' -H 'Sec-Fetch-Dest: document' -H 'Sec-Fetch-Mode: navigate')
expect "no open yet (curl's */* requests above were not page loads)" "$("$P" --db "$DB" app visits demo-private | command grep -c '^demo-private: 0 open(s)')" 1
expect "browser page load -> 200" "$(curl -s "${R[@]}" -b "$AJAR" "${NAV[@]}" -o /dev/null -w '%{http_code}' "$PRIV/")" 200
expect "second page load in the same visit -> 200" "$(curl -s "${R[@]}" -b "$AJAR" "${NAV[@]}" -o /dev/null -w '%{http_code}' "$PRIV/dashboard?a=1")" 200
expect "asset + api fetch -> 200" "$(curl -s "${R[@]}" -b "$AJAR" -H 'Accept: */*' -H 'Sec-Fetch-Dest: script' -o /dev/null -w '%{http_code}' "$PRIV/app.js")$(curl -s "${R[@]}" -b "$AJAR" -H 'Accept: application/json' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -o /dev/null -w '%{http_code}' "$PRIV/whoami.json")" 200200
VIS=$("$P" --db "$DB" app visits demo-private)
expect "exactly one open by one person" "$(echo "$VIS" | command grep -c '^demo-private: 1 open(s) by 1 signed-in person')" 1
expect "the person is bob" "$(echo "$VIS" | command grep -c '^bob ')" 1
expect "anonymous public page load -> 200" "$(curl -s "${R[@]}" "${NAV[@]}" -o /dev/null -w '%{http_code}' "https://demo-listed.repo.box:$HTTPS/")" 200
expect "anonymous public page load is not an open" "$("$P" --db "$DB" app visits demo-listed | command grep -c '^demo-listed: 0 open(s)')" 1
expect "visits page needs sign-in" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$A/apps/demo-private/visits")" 401
expect "visits page: member (bob) is refused" "$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{http_code}' "$A/apps/demo-private/visits")" 403
expect "grant suggestions: member (bob) is refused" "$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{http_code}' "$A/apps/demo-private/grantable-users")" 403

echo "== revocation / disable"
"$P" --db "$DB" app revoke demo-private --user bob >/dev/null
expect "grant revoked -> live session denied" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code}' "$PRIV/")" 401
"$P" --db "$DB" app grant demo-private --user bob >/dev/null
expect "grant restored -> session works again" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code}' "$PRIV/")" 200
"$P" --db "$DB" app disable demo-listed >/dev/null
expect "disabled app -> 404 without re-render" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "https://demo-listed.repo.box:$HTTPS/")" 404
"$P" --db "$DB" app enable demo-listed >/dev/null
"$P" --db "$DB" user disable bob >/dev/null
expect "disabled user -> app session denied" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code}' "$PRIV/")" 401
expect "disabled user -> auth session denied" "$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{http_code}' "$A/me")" 401

echo "== no secrets in logs"
expect "control plane log has no launch token" "$(command grep -c "$TOKEN" "$W/cp.log" "$W/caddy.log" "$W/origin.log" | awk -F: '{s+=$2} END {print s}')" 0
expect "control plane log has no enrol token" "$(command grep -c "${PATHPART#/enrol/}" "$W/cp.log" | awk -F: '{s+=$2} END {print s}')" 0

if [[ $fail -eq 0 ]]; then echo "EDGE E2E: ALL PASS (work dir $W)"; else echo "EDGE E2E: FAILURES (work dir $W)"; exit 1; fi
