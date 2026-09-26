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
#   - the same-origin AI endpoint (/_repo_box/ai/v1/*): reserved before the
#     origin, gate + session required, forged headers stripped, policy and
#     limits enforced, real broker with the bridge secret in front of a fake
#     ChatMock, no prompt/secret in any log
#
# Usage: scripts/edge-e2e.sh   (needs `caddy` + python3; uses ports 3930-3933/8443/8080)

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
expect "no log has the AI prompt" "$(command grep -c "$CANARY" "$W/cp.log" "$W/caddy.log" "$W/broker.log" "$W/origin.log" "$W/chatmock.log" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has the bridge secret" "$(command grep -c "$SECRET" "$W/cp.log" "$W/caddy.log" "$W/broker.log" "$W/origin.log" | awk -F: '{s+=$2} END {print s}')" 0
expect "AI requests were logged (metadata only)" "$([[ $(command grep -c 'chat app=demo-private' "$W/cp.log") -gt 0 ]] && echo yes)" yes

if [[ $fail -eq 0 ]]; then echo "EDGE E2E: ALL PASS (work dir $W)"; else echo "EDGE E2E: FAILURES (work dir $W)"; exit 1; fi
