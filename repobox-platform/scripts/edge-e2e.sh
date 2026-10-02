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
#   - onboarding link: one URL takes a brand-new device from nothing to a
#     signed-in, granted app session on a clean app URL; strangers, replays
#     and disabled apps are refused; the token never reaches a log; and, when
#     Playwright's Chromium is installed, the same chain in a real browser
#     (through a CONNECT proxy to this Caddy, so Origin/cookies are real),
#     starting from the owner's manage page: "Onboard new user"
#   - the same-origin AI endpoint (/_repo_box/ai/v1/*): reserved before the
#     origin, gate + session required, forged headers stripped, policy and
#     limits enforced, real broker with the bridge secret in front of a fake
#     ChatMock, no prompt/secret in any log
#
# Usage: scripts/edge-e2e.sh   (needs `caddy` + python3; uses ports 3930-3933/8443/8080)

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build -p repobox-platform 2>/dev/null
P="${E2E_BIN:-$PWD/../target/debug/repobox-platform}"
W="${E2E_DIR:-$(mktemp -d)}"
export XDG_DATA_HOME="$W/caddy-data" XDG_CONFIG_HOME="$W/caddy-config"
DB="$W/platform.db"
APPS="$W/apps"
GATE=127.0.0.1:3930
ORIGIN=127.0.0.1:3931
BROKER=127.0.0.1:3932
FAKE_CHATMOCK=127.0.0.1:3933
HTTPS=8443
mkdir -p "$APPS/demo-unlisted" "$APPS/demo-listed"
cp demo/demo-unlisted/index.html "$APPS/demo-unlisted/"
cp demo/demo-listed/index.html "$APPS/demo-listed/"

