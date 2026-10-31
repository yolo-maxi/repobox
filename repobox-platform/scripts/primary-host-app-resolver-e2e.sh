#!/usr/bin/env bash
#
# Local end-to-end proof of `primary-host --resolver app` (Kodu mode) with a REAL
# Caddy and a fake app origin that implements the agreed contract:
#   - anonymous GET/HEAD / : the app answers 401 + X-Kodu-Auth: anonymous-root
#     (+ a stale-cookie clear) -> the edge serves the static landing and keeps
#     the Set-Cookie; a 401 without the marker, or on POST, passes through
#   - /login/, /api/kodu/v1/auth/* reach the app; X-Kodu-Claim / X-Kodu-Csrf
#     reach it intact; browser X-Kodu-Edge / X-Kodu-Edge-Proof / X-RepoBox-* never do
#   - the edge adds the shared secret; the origin refuses requests without it
#   - signed-in / , deep links and a WebSocket upgrade reach the app; renewal
#     Set-Cookie passes through; /api/health and /_repo_box/* are 404 at the edge
#   - the launcher keeps device sign-in on <name>.repo.box
#
# Usage: LANDING_DIR=<dist-landing> LANDING_HEADERS=<headers.json> scripts/primary-host-app-resolver-e2e.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build -p repobox-platform 2>/dev/null
P="${E2E_BIN:-$PWD/../target/debug/repobox-platform}"
W="${E2E_DIR:-$(mktemp -d)}"
export XDG_DATA_HOME="$W/caddy-data" XDG_CONFIG_HOME="$W/caddy-config"
DB="$W/platform.db"; GATE=127.0.0.1:3950; ORIGIN=127.0.0.1:3951; HTTPS=8446; HOST=kodu.work
: "${LANDING_DIR:?}" "${LANDING_HEADERS:?}"

"$P" --db "$DB" user create fran --display-name Fran --admin >/dev/null
"$P" --db "$DB" app register fieldwork --title "kodu.work" --owner fran --kind proxy --target "$ORIGIN" --visibility private --identity platform >/dev/null
"$P" --db "$DB" app primary-host fieldwork --host "$HOST" --landing --resolver app --review-note "e2e: app resolver with shared edge secret" >/dev/null
"$P" --db "$DB" app show fieldwork | grep "^primary host"
"$P" --db "$DB" routes render --gate "$GATE" --out "$W/apps.caddy" 2>/dev/null
mkdir -p "$W/srv/$HOST"; cp -r "$LANDING_DIR" "$W/srv/$HOST/current"
python3 deploy/landing-headers.py "$LANDING_HEADERS" "$W/srv/$HOST/headers.caddy" >/dev/null
python3 deploy/primary-host-edge-secret.py "$HOST" --root "$W" >/dev/null
SECRET=$(cat "$W/etc/repobox-platform/$HOST.edge-secret")
sed -i -e "s#/srv/$HOST/#$W/srv/$HOST/#g" -e "s#/etc/caddy/repobox-platform/secrets/#$W/etc/caddy/repobox-platform/secrets/#g" "$W/apps.caddy"
cat > "$W/Caddyfile" <<CADDY
{
	admin off
	local_certs
	https_port $HTTPS
	http_port 8093
	skip_install_trust
}
auth.repo.box {
	handle {
		request_header -X-RepoBox-*
		reverse_proxy $GATE
	}
}
import $W/apps.caddy
CADDY
caddy validate --config "$W/Caddyfile" --adapter caddyfile >/dev/null 2>&1 && echo "  caddy validate: OK"

