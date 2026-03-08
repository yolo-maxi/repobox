#!/bin/bash

# build-homepage.sh
# Rebuild the homepage with daily activity feed
# Integrates the generated daily-activity.html into index.html
#
# Usage: ./tools/build-homepage.sh
# Dependencies: ./tools/daily-activity.sh
#
# This script is idempotent and safe to run multiple times.

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOMEPAGE="$REPO_ROOT/public/index.html"
ACTIVITY_DATA="$REPO_ROOT/public/data/daily-activity.html"

echo "🔨 Building homepage with daily activity feed..."

# First, generate the latest activity data
echo "📊 Generating daily activity data..."
"$SCRIPT_DIR/daily-activity.sh"

# Check if activity data exists
if [[ ! -f "$ACTIVITY_DATA" ]]; then
    echo "❌ Activity data not found at $ACTIVITY_DATA"
    exit 1
fi

# Create backup of current homepage if it doesn't have .backup extension
if [[ ! -f "${HOMEPAGE}.backup" ]]; then
    echo "💾 Creating backup of original homepage..."
    cp "$HOMEPAGE" "${HOMEPAGE}.backup"
fi

# Use the backup as the source template
TEMPLATE="${HOMEPAGE}.backup"

echo "🔍 Finding insertion point..."

# Find the line number after the first Projects section ends (</section>)
# We want to insert after "Things We've Shipped" section
INSERTION_LINE=$(grep -n "Things We've Shipped" "$TEMPLATE" | head -1 | cut -d: -f1)
INSERTION_LINE=$((INSERTION_LINE + 10)) # Move past the section to find its closing

# Find the actual end of the section
while IFS= read -r line; do
    INSERTION_LINE=$((INSERTION_LINE + 1))
    if echo "$line" | grep -q "</section>"; then
        break
    fi
done < <(tail -n +$INSERTION_LINE "$TEMPLATE")

echo "📍 Inserting activity feed after line $INSERTION_LINE"

# Create the new homepage
{
    # Everything before the insertion point
    head -n $INSERTION_LINE "$TEMPLATE"
    
    # Add a blank line and the activity feed
    echo ""
    cat "$ACTIVITY_DATA"
    echo ""
    
    # Everything after the insertion point  
    tail -n +$((INSERTION_LINE + 1)) "$TEMPLATE"
} > "$HOMEPAGE"

echo "✅ Homepage rebuilt with daily activity feed"
echo "📄 Output: $HOMEPAGE"

# Validate the HTML structure
if grep -q "What We Shipped Today" "$HOMEPAGE" && grep -q "What We Build" "$HOMEPAGE"; then
    echo "✅ HTML validation passed - both sections found"
else
    echo "❌ HTML validation failed - restoring backup"
    cp "${HOMEPAGE}.backup" "$HOMEPAGE"
    exit 1
fi