#!/usr/bin/env bash
#
# Local end-to-end proof of `primary-host-v1` with a REAL Caddy (internal CA,
# https on a high port) and the real control plane on loopback:
#   - anonymous GET / on the primary host -> the static landing (200), from the
#     landing root, with the build's CSP, private no-cache and Vary: Cookie; the
#     origin never sees the request
#   - /_landing/* -> landing files, cookies stripped, no gate, immutable assets
#   - any other anonymous path -> the gate's 401; spoofed identity stays 401
#   - a stale/foreign cookie on / -> landing, never the app
#   - live-page-shaped paths stay closed on the primary host (lane is off) and
#     stay open on <name>.repo.box
#   - launcher -> https://<primary>/?rb_launch=… -> host-only cookie -> the
#     origin sees the gate identity on /
#   - a signed-in user without a grant -> landing on /, 401 elsewhere
#   - www.<primary> -> 301 to the bare host
#   - no log has a launch code
#
# Usage: LANDING_DIR=<dist-landing> LANDING_HEADERS=<dist-landing.headers.json> scripts/primary-host-e2e.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build -p repobox-platform 2>/dev/null
P="${E2E_BIN:-$PWD/../target/debug/repobox-platform}"
W="${E2E_DIR:-$(mktemp -d)}"
export XDG_DATA_HOME="$W/caddy-data" XDG_CONFIG_HOME="$W/caddy-config"
DB="$W/platform.db"
GATE=127.0.0.1:3940
ORIGIN=127.0.0.1:3941
HTTPS=8444
HOST=kodu.work
: "${LANDING_DIR:?set LANDING_DIR}" "${LANDING_HEADERS:?set LANDING_HEADERS}"

echo "== registry"
"$P" --db "$DB" user create fran --display-name Fran --admin >/dev/null
"$P" --db "$DB" user create stranger --display-name Stranger >/dev/null
"$P" --db "$DB" app register fieldwork --title "kodu.work" --owner fran --kind proxy --target "$ORIGIN" --visibility private --identity platform >/dev/null
"$P" --db "$DB" app live-pages fieldwork --enable --review-note "e2e: lane stays on the platform host only" >/dev/null
if "$P" --db "$DB" app primary-host fieldwork --host "$HOST" --landing >/dev/null 2>&1; then echo "  FAIL primary host without review note accepted"; exit 1; fi
"$P" --db "$DB" app primary-host fieldwork --host "$HOST" --landing --review-note "e2e: DNS by operator, boundary checked by this script"
"$P" --db "$DB" routes render --gate "$GATE" --out "$W/apps.caddy" 2>/dev/null
# The rendered route reads /srv/<host>/…; point those paths at this run's copy.
mkdir -p "$W/srv/$HOST"
cp -r "$LANDING_DIR" "$W/srv/$HOST/current"
head -c 4096 /dev/urandom > "$W/srv/$HOST/current/hackathon.mp4"
python3 deploy/landing-headers.py "$LANDING_HEADERS" "$W/srv/$HOST/headers.caddy" >/dev/null
sed -i "s#/srv/$HOST/#$W/srv/$HOST/#g" "$W/apps.caddy"

