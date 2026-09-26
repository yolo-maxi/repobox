#!/usr/bin/env bash
#
# Install / update the platform AI bridge on the ChatMock host (Hetzner):
#
#   repo.box 127.0.0.1:3232  --(reverse SSH tunnel)-->  Hetzner 127.0.0.1:8127
#   repobox-ai-broker (bridge secret, limits, model allowlist ∩ live models)
#   --> ChatMock 127.0.0.1:8111
#
# Run from the Hetzner build box. Steps:
#   1. build the static musl binary (same one deploy.sh ships to repo.box)
#   2. install it under /opt/repobox-platform/bin (previous copy kept as .prev)
#   3. create the bridge secret once (root 0600, /etc/repobox-platform/
#      ai-bridge.secret) and copy it to the same path on repo.box through
#      ssh stdin; the value is never printed, logged or put on a command line
#   4. install + (re)start repobox-ai-broker and repobox-ai-tunnel
#   5. prove: broker bound to loopback only, secret required, tunnel end on
#      repo.box bound to loopback only, secret required through the tunnel
#
#   ./scripts/deploy-ai-bridge.sh            install/update
#   ./scripts/deploy-ai-bridge.sh rollback   stop + disable broker and tunnel
#                                            (the app AI endpoint then answers
#                                            502; app routing is untouched)
#
# The control plane on repo.box reads the secret via systemd LoadCredential,
# so run this before the first `deploy.sh` that ships the AI-enabled unit.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
CRATE_DIR="$PWD"
cd ..
REPO="$PWD"

HOST="${DEPLOY_HOST:-fran@204.168.190.248}"
TARGET=x86_64-unknown-linux-musl
BIN="$REPO/target/$TARGET/release/repobox-platform"
SECRET=/etc/repobox-platform/ai-bridge.secret
UNITS="repobox-ai-broker.service repobox-ai-tunnel.service"

log() { printf '\n=== %s ===\n' "$*"; }
remote() { ssh -o BatchMode=yes "$HOST" "$@"; }

if [[ "${1:-}" == "rollback" ]]; then
  log "rollback: stop + disable $UNITS"
  sudo -n systemctl disable --now $UNITS || true
  systemctl is-active $UNITS || true
  exit 0
fi

log "build"
REPOBOX_PLATFORM_GIT_SHA="$(git rev-parse --short HEAD)" \
  cargo build --release -p repobox-platform --target "$TARGET"
ldd "$BIN" 2>&1 | command grep -qiE "statically linked|not a dynamic executable" || { echo "FATAL: binary is not static" >&2; exit 1; }

log "install binary"
sudo -n install -d -o root -g root -m 0755 /opt/repobox-platform /opt/repobox-platform/bin
if [[ -f /opt/repobox-platform/bin/repobox-platform ]]; then
  sudo -n cp -p /opt/repobox-platform/bin/repobox-platform /opt/repobox-platform/bin/repobox-platform.prev
fi
sudo -n install -o root -g root -m 0755 "$BIN" /opt/repobox-platform/bin/repobox-platform

log "bridge secret (never printed)"
sudo -n install -d -o root -g root -m 0700 /etc/repobox-platform
if ! sudo -n test -s "$SECRET"; then
  head -c 48 /dev/urandom | base64 | tr -d '/+=\n' | sudo -n sh -c "umask 077; cat > '$SECRET'"
  echo "created $SECRET"
fi
sudo -n chmod 0600 "$SECRET"
local_sum=$(sudo -n sha256sum "$SECRET" | cut -d' ' -f1)
remote_sum=$(remote "sudo -n sha256sum '$SECRET' 2>/dev/null | cut -d' ' -f1" || true)
if [[ "$local_sum" != "$remote_sum" ]]; then
  sudo -n cat "$SECRET" | remote "set -e
    sudo -n install -d -o root -g root -m 0700 /etc/repobox-platform
    sudo -n sh -c \"umask 077; cat > '$SECRET.new' && chmod 0600 '$SECRET.new' && mv '$SECRET.new' '$SECRET'\""
  echo "secret copied to $HOST:$SECRET (restart repobox-platform there to load a changed secret)"
else
  echo "secret already in place on $HOST"
fi

log "units"
for u in $UNITS; do
  sudo -n install -o root -g root -m 0644 "$CRATE_DIR/deploy/$u" "/etc/systemd/system/$u"
done
sudo -n systemctl daemon-reload
sudo -n systemctl enable repobox-ai-broker.service repobox-ai-tunnel.service
sudo -n systemctl restart repobox-ai-broker.service
sleep 1
sudo -n systemctl restart repobox-ai-tunnel.service
sleep 3
systemctl is-active $UNITS

log "verify"
fail=0
check() { if [[ "$2" == "$3" ]]; then printf '  ok   %-60s %s\n' "$1" "$2"; else printf '  FAIL %-60s got %s want %s\n' "$1" "$2" "$3"; fail=1; fi; }
auth_hdr() { printf 'Authorization: Bearer %s\n' "$(sudo -n cat "$SECRET")"; }
check "broker listens on loopback only" "$(ss -ltnH 'sport = :8127' | awk '{print $4}' | sort -u | tr '\n' ' ')" "127.0.0.1:8127 "
check "broker health" "$(curl -s -m5 http://127.0.0.1:8127/healthz)" "ok"
check "broker without secret -> 401" "$(curl -s -m5 -o /dev/null -w '%{http_code}' http://127.0.0.1:8127/v1/models)" 401
check "broker with secret lists models" "$(curl -s -m15 -H @<(auth_hdr) http://127.0.0.1:8127/v1/models | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["data"]) > 0)')" True
check "ChatMock still loopback only" "$(ss -ltnH 'sport = :8111' | awk '{print $4}' | sort -u | tr '\n' ' ')" "127.0.0.1:8111 "
check "tunnel end on repo.box is loopback only" "$(remote "ss -ltnH 'sport = :3232' | awk '{print \$4}' | sort -u | tr '\n' ' '")" "127.0.0.1:3232 "
check "through tunnel: health" "$(remote 'curl -s -m5 http://127.0.0.1:3232/healthz')" "ok"
check "through tunnel: no secret -> 401" "$(remote "curl -s -m5 -o /dev/null -w '%{http_code}' http://127.0.0.1:3232/v1/models")" 401
check "broker port not reachable publicly" "$(curl -s -m5 -o /dev/null -w '%{http_code}' "http://$(hostname -I | awk '{print $1}'):8127/healthz" || true)" 000
[[ "$fail" -eq 0 ]] || { echo "AI BRIDGE VERIFY FAILED (rollback: $0 rollback)" >&2; exit 1; }
log "AI bridge verified"
