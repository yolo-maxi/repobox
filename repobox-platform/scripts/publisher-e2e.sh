#!/usr/bin/env bash
#
# Local end-to-end proof of the external publisher path with a REAL Caddy and
# a REAL Docker daemon: an agent-held publisher token uploads a `docker save`
# archive over HTTPS, the deploy worker imports it under the platform's own
# name, runs it on loopback, health-gates it and applies the published route
# through caddy-apply.py (test mode); a signed-in user launches it through
# auth.repo.box and the app page calls the same-origin AI endpoint. Then:
# update, failed update, rollback, restart, logs, isolation, revocation,
# removal, and no secrets in any log.
#
# Usage: scripts/publisher-e2e.sh  (needs docker, caddy, python3, the
# node:22-alpine image or network to pull it; ports 3940-3943, 8444, 8081,
# 2999, loopback 4700-4800)

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build -p repobox-platform 2>/dev/null
P="${E2E_BIN:-$PWD/../target/debug/repobox-platform}"
W="${E2E_DIR:-$(mktemp -d)}"
export XDG_DATA_HOME="$W/caddy-data" XDG_CONFIG_HOME="$W/caddy-config"
DB="$W/platform.db"
GATE=127.0.0.1:3940
BROKER=127.0.0.1:3942
FAKE_CHATMOCK=127.0.0.1:3943
HTTPS=8444
ADMIN=localhost:2999
APP=e2e-probe-$RANDOM
SPOOL="$W/spool"
mkdir -p "$W/caddy/repobox-platform" "$W/published" "$SPOOL"/{jobs,uploads,queries,results,work} "$W/subdomains/legacy-static"
echo '# PUBLISHED by `repobox-platform publisher-worker`. Do not edit by hand.' > "$W/caddy/repobox-platform/published.caddy"

fail=0
expect() { # label got want
  if [[ "$2" == "$3" ]]; then printf '  ok   %-62s %s\n' "$1" "$2"; else printf '  FAIL %-62s got %s want %s\n' "$1" "$2" "$3"; fail=1; fi
}

echo "== registry"
"$P" --db "$DB" user create fran --display-name Fran --admin >/dev/null
"$P" --db "$DB" user create bob --display-name Bob >/dev/null
"$P" --db "$DB" app register fran-app --title "Fran's app" --owner fran --kind static --target "$W/subdomains/legacy-static" --visibility public_unlisted >/dev/null
"$P" --db "$DB" publisher create --handle e2e-muse --display-name "E2E Muse" >/dev/null
"$P" --db "$DB" publisher create --handle e2e-other >/dev/null
"$P" --db "$DB" publisher token create --publisher e2e-muse --name e2e-muse-1 --ttl-days 1 --out "$W/muse.token" >/dev/null
"$P" --db "$DB" publisher token create --publisher e2e-other --name e2e-other-1 --ttl-days 1 --out "$W/other.token" >/dev/null
TOK=$(head -1 "$W/muse.token"); OTHER=$(head -1 "$W/other.token")
expect "token file is 0600" "$(stat -c %a "$W/muse.token")" 600
"$P" --db "$DB" routes render --gate "$GATE" --apps-root "$W/subdomains" --out "$W/caddy/repobox-platform/apps.caddy" 2>/dev/null