cat > "$W/Caddyfile" <<CADDY
{
	admin off
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
import $W/apps.caddy
CADDY
caddy validate --config "$W/Caddyfile" --adapter caddyfile >/dev/null 2>&1 && echo "  caddy validate: OK"

# A tiny origin that records every request it sees and echoes the identity.
cat > "$W/origin.py" <<'PY'
import json, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
LOG = sys.argv[2]
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def handle_one(self):
        ident = {k: v for k, v in self.headers.items() if k.lower().startswith("x-repobox-")}
        with open(LOG, "a") as f:
            f.write(json.dumps({"path": self.path.split("?")[0], "auth": ident.get("X-RepoBox-Auth"), "cookie": "Cookie" in self.headers}) + "\n")
        b = ("ORIGIN " + json.dumps(ident, sort_keys=True)).encode()
        self.send_response(200); self.send_header("Content-Type", "text/plain"); self.send_header("Content-Length", str(len(b))); self.end_headers()
        if self.command != "HEAD": self.wfile.write(b)
    do_GET = do_POST = do_HEAD = handle_one
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
: > "$W/origin-requests.log"
python3 "$W/origin.py" "${ORIGIN#*:}" "$W/origin-requests.log" >"$W/origin.log" 2>&1 &
OP=$!
"$P" --db "$DB" serve --bind "$GATE" --public-base https://auth.repo.box --domain repo.box >"$W/cp.log" 2>&1 &
CP=$!
caddy run --config "$W/Caddyfile" --adapter caddyfile >"$W/caddy.log" 2>&1 &
CD=$!
cleanup() { kill $CP $OP $CD 2>/dev/null || true; wait 2>/dev/null || true; }
trap cleanup EXIT
for i in $(seq 1 40); do curl -sf "http://$GATE/healthz" >/dev/null && break; sleep 0.25; done
sleep 1

R=(-k --resolve "auth.repo.box:$HTTPS:127.0.0.1" --resolve "$HOST:$HTTPS:127.0.0.1" --resolve "www.$HOST:$HTTPS:127.0.0.1" --resolve "fieldwork.repo.box:$HTTPS:127.0.0.1")
A="https://auth.repo.box:$HTTPS"
K="https://$HOST:$HTTPS"
FW="https://fieldwork.repo.box:$HTTPS"
fail=0
expect() { if [[ "$2" == "$3" ]]; then printf '  ok   %-62s %s\n' "$1" "$2"; else printf '  FAIL %-62s got %s want %s\n' "$1" "$2" "$3"; fail=1; fi; }
hdr() { tr -d '\r' | awk -F': ' -v k="$1" 'tolower($1)==k {print $2}'; }
origin_seen() { wc -l < "$W/origin-requests.log" | tr -d ' '; }
title=$(command grep -o '<title>[^<]*</title>' "$LANDING_DIR/index.html" | head -1)
asset=$(cd "$LANDING_DIR" && find _landing -type f | sort | head -1)
immutable=$(cd "$LANDING_DIR" && find _landing/assets -type f 2>/dev/null | sort | head -1)

echo "== anonymous on $HOST"
H=$(curl -s "${R[@]}" -D - -o "$W/anon.html" "$K/")
expect "GET / -> landing 200" "$(echo "$H" | head -1 | awk '{print $2}')" 200
expect "GET / is the landing document" "$(command grep -c -F "$title" "$W/anon.html")" 1
expect "GET / carries the build CSP" "$(echo "$H" | hdr content-security-policy | command grep -c "frame-ancestors 'none'")" 1
expect "GET / cache: private, no-cache" "$(echo "$H" | hdr cache-control)" "private, no-cache"
expect "GET / Vary: Cookie" "$(echo "$H" | hdr vary | command grep -c -x Cookie)" 1
expect "GET / sets no cookie" "$(echo "$H" | hdr set-cookie | wc -l | tr -d ' ')" 0
expect "HEAD / -> 200" "$(curl -s "${R[@]}" -I -o /dev/null -w '%{http_code}' "$K/")" 200
expect "GET /$asset -> 200" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -b 'x=1' "$K/$asset")" 200
if [[ -n "$immutable" ]]; then
  expect "landing asset cache is immutable" "$(curl -s "${R[@]}" -D - -o /dev/null "$K/$immutable" | hdr cache-control)" "public, max-age=31536000, immutable"
fi
expect "GET /hackathon.mp4 (with cookie) -> 200 video/mp4" "$(curl -s "${R[@]}" -b 'a=1' -o /dev/null -w '%{http_code} %{content_type}' "$K/hackathon.mp4")" "200 video/mp4"
expect "range on /hackathon.mp4 -> 206" "$(curl -s "${R[@]}" -r 1000-1999 -o /dev/null -w '%{http_code} %{size_download}' "$K/hackathon.mp4")" "206 1000"
expect "missing /other.mp4 -> gate 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/other.mp4")" 401
expect "GET /_landing/nope -> 404" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/_landing/nope")" 404
expect "POST / -> gate 401 (no landing, no origin)" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST "$K/")" 401
expect "POST /_landing/x -> gate 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST "$K/_landing/x")" 401
for p in /index.html /api/me /assets/index.js /motion/0f8fad5b-d9cb-469f-a165-70867728950e /headers.caddy /fran/notes-on-rivers /live/fran/rivers/page.json "/?x=1"; do
  code=$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K$p")
  if [[ "$p" == "/?x=1" ]]; then expect "GET $p -> landing" "$code" 200; else expect "GET $p -> 401" "$code" 401; fi
done
expect "spoofed identity on / -> still the landing" "$(curl -s "${R[@]}" -H 'X-RepoBox-User: fran' -H 'X-RepoBox-User-Id: 1' -H 'X-RepoBox-Auth: session' -H 'X-RepoBox-Role: admin' -H 'X-RepoBox-App: fieldwork' -H 'X-RepoBox-Gate-Primary-Host: 0' "$K/" | command grep -c -F "$title")" 1
expect "spoofed identity on /api/me -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -H 'X-RepoBox-User: fran' -H 'X-RepoBox-User-Id: 1' -H 'X-RepoBox-Auth: session' -H 'X-RepoBox-Role: admin' "$K/api/me")" 401
expect "stale app cookie on / -> landing" "$(curl -s "${R[@]}" -b '__Host-rb_app=stale' "$K/" | command grep -c -F "$title")" 1
expect "origin saw no anonymous request on $HOST" "$(origin_seen)" 0

