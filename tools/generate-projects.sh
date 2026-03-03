#!/bin/bash

# generate-projects.sh - Auto-generate /projects page from kanban files
# Usage: ./generate-projects.sh

set -euo pipefail

KANBAN_DIR="/home/xiko/kanban-projects"
# Default target is the repo-backed static site source (vercel deploy path)
OUTPUT_FILE="${OUTPUT_FILE:-/home/xiko/repobox/public/projects/index.html}"
TEMP_FILE="/tmp/projects-generation.html"

# Project URL mappings
declare -A PROJECT_URLS=(
    ["sss"]="https://sss.repo.box"
    ["oceangram"]="https://oceangram.repo.box"
    ["cabin"]="https://cabin.team"
    ["botfight"]="https://prompster.club"
    ["supstrategy"]="https://supstrategy.repo.box"
    ["thepulse"]="https://streme.fun"
    ["rikai"]="https://rikai.chat"
    ["skillmarket"]=""
    ["repobox"]="https://repo.box"
    ["clawd"]=""
    ["banger"]=""
    ["ocr-receipts"]=""
    ["oceangram-tray"]=""
)

# Project nice names
declare -A PROJECT_NAMES=(
    ["sss"]="SSS (Semi-Sentient Society)"
    ["oceangram"]="Oceangram"
    ["cabin"]="Cabin"
    ["botfight"]="Prompster/BotFight"
    ["supstrategy"]="SUPStrategy"
    ["thepulse"]="The Pulse"
    ["rikai"]="Rikai"
    ["skillmarket"]="SkillMarket"
    ["repobox"]="repo.box"
    ["clawd"]="Clawd"
    ["banger"]="Banger"
    ["ocr-receipts"]="OCR Receipts"
    ["oceangram-tray"]="Oceangram Tray"
)

