#!/usr/bin/env bash
#
# Exercise the /api/hire contract end to end against the PACKAGED standalone
# tree, in one driver, with one exit code.
#
# WHY THIS EXISTS
# ---------------
# fi_5bd1f3d595c06ae5248b declares three evaluation gates:
#
#   1. POST /api/hire with valid and invalid payloads
#   2. success/error text matches real delivery
#   3. pnpm build
#
# Gate 3 has been run many times. Gates 1 and 2 had NEVER been run: the string
# "api/hire" appears zero times in 1033 home-heartbeat ledger entries, and this
# repository has no test runner at all (package.json scripts are dev/build/
# start/lint only, devDependencies carry no vitest/jest/playwright). The route
# handler's validation, its durable-log write, and its notification failure
# path were asserted in prose and never executed.
#
# This is ad-hoc verification, NOT a test suite. It makes exactly the
# assertions listed below and nothing else.
#
# WHAT IT ASSERTS
#   A. Valid payload -> 200 {success:true} and ONE new durable log line whose
#      contents round-trip the submitted fields.
#   B. Each invalid payload -> 400 with the specific documented error string,
#      and NO durable log line written (rejection must not persist).
#   C. A submission whose notification cannot be delivered -> 502 with the
#      user-facing "could not be delivered" text, NOT a false 200.
#   D. The success/error strings the API returns are the ones the UI renders
#      its states from (HireForm.tsx treats !response.ok as the error branch).
#   E. The harness leaves no test data behind: the logs/ directory it caused
#      the packaged server to create is removed, so a subsequent rsync cannot
#      ship synthetic submissions over real user data.
#
# NETWORK NOTE
#   Case C uses a deliberately invalid bot token. The route hardcodes
#   api.telegram.org, so it cannot be redirected at a local stub. With an
#   invalid token Telegram answers 401/404 and the route returns 502; if the
#   host has no network the fetch throws and the route ALSO returns 502. Both
#   paths are the assertion, so the case is robust either way, and no message
#   can be delivered to anyone because the token is not a real one.
#
# USAGE
#   ./scripts/verify-hire-flow.sh              # build + package + verify
#   ./scripts/verify-hire-flow.sh --no-build   # verify an existing build
#
# EXIT CODES
#   0 all assertions passed
#   1 an assertion failed
#   2 could not run (missing build, server never became ready)

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2
APP_DIR="$PWD"
STANDALONE="$APP_DIR/.next/standalone"
PORT="${HIRE_VERIFY_PORT:-3491}"
BASE="http://127.0.0.1:$PORT"
LOGDIR="$STANDALONE/logs"
LOGFILE="$LOGDIR/hire-submissions.jsonl"
SERVERLOG="$(mktemp -t hire-verify-server.XXXXXX.log)"

pass=0
fail=0

ok()   { printf '  PASS  %s\n' "$1"; pass=$((pass + 1)); }
bad()  { printf '  FAIL  %s\n' "$1"; fail=$((fail + 1)); }
log()  { printf '\n=== %s ===\n' "$*"; }

# assert_eq <label> <expected> <actual>
assert_eq() {
  if [[ "$2" == "$3" ]]; then ok "$1"; else bad "$1 (expected '$2', got '$3')"; fi
}

# assert_contains <label> <needle> <haystack>
assert_contains() {
  if [[ "$3" == *"$2"* ]]; then ok "$1"; else bad "$1 (missing '$2' in: ${3:0:200})"; fi
}

# --- 0. preconditions ------------------------------------------------------
if [[ "${1:-}" != "--no-build" ]]; then
  log "build + package"
  # package-standalone.sh already builds, flattens public/, and proves the
  # packaged tree serves. Reuse it rather than reimplementing the packaging.
  "$APP_DIR/scripts/package-standalone.sh" >/tmp/hire-verify-package.log 2>&1 || {
    echo "FATAL: package-standalone.sh failed; see /tmp/hire-verify-package.log" >&2
    tail -20 /tmp/hire-verify-package.log >&2
    exit 2
  }
  echo "packaged (log: /tmp/hire-verify-package.log)"
else
  log "build skipped (--no-build)"
fi

[[ -f "$STANDALONE/server.js" ]] || {
  echo "FATAL: $STANDALONE/server.js missing - nothing to verify" >&2
  exit 2
}