cat > "$W/origin.py" <<'PY'
import json, sys, hmac
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer as HTTPServer
SECRET, LOG = sys.argv[2], sys.argv[3]
class H(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def log_message(self, *a): pass
    def send(self, code, body=b"", headers=()):
        self.send_response(code)
        for k, v in headers: self.send_header(k, v)
        self.send_header("Content-Length", str(len(body))); self.end_headers()
        if self.command != "HEAD": self.wfile.write(body)
    def handle_one(self):
        path = self.path.split("?")[0]; q = self.path.partition("?")[2]
        h = {k.lower(): v for k, v in self.headers.items()}
        cookie = h.get("cookie", "")
        with open(LOG, "a") as f:
            f.write(json.dumps({"m": self.command, "path": path, "edge_ok": hmac.compare_digest(h.get("x-kodu-edge", ""), SECRET),
                                "edge_proof": "x-kodu-edge-proof" in h, "repobox": [k for k in h if k.startswith("x-repobox-")],
                                "claim": h.get("x-kodu-claim"), "csrf": h.get("x-kodu-csrf"), "xff": h.get("x-forwarded-for")}) + "\n")
        if not hmac.compare_digest(h.get("x-kodu-edge", ""), SECRET):
            return self.send(403, b"edge")
        signed = "__Host-kodu=sess" in cookie
        if h.get("upgrade", "").lower() == "websocket":
            if not signed: return self.send(401, b"{}")
            self.send_response(101); self.send_header("Upgrade", "websocket"); self.send_header("Connection", "Upgrade"); self.end_headers(); return
        if path == "/login/": return self.send(200, b"LOGIN", [("Content-Type", "text/html")])
        if path == "/api/kodu/v1/auth/redeem" and self.command == "POST":
            if h.get("x-kodu-claim") == "good": return self.send(200, b'{"ok":true}', [("Set-Cookie", "__Host-kodu=sess; Path=/; Secure; HttpOnly; SameSite=Lax"), ("Content-Type", "application/json")])
            return self.send(410, b'{"code":"used"}')
        if path == "/api/kodu/v1/auth/signout" and self.command == "POST":
            return self.send(200 if h.get("x-kodu-csrf") == "c1" else 403, b"{}")
        if path == "/api/health": return self.send(200, b"HEALTH-LEAK")
        if signed:
            extra = [("Set-Cookie", "__Host-kodu=sess; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=2592000")] if "renew=1" in q else []
            return self.send(200, b"APP " + path.encode(), [("Content-Type", "text/html")] + extra)
        if path == "/" and "plain=1" not in q:
            clear = [("Set-Cookie", "__Host-kodu=; Path=/; Secure; HttpOnly; Max-Age=0")] if "__Host-kodu=" in cookie else []
            return self.send(401, b"ROOT401", [("X-Kodu-Auth", "anonymous-root"), ("Content-Type", "text/html")] + clear)
        return self.send(401, b'{"error":"sign in","login":"/login/"}', [("Content-Type", "application/json")])
    do_GET = do_POST = do_HEAD = handle_one
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
: > "$W/origin.log"
python3 "$W/origin.py" "${ORIGIN#*:}" "$SECRET" "$W/origin.log" >"$W/origin.err" 2>&1 & OP=$!
"$P" --db "$DB" serve --bind "$GATE" --public-base https://auth.repo.box --domain repo.box >"$W/cp.log" 2>&1 & CP=$!
caddy run --config "$W/Caddyfile" --adapter caddyfile >"$W/caddy.log" 2>&1 & CD=$!
trap 'kill $OP $CP $CD 2>/dev/null || true; wait 2>/dev/null || true' EXIT
for i in $(seq 1 40); do curl -sf "http://$GATE/healthz" >/dev/null && break; sleep 0.25; done; sleep 1

R=(-k --resolve "$HOST:$HTTPS:127.0.0.1" --resolve "auth.repo.box:$HTTPS:127.0.0.1")
K="https://$HOST:$HTTPS"; fail=0
expect() { if [[ "$2" == "$3" ]]; then printf '  ok   %-64s %s\n' "$1" "$2"; else printf '  FAIL %-64s got %s want %s\n' "$1" "$2" "$3"; fail=1; fi; }
hdr() { tr -d '\r' | awk -F': ' -v k="$1" 'tolower($1)==k {print $2}'; }
last() { tail -1 "$W/origin.log" | python3 -c "import json,sys; print(json.load(sys.stdin)$1)"; }
ws_status() { # cookie-or-empty -> HTTP status of a WebSocket upgrade on $HOST:/?writingBlocks=2 (raw TLS socket)
  python3 - "$HTTPS" "$HOST" "$1" 2>&1 <<'PY' | tail -1
import socket, ssl, sys
port, host, cookie = int(sys.argv[1]), sys.argv[2], sys.argv[3]
ctx = ssl.create_default_context(); ctx.check_hostname = False; ctx.verify_mode = ssl.CERT_NONE; ctx.set_alpn_protocols(["http/1.1"])
s = ctx.wrap_socket(socket.create_connection(("127.0.0.1", port), timeout=5), server_hostname=host)
req = f"GET /?writingBlocks=2 HTTP/1.1\r\nHost: {host}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nOrigin: https://{host}\r\n"
if cookie: req += f"Cookie: {cookie}\r\n"
s.sendall((req + "\r\n").encode()); print(s.recv(64).split(b" ")[1].decode())
PY
}
title=$(command grep -o '<title>[^<]*</title>' "$LANDING_DIR/index.html" | head -1)

echo "== anonymous"
H=$(curl -s "${R[@]}" -D - -o "$W/a.html" "$K/")
expect "GET / -> 200 landing (app's marked root 401 intercepted)" "$(echo "$H" | head -1 | awk '{print $2}') $(command grep -c -F "$title" "$W/a.html")" "200 1"
expect "edge added the shared secret on that request" "$(last "['edge_ok']")" True
expect "landing keeps private no-cache + Vary: Cookie" "$(echo "$H" | hdr cache-control) $(echo "$H" | hdr vary | command grep -c -x Cookie)" "private, no-cache 1"
H=$(curl -s "${R[@]}" -D - -o /dev/null -b '__Host-kodu=stale' "$K/")
expect "stale cookie on / -> landing AND the app's cookie clear is kept" "$(echo "$H" | head -1 | awk '{print $2}') $(echo "$H" | hdr set-cookie | command grep -c 'Max-Age=0')" "200 1"
expect "unmarked 401 on / passes through (no landing)" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/?plain=1")" 401
expect "POST / 401 passes through" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST "$K/")" 401
expect "GET /notes/x anonymous -> app's 401 with login hint" "$(curl -s "${R[@]}" "$K/notes/x" | command grep -c '"login":"/login/"')" 1
expect "GET /login/ -> app 200" "$(curl -s "${R[@]}" "$K/login/")" LOGIN
expect "/api/health is 404 at the edge" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/api/health") $(command grep -c '"/api/health"' "$W/origin.log")" "404 0"
expect "/_repo_box/ai/v1/models is 404 at the edge" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/_repo_box/ai/v1/models")" 404
expect "/_landing/* static, never reaches the app" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' "$K/_landing/$(cd "$LANDING_DIR/_landing" && ls | head -1)") $(command grep -c '_landing' "$W/origin.log")" "200 0"

