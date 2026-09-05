#!/usr/bin/env bash
#
# Ship the verified standalone release tree to the repo.box web host.
#
# READ scripts/package-standalone.sh FIRST. This script only transports a tree
# that package-standalone.sh has already built and proven; it does not build,
# and it refuses to run against an unverified tree.
#
# WHICH HOST
# ----------
# repo.box is NOT served from the Hetzner build box. It is served from
# 204.168.190.248 by repobox-landing.service (User=fran,
# WorkingDirectory=/srv/apps/repobox-landing, node on 127.0.0.1:3480 behind
# Caddy). Checking the build box's Caddy or filesystem proves nothing about
# production; that mistake has already been made once and recorded on
# fi_dda49a9045a5c36dd06c.
#
# WHAT IT PROTECTS
# ----------------
#  * logs/hire-submissions.jsonl lives INSIDE the deploy directory and is real
#    user data. It is excluded from the sync and additionally copied out to a
#    timestamped backup before anything is written.
#  * The whole previous release tree is backed up before the swap, so a bad
#    deploy can be rolled back to the exact bytes that were serving before.
#  * If the post-deploy live sweep fails, the script rolls back automatically
#    and exits non-zero. A green systemctl is not accepted as evidence.
#
# USAGE
#   ./scripts/deploy-standalone.sh              # deploy to production
#   ./scripts/deploy-standalone.sh --dry-run    # show what would transfer
#   STAGING=1 ./scripts/deploy-standalone.sh    # deploy to a staging dir+port,
#                                               # production untouched

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
APP_DIR="$PWD"
STANDALONE="$APP_DIR/.next/standalone"

HOST="${DEPLOY_HOST:-fran@204.168.190.248}"
DRY_RUN=""
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN="--dry-run"

if [[ -n "${STAGING:-}" ]]; then
  # /srv/apps is root-owned, so a staging tree cannot be created beside the
  # production one without sudo. Stage under the deploy user's home instead;
  # the point of staging is to exercise the transport and the tree, not the
  # exact path.
  TARGET="${STAGING_DIR:-/home/fran/staging-repobox-landing}"
  SERVICE=""
  PROBE_PORT="${PROBE_PORT:-3489}"
else
  TARGET="/srv/apps/repobox-landing"
  SERVICE="repobox-landing.service"
  PROBE_PORT="3480"
fi

BACKUP_ROOT="/home/fran/backups/repobox-landing"
STAMP="$(date -u +%Y%m%d-%H%M%S)"

# Routes that must answer 200 after the deploy. Same list the packaging step
# verifies locally, plus the real nav, so a partial tree cannot pass.
SWEEP_PATHS=(
  / /agents /building /git /hire /made-by-agents /packages /playground
  /portfolio /projects /proof /trust /blog
  /llms.txt /feed.xml /SKILL.md /favicon.svg
)

log() { printf '\n=== %s ===\n' "$*"; }

# --- 0. refuse to ship an unverified or stale tree --------------------------
log "preflight"

[[ -f "$STANDALONE/server.js" ]] || {
  echo "FATAL: $STANDALONE/server.js missing. Run scripts/package-standalone.sh first." >&2
  exit 1
}

if [[ -d "$STANDALONE/public/public" ]]; then
  echo "FATAL: nested public/public present - tree was not packaged correctly." >&2
  exit 1
fi

src_count=$(find "$APP_DIR/public" -mindepth 1 -maxdepth 1 | wc -l)
out_count=$(find "$STANDALONE/public" -mindepth 1 -maxdepth 1 | wc -l)
[[ "$src_count" -eq "$out_count" ]] || {
  echo "FATAL: packaged public/ has $out_count entries, source has $src_count." >&2
  echo "Re-run scripts/package-standalone.sh." >&2
  exit 1
}

[[ -d "$STANDALONE/.next/static" ]] || {
  echo "FATAL: .next/static missing from the packaged tree." >&2
  exit 1
}

# The standalone tree must be newer than the newest source file, otherwise we
# would be shipping a stale build of older code. Only scan directories that
# actually exist: this app keeps its code in src/, not app/, and a missing
# directory would make find exit non-zero and abort the script under pipefail.
SRC_DIRS=()
for d in src app public content; do
  [[ -d "$APP_DIR/$d" ]] && SRC_DIRS+=("$APP_DIR/$d")