# The packaged server chdir()s to its own directory, so any logs/ it writes
# land in the build output. A logs/ can ALSO arrive there without us: `next
# build` output-file-tracing copies the source tree's own
# repobox-landing/logs/hire-submissions.jsonl (a real March submission) into
# .next/standalone/logs/ on every build. So its mere presence is not evidence
# of leftover test data.
#
# Distinguish the two rather than refusing on both, or the --build path can
# never run unattended:
#   - byte-identical to the source-tree copy  -> build output, snapshot + clear
#   - anything else                           -> unattributable, refuse (exit 2)
# Never delete a file we cannot account for.
SRC_LOG="$APP_DIR/logs/hire-submissions.jsonl"
if [[ -e "$LOGDIR" ]]; then
  if [[ -f "$LOGFILE" && -f "$SRC_LOG" ]] \
     && [[ "$(md5sum <"$LOGFILE" | cut -d' ' -f1)" == "$(md5sum <"$SRC_LOG" | cut -d' ' -f1)" ]] \
     && [[ "$(find "$LOGDIR" -type f | wc -l)" == "1" ]]; then
    snap="${HIRE_VERIFY_BACKUP_DIR:-$HOME/.heartbeat-backups}"
    mkdir -p "$snap"
    cp -a "$LOGFILE" "$snap/hire-submissions.build-tree.$(date +%Y%m%dT%H%M%S).jsonl"
    rm -rf "$LOGDIR"
    echo "cleared build-traced logs/ (byte-identical to source copy; snapshot in $snap)"
  else
    echo "FATAL: $LOGDIR exists and does not match the source-tree copy." >&2
    echo "Refusing to run so unattributable submissions cannot be deleted or" >&2
    echo "mistaken for this run's output. Inspect it by hand, then re-run." >&2
    exit 2
  fi
fi

# --- 1. boot the packaged server ------------------------------------------
# No REPOBOX_HIRE_TELEGRAM_* vars: this instance is deliberately unconfigured,
# which is the branch that must still return 200 and still persist.
log "boot packaged tree on :$PORT (notifications unconfigured)"

env -u REPOBOX_HIRE_TELEGRAM_BOT_TOKEN \
    -u REPOBOX_HIRE_TELEGRAM_CHAT_ID \
    -u REPOBOX_HIRE_TELEGRAM_THREAD_ID \
    PORT="$PORT" HOSTNAME=127.0.0.1 NODE_ENV=production \
    node "$STANDALONE/server.js" >"$SERVERLOG" 2>&1 &
SERVER_PID=$!

# SC2317: every command below runs from the EXIT trap, which shellcheck cannot
# see. Load bearing: without it shellcheck exits 1 on info severity alone.
# shellcheck disable=SC2317
cleanup() {
  # Kill by listening port, never pkill: a pattern describing this process is
  # also present in any ssh command line carrying it.
  fuser -k -n tcp "$PORT" >/dev/null 2>&1 || true
  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  rm -rf "$LOGDIR"
}
trap cleanup EXIT

ready=0
for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "$BASE/hire"; then ready=1; break; fi
  sleep 0.5
done
if [[ "$ready" -ne 1 ]]; then
  echo "FATAL: packaged server never became ready on $BASE" >&2
  tail -30 "$SERVERLOG" >&2
  exit 2
fi

post() {
  curl -s -o /tmp/hire-verify-body.json -w '%{http_code}' \
    -H 'Content-Type: application/json' \
    -X POST "$BASE/api/hire" --data "$1"
}

logcount() { [[ -f "$LOGFILE" ]] && wc -l <"$LOGFILE" | tr -d ' ' || echo 0; }

# --- 2. A: valid payload persists and succeeds -----------------------------
log "A. valid payload"

MARKER="hire-flow-verify-$$-$(date +%s)"
VALID=$(cat <<JSON
{"description":"$MARKER automated contract check, not a real enquiry",
 "projectType":"Automation","budget":"","timeline":"1-2 weeks",
 "email":"verify+$$@example.invalid"}
JSON
)

before=$(logcount)
code=$(post "$VALID")
body=$(cat /tmp/hire-verify-body.json)
after=$(logcount)

assert_eq      "valid payload returns 200"              "200" "$code"
assert_contains "valid payload reports success:true"    '"success":true' "$body"
assert_eq      "valid payload appends exactly one line" "1" "$((after - before))"

if [[ -f "$LOGFILE" ]]; then
  if node -e '
    const fs=require("fs");
    const marker=process.argv[1], file=process.argv[2];
    const lines=fs.readFileSync(file,"utf8").trim().split("\n").filter(Boolean);
    const rec=JSON.parse(lines[lines.length-1]);
    if(!rec.data) throw new Error("no data key");
    if(!String(rec.data.description).includes(marker)) throw new Error("description not round-tripped");
    if(rec.data.projectType!=="Automation") throw new Error("projectType not round-tripped");
    if(!rec.timestamp) throw new Error("no timestamp");
    if(!String(rec.notificationBody).includes(marker)) throw new Error("body not composed");
  ' "$MARKER" "$LOGFILE" 2>/tmp/hire-verify-node.err; then
    ok "durable line round-trips the submitted fields"
  else
    bad "durable line malformed: $(cat /tmp/hire-verify-node.err)"
  fi
else
  bad "durable line round-trips the submitted fields (no log file written at all)"
fi

# --- 3. B: invalid payloads are refused and persist nothing ----------------
log "B. invalid payloads"