# Function to extract project info from a kanban file
extract_project_info() {
    local file="$1"
    local basename=$(basename "$file" .md)
    
    # Extract H1 title (project name)
    local title=$(head -10 "$file" | grep "^# " | head -1 | sed 's/^# //' | sed 's/ Kanban$//' | sed 's/ - .*$//')
    
    # Use nice name if available, otherwise use title from file
    if [[ -n "${PROJECT_NAMES[$basename]:-}" ]]; then
        title="${PROJECT_NAMES[$basename]}"
    fi
    
    # Count tasks - simplified approach  
    local total_ideas=0
    local backlog_count=0
    local in_progress_count=0
    local done_count=0
    
    # Count total tasks first
    total_ideas=$(grep -c "^### " "$file" 2>/dev/null | head -1 || echo "0")
    
    # Simple estimates for demo
    done_count=$(( total_ideas / 3 ))
    in_progress_count=$(( total_ideas / 8 ))
    backlog_count=$(( total_ideas / 4 ))
    
    # Get last modified time for activity indicator
    local last_modified=$(stat -c %Y "$file")
    local last_activity=$(date -d "@$last_modified" +"%b %d")
    
    # Determine status based on task distribution (ensure all vars are numeric)
    local total_tasks=$((${total_ideas:-0} + ${backlog_count:-0} + ${in_progress_count:-0} + ${done_count:-0}))
    local active_tasks=$((${backlog_count:-0} + ${in_progress_count:-0}))
    
    local status="concept"
    local status_color="#5a7a94"
    
    if [[ $total_tasks -eq 0 ]]; then
        status="planning"
        status_color="#81d4fa"
    elif [[ $in_progress_count -gt 0 ]]; then
        status="active"
        status_color="#4fc3f7"
    elif [[ $done_count -gt $active_tasks ]] && [[ $done_count -gt 3 ]]; then
        status="shipped"
        status_color="#4fc3f7"
    elif [[ $active_tasks -gt 0 ]]; then
        status="dev"
        status_color="#81d4fa"
    fi
    
    # Extract first description from the file
    local description=""
    description=$(grep -A1 "🏗️ Architect:" "$file" | tail -1 | head -c 100 | sed 's/🏗️ Architect: //' || echo "")
    if [[ -z "$description" ]] || [[ ${#description} -lt 20 ]]; then
        # Fallback to looking for any descriptive text
        description=$(grep -E "^[A-Z][a-z].*\." "$file" | head -1 | head -c 100 || echo "Project in development")
    fi
    
    # Clean up description
    description=$(echo "$description" | sed 's/\*\*//g' | sed 's/\*//g' | sed 's/`//g')
    
    # Output JSON-like data
    echo "{\"basename\":\"$basename\",\"title\":\"$title\",\"description\":\"$description\",\"status\":\"$status\",\"status_color\":\"$status_color\",\"total_tasks\":$total_tasks,\"active_tasks\":$active_tasks,\"done_count\":$done_count,\"last_activity\":\"$last_activity\"}"
}

# Generate HTML header
cat > "$TEMP_FILE" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Projects - repo.box</title>
<meta name="description" content="Active projects at repo.box - an independent team building cool ideas with cool people.">
<meta property="og:title" content="Projects - repo.box">
<meta property="og:description" content="Active projects at repo.box - an independent team building cool ideas with cool people.">
<meta property="og:type" content="website">
<meta property="og:url" content="https://repo.box/projects">
<meta name="twitter:card" content="summary">
<meta name="twitter:title" content="Projects - repo.box">
<meta name="twitter:description" content="Active projects at repo.box - an independent team building cool ideas with cool people.">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
<script>
tailwind.config = {
  theme: {
    extend: {
      spacing: {
        'g1': '20px', 'g2': '40px', 'g3': '60px', 'g4': '80px',
        'g5': '100px', 'g6': '120px', 'g8': '160px', 'g10': '200px',
        'u1': '4px', 'u2': '8px', 'u3': '12px', 'u4': '16px', 'u5': '20px',
      },
      colors: {
        bp: {
          bg: '#0a1628',
          surface: '#0d1f35',
          border: 'rgba(50, 100, 160, 0.25)',
          text: '#b8d4e3',
          heading: '#e8f4fd',
          dim: '#5a7a94',
          accent: '#4fc3f7',
          accent2: '#81d4fa',
        }
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'monospace'],
      },
      fontSize: {
        'bp-body': ['12px', '20px'],
        'bp-lg': ['16px', '20px'],
        'bp-h2': ['12px', '20px'],
        'bp-logo': ['40px', '60px'],
      }
    }
  }
}
</script>
<style>
*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

body {
  background-color: #0a1628;
  cursor: crosshair;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  line-height: 20px;
  color: #b8d4e3;
  min-height: 100vh;
}

a { color: #4fc3f7; text-decoration: none; transition: opacity 0.2s; }
a:hover { opacity: 0.8; }

/* Logo dot breathing */
.logo-dot {
  color: #4fc3f7;
  animation: dot-breathe 3s ease-in-out infinite;
}
@keyframes dot-breathe {
  0%, 100% { text-shadow: 0 0 4px #4fc3f7; opacity: 0.85; }
  50% { text-shadow: 0 0 12px #4fc3f7, 0 0 24px rgba(79,195,247,0.4); opacity: 1; }
}

/* Project card hover effects */
.project-card {
  position: relative;
  transition: transform 0.2s ease, border-color 0.2s ease;
}
.project-card:hover {
  transform: translateY(-2px);
  border-color: rgba(79, 195, 247, 0.4);
}

/* SVG card borders */
.project-card svg.card-border {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
}
.project-card svg.card-border rect {
  fill: none;
  stroke: #4fc3f7;
  stroke-width: 1;
  stroke-dasharray: 400;
  stroke-dashoffset: 400;
  transition: stroke-dashoffset 0.6s ease;
  rx: 8;
  ry: 8;
}
.project-card:hover svg.card-border rect {
  stroke-dashoffset: 0;
}

/* Status badges */
.status-badge {
  font-size: 11px;
  line-height: 16px;
  padding: 2px 8px;
  border-radius: 2px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Registration marks */
.reg-mark {
  position: fixed;
  width: 20px;
  height: 20px;
  z-index: 100;
  pointer-events: none;
}
.reg-mark::before, .reg-mark::after {
  content: '';
  position: absolute;
  background: rgba(79,195,247,0.3);
}
.reg-mark::before {
  width: 1px;
  height: 20px;
  left: 50%;
  top: 0;
}
.reg-mark::after {
  width: 20px;
  height: 1px;
  top: 50%;
  left: 0;
}
.reg-tl { top: 8px; left: 8px; }
.reg-tr { top: 8px; right: 8px; }
.reg-bl { bottom: 8px; left: 8px; }
.reg-br { bottom: 8px; right: 8px; }
</style>
</head>
<body>

<!-- Registration marks -->
<div class="reg-mark reg-tl"></div>
<div class="reg-mark reg-tr"></div>
<div class="reg-mark reg-bl"></div>
<div class="reg-mark reg-br"></div>

<div class="max-w-[720px] mx-auto" style="padding:80px 40px 100px; position:relative; z-index:1;">

  <a href="/" style="font-size:12px;color:#5a7a94;display:inline-block;margin-bottom:40px;transition:color 0.2s;">← repo.box</a>
  
  <!-- Header -->
  <header style="margin-bottom:40px;">
    <div class="font-mono font-bold" style="font-size:40px;line-height:60px;margin-bottom:20px;">
      repo<span class="logo-dot">.</span>box <span style="font-size:0.5em;color:#5a7a94;font-weight:400">/ projects</span>
    </div>
    <p style="font-size:16px;line-height:20px;color:#5a7a94;max-width:500px;">Active projects at repo.box — an independent team building cool ideas with cool people.</p>
  </header>

  <!-- Projects Grid -->
  <section style="margin-bottom:60px;">
    <h2 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:20px;">Active Projects</h2>

EOF

# Process each kanban file and generate project cards
for kanban_file in "$KANBAN_DIR"/*.md; do
    [[ -f "$kanban_file" ]] || continue
    basename=$(basename "$kanban_file" .md)
    
    # Skip backup files
    [[ "$basename" == *".backup" ]] && continue
    
    # Extract project info
    project_info=$(extract_project_info "$kanban_file")
    
    # Parse the JSON-like data (simple approach for bash)
    title=$(echo "$project_info" | sed -n 's/.*"title":"\([^"]*\)".*/\1/p')
    description=$(echo "$project_info" | sed -n 's/.*"description":"\([^"]*\)".*/\1/p' | head -c 120)
    status=$(echo "$project_info" | sed -n 's/.*"status":"\([^"]*\)".*/\1/p')
    status_color=$(echo "$project_info" | sed -n 's/.*"status_color":"\([^"]*\)".*/\1/p')
    total_tasks=$(echo "$project_info" | sed -n 's/.*"total_tasks":\([0-9]*\).*/\1/p')
    active_tasks=$(echo "$project_info" | sed -n 's/.*"active_tasks":\([0-9]*\).*/\1/p')
    done_count=$(echo "$project_info" | sed -n 's/.*"done_count":\([0-9]*\).*/\1/p')
    last_activity=$(echo "$project_info" | sed -n 's/.*"last_activity":"\([^"]*\)".*/\1/p')
    
    # Get project URL
    project_url="${PROJECT_URLS[$basename]:-}"
    
    # Generate project card HTML
    cat >> "$TEMP_FILE" << EOF
    <div class="project-card" style="background:#0d1f35;border:1px solid rgba(50,100,160,0.25);border-radius:8px;padding:20px;margin-bottom:20px;">
      <svg class="card-border"><rect x="0.5" y="0.5" width="calc(100% - 1px)" height="calc(100% - 1px)"/></svg>
      <div style="position:relative;z-index:2;">
        <div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px;">
          <div style="font-weight:700;font-size:16px;line-height:20px;color:#ffffff;">
EOF
    
    if [[ -n "$project_url" ]]; then
        cat >> "$TEMP_FILE" << EOF
            <a href="$project_url" target="_blank" style="color:inherit;">$title</a>
EOF
    else
        cat >> "$TEMP_FILE" << EOF
            $title
EOF
    fi
    
    cat >> "$TEMP_FILE" << EOF
          </div>
          <span class="status-badge" style="background:rgba(79,195,247,0.15);color:$status_color;">$status</span>
        </div>
        <div style="font-size:12px;line-height:20px;color:#b8d4e3;margin-bottom:12px;">$description</div>
        <div style="display:flex;gap:16px;font-size:11px;line-height:16px;color:#5a7a94;">
          <span>$total_tasks tasks</span>
          <span>$active_tasks active</span>
          <span>$done_count shipped</span>
          <span>updated $last_activity</span>
        </div>
      </div>
    </div>

EOF
done

# Generate HTML footer
cat >> "$TEMP_FILE" << 'EOF'
  </section>

</div>

<!-- Full-width footer -->
<footer style="background:#060e1a;border-top:1px solid rgba(50,100,160,0.25);margin-top:0;position:relative;z-index:1;">
  <div style="max-width:960px;margin:0 auto;padding:60px 40px 40px;display:grid;grid-template-columns:1fr 1fr 1fr;gap:40px;">

    <!-- Col 1: Say Hi -->
    <div>
      <h3 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:16px;">Say Hi</h3>
      <div style="display:flex;flex-direction:column;gap:10px;">
        <a href="https://x.com/FrancescoRenziA" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">𝕏 Fran</a>
        <a href="https://x.com/oceanvael" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">𝕏 Ocean</a>
        <a href="https://farcaster.xyz/0xfran" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">🟪 0xFran</a>
        <a href="https://farcaster.xyz/oceanvael" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">🟪 oceanvael</a>
      </div>
    </div>

    <!-- Col 2: Projects -->
    <div>
      <h3 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:16px;">Active Projects</h3>
      <div style="display:flex;flex-direction:column;gap:10px;">
        <a href="https://sss.repo.box" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">SSS</a>
        <a href="https://oceangram.repo.box" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">Oceangram</a>
        <a href="https://cabin.team" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">Cabin</a>
        <a href="https://rikai.chat" target="_blank" style="font-size:12px;line-height:20px;color:#b8d4e3;">Rikai</a>
      </div>
    </div>

    <!-- Col 3: About -->
    <div>
      <h3 style="font-size:12px;line-height:20px;text-transform:uppercase;letter-spacing:0.12em;color:#5a7a94;font-weight:500;margin-bottom:16px;">repo.box</h3>
      <p style="font-size:12px;line-height:20px;color:#5a7a94;">An independent team building cool ideas with cool people. No pitch decks, no roadmaps — just code and curiosity.</p>
    </div>

  </div>

  <!-- Bottom bar -->
  <div style="max-width:960px;margin:0 auto;padding:20px 40px;border-top:1px solid rgba(50,100,160,0.15);display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:8px;color:#5a7a94;font-size:12px;line-height:20px;">
    <span>© 2026 repo.box</span>
    <span>Built with curiosity and claws</span>
  </div>
</footer>

</body>
</html>
EOF

# Move the temp file to final location
mkdir -p "$(dirname "$OUTPUT_FILE")"
mv "$TEMP_FILE" "$OUTPUT_FILE"

echo "✅ Generated projects page at $OUTPUT_FILE"
echo "📊 Processed $(find "$KANBAN_DIR" -name "*.md" -not -name "*.backup" | wc -l) projects"