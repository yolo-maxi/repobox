#!/bin/bash
# Sign a commit for repo.box dogfooding
# Usage: sign-commit.sh <address>
# Reads commit content from stdin, outputs signed commit to stdout

set -euo pipefail

ADDRESS="${1:?Usage: sign-commit.sh <address>}"
REPOBOX_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Read commit content from stdin
COMMIT_CONTENT=$(cat)

# Use cargo to run a signing helper inline
cd "$REPOBOX_DIR"
SIG_HEX=$(echo -n "$COMMIT_CONTENT" | cargo run --release -p repobox-core --example sign -- "$ADDRESS" 2>/dev/null)

# Build the signed commit (insert gpgsig header after committer line)
echo "$COMMIT_CONTENT" | awk -v sig="$SIG_HEX" '
/^$/ && !done {
    print "gpgsig " sig
    done = 1
}
{ print }
'
