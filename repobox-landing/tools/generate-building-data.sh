#!/bin/bash

# Generate live building status data from git repositories
# Outputs JSON data for the /building page showing real development status

set -e

OUTPUT_FILE="public/data/building-status.json"
CURRENT_DATE=$(date -u +"%Y-%m-%d")
WEEK_AGO=$(date -u -d "7 days ago" +"%Y-%m-%d")

# Create output directory
mkdir -p public/data

# Array of project directories and their metadata
declare -A PROJECTS
PROJECTS[repobox]="/home/xiko/repobox|both|git,security,infrastructure,cli|https://repo.box"
PROJECTS[sss]="/home/xiko/sss|ocean|dao,verification,on-chain,superfluid|https://sss.repo.box"
PROJECTS[oceangram]="/home/xiko/oceangram|ocean|vscode,telegram,developer-tools,integration|https://marketplace.visualstudio.com/items?itemName=ocean.oceangram"
PROJECTS[archipelago]="/home/xiko/archipelago|ocean|dashboard,telegram,collaboration,monitoring|https://archipelago.repo.box"
PROJECTS[supstrategy]="/home/xiko/supstrategy|ocean|trading,superfluid,defi,monitoring|https://supstrategy.repo.box"
PROJECTS[cabin]="/home/xiko/cabin|ocean|travel,ai-agent,crypto,booking|"
PROJECTS[botfight]="/home/xiko/botfight|ocean|gaming,ai-behavior,social,multiplayer|"
PROJECTS[rikai]="/home/xiko/rikai|fran|education,language,reading,ai-assistant|"
PROJECTS[agentation]="/home/xiko/agentation|both|ai-tools,feedback,annotation,interaction|https://agentation.repo.box"

# Descriptions for each project
declare -A DESCRIPTIONS
DESCRIPTIONS[repobox]="Git permission layer that makes repositories safe for AI agents. Server-first security architecture with CLI tools and browser explorer."
DESCRIPTIONS[sss]="Verified agent DAO with on-chain reputation system. Features corvée work assignment, lobster verification tests, and SUP token rewards."
DESCRIPTIONS[oceangram]="Telegram interface for VS Code with 76 integrated services. Enables coding through chat with full IDE integration and multi-service orchestration."
DESCRIPTIONS[archipelago]="Real-time multi-topic visibility dashboard for Telegram teams. Provides unified view across chat topics with activity monitoring."
DESCRIPTIONS[supstrategy]="AI-powered Superfluid token trading monitor with smart signals. Tracks pool activity, price movements, and provides trading insights."
DESCRIPTIONS[cabin]="AI group travel agent that finds and books real flights with USDC payments. Features multi-destination planning and crypto-native booking flow."
DESCRIPTIONS[botfight]="AI social deduction arena where agents play Mafia with evolving strategies. Real-time multiplayer with behavioral learning systems."
DESCRIPTIONS[rikai]="Interactive language reading assistant with real-time vocabulary help. Features context-aware definitions and progress tracking."
DESCRIPTIONS[agentation]="Visual feedback and annotation tool for AI agent interactions. Enables humans to provide context-aware guidance to agents."

# Function to get commit count in last 7 days for a repo
get_commit_count() {
    local repo_path="$1"
    if [[ -d "$repo_path/.git" ]]; then
        cd "$repo_path"
        git log --oneline --since="$WEEK_AGO" 2>/dev/null | wc -l
    else
        echo "0"
    fi
}

# Function to get last commit date for a repo
get_last_commit_date() {
    local repo_path="$1"
    if [[ -d "$repo_path/.git" ]]; then
        cd "$repo_path"
        git log -1 --format=%cd --date=short 2>/dev/null || echo "$CURRENT_DATE"
    else
        echo "$CURRENT_DATE"
    fi
}

# Function to determine status based on commit activity and known deployment state
get_project_status() {
    local project_id="$1"
    local commit_count="$2"
    local repo_path="$3"
    
    # Manual status overrides for specific projects
    case "$project_id" in
        "agentation")
            echo "shipped" # Known to be stable in production
            ;;
        "botfight"|"rikai")
            echo "concept" # Currently paused/planned
            ;;
        "archipelago"|"cabin")
            echo "beta" # Deployed but under active iteration
            ;;
        *)
            # Auto-determine based on commit activity
            if [[ $commit_count -gt 3 ]]; then
                echo "active"
            elif [[ $commit_count -gt 0 ]]; then
                echo "beta"
            else
                echo "concept"
            fi
            ;;
    esac
}

echo "Generating building status data..."

# Start JSON output
cat > "$OUTPUT_FILE" << 'EOL'
{
  "generated": "TIMESTAMP_PLACEHOLDER",
  "projects": [
EOL

first=true
total_projects=0
total_commits=0
active_count=0
ocean_count=0
fran_count=0
both_count=0

for project_id in "${!PROJECTS[@]}"; do
    IFS='|' read -r repo_path team tags link <<< "${PROJECTS[$project_id]}"
    
    # Get git stats
    commit_count=$(get_commit_count "$repo_path")
    last_commit=$(get_last_commit_date "$repo_path")
    status=$(get_project_status "$project_id" "$commit_count" "$repo_path")
    
    # Update counters
    total_projects=$((total_projects + 1))
    total_commits=$((total_commits + commit_count))
    
    if [[ "$status" == "active" ]]; then
        active_count=$((active_count + 1))
    fi
    
    case "$team" in
        "ocean") ocean_count=$((ocean_count + 1)) ;;
        "fran") fran_count=$((fran_count + 1)) ;;
        "both") both_count=$((both_count + 1)) ;;
    esac
    
    # Add comma separator for JSON array
    if [[ "$first" == "false" ]]; then
        echo "," >> "$OUTPUT_FILE"
    fi
    first=false
    
    # Generate project JSON entry
    cat >> "$OUTPUT_FILE" << EOF
    {
      "id": "$project_id",
      "name": "$(echo "$project_id" | sed 's/\b./\u&/g')",
      "description": "${DESCRIPTIONS[$project_id]}",
      "status": "$status",
      "link": "${link:-null}",
      "lastCommitDate": "$last_commit",
      "lastActivity": "$last_commit",
      "tags": [$(echo "$tags" | sed 's/,/","/g' | sed 's/^/"/' | sed 's/$/"/')],
      "contributorAttribution": "$team",
      "commitCount7d": $commit_count,
      "team": "$team",
      "repositoryPath": "$repo_path"
    }
EOF
done

# Close projects array and add stats
cat >> "$OUTPUT_FILE" << EOF
  ],
  "stats": {
    "totalProjects": $total_projects,
    "activeCount": $active_count,
    "commitsThisWeek": $total_commits,
    "oceanProjects": $ocean_count,
    "franProjects": $fran_count,
    "bothProjects": $both_count
  }
}
EOF

# Replace timestamp placeholder with actual timestamp
sed -i "s/TIMESTAMP_PLACEHOLDER/$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")/g" "$OUTPUT_FILE"

echo "✅ Building status data generated at $OUTPUT_FILE"
echo "📊 Stats: $total_projects projects, $active_count active, $total_commits commits this week"