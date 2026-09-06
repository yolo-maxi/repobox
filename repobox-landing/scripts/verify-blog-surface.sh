#!/usr/bin/env bash
# Verifies the blog surface: every post is wired into the index, feed and
# sitemap exactly once, the feed is structurally sound, no sitemap entry points
# at a missing page, and the packaged tree serves each file byte-identically.
#
# This package has NO test runner. These are ad-hoc assertions, not a suite --
# say so when reporting a result from them.
#
# Run after adding or editing a post:  ./scripts/verify-blog-surface.sh
# Requires a packaged tree:            ./scripts/package-standalone.sh
set -uo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2
LANDING=$PWD
PUB=$LANDING/public
SLUG=${1:-powerless-landing-page-agent}
PORT=${PORT:-3495}
FAIL=0

ok(){ printf 'ok   %s\n' "$1"; }
bad(){ printf 'FAIL %s -- %s\n' "$1" "${2:-}"; FAIL=$((FAIL+1)); }
# is NAME ACTUAL EXPECTED
is(){ if [ "$2" = "$3" ]; then ok "$1 ($3)"; else bad "$1" "want $3, got $2"; fi; }
# run NAME CMD... -- asserts exit 0
run(){ local n=$1; shift; if "$@" >/dev/null 2>&1; then ok "$n"; else bad "$n" "exit $?"; fi; }

# shellcheck disable=SC2317  # invoked via trap
cleanup(){ fuser -k -n tcp "$PORT" >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "== 1. XML well-formedness =="
for f in feed.xml sitemap.xml; do
  run "$f parses" python3 -c "import xml.dom.minidom;xml.dom.minidom.parse('$PUB/$f')"
done

echo
echo "== 2. the new post is wired exactly once into each surface =="
is "index.html references post" "$(grep -c "$SLUG.html" "$PUB/blog/index.html")" 1
is "feed.xml <item> for post"   "$(grep -c "<link>https://repo.box/blog/$SLUG.html</link>" "$PUB/feed.xml")" 1
is "sitemap.xml <loc> for post" "$(grep -c "<loc>https://repo.box/blog/$SLUG.html</loc>" "$PUB/sitemap.xml")" 1

echo
echo "== 3. feed integrity: guids unique, guid==link, every item titled/dated =="
run "feed items structurally sound" python3 - "$PUB/feed.xml" <<'PY'
import sys, xml.etree.ElementTree as ET
items = ET.parse(sys.argv[1]).getroot().find("channel").findall("item")
guids = [i.findtext("guid") for i in items]
assert len(set(guids)) == len(guids), "duplicate guid"
assert guids == [i.findtext("link") for i in items], "guid/link divergence"
assert all(i.findtext("title") and i.findtext("pubDate") for i in items), "missing title/pubDate"
PY

echo
echo "== 4. every sitemap blog <loc> maps to a real file or app route =="
run "no dangling blog sitemap entry" python3 - "$PUB/sitemap.xml" "$LANDING" <<'PY'
import sys, os, re, xml.etree.ElementTree as ET
locs = [e.text for e in ET.parse(sys.argv[1]).getroot().iter() if e.tag.endswith("loc")]
root = sys.argv[2]
missing = [
    l for l in locs
    if (p := re.sub(r"^https://repo\.box/?", "", l)).startswith("blog/")
    and not os.path.exists(f"{root}/public/{p}")
    and not os.path.isdir(f"{root}/src/app/{p}")
]
assert not missing, f"dangling: {missing}"
PY

echo
echo "== 5. packaged tree serves the changed files, byte-identical to source =="
if [ ! -f "$LANDING/.next/standalone/server.js" ]; then
  bad "packaged tree present" "run scripts/package-standalone.sh first"
else
  ( cd "$LANDING/.next/standalone" && PORT=$PORT HOSTNAME=127.0.0.1 NODE_ENV=production \
      node server.js >/tmp/hermes-verify-server.log 2>&1 & )
  for _ in $(seq 1 30); do curl -sf -o /dev/null "http://127.0.0.1:$PORT/" && break; sleep 1; done
  B=http://127.0.0.1:$PORT
  for p in /blog "/blog/$SLUG.html" "/blog/$SLUG.txt" /feed.xml /blog/github-wasnt-built-for-this.html; do
    is "GET $p" "$(curl -s -o /dev/null -w '%{http_code}' "$B$p")" 200
  done
  for f in blog/index.html "blog/$SLUG.html" "blog/$SLUG.txt" feed.xml; do
    is "served $f == source" \
       "$(curl -s "$B/$f" | md5sum | cut -d' ' -f1)" \
       "$(md5sum "$PUB/$f" | cut -d' ' -f1)"
  done
  # The app does serve public/sitemap.xml; on the live host a Caddy
  # `handle /sitemap.xml` block rooted at /var/www/repo.box/subdomains/root
  # SHADOWS it. So assert the app copy is correct here, and record that editing
  # it alone does not change production -- the Caddy root needs the same update.
  is "served sitemap.xml == source" \
     "$(curl -s "$B/sitemap.xml" | md5sum | cut -d' ' -f1)" \
     "$(md5sum "$PUB/sitemap.xml" | cut -d' ' -f1)"
  echo "note repo.box serves /sitemap.xml from the Caddy static root, not this app;"
  echo "     shipping this tree alone leaves the live sitemap unchanged."
fi

echo
echo "== 6. claim harness is green against the committed post =="
run "verify-concierge-post-claims.mjs" node "$LANDING/scripts/verify-concierge-post-claims.mjs"

echo
echo "----"
if [ $FAIL -eq 0 ]; then
  echo "AD-HOC VERIFICATION PASSED (no suite exists for this package)"
else
  echo "$FAIL ASSERTION(S) FAILED"
fi
exit $((FAIL > 0))
