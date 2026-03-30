#!/bin/bash

# generate-activity-heatmap.sh
# Generate 365-day activity heatmap data for repo.box homepage
# Aggregates git commits from all active repo.box repositories
#
# Usage: ./tools/generate-activity-heatmap.sh
# Output: repobox-landing/public/heatmap-data.json
#
# This script is idempotent and cron-friendly.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$(dirname "$SCRIPT_DIR")" && pwd)"
OUTPUT_FILE="$REPO_ROOT/repobox-landing/public/heatmap-data.json"

echo "🔍 Generating activity heatmap data for repo.box..."

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Date range: 365 days from today
END_DATE=$(date '+%Y-%m-%d')
START_DATE=$(date -d '365 days ago' '+%Y-%m-%d')

echo "📅 Date range: $START_DATE to $END_DATE"

# List of repositories to scan
REPOS=(
    "/home/xiko/repobox"
    "/home/xiko/clawd"
    "/home/xiko/sss"
    "/home/xiko/oceangram"
    "/home/xiko/cabin"
    "/home/xiko/botfight"
    "/home/xiko/streme-frontend"
    "/home/xiko/rikai-ui"
    "/home/xiko/beamr-economy"
    "/home/xiko/prompster"
    "/home/xiko/prompster-app"
    "/home/xiko/langbot"
    "/home/xiko/yolomaxi-telegram"
    "/home/xiko/SUPStrategy"
    "/home/xiko/kanban-projects"
    "/home/xiko/skillmarket"
    "/home/xiko/pool-admin"
    "/home/xiko/tradestrategy-work"
)

# Create temporary file to collect data
TEMP_FILE=$(mktemp)
TEMP_SUMMARY=$(mktemp)
trap "rm -f $TEMP_FILE $TEMP_SUMMARY" EXIT

# Scan each repository
for repo in "${REPOS[@]}"; do
    if [[ -d "$repo/.git" ]]; then
        repo_name=$(basename "$repo")
        
        echo "📦 Scanning $repo_name..."
        
        # Get commits for the date range
        cd "$repo"
        git log --since="$START_DATE 00:00:00" --until="$END_DATE 23:59:59" \
            --pretty=format:"%ad|%H" --date=format:'%Y-%m-%d' \
            --no-merges 2>/dev/null | while IFS='|' read -r date hash; do
                echo "$date|$repo_name" >> "$TEMP_FILE"
            done || true
    fi
done

# Aggregate data per day
sort "$TEMP_FILE" | uniq -c | while read count date_repo; do
    echo "$date_repo|$count" >> "$TEMP_SUMMARY"
done

# Generate date range array
current_date="$START_DATE"
date_range=""
while [[ "$current_date" != $(date -d "$END_DATE + 1 day" '+%Y-%m-%d') ]]; do
    date_range="$date_range $current_date"
    current_date=$(date -d "$current_date + 1 day" '+%Y-%m-%d')
done

# Determine activity levels based on commit count
get_activity_level() {
    local count=$1
    if [[ $count -eq 0 ]]; then
        echo "empty"
    elif [[ $count -le 3 ]]; then
        echo "light"
    elif [[ $count -le 10 ]]; then
        echo "medium"
    else
        echo "intense"
    fi
}

# Generate JSON output
echo "📝 Generating JSON output..."

# Create JSON file
echo '{' > "$OUTPUT_FILE"
echo "  \"startDate\": \"$START_DATE\"," >> "$OUTPUT_FILE"
echo "  \"endDate\": \"$END_DATE\"," >> "$OUTPUT_FILE"
echo "  \"data\": {" >> "$OUTPUT_FILE"

first_entry=true
for date in $date_range; do
    # Count commits and repos for this date
    commit_count=0
    repo_list=""
    
    # Sum up all commits for this date
    if [[ -s "$TEMP_SUMMARY" ]]; then
        while IFS='|' read -r entry_date entry_repo count_str; do
            if [[ "$entry_date" == "$date" ]]; then
                commit_count=$((commit_count + count_str))
                if [[ -n "$repo_list" ]]; then
                    repo_list="$repo_list,$entry_repo"
                else
                    repo_list="$entry_repo"
                fi
            fi
        done < "$TEMP_SUMMARY"
    fi
    
    # Count unique repos
    if [[ -n "$repo_list" ]]; then
        repo_count=$(echo "$repo_list" | tr ',' '\n' | sort -u | wc -l)
    else
        repo_count=0
    fi
    
    level=$(get_activity_level "$commit_count")
    
    # Add comma for all entries except the first
    if [[ "$first_entry" == "true" ]]; then
        first_entry=false
    else
        echo "," >> "$OUTPUT_FILE"
    fi
    
    # Add JSON entry
    echo "    \"$date\": {" >> "$OUTPUT_FILE"
    echo "      \"count\": $commit_count," >> "$OUTPUT_FILE"
    echo "      \"level\": \"$level\"," >> "$OUTPUT_FILE"
    echo "      \"repos\": \"$repo_list\"," >> "$OUTPUT_FILE"
    echo "      \"repoCount\": $repo_count" >> "$OUTPUT_FILE"
    echo -n "    }" >> "$OUTPUT_FILE"
done

echo "" >> "$OUTPUT_FILE"
echo "  }" >> "$OUTPUT_FILE"
echo "}" >> "$OUTPUT_FILE"

# Calculate summary statistics
total_commits=$(grep -o '"count": [0-9]*' "$OUTPUT_FILE" | cut -d' ' -f2 | paste -sd+ | bc || echo "0")
active_days=$(grep -c '"count": [1-9]' "$OUTPUT_FILE" || echo "0")
total_days=$(echo "$date_range" | wc -w)

echo "✅ Generated heatmap data successfully"
echo "📊 Total commits: $total_commits"
echo "📅 Active days: $active_days out of $total_days"
echo "📄 Output: $OUTPUT_FILE"

# Validate JSON
if command -v jq >/dev/null 2>&1; then
    if jq empty "$OUTPUT_FILE" >/dev/null 2>&1; then
        echo "✅ JSON validation passed"
    else
        echo "❌ JSON validation failed"
        exit 1
    fi
else
    echo "⚠️  jq not available for JSON validation"
fi