cat > "$W/caddy/Caddyfile" <<CADDY
{
	admin $ADMIN
	local_certs
	https_port $HTTPS
	http_port 8081
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
legacy-caddy-site.repo.box {
	respond "legacy"
}
import $W/caddy/repobox-platform/apps.caddy
import $W/caddy/repobox-platform/published.caddy
CADDY
caddy validate --config "$W/caddy/Caddyfile" --adapter caddyfile >/dev/null 2>&1 && echo "caddy validate: OK"

echo "== services"
( umask 077; head -c 48 /dev/urandom | base64 | tr -d '/+=\n' > "$W/bridge.secret" )
cat > "$W/fake_chatmock.py" <<'PY'
import json, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def send(self, obj):
        b = json.dumps(obj).encode()
        self.send_response(200); self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
    def do_GET(self):
        self.send({"object": "list", "data": [{"id": m} for m in ("gpt-5.6-terra", "gpt-5.6-luna")]})
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        self.send({"id": "r", "object": "chat.completion", "created": 1, "model": body["model"],
                   "choices": [{"index": 0, "finish_reason": "stop", "message": {"role": "assistant", "content": "pong-from-fake-model"}}],
                   "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}})
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
python3 "$W/fake_chatmock.py" "${FAKE_CHATMOCK#*:}" >"$W/chatmock.log" 2>&1 &
PIDS=($!)
"$P" ai-broker --bind "$BROKER" --upstream "http://$FAKE_CHATMOCK" --secret-file "$W/bridge.secret" >"$W/broker.log" 2>&1 &
PIDS+=($!)
"$P" --db "$DB" serve --bind "$GATE" --public-base https://auth.repo.box --domain repo.box \
  --ai-upstream "http://$BROKER" --ai-secret-file "$W/bridge.secret" \
  --publisher-spool "$SPOOL" --publisher-host-file "$W/caddy/Caddyfile" --publisher-host-file "$W/caddy/repobox-platform/apps.caddy" \
  --publisher-reserved-dir "$W/subdomains" --publisher-min-free-mib 512 >"$W/cp.log" 2>&1 &
PIDS+=($!)
caddy run --config "$W/caddy/Caddyfile" --adapter caddyfile >"$W/caddy.log" 2>&1 &
PIDS+=($!)
# The .path units, emulated: run the workers whenever their queue is non-empty.
WARGS=(--spool "$SPOOL" --root "$W/published" --routes-file "$W/caddy/repobox-platform/published.caddy"
  --apply-cmd python3 "$PWD/deploy/caddy-apply.py" --service-user "$(id -un)" --gate "$GATE"
  --network "" --no-firewall --ports 4700-4800 --health-timeout-secs 60)
( export CADDY_APPLY_TEST_DIR="$W/caddy" CADDY_APPLY_TEST_ADMIN="$ADMIN"
  while true; do
    if [ -n "$(ls -A "$SPOOL/jobs")" ]; then "$P" publisher-worker "${WARGS[@]}" >>"$W/worker.log" 2>&1 || true; fi
    if [ -n "$(ls -A "$SPOOL/queries")" ]; then "$P" publisher-query "${WARGS[@]}" >>"$W/worker.log" 2>&1 || true; fi
    sleep 0.5
  done ) &
PIDS+=($!)
cleanup() {
  kill "${PIDS[@]}" 2>/dev/null || true; wait 2>/dev/null || true
  for c in $(docker ps -aq --filter "label=repobox.app=$APP"); do docker rm -f "$c" >/dev/null; done
  for i in $(docker image ls -q "repobox-pub/$APP" | sort -u); do docker image rm -f "$i" >/dev/null 2>&1 || true; done
  docker volume rm "rbpub-$APP-data" >/dev/null 2>&1 || true
}
trap cleanup EXIT
for i in $(seq 1 40); do curl -sf "http://$GATE/healthz" >/dev/null && break; sleep 0.25; done
sleep 1

R=(-k --resolve "auth.repo.box:$HTTPS:127.0.0.1" --resolve "$APP.repo.box:$HTTPS:127.0.0.1")
A="https://auth.repo.box:$HTTPS"
APPURL="https://$APP.repo.box:$HTTPS"
api() { curl -s "${R[@]}" -H "Authorization: Bearer $TOK" "$@"; }
jq_() { python3 -c "
import json,sys
raw=sys.stdin.read()
try:
    d=json.loads(raw); print(eval(sys.argv[1]))
except Exception as e:
    print('JQ-ERROR', type(e).__name__, raw[:300].replace(chr(10),' '))" "$1"; }

echo "== build the image an agent would build, and save it"
docker build -q -t e2e-publisher-probe:latest demo/publisher-probe >/dev/null
BEFORE=$(docker image inspect -f '{{.Id}}' e2e-publisher-probe:latest)
docker save e2e-publisher-probe:latest | gzip > "$W/probe.tar.gz"
echo "  archive: $(stat -c %s "$W/probe.tar.gz") bytes"
MANIFEST="{\"name\":\"$APP\",\"title\":\"E2E probe\",\"version\":\"1.0.0\",\"runtime\":{\"health_path\":\"/healthz\",\"memory_mb\":128,\"env\":{\"APP_VERSION\":\"v1\"}},\"provenance\":{\"repository\":\"https://example.com/probe\",\"commit\":\"abc1234\"}}"

echo "== public docs through the edge"
expect "anonymous /docs -> 200" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$A/docs")" 200
expect "/docs has the publisher quickstart" "$(curl -s "${R[@]}" "$A/docs" | command grep -c 'id="publish"')" 1
expect "discovery names the docs" "$(curl -s "${R[@]}" "$A/api/platform/v1" | jq_ 'd["docs"]')" "https://auth.repo.box/docs"
expect "401 points at the docs" "$(curl -s "${R[@]}" "$A/api/platform/v1/publisher/whoami" | jq_ '"https://auth.repo.box/docs" in d["error"]["message"]')" True

echo "== unauthorized requests"
expect "anonymous upload -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -F "manifest=$MANIFEST;type=application/json" -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases")" 401
expect "no upload was stored" "$(ls -A "$SPOOL/uploads" | wc -l)" 0
expect "service-token-shaped bearer -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'Authorization: Bearer rbp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' "$A/api/platform/v1/publisher/apps")" 401
expect "not multipart -> 415" "$(api -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' -d "$MANIFEST" "$A/api/platform/v1/publisher/releases")" 415
expect "legacy Caddy host name -> 409" "$(api -o /dev/null -w '%{http_code}' -F 'manifest={"name":"legacy-caddy-site","title":"x"};type=application/json' -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases")" 409
expect "operator app name -> 409" "$(api -o /dev/null -w '%{http_code}' -F 'manifest={"name":"fran-app","title":"x"};type=application/json' -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases")" 409

echo "== deploy v1 over HTTPS"
OUT=$(api -F "manifest=$MANIFEST;type=application/json" -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases?wait=240")
R1=$(echo "$OUT" | jq_ 'd["release"]["id"]')
expect "v1 release is live" "$(echo "$OUT" | jq_ 'd["release"]["status"]')" live
expect "launcher URL" "$(echo "$OUT" | jq_ 'd["app"]["launcher_url"]')" "https://auth.repo.box/$APP"
expect "direct host marked edge-gated" "$(echo "$OUT" | jq_ 'd["app"]["direct_url_access"][:10]')" edge-gated
expect "artifact digest recorded" "$(echo "$OUT" | jq_ 'd["release"]["artifact"]["sha256"]')" "sha256:$(sha256sum "$W/probe.tar.gz" | cut -d' ' -f1)"
expect "archive format" "$(echo "$OUT" | jq_ 'd["release"]["artifact"]["format"]')" docker-save+gzip
expect "private + platform identity + AI on" "$(echo "$OUT" | jq_ 'd["app"]["visibility"]+"/"+d["app"]["identity"]+"/"+str(d["app"]["ai"]["policy"]["enabled"])')" "private/platform/True"
expect "archive tag never applied on the host (same image id)" "$(docker image inspect -f '{{.Id}}' e2e-publisher-probe:latest)" "$BEFORE"
expect "imported under the platform name only" "$(docker image ls --format '{{.Repository}}:{{.Tag}}' | command grep -c "^repobox-pub/$APP:$R1\$")" 1
expect "published route has the gated shape" "$(command grep -c "forward_auth $GATE" "$W/caddy/repobox-platform/published.caddy")" 1
PORT=$(command grep -oE 'reverse_proxy 127\.0\.0\.1:[0-9]+$' "$W/caddy/repobox-platform/published.caddy" | head -1 | cut -d: -f2)
expect "container published on loopback only" "$(docker ps --filter "label=repobox.app=$APP" --format '{{.Ports}}' | command grep -c "^127.0.0.1:$PORT->8080/tcp$")" 1
expect "deploy log tells the story" "$(api "$A/api/platform/v1/publisher/releases/$R1/log" | command grep -c -E '^== (import|image|start|healthy|apply route)')" 5

echo "== the app through the edge"
expect "anonymous direct host -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$APPURL/")" 401
expect "spoofed identity headers -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'X-RepoBox-User: fran' -H 'X-RepoBox-Auth: session' "$APPURL/whoami.json")" 401
"$P" --db "$DB" app grant "$APP" --user bob >/dev/null
"$P" --db "$DB" user enrol bob --out "$W/bob.url" >/dev/null
LINK=$(head -1 "$W/bob.url"); JAR="$W/jar"; AJAR="$W/appjar"
curl -s "${R[@]}" -c "$JAR" -o /dev/null -X POST -H 'Origin: https://auth.repo.box' "$A${LINK#https://auth.repo.box}"
LOC=$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{redirect_url}' "$A/$APP")
expect "launcher redirects with a one-time code" "$(echo "$LOC" | command grep -c "^https://$APP.repo.box/?rb_launch=")" 1
expect "code redeemed -> clean redirect" "$(curl -s "${R[@]}" -c "$AJAR" -o /dev/null -w '%{http_code}' "$APPURL/?rb_launch=${LOC#*rb_launch=}")" 302
expect "app page renders v1" "$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/" | command grep -o 'id="version">v1' )" 'id="version">v1'
WHO=$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/whoami.json")
expect "origin sees the gate identity (bob)" "$(echo "$WHO" | jq_ 'd["identity"]["x-repobox-user"]')" bob
expect "origin runs the release" "$(echo "$WHO" | jq_ 'd["release"]')" "$R1"
expect "app runs as a non-root user" "$(echo "$WHO" | jq_ 'd["uid"]!=0')" True
expect "/data is writable" "$(echo "$WHO" | jq_ 'd["boots"]')" 1
AI=$(curl -s "${R[@]}" -b "$AJAR" -X POST -H 'Content-Type: application/json' -H "Origin: $APPURL" -d '{"messages":[{"role":"user","content":"ping"}],"max_tokens":20}' "$APPURL/_repo_box/ai/v1/chat/completions")
expect "same-origin AI call from the app works" "$(echo "$AI" | jq_ 'd["choices"][0]["message"]["content"]')" pong-from-fake-model
expect "AI without the app session -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST -H 'Content-Type: application/json' -d '{"messages":[{"role":"user","content":"x"}]}' "$APPURL/_repo_box/ai/v1/chat/completions")" 401

echo "== update to v2 (same name, new archive)"
M2=${MANIFEST//\"v1\"/\"v2\"}; M2=${M2//1.0.0/2.0.0}
OUT=$(api -F "manifest=$M2;type=application/json" -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases?wait=240")
R2=$(echo "$OUT" | jq_ 'd["release"]["id"]')
expect "v2 live (version 2)" "$(echo "$OUT" | jq_ 'd["release"]["status"]+"/"+str(d["release"]["version"])')" live/2
expect "app page now v2 (same session)" "$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/" | command grep -o 'id="version">v2')" 'id="version">v2'
expect "/data survived the update" "$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/whoami.json" | jq_ 'd["boots"]')" 2
expect "one container runs" "$(docker ps -q --filter "label=repobox.app=$APP" | wc -l)" 1
expect "v1 superseded, retained" "$(api "$A/api/platform/v1/publisher/releases/$R1" | jq_ 'd["release"]["status"]+"/"+str(d["release"]["retained_for_rollback"])')" superseded/True

echo "== failed update keeps v2 serving"
M3=${M2//\/healthz/\/does-not-exist}
OUT=$(api -F "manifest=$M3;type=application/json" -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases?wait=240")
expect "unhealthy release fails" "$(echo "$OUT" | jq_ 'd["release"]["status"]+"/"+d["release"]["failure"]["code"]')" failed/unhealthy
expect "v2 still serves" "$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/" | command grep -o 'id="version">v2')" 'id="version">v2'

echo "== rollback, restart, logs"
OUT=$(api -X POST "$A/api/platform/v1/publisher/apps/$APP/rollback?wait=240")
expect "rollback to v1 is live" "$(echo "$OUT" | jq_ 'd["release"]["status"]+"/"+d["release"]["rollback_of"]')" "live/$R1"
expect "app page back to v1" "$(curl -s "${R[@]}" -b "$AJAR" "$APPURL/" | command grep -o 'id="version">v1')" 'id="version">v1'
OUT=$(api -X POST "$A/api/platform/v1/publisher/apps/$APP/restart?wait=120")
expect "restart done" "$(echo "$OUT" | jq_ 'd["release"]["status"]')" done
LOGS=$(api "$A/api/platform/v1/publisher/apps/$APP/logs?tail=50")
expect "runtime logs: container running" "$(echo "$LOGS" | jq_ 'd["container"]["state"]')" running
expect "runtime logs carry app output" "$([[ $(echo "$LOGS" | jq_ 'd["logs"].count("listening on 8080")') -ge 1 ]] && echo yes)" yes

echo "== isolation"
expect "other publisher cannot see the app" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $OTHER" "$A/api/platform/v1/publisher/apps/$APP")" 404
expect "other publisher cannot take the name" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $OTHER" -F "manifest=$MANIFEST;type=application/json" -F "image=@$W/probe.tar.gz" "$A/api/platform/v1/publisher/releases")" 409
expect "publisher lists only its own app" "$(api "$A/api/platform/v1/publisher/apps" | jq_ '",".join(a["name"] for a in d["apps"])')" "$APP"
expect "operator app unreachable" "$(api -o /dev/null -w '%{http_code}' -X POST "$A/api/platform/v1/publisher/apps/fran-app/restart")" 404
expect "publisher token opens nothing on the service API" "$(api -o /dev/null -w '%{http_code}' "$A/api/platform/v1/apps")" 401

echo "== revoke, remove"
"$P" --db "$DB" publisher token revoke e2e-muse-1 >/dev/null
expect "revoked token -> 401" "$(api -o /dev/null -w '%{http_code}' "$A/api/platform/v1/publisher/whoami")" 401
"$P" --db "$DB" publisher remove-app "$APP" --purge-data --spool "$SPOOL" --yes >/dev/null
for i in $(seq 1 60); do [ -z "$(docker ps -aq --filter "label=repobox.app=$APP")" ] && ! command grep -q "$APP" "$W/caddy/repobox-platform/published.caddy" && break; sleep 0.5; done
expect "containers removed" "$(docker ps -aq --filter "label=repobox.app=$APP" | wc -l)" 0
expect "images removed" "$(docker image ls -q "repobox-pub/$APP" | wc -l)" 0
expect "route removed" "$(command grep -c "$APP" "$W/caddy/repobox-platform/published.caddy")" 0
expect "gate: removed app -> 404" "$(curl -s -o /dev/null -w '%{http_code}' -H 'X-RepoBox-Gate: 1' -H "X-RepoBox-Gate-App: $APP" "http://$GATE/gate/verify")" 404

echo "== no secrets in logs"
ALL=("$W/cp.log" "$W/caddy.log" "$W/worker.log" "$W/broker.log")
expect "no log has the publisher token" "$(command grep -c -- "${TOK#rbpub_}" "${ALL[@]}" | awk -F: '{s+=$2} END {print s}')" 0
expect "no log has a launch code" "$(command grep -c -- 'rb_launch=' "${ALL[@]}" | awk -F: '{s+=$2} END {print s}')" 0
docker image rm e2e-publisher-probe:latest >/dev/null 2>&1 || true

if [[ $fail -eq 0 ]]; then echo "PUBLISHER E2E: ALL PASS (work dir $W)"; else echo "PUBLISHER E2E: FAILURES (work dir $W)"; exit 1; fi
