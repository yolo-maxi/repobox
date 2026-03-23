#!/usr/bin/env bash

set -euo pipefail

# Deterministic production smoke test: verify unsigned pushes are rejected on
# addressless smart-HTTP receive-pack.

REMOTE_PRIMARY="${1:-${REPOBOX_SMOKE_REMOTE:-https://git.repo.box/repobox.git}}"
REMOTE_FALLBACK="${2:-${REPOBOX_SMOKE_REMOTE_FALLBACK:-http://127.0.0.1:3490/repobox.git}}"
WORKDIR="$(mktemp -d -t repobox-unsigned-smoke-XXXXXX)"
STATE_FILE="/home/xiko/repobox/.state/unsigned-push-smoke-result.json"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
PUSH_OUTPUT=""
PUSH_RC=""

cleanup() {
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

log() {
  printf '%s %s\n' "[$TS]" "$1"
}

build_smoke_remote() {
  local base_remote="$1"
  local stem="$base_remote"

  stem="${stem%/}"
  if [[ "$stem" == *.git ]]; then
    stem="${stem%.git}"
  fi

  printf '%s-smoke-%s.git' "$stem" "$RUN_ID"
}

run_push_test() {
  local base_remote="$1"
  local remote="$(build_smoke_remote "$base_remote")"
  local repo_dir="$WORKDIR/repo"
  local log_file="$repo_dir/push.log"

  rm -rf "$repo_dir"
  git init "$repo_dir" >/dev/null
  git -C "$repo_dir" config user.name "repo.box smoke"
  git -C "$repo_dir" config user.email "smoke@repobox"
  printf 'repobox unsigned smoke\n' > "$repo_dir/smoke.txt"
  git -C "$repo_dir" add smoke.txt
  git -C "$repo_dir" commit -m "smoke: unsigned push should fail" >/dev/null
  git -C "$repo_dir" remote add origin "$remote"

  set +e
  PUSH_OUTPUT="$(git -C "$repo_dir" push -u origin HEAD:refs/heads/main 2>&1)"
  PUSH_RC="$?"
  set -e

  printf '%s\n' "$PUSH_OUTPUT" > "$log_file"

  if [[ "$PUSH_RC" -eq 0 ]]; then
    cat > "$STATE_FILE" <<EOF
{
  "timestamp_utc": "$TS",
  "status": "failed",
  "reason": "unsigned push unexpectedly succeeded",
  "remote": "$remote",
  "push_exit_code": $PUSH_RC
}
EOF
    return 1
  fi

  if ! printf '%s\n' "$PUSH_OUTPUT" | grep -Ei "Unsigned commit rejected|All commits pushed to repo.box must be EVM-signed|Could not determine pusher identity|remote rejected|curl 22|HTTP 403|HTTP 400|pre-receive hook declined" >/dev/null; then
    cat > "$STATE_FILE" <<EOF
{
  "timestamp_utc": "$TS",
  "status": "failed",
  "reason": "unexpected rejection text for unsigned push",
  "remote": "$remote",
  "push_exit_code": $PUSH_RC
}
EOF
    return 1
  fi

  cat > "$STATE_FILE" <<EOF
{
  "timestamp_utc": "$TS",
  "status": "passed",
  "reason": "unsigned push rejected as expected",
  "remote": "$remote",
  "push_exit_code": $PUSH_RC
}
EOF
  return 0
}

log "Running unsigned push smoke (primary base: $REMOTE_PRIMARY)"
if run_push_test "$REMOTE_PRIMARY"; then
  log "Unsigned push smoke passed on primary remote"
  exit 0
fi

if [[ "$PUSH_OUTPUT" == *"Could not connect to"* || "$PUSH_OUTPUT" == *"connection refused"* || "$PUSH_OUTPUT" == *"Could not resolve host"* || "$PUSH_OUTPUT" == *"Failed to connect"* || "$PUSH_OUTPUT" == *"gnutls_handshake"* || "$PUSH_OUTPUT" == *"unexpected TLS packet"* ]]; then
  if [[ -n "$REMOTE_FALLBACK" && "$REMOTE_FALLBACK" != "$REMOTE_PRIMARY" ]]; then
    log "Primary remote unreachable, retrying fallback base: $REMOTE_FALLBACK"
    if run_push_test "$REMOTE_FALLBACK"; then
      log "Unsigned push smoke passed on fallback remote"
      exit 0
    fi
    cat "$STATE_FILE"
    log "Fallback check failed"
    exit 1
  fi
fi

cat "$STATE_FILE"
log "Primary check failed"
exit 1
