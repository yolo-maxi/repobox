#!/bin/bash

# daily-activity.sh
# Generate daily commit activity for repo.box homepage
# Scans git repos in /home/xiko/ and collects today's commits
#
# Usage: ./tools/daily-activity.sh
# Output: public/data/daily-activity.html
#
# This script is idempotent and cron-friendly.

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_FILE="$REPO_ROOT/public/data/daily-activity.html"

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

echo "🔍 Generating daily activity feed for repo.box..."

# Get today's date in git format (allow override via environment)
TODAY=${TODAY:-$(date '+%Y-%m-%d')}
echo "📅 Scanning commits for: $TODAY"

# Create a temporary file to collect commits
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

# List of repositories to scan
REPOS=(
    "/home/xiko/archipelago"
    "/home/xiko/banger"
    "/home/xiko/beamr-economy"
    "/home/xiko/botfight"
    "/home/xiko/cabin"
    "/home/xiko/kanban-app"
    "/home/xiko/kanban-projects"
    "/home/xiko/langbot"
    "/home/xiko/markdown-kanban"
    "/home/xiko/oceangram"
    "/home/xiko/oceangram-daemon"
    "/home/xiko/oceangram-tray"
    "/home/xiko/pool-admin"
    "/home/xiko/prompster"
    "/home/xiko/prompster-app"
    "/home/xiko/prompster-bot"
    "/home/xiko/repobox"
    "/home/xiko/rikai-admin-bot"
    "/home/xiko/rikai-ui"
    "/home/xiko/skillmarket"
    "/home/xiko/sss"
    "/home/xiko/sss-github"
    "/home/xiko/streme-frontend"
    "/home/xiko/SUPStrategy"
    "/home/xiko/takopi"
    "/home/xiko/telegram-tt"
    "/home/xiko/toolsdk-mcp-registry"
    "/home/xiko/tradestrategy-work"
    "/home/xiko/yolomaxi-telegram"
)

# Scan each repository
for repo in "${REPOS[@]}"; do
    if [[ -d "$repo/.git" ]]; then
        repo_name=$(basename "$repo")
        
        # Get today's commits (since midnight UTC)
        commits=$(cd "$repo" && git log --since="$TODAY 00:00:00" --until="$TODAY 23:59:59" \
            --pretty=format:"%H|%an|%s|%ad" --date=format:'%H:%M' \
            --no-merges 2>/dev/null || true)
        
        if [[ -n "$commits" ]]; then
            echo "📦 Found commits in $repo_name"
            
            # Add to temp file with repo name prefix
            while IFS='|' read -r hash author message time; do
                echo "$repo_name|$author|$message|$time" >> "$TEMP_FILE"
            done <<< "$commits"
        fi
    fi
done

# Count total commits
commit_count=$(wc -l < "$TEMP_FILE" || echo "0")

# Generate HTML output
cat > "$OUTPUT_FILE" << 'HTML_START'
<section class="reveal" style="margin-bottom:60px;">
  <h2 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:20px;">What We Shipped Today</h2>
  
HTML_START

if [[ $commit_count -eq 0 ]]; then
    # No commits today
    cat >> "$OUTPUT_FILE" << 'HTML_EMPTY'
  <div style="background:#0d1f35;border:1px solid rgba(50,100,160,0.25);border-radius:8px;padding:20px;">
    <div style="font-size:12px;line-height:20px;color:#5a7a94;text-align:center;">
      No commits yet today. Check back later! 🚧
    </div>
  </div>
HTML_EMPTY
else
    # Sort commits by repo name, newest first (by time)
    sort -t'|' -k1,1 -k4,4r "$TEMP_FILE" > "${TEMP_FILE}.sorted"
    
    current_project=""
    
    while IFS='|' read -r project author message time; do
        if [[ "$project" != "$current_project" ]]; then
            # Close previous project div if exists
            if [[ -n "$current_project" ]]; then
                echo "    </div>" >> "$OUTPUT_FILE"
                echo "  </div>" >> "$OUTPUT_FILE"
            fi
            
            # Start new project section
            cat >> "$OUTPUT_FILE" << HTML_PROJECT_START
  <div style="background:#0d1f35;border:1px solid rgba(50,100,160,0.25);border-radius:8px;padding:20px;margin-bottom:16px;">
    <div style="font-size:12px;line-height:20px;color:#4fc3f7;font-weight:600;margin-bottom:12px;">
      📦 $project
    </div>
    <div style="margin-left:12px;">
HTML_PROJECT_START
            current_project="$project"
        fi
        
        # Clean up the commit message (truncate if too long)
        if [[ ${#message} -gt 80 ]]; then
            clean_message="${message:0:77}..."
        else
            clean_message="$message"
        fi
        
        # Add commit entry
        cat >> "$OUTPUT_FILE" << HTML_COMMIT
      <div style="display:flex;align-items:flex-start;gap:8px;margin-bottom:8px;font-size:12px;line-height:20px;">
        <span style="color:#81d4fa;font-weight:500;white-space:nowrap;">$time</span>
        <span style="color:#b8d4e3;">$clean_message</span>
      </div>
HTML_COMMIT
    done < "${TEMP_FILE}.sorted"
    
    # Close the last project div
    if [[ -n "$current_project" ]]; then
        echo "    </div>" >> "$OUTPUT_FILE"
        echo "  </div>" >> "$OUTPUT_FILE"
    fi
    
    rm -f "${TEMP_FILE}.sorted"
fi

# Close section
echo "</section>" >> "$OUTPUT_FILE"

echo "📝 Generated daily activity HTML"
echo "📊 Total commits today: $commit_count"

if [[ $commit_count -gt 0 ]]; then
    echo ""
    echo "📋 Summary by project:"
    sort "$TEMP_FILE" | cut -d'|' -f1 | uniq -c | while read count project; do
        echo "  $project: $count commits"
    done
fi

echo "📄 Output: $OUTPUT_FILE"