echo "== platform host keeps the lane"
expect "fieldwork.repo.box live page shape reaches origin anonymously" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$FW/fran/notes-on-rivers")" 200
expect "fieldwork.repo.box / -> 401" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$FW/")" 401
: > "$W/origin-requests.log"

echo "== www"
expect "www -> 301 bare host" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{redirect_url}' "https://www.$HOST:$HTTPS/a?b=1")" "301 https://$HOST/a?b=1"

echo "== launch (device session for fran)"
"$P" --db "$DB" user enrol fran --out "$W/fran.url" >/dev/null
PATHPART=$(head -1 "$W/fran.url"); PATHPART=${PATHPART#https://auth.repo.box}
JAR="$W/jar"
expect "enrol POST signs the device in" "$(curl -s "${R[@]}" -c "$JAR" -o /dev/null -w '%{http_code}' -X POST -H 'Origin: https://auth.repo.box' "$A$PATHPART")" 303
LOC=$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{redirect_url}' "$A/fieldwork")
case "$LOC" in "https://$HOST/?rb_launch="*) printf '  ok   %-62s %s\n' "launcher -> https://$HOST/?rb_launch=…" "(code elided)";; *) echo "  FAIL launcher redirect: ${LOC%%rb_launch=*}"; fail=1;; esac
CODE=${LOC#*rb_launch=}
LOC2=$(curl -s "${R[@]}" -b "$JAR" -o /dev/null -w '%{redirect_url}' "$A/fieldwork?next=/motion/0f8fad5b-d9cb-469f-a165-70867728950e")
case "$LOC2" in "https://$HOST/motion/0f8fad5b-d9cb-469f-a165-70867728950e?rb_launch="*) printf '  ok   %-62s %s\n' "deep link launcher keeps next on $HOST" "(code elided)";; *) echo "  FAIL deep link: ${LOC2%%rb_launch=*}"; fail=1;; esac
AJAR="$W/appjar"
expect "gate redeems on $HOST -> clean redirect" "$(curl -s "${R[@]}" -c "$AJAR" -o /dev/null -w '%{http_code} %{size_download} %{redirect_url}' "$K/?rb_launch=$CODE")" "302 0 $K/"
expect "app cookie is host-only on $HOST" "$(awk -v h="$HOST" '($1==h || $1=="#HttpOnly_"h) && $2=="FALSE" && $6 ~ /^__Host-rb_app/' "$AJAR" | wc -l | tr -d ' ')" 1
body=$(curl -s "${R[@]}" -b "$AJAR" "$K/")
expect "signed-in / reaches the origin" "${body%% *}" ORIGIN
expect "origin sees gate identity (id 1, session)" "$(echo "${body#ORIGIN }" | python3 -c 'import json,sys; d={k.lower():v for k,v in json.load(sys.stdin).items()}; print(d.get("x-repobox-user-id"), d.get("x-repobox-auth"), d.get("x-repobox-app"))')" "1 session fieldwork"
expect "signed-in /api/me reaches the origin" "$(curl -s "${R[@]}" -b "$AJAR" "$K/api/me" | cut -c1-6)" ORIGIN
expect "kodu.work cookie does not open fieldwork.repo.box" "$(curl -s "${R[@]}" -b "$AJAR" -o /dev/null -w '%{http_code}' "$FW/")" 401
expect "code replay -> body-less redirect to launcher" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code} %{size_download}' "$K/?rb_launch=$CODE")" "303 0"

echo "== signed-in stranger without a grant"
"$P" --db "$DB" user enrol stranger --out "$W/s.url" >/dev/null
SP=$(head -1 "$W/s.url"); SP=${SP#https://auth.repo.box}
SJ="$W/sjar"
curl -s "${R[@]}" -c "$SJ" -o /dev/null -X POST -H 'Origin: https://auth.repo.box' "$A$SP"
expect "stranger launch -> 403 no access" "$(curl -s "${R[@]}" -b "$SJ" -o /dev/null -w '%{http_code}' "$A/fieldwork")" 403
: > "$W/origin-requests.log"
expect "stranger on / -> landing" "$(curl -s "${R[@]}" -b "$SJ" "$K/" | command grep -c -F "$title")" 1
expect "origin saw nothing from the stranger" "$(origin_seen)" 0

echo "== logs"
expect "no log has the launch code" "$(cat "$W"/*.log | command grep -c -- "$CODE" || true)" 0

[[ "$fail" -eq 0 ]] && echo "PRIMARY HOST E2E: PASS" || { echo "PRIMARY HOST E2E: FAIL ($W)"; exit 1; }