done
newest_src=""
if [[ ${#SRC_DIRS[@]} -gt 0 ]]; then
  newest_src=$(find "${SRC_DIRS[@]}" -type f -printf '%T@\n' | sort -n | tail -1 | cut -d. -f1)
fi
tree_time=$(stat -c %Y "$STANDALONE/server.js")
if [[ -n "$newest_src" && "$tree_time" -lt "$newest_src" ]]; then
  echo "FATAL: packaged tree is older than the newest source file - rebuild." >&2
  exit 1
fi

echo "tree ok: public=$out_count entries, static present, newer than source"
echo "target:  $HOST:$TARGET"
[[ -n "$DRY_RUN" ]] && echo "mode:    DRY RUN"
[[ -n "${STAGING:-}" ]] && echo "mode:    STAGING (production untouched)"

# --- 1. back up user data and the current release --------------------------
if [[ -z "$DRY_RUN" ]]; then
  log "backup"
  ssh -o BatchMode=yes "$HOST" "
    set -e
    mkdir -p '$BACKUP_ROOT'
    if [ -f '$TARGET/logs/hire-submissions.jsonl' ]; then
      cp -a '$TARGET/logs/hire-submissions.jsonl' \
            '$BACKUP_ROOT/hire-submissions.jsonl.$STAMP'
      echo \"hire submissions backed up: \$(wc -l < '$TARGET/logs/hire-submissions.jsonl') lines\"
    fi
    if [ -d '$TARGET' ]; then
      rm -rf '$BACKUP_ROOT/tree-previous'
      cp -a '$TARGET' '$BACKUP_ROOT/tree-previous'
      echo \"previous release tree backed up to $BACKUP_ROOT/tree-previous\"
    fi
  "
fi

# --- 2. transport ----------------------------------------------------------
# --delete keeps the target from accumulating removed files, but logs/ is
# excluded so live user submissions are never touched by the sync.
log "rsync"
ssh -o BatchMode=yes "$HOST" "mkdir -p '$TARGET'"
rsync -az --delete $DRY_RUN \
  --exclude 'logs/' \
  --itemize-changes \
  "$STANDALONE/" "$HOST:$TARGET/"

if [[ -n "$DRY_RUN" ]]; then
  log "dry run complete - nothing was changed"
  exit 0
fi

ssh -o BatchMode=yes "$HOST" "mkdir -p '$TARGET/logs'"

# --- 3. restart and sweep --------------------------------------------------
if [[ -n "$SERVICE" ]]; then
  log "restart $SERVICE"
  ssh -o BatchMode=yes "$HOST" "sudo -n systemctl restart '$SERVICE' && sleep 3 && systemctl is-active '$SERVICE'"
else
  log "staging: no service restart; probing the tree directly"
  # NOTE: never use `pkill -f <pattern>` here. Any pattern describing the
  # server (a PORT= string, or the server.js path) is ALSO present in the ssh
  # command line that carries it, so pkill matches the remote shell running
  # this very block and kills the session before node ever starts - the
  # symptom is ssh exit 255, or a silent no-op with no probe log at all.
  # Kill by listening port instead, which cannot match our own command line.
  ssh -o BatchMode=yes "$HOST" "
    fuser -k -n tcp $PROBE_PORT 2>/dev/null || true
    sleep 1
    cd '$TARGET'
    PORT=$PROBE_PORT HOSTNAME=127.0.0.1 NODE_ENV=production \
      setsid /home/fran/.local/bin/node '$TARGET/server.js' \
      >/tmp/staging-probe.log 2>&1 < /dev/null &
    disown 2>/dev/null || true
    for i in \$(seq 1 20); do
      curl -sf -o /dev/null http://127.0.0.1:$PROBE_PORT/ && break
      sleep 0.5
    done
    echo 'staging probe server started'
  " || true
fi

log "verify: live sweep"
fail=0
for p in "${SWEEP_PATHS[@]}"; do
  if [[ -n "${STAGING:-}" ]]; then
    code=$(ssh -o BatchMode=yes "$HOST" "curl -s -o /dev/null -w '%{http_code}' 'http://127.0.0.1:$PROBE_PORT$p'" || echo 000)
  else
    code=$(curl -s -o /dev/null -w '%{http_code}' "https://repo.box$p" || echo 000)
  fi
  printf '  %-18s %s\n' "$p" "$code"
  [[ "$code" == "200" ]] || fail=1
done

# --- 4. roll back if the sweep failed --------------------------------------
if [[ "$fail" -ne 0 ]]; then
  echo >&2
  echo "SWEEP FAILED - rolling back to the previous release tree." >&2
  ssh -o BatchMode=yes "$HOST" "
    set -e
    if [ -d '$BACKUP_ROOT/tree-previous' ]; then
      rm -rf '$TARGET'
      cp -a '$BACKUP_ROOT/tree-previous' '$TARGET'
      ${SERVICE:+sudo -n systemctl restart '$SERVICE'}
      echo 'rolled back'
    else
      echo 'NO BACKUP TO ROLL BACK TO' >&2
      exit 1
    fi
  "
  echo "Rollback done. Production is on the previous tree. Deploy NOT accepted." >&2
  exit 1
fi

# --- 5. confirm user data survived -----------------------------------------
log "post-deploy data check"
ssh -o BatchMode=yes "$HOST" "
  if [ -f '$TARGET/logs/hire-submissions.jsonl' ]; then
    echo \"hire submissions present: \$(wc -l < '$TARGET/logs/hire-submissions.jsonl') lines\"
  else
    echo 'hire submissions file absent (none had been recorded, or excluded correctly)'
  fi
  echo \"BUILD_ID: \$(cat '$TARGET/.next/BUILD_ID' 2>/dev/null)\"
"

log "deploy verified"
echo "Backups kept at $BACKUP_ROOT (tree-previous, hire-submissions.jsonl.$STAMP)."
echo
echo "REMINDER: /sitemap.xml, /robots.txt, /releases/*, /install.sh and /assets/*"
echo "are served by Caddy from /var/www/repo.box/subdomains/root, not by this app."
echo "They are deliberately NOT in the sweep above; changing them is a separate step."
