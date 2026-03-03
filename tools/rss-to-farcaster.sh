#!/usr/bin/env bash
# rss-to-farcaster.sh
# RSS-to-Farcaster auto-publish automation with safeguards

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FEED_FILE="$REPO_ROOT/public/feed.xml"
STATE_DIR="$REPO_ROOT/.state"
STATE_FILE="$STATE_DIR/farcaster-last-guid.txt"
CAST_GUARD="/home/xiko/clawd/scripts/farcaster-cast-guard.sh"

# Default mode
MODE="dry-run"

usage() {
  echo "Usage: ./rss-to-farcaster.sh [--dry-run | --post]"
  echo "Examples:"
  echo "  ./rss-to-farcaster.sh               # dry-run (default)"
  echo "  ./rss-to-farcaster.sh --dry-run     # explicit dry-run"
  echo "  ./rss-to-farcaster.sh --post        # actually post"
}

# Parse arguments
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      MODE="dry-run"
      ;;
    --post)
      MODE="post"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "❌ Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
  shift
done

# Ensure state directory exists
mkdir -p "$STATE_DIR"

# Check if feed file exists
if [ ! -f "$FEED_FILE" ]; then
  echo "❌ RSS feed not found: $FEED_FILE"
  exit 1
fi

# Check if cast guard exists (for post mode)
if [ "$MODE" = "post" ] && [ ! -x "$CAST_GUARD" ]; then
  echo "❌ Cast guard script not found or not executable: $CAST_GUARD"
  exit 1
fi

# Check for xmllint
if ! command -v xmllint >/dev/null 2>&1; then
  echo "❌ xmllint not found. Please install libxml2-utils."
  exit 1
fi

# Extract the most recent item from RSS feed
TITLE=$(xmllint --xpath "string(//item[1]/title)" "$FEED_FILE" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
LINK=$(xmllint --xpath "string(//item[1]/link)" "$FEED_FILE" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
GUID=$(xmllint --xpath "string(//item[1]/guid)" "$FEED_FILE" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
DESCRIPTION=$(xmllint --xpath "string(//item[1]/description)" "$FEED_FILE" 2>/dev/null | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

# Validate required fields
if [ -z "$TITLE" ] || [ -z "$LINK" ] || [ -z "$GUID" ]; then
  echo "❌ Missing required fields in RSS item (title/link/guid)"
  exit 1
fi

# Read last posted GUID
LAST_GUID=""
if [ -f "$STATE_FILE" ]; then
  LAST_GUID=$(cat "$STATE_FILE" 2>/dev/null || true)
fi

# Check if this item was already posted
if [ -n "$LAST_GUID" ] && [ "$GUID" = "$LAST_GUID" ]; then
  # Nothing new to post
  if [ "$MODE" = "dry-run" ]; then
    echo "✅ No new posts (latest GUID already posted: $GUID)"
  fi
  exit 0
fi

# Generate sanitized cast text
# Truncate description to fit within limits
MAX_DESC_LEN=150
TRUNCATED_DESC="$DESCRIPTION"
if [ ${#DESCRIPTION} -gt $MAX_DESC_LEN ]; then
  TRUNCATED_DESC="${DESCRIPTION:0:$MAX_DESC_LEN}..."
fi

# Generate cast text
CAST_TEXT="New post: $TITLE

$TRUNCATED_DESC

$LINK"

# Validate cast text length (Farcaster limit is 320 chars)
if [ ${#CAST_TEXT} -gt 320 ]; then
  # Try with shorter description
  MAX_DESC_LEN=100
  TRUNCATED_DESC="$DESCRIPTION"
  if [ ${#DESCRIPTION} -gt $MAX_DESC_LEN ]; then
    TRUNCATED_DESC="${DESCRIPTION:0:$MAX_DESC_LEN}..."
  fi
  
  CAST_TEXT="New post: $TITLE

$TRUNCATED_DESC

$LINK"
  
  # If still too long, try title + link only
  if [ ${#CAST_TEXT} -gt 320 ]; then
    CAST_TEXT="New post: $TITLE

$LINK"
  fi
fi

# Output based on mode
if [ "$MODE" = "dry-run" ]; then
  echo "📝 NEW POST DETECTED (dry-run mode)"
  echo "Title: $TITLE"
  echo "Link: $LINK"
  echo "GUID: $GUID"
  echo ""
  echo "Cast text (${#CAST_TEXT} chars):"
  echo "---"
  echo "$CAST_TEXT"
  echo "---"
  echo ""
  echo "To actually post, run: $0 --post"
else
  echo "🚀 POSTING NEW BLOG POST"
  echo "Title: $TITLE"
  echo "GUID: $GUID"
  echo ""
  
  # Use cast guard to post safely
  if "$CAST_GUARD" "$CAST_TEXT"; then
    # Update state file only after successful posting
    echo "$GUID" > "$STATE_FILE"
    echo "✅ Posted and updated state file"
  else
    echo "❌ Cast guard blocked the post or posting failed"
    exit 1
  fi
fi