echo "== registry"
expect_fail() { if "$@" >/dev/null 2>&1; then echo "  FAIL expected refusal: $*"; exit 1; else echo "  ok   refused: private app without --identity platform"; fi; }
expect_fail "$P" --db "$DB" app register undeclared --title "Undeclared" --owner fran --kind proxy --target "$ORIGIN" --visibility private
"$P" --db "$DB" user create fran --display-name Fran --admin
"$P" --db "$DB" user create bob --display-name Bob
"$P" --db "$DB" app register demo-private  --title "Private demo"  --owner fran --kind proxy  --target "$ORIGIN" --visibility private --identity platform
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
	header {
		?Referrer-Policy "strict-origin-when-cross-origin"
		-Server
	}
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
( umask 077; head -c 48 /dev/urandom | base64 | tr -d '/+=\n' > "$W/bridge.secret" )
SECRET=$(cat "$W/bridge.secret")
cat > "$W/fake_chatmock.py" <<'PY'
import json, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
LOG = sys.argv[2]
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def send(self, obj):
        b = json.dumps(obj).encode()
        self.send_response(200); self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
    def do_GET(self):
        self.send({"object": "list", "data": [{"id": m} for m in ("gpt-5.6-terra", "gpt-5.6-luna", "gpt-5.5")]})
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        with open(LOG, "a") as f:  # header *names* and body keys only
            f.write(json.dumps({"headers": sorted(k.lower() for k in self.headers.keys()), "keys": sorted(body)}) + "\n")
        self.send({"id": "resp_e2e", "object": "chat.completion", "created": 1, "model": body["model"],
                   "choices": [{"index": 0, "finish_reason": "stop", "message": {"role": "assistant", "content": "fake-model-says-hi"}}],
                   "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}})
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
python3 "$W/fake_chatmock.py" "${FAKE_CHATMOCK#*:}" "$W/chatmock-requests.log" >"$W/chatmock.log" 2>&1 &
FC=$!
"$P" ai-broker --bind "$BROKER" --upstream "http://$FAKE_CHATMOCK" --secret-file "$W/bridge.secret" >"$W/broker.log" 2>&1 &
BK=$!
"$P" --db "$DB" serve --bind "$GATE" --public-base https://auth.repo.box --domain repo.box \
  --ai-upstream "http://$BROKER" --ai-secret-file "$W/bridge.secret" >"$W/cp.log" 2>&1 &
CP=$!
"$P" demo-origin --bind "$ORIGIN" --app demo-private >"$W/origin.log" 2>&1 &
OP=$!
caddy run --config "$W/Caddyfile" --adapter caddyfile >"$W/caddy.log" 2>&1 &
CD=$!
cleanup() { kill $CP $OP $CD $BK $FC 2>/dev/null || true; wait 2>/dev/null || true; }
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
expect "code replay rejected (body-less redirect, no code in target)" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' "$PRIV/?rb_launch=$TOKEN")" "303 0 https://auth.repo.box/demo-private?launch_error=used&next=/"
expect "launcher explains the rejected code" "$(curl -s "${R[@]}" -w ' %{http_code}' "$A/demo-private?launch_error=used&next=/" | command grep -o 'already been used\| 403' | tr -d '\n')" "already been used 403"
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

echo "== platform AI endpoint (same origin, gate + session, broker)"
AIURL="$PRIV/_repo_box/ai/v1/chat/completions"
CANARY="prompt-canary-$RANDOM$RANDOM"
AIBODY="{\"messages\":[{\"role\":\"user\",\"content\":\"$CANARY\"}],\"max_tokens\":50}"
ai() { curl -s "${R[@]}" -X POST -H 'Content-Type: application/json' "$@"; }
expect "registration default: private platform app has AI on" "$("$P" --db "$DB" app show demo-private | command grep -c '^ai:          on')" 1
expect "registration default: public app has AI off" "$("$P" --db "$DB" app show demo-listed | command grep -c '^ai:          off')" 1
expect "AI anonymous -> 401" "$(ai -o /dev/null -w '%{http_code}' -d "$AIBODY" "$AIURL")" 401
expect "AI anonymous answer is JSON" "$(ai -d "$AIBODY" "$AIURL" | command grep -c '"code":"unauthenticated"')" 1
expect "AI anonymous + forged identity -> 401" "$(ai -o /dev/null -w '%{http_code}' -H 'X-RepoBox-Auth: session' -H 'X-RepoBox-User-Id: 1' -H 'X-RepoBox-User: fran' -H 'X-RepoBox-Gate: 1' -H 'X-RepoBox-Gate-App: demo-private' -d "$AIBODY" "$AIURL")" 401
out=$(ai -b "$AJAR" -H "Origin: $PRIV" -d "$AIBODY" "$AIURL")
expect "AI with bob's app session -> completion" "$(echo "$out" | command grep -c 'fake-model-says-hi')" 1
expect "AI response is a chat.completion" "$(echo "$out" | command grep -c '"object":"chat.completion"')" 1
BOB_ID=$("$P" --db "$DB" user sessions bob | sed -nE '1s/.*\(id ([0-9]+),.*/\1/p')
out=$(ai -b "$AJAR" -H 'X-RepoBox-User-Id: 1' -H 'X-RepoBox-User: fran' -H 'X-RepoBox-Role: admin' -d "$AIBODY" "$AIURL" -o /dev/null -w '%{http_code}')
expect "AI with session + forged identity: served as bob" "$out/$(tail -1 "$W/broker.log" | command grep -c "user=$BOB_ID ")" "200/1"
expect "AI cross-origin -> 403" "$(ai -b "$AJAR" -H 'Origin: https://evil.repo.box' -o /dev/null -w '%{http_code}' -d "$AIBODY" "$AIURL")" 403
expect "AI text/plain -> 415" "$(curl -s "${R[@]}" -b "$AJAR" -X POST -H 'Content-Type: text/plain' -o /dev/null -w '%{http_code}' -d "$AIBODY" "$AIURL")" 415
expect "AI stream:true -> 400" "$(ai -b "$AJAR" -o /dev/null -w '%{http_code}' -d '{"stream":true,"messages":[{"role":"user","content":"x"}]}' "$AIURL")" 400
expect "AI unsupported model -> 400" "$(ai -b "$AJAR" -o /dev/null -w '%{http_code}' -d '{"model":"gpt-4o","messages":[{"role":"user","content":"x"}]}' "$AIURL")" 400
expect "AI oversized input -> 413" "$(python3 -c 'import json;print(json.dumps({"messages":[{"role":"user","content":"x"*40000}]}))' | ai -b "$AJAR" -o /dev/null -w '%{http_code}' --data-binary @- "$AIURL")" 413
expect "AI models list" "$(curl -s "${R[@]}" -b "$AJAR" "$PRIV/_repo_box/ai/v1/models" | command grep -o '"id"' | wc -l | tr -d ' ')" 2
expect "other reserved path -> 404 (never the origin)" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code}' "$PRIV/_repo_box/whoami.json")" 404
expect "unknown AI path -> 404" "$(ai -b "$AJAR" -o /dev/null -w '%{http_code}' -d '{}' "$PRIV/_repo_box/ai/v1/embeddings")" 404
expect "AI path not served on auth.repo.box (/gate/*)" "$(ai -o /dev/null -w '%{http_code}' -H 'X-RepoBox-Gate: 1' -H 'X-RepoBox-Gate-App: demo-private' -d "$AIBODY" "$A/gate/ai/v1/chat/completions")" 404
"$P" --db "$DB" app ai disable demo-private >/dev/null
expect "AI disabled by policy -> 403" "$(ai -b "$AJAR" -o /dev/null -w '%{http_code}' -d "$AIBODY" "$AIURL")" 403
"$P" --db "$DB" app ai enable demo-private >/dev/null
expect "public app: enabling AI without a policy is refused" "$("$P" --db "$DB" app ai enable demo-listed >/dev/null 2>&1; echo $?)" 1
expect "direct broker without secret -> 401" "$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'Content-Type: application/json' -d "$AIBODY" "http://$BROKER/v1/chat/completions")" 401
expect "direct control-plane AI hop with forged headers, no cookie -> 401" "$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'Content-Type: application/json' -H 'X-RepoBox-Gate: 1' -H 'X-RepoBox-Gate-App: demo-private' -H 'X-RepoBox-Auth: session' -H "X-RepoBox-User-Id: $BOB_ID" -d "$AIBODY" "http://$GATE/gate/ai/v1/chat/completions")" 401
expect "ChatMock saw no credential, cookie or identity header" "$(command grep -cE 'authorization|cookie|x-repobox' "$W/chatmock-requests.log")" 0
expect "ChatMock saw only rebuilt fields" "$(sort -u "$W/chatmock-requests.log" | python3 -c 'import sys,json; print(sorted({k for l in sys.stdin for k in json.loads(l)["keys"]}))')" "['max_tokens', 'messages', 'model', 'stream']"

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

echo "== identity contract in the manifest"
expect "rendered routes: the private app carries the contract" "$(command grep -c '^# identity: platform' "$W/apps.caddy")" 1
expect "rendered routes: undeclared public apps are marked pending" "$(command grep -c '^# identity: pending (public app' "$W/apps.caddy")" 2
expect "app show states the contract" "$("$P" --db "$DB" app show demo-private | command grep -c '^identity:    platform')" 1
"$P" --db "$DB" app register legacy-pub --title "Legacy public" --owner fran --kind static --target "$APPS/demo-listed" --visibility public_listed >/dev/null
expect "pending public app cannot be made private (CLI)" "$("$P" --db "$DB" app visibility legacy-pub private >/dev/null 2>&1; echo $?)" 1
expect "attest, then private is allowed" "$("$P" --db "$DB" app attest legacy-pub --note e2e >/dev/null && "$P" --db "$DB" app visibility legacy-pub private >/dev/null && echo ok)" ok
"$P" --db "$DB" app remove legacy-pub >/dev/null

echo "== onboarding link: one private URL from a new device into the app"
"$P" --db "$DB" app invite demo-private --new carol --display-name Carol --out "$W/carol.url" >/dev/null
expect "signup link does not create the account yet" "$("$P" --db "$DB" user list | command grep -c '^carol ')" 0
expect "onboarding link file is 0600" "$(stat -c %a "$W/carol.url")" 600
OLINK=$(head -1 "$W/carol.url")
OPATH=${OLINK#https://auth.repo.box}
ORAW=${OPATH#/invite/}
CJ="$W/carol.jar"
expect "open link -> body-less 303 to the clean /invite" "$(curl -s "${R[@]}" -c "$CJ" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' "$A$OPATH")" "303 0 $A/invite"
expect "token moved into a host-only cookie" "$(command grep -c '__Host-rb_invite' "$CJ")" 1
page=$(curl -s "${R[@]}" -b "$CJ" -D "$W/invite.h" "$A/invite")
expect "page greets the recipient" "$(echo "$page" | command grep -c 'Welcome, <strong>Carol</strong>')" 1
expect "page carries no token" "$(echo "$page" | command grep -c -- "$ORAW")" 0
expect "page: same-origin referrer policy survives the edge" "$(command grep -ci '^referrer-policy: same-origin' "$W/invite.h")" 1
expect "other pages keep the edge default policy" "$(curl -s "${R[@]}" -D - -o /dev/null "$A/" | command grep -ci '^referrer-policy: strict-origin-when-cross-origin')" 1
BOBC=$(awk '/__Host-rb_auth/ {print $7}' "$JAR")
expect "stranger signed in (bob) -> refused" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H "Cookie: __Host-rb_invite=$ORAW; __Host-rb_auth=$BOBC" "$A/invite")" 403
expect "stranger POST -> nothing consumed" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{redirect_url}' -X POST -H 'Origin: https://auth.repo.box' -H "Cookie: __Host-rb_invite=$ORAW; __Host-rb_auth=$BOBC" "$A/invite")" "303 $A/invite?err=sign_out_first"
expect "cross-site POST -> nothing consumed" "$(curl -s "${R[@]}" -b "$CJ" -o /dev/null -w '%{http_code} %{redirect_url}' -X POST -H 'Origin: https://evil.example' "$A/invite")" "303 $A/invite"
expect "link still active" "$("$P" --db "$DB" app invites demo-private | awk '$2=="new:carol" {print $3}')" active
expect "one click: account created, signed in, granted -> launcher" "$(curl -s "${R[@]}" -b "$CJ" -c "$CJ" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' -X POST -H 'Origin: https://auth.repo.box' "$A/invite")" "303 0 $A/demo-private"
expect "device cookie set, invite cookie gone" "$(command grep -c '__Host-rb_auth' "$CJ")/$(command grep -c '__Host-rb_invite' "$CJ")" "1/0"
OLOC=$(curl -s "${R[@]}" -b "$CJ" -o /dev/null -w '%{redirect_url}' "$A/demo-private")
case "$OLOC" in https://demo-private.repo.box/?rb_launch=*) printf '  ok   %-55s %s\n' "launcher mints the launch code" "(code elided)";; *) echo "  FAIL launcher: $OLOC"; fail=1;; esac
OCODE=${OLOC#*rb_launch=}
CAJ="$W/carol.appjar"
expect "gate: app session + clean app URL" "$(curl -s "${R[@]}" -c "$CAJ" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' "$PRIV/?rb_launch=$OCODE")" "302 0 $PRIV/"
who=$(curl -s "${R[@]}" -b "$CAJ" "$PRIV/whoami.json")
expect "origin sees X-RepoBox-User=carol" "$(echo "$who" | command grep -c '"x-repobox-user":"carol"')" 1
who=$(curl -s "${R[@]}" -b "$CAJ" -H 'X-RepoBox-User: fran' -H 'X-RepoBox-Role: admin' "$PRIV/whoami.json")
expect "spoofed headers with carol's session: still carol/member" "$(echo "$who" | command grep -c '"x-repobox-user":"carol".*"x-repobox-role":"member"\|"x-repobox-role":"member".*"x-repobox-user":"carol"')" 1
expect "carol holds exactly the one intended grant" "$("$P" --db "$DB" app show demo-private | command grep -c '^  - carol ')/$("$P" --db "$DB" app show demo-listed | command grep -c '^  - carol ')" "1/0"
expect "replay -> refused, body-less, no token in target" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' "$A$OPATH")" "303 0 $A/invite?e=used"
expect "replayed cookie -> refused" "$(curl -s "${R[@]}" -o /dev/null -w '%{redirect_url}' -X POST -H 'Origin: https://auth.repo.box' -H "Cookie: __Host-rb_invite=$ORAW" "$A/invite")" "$A/invite?e=used"
expect "used link records carol" "$("$P" --db "$DB" app invites demo-private | awk '$2=="new:carol" {print $3, $NF}')" "used carol"
"$P" --db "$DB" app invite demo-private --open --out "$W/open.url" >/dev/null
O2=$(head -1 "$W/open.url"); O2=${O2#https://auth.repo.box}
"$P" --db "$DB" app disable demo-private >/dev/null
expect "app disabled -> link refused" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{redirect_url}' "$A$O2")" "303 $A/invite?e=app_off"
"$P" --db "$DB" app enable demo-private >/dev/null
O2ID=$("$P" --db "$DB" app invites demo-private | awk '$2=="anyone" && $3=="active" && !f {print $1; f=1}')
"$P" --db "$DB" app invite-revoke demo-private --id "$O2ID" >/dev/null
expect "revoked link refused" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{redirect_url}' "$A$O2")" "303 $A/invite?e=revoked"
expect "app host anonymous + spoofed identity still 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'X-RepoBox-User: carol' -H 'X-RepoBox-Auth: session' "$PRIV/")" 401

CHROME="${E2E_CHROME:-$HOME/.cache/ms-playwright/chromium-1148/chrome-linux/chrome}"
if [[ -x "$CHROME" ]] && NODE_PATH="${E2E_NODE_PATH:-$HOME/idea-products/nomad-calendar/node_modules}" node -e 'require("playwright")' 2>/dev/null; then
  echo "== onboarding link in a real browser (Chromium through Caddy)"
  "$P" --db "$DB" user enrol fran --out "$W/fran.url" >/dev/null
  # Chromium talks to the real https://*.repo.box URLs (port 443) through a
  # CONNECT proxy that tunnels every :443 to this Caddy, so Origin, cookies
  # and the cross-host launch hop behave exactly as in production.
  cat > "$W/connect_proxy.py" <<'PY'
import socket, sys, threading
listen, target = int(sys.argv[1]), int(sys.argv[2])
def pipe(a, b):
    try:
        while (d := a.recv(65536)):
            b.sendall(d)
    except OSError:
        pass
    finally:
        for s in (a, b):
            try: s.shutdown(socket.SHUT_RDWR)
            except OSError: pass
def handle(c):
    head = b""
    while b"\r\n\r\n" not in head:
        d = c.recv(4096)
        if not d: return c.close()
        head += d
    line = head.split(b"\r\n")[0].split()
    if len(line) < 2 or line[0] != b"CONNECT" or not line[1].endswith(b":443"):
        c.sendall(b"HTTP/1.1 403 Forbidden\r\n\r\n"); return c.close()
    u = socket.create_connection(("127.0.0.1", target))
    c.sendall(b"HTTP/1.1 200 Connection established\r\n\r\n")
    threading.Thread(target=pipe, args=(c, u), daemon=True).start()
    pipe(u, c)
srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
srv.bind(("127.0.0.1", listen)); srv.listen(64)
while True:
    conn, _ = srv.accept()
    threading.Thread(target=handle, args=(conn,), daemon=True).start()
PY
  python3 "$W/connect_proxy.py" 3934 "$HTTPS" >"$W/proxy.log" 2>&1 &
  PX=$!
  sleep 0.5
  cat > "$W/browser.js" <<'JS'
// The owner's exact flow: sign in, open the app's manage page, "Onboard new
// user", enter handle + display name, take the one link; then the new
// person opens it in a fresh browser and clicks once.
const { chromium } = require('playwright');
const [chrome, franFile] = process.argv.slice(2);
const franLink = require('fs').readFileSync(franFile, 'utf8').split('\n')[0];
(async () => {
  const b = await chromium.launch({ executablePath: chrome, proxy: { server: 'http://127.0.0.1:3934' } });
  const owner = await (await b.newContext({ ignoreHTTPSErrors: true })).newPage();
  await owner.goto(franLink);
  await owner.click('button[type=submit]');
  await owner.waitForURL(u => u.pathname === '/');
  await owner.goto('https://auth.repo.box/apps/demo-private');
  await owner.click('a:has-text("Onboard new user")');
  await owner.fill('#onboard-name', 'dave');
  await owner.fill('#onboard-dn', 'Dave');
  await owner.click('button:has-text("Create signup link")');
  const link = (await owner.textContent('.secret')).trim();
  const shown = link.startsWith('https://auth.repo.box/invite/');
  const raw = link.split('/').pop();
  const page = await (await b.newContext({ ignoreHTTPSErrors: true })).newPage();
  const seen = [];
  page.on('framenavigated', f => { if (f === page.mainFrame()) seen.push(f.url()); });
  await page.goto(link);
  const clean = page.url() === 'https://auth.repo.box/invite' && (await page.textContent('body')).includes('Welcome, Dave');
  await page.click('button:has-text("Create account and open Private demo")');
  await page.waitForURL(u => u.hostname === 'demo-private.repo.box', { timeout: 15000 }).catch(() => {});
  const landed = page.url() === 'https://demo-private.repo.box/';
  const who = landed && (await page.textContent('body')).includes('dave');
  const tokenFree = seen.slice(1).every(u => !u.includes(raw) && !u.includes('rb_launch'));
  console.log(`${+shown}${+clean}${+landed}${+who}${+tokenFree}`);
  await b.close();
})().catch(e => { console.log('error ' + String(e.message).split('\n')[0].replace(/[A-Za-z0-9_-]{43}/g, '<redacted>')); process.exit(1); });
JS
  expect "browser: owner page 'Onboard new user' -> link -> one click -> app as dave" "$(NODE_PATH="${E2E_NODE_PATH:-$HOME/idea-products/nomad-calendar/node_modules}" node "$W/browser.js" "$CHROME" "$W/fran.url")" 11111
  expect "browser: dave's account exists as a member" "$("$P" --db "$DB" user list | awk '$1=="dave" {print $2, $3}')" "member active"
  expect "browser: dave holds exactly the intended grant" "$("$P" --db "$DB" app show demo-private | command grep -c '^  - dave ')/$("$P" --db "$DB" app show demo-unlisted | command grep -c '^  - dave ')" "1/0"
  kill $PX 2>/dev/null || true
else
  echo "== (skipped: no Chromium/Playwright for the real-browser onboarding check)"
fi

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

echo "== abrupt clients on code-bearing requests (Caddy logs copy failures with the URI)"
cat > "$W/abrupt.py" <<'PY'
import socket, ssl, sys
host, port, path, n = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4])
ctx = ssl.create_default_context(); ctx.check_hostname = False; ctx.verify_mode = ssl.CERT_NONE
bodies = 0
for _ in range(n):
    s = ctx.wrap_socket(socket.create_connection(("127.0.0.1", port)), server_hostname=host)
    s.sendall(f"GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: text/html\r\n\r\n".encode())
    head = s.recv(4096)          # status line + headers, then hang up at once
    if b"content-length: 0" not in head.lower():
        bodies += 1
    s.close()
print(bodies)
PY
for p in "/?rb_launch=$TOKEN" "/deep/x?a=1&rb_launch=$TOKEN" "/?rb_launch=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"; do
  expect "code-bearing responses carry no body (40 abrupt clients)" "$(python3 "$W/abrupt.py" demo-private.repo.box "$HTTPS" "$p" 40)" 0
done
expect "onboarding link answers carry no body (20 abrupt clients)" "$(python3 "$W/abrupt.py" auth.repo.box "$HTTPS" "$OPATH" 20)" 0
expect "public app: stray code answer carries no body" "$(python3 "$W/abrupt.py" demo-listed.repo.box "$HTTPS" "/?rb_launch=$TOKEN" 20)" 0
sleep 0.5

echo "== no secrets in logs"
ALL_LOGS=("$W/cp.log" "$W/caddy.log" "$W/origin.log" "$W/broker.log" "$W/chatmock.log")
expect "no log has the launch token" "$(command grep -c -- "$TOKEN" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has any rb_launch= query" "$(command grep -c -- 'rb_launch=' "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has the onboarding token" "$(command grep -c -- "$ORAW" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has any /invite/ token path" "$(command grep -c -- '/invite/' "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has the enrol token" "$(command grep -c -- "${PATHPART#/enrol/}" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has an app session cookie value" "$(command grep -c -- "$(awk '/__Host-rb_app/ {print $7}' "$AJAR")" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has the AI prompt" "$(command grep -c -- "$CANARY" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has the bridge secret" "$(command grep -c -- "$SECRET" "${ALL_LOGS[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "AI requests were logged (metadata only)" "$([[ $(command grep -c 'chat app=demo-private' "$W/cp.log") -gt 0 ]] && echo yes)" yes

if [[ $fail -eq 0 ]]; then echo "EDGE E2E: ALL PASS (work dir $W)"; else echo "EDGE E2E: FAILURES (work dir $W)"; exit 1; fi