echo "== edge headers"
curl -s "${R[@]}" -o /dev/null -H 'X-Kodu-Edge: forged' -H 'X-Kodu-Edge-Proof: v1.1.ff' -H 'X-RepoBox-User-Id: 1' -H 'X-RepoBox-Auth: session' "$K/notes/y"
expect "forged X-Kodu-Edge replaced by the edge's secret" "$(last "['edge_ok']")" True
expect "browser X-Kodu-Edge-Proof stripped" "$(last "['edge_proof']")" False
expect "browser X-RepoBox-* stripped" "$(last "['repobox']")" "[]"
expect "X-Forwarded-For is the client, not a forged value" "$(curl -s "${R[@]}" -o /dev/null -H 'X-Forwarded-For: 6.6.6.6' "$K/notes/z"; last "['xff']")" 127.0.0.1

echo "== sign-in"
J="$W/jar"
expect "redeem with X-Kodu-Claim reaches the app; Set-Cookie passes" "$(curl -s "${R[@]}" -c "$J" -o /dev/null -w '%{http_code}' -X POST -H 'X-Kodu-Claim: good' -H 'Content-Type: application/json' "$K/api/kodu/v1/auth/redeem") $(last "['claim']")" "200 good"
expect "host-only __Host-kodu cookie stored" "$(awk '($1=="kodu.work"||$1=="#HttpOnly_kodu.work") && $6=="__Host-kodu"' "$J" | wc -l | tr -d ' ')" 1
expect "replayed claim -> app's 410 passes through" "$(curl -s "${R[@]}" -o /dev/null -w '%{http_code}' -X POST -H 'X-Kodu-Claim: used' "$K/api/kodu/v1/auth/redeem")" 410
expect "signed-in / -> the app (not the landing)" "$(curl -s "${R[@]}" -b "$J" "$K/")" "APP /"
expect "signed-in deep link -> the app" "$(curl -s "${R[@]}" -b "$J" "$K/notes/0f8fad5b-d9cb-469f-a165-70867728950e")" "APP /notes/0f8fad5b-d9cb-469f-a165-70867728950e"
expect "renewal Set-Cookie on a signed-in response passes through" "$(curl -s "${R[@]}" -b "$J" -D - -o /dev/null "$K/notes/a?renew=1" | hdr set-cookie | command grep -c 'Max-Age=2592000')" 1
expect "WebSocket upgrade signed-in -> 101" "$(ws_status '__Host-kodu=sess')" 101
expect "WebSocket upgrade anonymous -> app's 401 (no landing)" "$(ws_status '')" 401
expect "signout with X-Kodu-Csrf reaches the app" "$(curl -s "${R[@]}" -b "$J" -o /dev/null -w '%{http_code}' -X POST -H 'X-Kodu-Csrf: c1' "$K/api/kodu/v1/auth/signout") $(last "['csrf']")" "200 c1"
expect "every app request carried the edge secret" "$(python3 -c "import json; print(all(json.loads(l)['edge_ok'] for l in open('$W/origin.log')))")" True

echo "== launcher"
"$P" --db "$DB" user enrol fran --out "$W/f.url" >/dev/null
PP=$(head -1 "$W/f.url"); PP=${PP#https://auth.repo.box}
curl -s "${R[@]}" -c "$W/dj" -o /dev/null -X POST -H 'Origin: https://auth.repo.box' "https://auth.repo.box:$HTTPS$PP"
LOC=$(curl -s "${R[@]}" -b "$W/dj" -o /dev/null -w '%{redirect_url}' "https://auth.repo.box:$HTTPS/fieldwork")
case "$LOC" in "https://fieldwork.repo.box/?rb_launch="*) printf '  ok   %-64s %s\n' "device launcher stays on fieldwork.repo.box" "(code elided)";; *) echo "  FAIL launcher: ${LOC%%rb_launch=*}"; fail=1;; esac
expect "no log has the edge secret" "$(cat "$W"/*.log "$W"/*.err 2>/dev/null | command grep -c -- "$SECRET" || true)" 0

[[ "$fail" -eq 0 ]] && echo "APP RESOLVER E2E: PASS" || { echo "APP RESOLVER E2E: FAIL ($W)"; exit 1; }
