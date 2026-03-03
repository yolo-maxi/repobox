#!/bin/bash

# Weekly Build Report Generator
# Aggregates git activity from key repos and generates a markdown report

set -e

# Repository mapping
declare -A REPOS
REPOS[repobox]="/home/xiko/repobox"
REPOS[oceangram]="/home/xiko/oceangram"
REPOS[sss]="/home/xiko/sss" 
REPOS[clawd]="/home/xiko/clawd"

# Date calculation - last 7 days
SINCE_DATE=$(date -d '7 days ago' '+%Y-%m-%d')
REPORT_DATE=$(date '+%Y-%m-%d')

echo "# Weekly Build Report - $REPORT_DATE"
echo
echo "Activity summary for the week ending $REPORT_DATE (since $SINCE_DATE)"
echo

# Counters
TOTAL_COMMITS=0
PROJECTS_TOUCHED=0

# What shipped section
echo "## What Shipped This Week"
echo

for repo_name in "${!REPOS[@]}"; do
    repo_path="${REPOS[$repo_name]}"
    
    if [[ ! -d "$repo_path" ]]; then
        continue
    fi
    
    # Check if repo has commits in the last 7 days
    commit_count=$(cd "$repo_path" && git log --since="$SINCE_DATE" --oneline 2>/dev/null | wc -l || echo "0")
    
    if [[ $commit_count -gt 0 ]]; then
        echo "### $repo_name"
        PROJECTS_TOUCHED=$((PROJECTS_TOUCHED + 1))
        TOTAL_COMMITS=$((TOTAL_COMMITS + commit_count))
        
        # Get summary of what was worked on
        cd "$repo_path"
        latest_commits=$(git log --since="$SINCE_DATE" --pretty=format:"- %s" --max-count=5 2>/dev/null || echo "- No recent commits")
        echo "$latest_commits"
        echo
    fi
done

if [[ $PROJECTS_TOUCHED -eq 0 ]]; then
    echo "No significant activity across tracked repositories this week."
    echo
fi

# Key commits section
echo "## Key Commits"
echo

for repo_name in "${!REPOS[@]}"; do
    repo_path="${REPOS[$repo_name]}"
    
    if [[ ! -d "$repo_path" ]]; then
        continue
    fi
    
    # Get detailed commits for the week
    cd "$repo_path"
    commits=$(git log --since="$SINCE_DATE" --pretty=format:"**%h** %s (%cr)" --max-count=3 2>/dev/null || echo "")
    
    if [[ -n "$commits" ]]; then
        echo "### $repo_name"
        echo "$commits"
        echo
    fi
done

# Metrics section
echo "## Metrics"
echo
echo "- **Total commits**: $TOTAL_COMMITS"
echo "- **Projects touched**: $PROJECTS_TOUCHED"
echo "- **Reporting period**: $SINCE_DATE to $REPORT_DATE"
echo

# Next week focus (placeholder)
echo "## Next Week Focus"
echo
echo "- Continue development across active projects"
echo "- Maintain momentum on recent improvements"
echo "- Focus on shipping incremental value"
echo
