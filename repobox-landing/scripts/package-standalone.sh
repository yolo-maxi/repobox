#!/usr/bin/env bash
#
# Package the repobox-landing Next standalone build into a shippable release
# tree, and prove the packaged tree actually serves the public assets.
#
# WHY THIS EXISTS
# ---------------
# `next build` with output:"standalone" does NOT copy public/ into
# .next/standalone/public. On this app it emits a .next/standalone/public
# containing only heatmap-data.json (the one asset imported from code), and
# nothing else. A standalone server started against that tree returns 404 for
# /sitemap.xml, /llms.txt, /robots.txt, /SKILL.md, /feed.xml, /favicon.svg and
# every /blog/* static file.
#
# That is the exact production 404 signature diagnosed on fi_6577f2d0ab9f8e4550a1.
# It is reproduced by the build every single time, so it is not a one-off rsync
# mistake and it cannot be fixed in source. The packaging step below is the fix.
#
# The same applies to .next/static, which standalone also does not copy.
#
# USAGE
#   ./scripts/package-standalone.sh            # build + package + verify
#   ./scripts/package-standalone.sh --no-build # package an existing build
#
# Output tree: .next/standalone/  (self-contained, ready to rsync)
#
# DEPLOY NOTE - TWO STATIC ROOTS
# ------------------------------
# repo.box does not serve everything from this app. The Caddy site has explicit
# handle blocks that serve these paths from /var/www/repo.box/subdomains/root
# on the web host, bypassing Next entirely:
#
#     /sitemap.xml   /robots.txt   /releases/*   /install.sh   /assets/*
#
# So those files exist in TWO places and the Caddy copy wins. Changing them in
# repobox-landing/public/ alone will build and package correctly and still not
# appear on the live site. Any deploy that intends to change them must update
# the Caddy static root deliberately as a separate step. This script verifies
# the app-side copy only, and prints a reminder listing the overridden paths.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
APP_DIR="$PWD"
STANDALONE="$APP_DIR/.next/standalone"
VERIFY_PORT="${VERIFY_PORT:-3487}"

# Paths that must be served by the packaged app. If any of these 404 against
# the packaged tree, the flatten step did not work and the deploy would
# reintroduce the production 404s.
VERIFY_PATHS=(
  /sitemap.xml
  /robots.txt
  /llms.txt
  /SKILL.md
  /feed.xml
  /favicon.svg
  /blog/index.html
  /blog
  /
)

# Paths served by Caddy from /var/www/repo.box/subdomains/root, NOT by this app.
CADDY_OVERRIDDEN=(/sitemap.xml /robots.txt '/releases/*' /install.sh '/assets/*')

log() { printf '\n=== %s ===\n' "$*"; }

# --- 1. build --------------------------------------------------------------
if [[ "${1:-}" != "--no-build" ]]; then
  log "build"
  corepack pnpm install --frozen-lockfile
  corepack pnpm build
else
  log "build skipped (--no-build)"
fi

[[ -f "$STANDALONE/server.js" ]] || {
  echo "FATAL: $STANDALONE/server.js missing - build did not produce a standalone bundle" >&2
  exit 1
}

# --- 2. flatten public/ and .next/static ----------------------------------
# This is the step `next build` does not do. Replace wholesale rather than
# merging, so a stale nested public/public from an older build cannot survive.
log "package: flatten public/ and .next/static into standalone"

rm -rf "$STANDALONE/public"
mkdir -p "$STANDALONE/public"
cp -R "$APP_DIR/public/." "$STANDALONE/public/"

rm -rf "$STANDALONE/.next/static"
mkdir -p "$STANDALONE/.next"
cp -R "$APP_DIR/.next/static" "$STANDALONE/.next/static"

# Guard: the nested-public shape must not exist after packaging.
if [[ -d "$STANDALONE/public/public" ]]; then
  echo "FATAL: nested public/public survived packaging - this is the 404 bug" >&2
  exit 1
fi

src_count=$(find "$APP_DIR/public" -mindepth 1 -maxdepth 1 | wc -l)
out_count=$(find "$STANDALONE/public" -mindepth 1 -maxdepth 1 | wc -l)
echo "public/ entries: source=$src_count packaged=$out_count"
if [[ "$src_count" -ne "$out_count" ]]; then
  echo "FATAL: packaged public/ entry count does not match source" >&2
  exit 1
fi

# --- 3. verify the packaged tree actually serves ---------------------------
# Start the packaged server exactly as production runs it and sweep the routes.
# A green build is not evidence; this is.
log "verify: serving packaged tree on :$VERIFY_PORT"

PORT="$VERIFY_PORT" HOSTNAME=127.0.0.1 NODE_ENV=production \
  node "$STANDALONE/server.js" >/tmp/package-standalone-verify.log 2>&1 &
SERVER_PID=$!
cleanup() { kill "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true; }
trap cleanup EXIT

for _ in $(seq 1 40); do
  curl -sf -o /dev/null "http://127.0.0.1:$VERIFY_PORT/" && break
  sleep 0.5
done

fail=0
for p in "${VERIFY_PATHS[@]}"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$VERIFY_PORT$p" || echo 000)
  printf '  %-22s %s\n' "$p" "$code"
  [[ "$code" == "200" ]] || fail=1
done

if [[ "$fail" -ne 0 ]]; then
  echo >&2
  echo "FATAL: packaged tree does not serve every required path." >&2
  echo "Do NOT deploy this tree - it would reintroduce the production 404s." >&2
  echo "Server log: /tmp/package-standalone-verify.log" >&2
  exit 1
fi

log "packaged tree verified"
cat <<EOF
Release tree: $STANDALONE
Run it as:    PORT=3480 HOSTNAME=0.0.0.0 NODE_ENV=production node server.js

REMINDER - these paths are served by Caddy from
/var/www/repo.box/subdomains/root on the web host, NOT by this app:
$(printf '  %s\n' "${CADDY_OVERRIDDEN[@]}")
A change to those files in public/ will package correctly here and still not
appear on repo.box until the Caddy static root is updated as a separate step.

After deploying, sweep the live routes before calling the deploy done:
  for p in / /agents /building /git /hire /made-by-agents /packages /playground \\
           /portfolio /projects /proof /trust /blog /sitemap.xml /robots.txt \\
           /llms.txt /feed.xml /SKILL.md; do
    printf '%-18s %s\\n' "\$p" "\$(curl -s -o /dev/null -w '%{http_code}' https://repo.box\$p)"
  done
EOF