# label|expected-code|expected-error|payload
INVALID_CASES=(
  'missing description|400|Missing required fields|{"description":"","projectType":"Web App","timeline":"1-2 weeks","email":"a@b.co"}'
  'missing projectType|400|Missing required fields|{"description":"x","projectType":"","timeline":"1-2 weeks","email":"a@b.co"}'
  'missing timeline|400|Missing required fields|{"description":"x","projectType":"Web App","timeline":"","email":"a@b.co"}'
  'missing email|400|Missing required fields|{"description":"x","projectType":"Web App","timeline":"1-2 weeks","email":""}'
  'unknown projectType|400|Invalid project type|{"description":"x","projectType":"Crypto Rug","timeline":"1-2 weeks","email":"a@b.co"}'
  'malformed email|400|Invalid email format|{"description":"x","projectType":"Web App","timeline":"1-2 weeks","email":"not-an-email"}'
  'description too long|400|Description too long|{"description":"PADDING","projectType":"Web App","timeline":"1-2 weeks","email":"a@b.co"}'
  'not JSON at all|500|Internal server error|this is not json'
)

base_count=$(logcount)
for row in "${INVALID_CASES[@]}"; do
  IFS='|' read -r label want_code want_err payload <<<"$row"
  if [[ "$label" == "description too long" ]]; then
    long=$(head -c 501 /dev/zero | tr '\0' 'x')
    payload=${payload/PADDING/$long}
  fi
  code=$(post "$payload")
  body=$(cat /tmp/hire-verify-body.json)
  assert_eq       "$label -> $want_code" "$want_code" "$code"
  assert_contains "$label -> \"$want_err\"" "$want_err" "$body"
done

assert_eq "no invalid payload wrote a durable line" "$base_count" "$(logcount)"

# --- 4. C: undeliverable notification must not report success --------------
# Restart the same packaged tree WITH notification env set to a token that
# cannot authenticate. The route must return 502, not a false 200.
log "C. notification failure path"

fuser -k -n tcp "$PORT" >/dev/null 2>&1 || true
wait "$SERVER_PID" 2>/dev/null || true

REPOBOX_HIRE_TELEGRAM_BOT_TOKEN='0:invalid-token-for-contract-verification' \
REPOBOX_HIRE_TELEGRAM_CHAT_ID='0' \
PORT="$PORT" HOSTNAME=127.0.0.1 NODE_ENV=production \
  node "$STANDALONE/server.js" >>"$SERVERLOG" 2>&1 &
SERVER_PID=$!

ready=0
for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "$BASE/hire"; then ready=1; break; fi
  sleep 0.5
done
if [[ "$ready" -ne 1 ]]; then
  bad "server did not come back up for the notification-failure case"
else
  code=$(post "$VALID")
  body=$(cat /tmp/hire-verify-body.json)
  assert_eq       "undeliverable notification returns 502"  "502" "$code"
  assert_contains "undeliverable notification says so"      "could not be delivered" "$body"
  assert_contains "undeliverable notification is not a success" \
                  "error" "$body"
fi

# --- 5. D: the UI renders its states from exactly these responses ----------
# HireForm.tsx branches on response.ok only, so the contract that matters is:
# 2xx -> the "Request Received!" panel, non-2xx -> the retry alert. Assert the
# component still reads it that way, so a copy change cannot silently detach
# the success panel from real delivery.
log "D. UI branch matches the API contract"

FORM="$APP_DIR/src/components/hire/HireForm.tsx"
form_code=$(grep -vE '^\s*(//|\*|/\*)' "$FORM")

if [[ "$form_code" == *"if (!response.ok)"* ]]; then
  ok "UI treats a non-2xx response as the error branch"
else
  bad "UI no longer branches on !response.ok - success panel may show on failure"
fi
if [[ "$form_code" == *"setSubmitted(true)"* ]]; then
  ok "UI shows the received panel only after a 2xx"
else
  bad "UI success panel is no longer gated on the response"
fi

# --- 6. E: leave nothing behind --------------------------------------------
log "E. no synthetic data left in the deployable tree"

fuser -k -n tcp "$PORT" >/dev/null 2>&1 || true
wait "$SERVER_PID" 2>/dev/null || true
rm -rf "$LOGDIR"
if [[ -e "$LOGDIR" ]]; then
  bad "harness left $LOGDIR in the packaged tree"
else
  ok "packaged tree carries no synthetic submissions"
fi

# --- summary ---------------------------------------------------------------
log "summary"
echo "  commit : $(git -C "$APP_DIR" rev-parse --short HEAD)"
echo "  passed : $pass"
echo "  failed : $fail"
echo "  server log: $SERVERLOG"
echo
echo "  NOTE: ad-hoc verification, $((pass + fail)) assertions. This repository"
echo "  has no test runner; this is not a suite and does not claim coverage"
echo "  beyond the assertions listed in this file's header."

[[ "$fail" -eq 0 ]] || exit 1
exit 0
