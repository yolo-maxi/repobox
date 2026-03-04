#!/bin/bash

# Generate portfolio.json from kanban project data

set -e

KANBAN_DIR="/home/xiko/kanban-projects"
OUTPUT_FILE="/home/xiko/repobox/public/data/portfolio.json"

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Start building JSON array
echo "[" > "$OUTPUT_FILE"

first=true

for md_file in "$KANBAN_DIR"/*.md; do
    # Skip backup files and non-existent files
    [[ "$md_file" == *.backup ]] && continue
    [[ ! -f "$md_file" ]] && continue
    
    filename=$(basename "$md_file" .md)
    
    # Extract project name from first heading (remove "# " and " Kanban" suffix)
    project_name=$(grep "^# " "$md_file" | head -1 | sed 's/^# //' | sed 's/ Kanban$//' | sed 's/ Team Kanban$//')
    
    # If no heading found, use filename
    if [ -z "$project_name" ]; then
        project_name="$filename"
    fi
    
    # Simple approach: count all tasks, then categorize by keywords in the task text
    total_tasks=$(grep "^### " "$md_file" | wc -l)
    
    # Count done tasks (look for completed or done markers)
    done_total=$(grep -E "^### .*✅|Completed.*[0-9]{4}-[0-9]{2}-[0-9]{2}" "$md_file" | wc -l)
    
    # Simple heuristic: if we have tasks, assume some are active
    if [ "$total_tasks" -gt 0 ]; then
        active=$((total_tasks - done_total))
        if [ "$active" -lt 0 ]; then active=0; fi
    else
        active=0
    fi
    
    # Find last completion date by looking for completed tasks with dates  
    last_completion=$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$md_file" | sort | tail -1)
    
    # Default to unknown if no date found
    if [ -z "$last_completion" ]; then
        last_completion="unknown"
    fi
    
    # Determine status badge based on activity
    if [ "$active" -gt 0 ]; then
        status="active"
    elif [ "$done_total" -gt 0 ]; then
        status="shipped"
    else
        status="concept"
    fi
    
    # Extract description from completed tasks or general content
    description=""
    
    # Look for a description in the first few completed tasks
    if [ -z "$description" ]; then
        description=$(grep -A5 "^### .*✅" "$md_file" | grep -v "^--$" | grep -v "^### " | grep -E "^[A-Za-z]" | head -1 | cut -c1-100)
    fi
    
    # Look for current task descriptions  
    if [ -z "$description" ]; then
        description=$(awk '/^### TASK/ {getline; if(/^[A-Za-z]/ && length($0) > 15) {print; exit}}' "$md_file" | cut -c1-100)
    fi
    
    # Default description
    if [ -z "$description" ]; then
        description="Project in development"
    fi
    
    # Clean up description (remove markdown, truncate)
    description=$(echo "$description" | sed 's/\*\*//g' | sed 's/__//g' | sed 's/  */ /g' | cut -c1-100)
    
    # Add comma separator for all except first
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> "$OUTPUT_FILE"
    fi
    
    # Write project JSON object  
    cat >> "$OUTPUT_FILE" << EOF
  {
    "name": "$project_name",
    "filename": "$filename",
    "description": "$description",
    "status": "$status",
    "tasks": {
      "total": $total_tasks,
      "active": $active,
      "done": $done_total
    },
    "last_update": "$last_completion"
  }
EOF
done

# Close JSON array
echo "" >> "$OUTPUT_FILE"
echo "]" >> "$OUTPUT_FILE"

echo "Generated portfolio data: $OUTPUT_FILE"
echo "Found $(jq length "$OUTPUT_FILE") projects"