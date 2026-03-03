#!/bin/bash

# generate-changelog.sh
# Deterministic changelog generator for repo.box homepage
# Reads from kanban projects and git commits to create build-in-public feed
#
# Usage: ./tools/generate-changelog.sh
# Output: public/data/changelog.json
# Sources: 
#   - /home/xiko/kanban-projects/*.md (Done sections)
#   - Recent git commits from repobox, oceangram, sss repos
#
# This script is idempotent - can be run multiple times safely.

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_FILE="$REPO_ROOT/public/data/changelog.json"

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

echo "🔍 Generating changelog for repo.box..."

# Create a Node.js script to do the heavy lifting
cat > /tmp/changelog-generator.js << 'EOF'
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const entries = [];

// Function to add entry
function addEntry(date, project, title, type, link = null) {
    entries.push({
        date: date,
        project: project,
        title: title,
        type: type,
        link: link
    });
}

// Scan kanban projects
console.log('🔍 Scanning kanban projects...');
const kanbanDir = '/home/xiko/kanban-projects';
if (fs.existsSync(kanbanDir)) {
    const files = fs.readdirSync(kanbanDir).filter(f => f.endsWith('.md'));
    
    for (const file of files) {
        const projectName = path.basename(file, '.md');
        const content = fs.readFileSync(path.join(kanbanDir, file), 'utf8');
        
        // Find Done sections
        const lines = content.split('\n');
        let inDoneSection = false;
        
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            // Check for Done section headers
            if (line.match(/^## ✅ Done \(Agent\)|^## ✅ Done \(Fran\)/)) {
                inDoneSection = true;
                continue;
            }
            
            // Exit Done section
            if (inDoneSection && line.match(/^## [^✅]/)) {
                inDoneSection = false;
                continue;
            }
            
            // Extract task in Done section
            if (inDoneSection && line.startsWith('### ')) {
                const title = line.substring(4).trim();
                
                // Look for completion date in following lines
                for (let j = i + 1; j < lines.length && j < i + 10; j++) {
                    const nextLine = lines[j];
                    
                    if (nextLine.match(/^\*\*Completed\*\*:/)) {
                        const match = nextLine.match(/^\*\*Completed\*\*:\s*(.+)$/);
                        if (match) {
                            const completionDate = match[1].trim();
                            addEntry(completionDate, projectName, title, 'feature');
                        }
                        break;
                    }
                    
                    // Stop if we hit another section
                    if (nextLine.startsWith('### ') || nextLine.startsWith('## ')) {
                        break;
                    }
                }
            }
        }
    }
}

// Scan git repos
console.log('🔍 Scanning git repos...');
const repos = {
    'repobox': '/home/xiko/repobox',
    'oceangram': '/home/xiko/oceangram',
    'sss': '/home/xiko/sss'
};

for (const [repoName, repoPath] of Object.entries(repos)) {
    if (fs.existsSync(path.join(repoPath, '.git'))) {
        try {
            const output = execSync(
                'git log --since="14 days ago" --pretty=format:"%ci|%s" --no-merges',
                { cwd: repoPath, encoding: 'utf8', maxBuffer: 1024 * 1024 }
            );
            
            const commits = output.trim().split('\n').slice(0, 15);
            
            for (const commit of commits) {
                if (commit.trim()) {
                    const [date, message] = commit.split('|', 2);
                    if (message && !message.startsWith('Merge ')) {
                        // Clean up commit message
                        const cleanMessage = message
                            .replace(/^[a-z]+\([^)]*\):\s*/, '')
                            .replace(/^[a-z]+:\s*/, '')
                            .trim();
                        
                        addEntry(date, repoName, cleanMessage, 'commit');
                    }
                }
            }
        } catch (err) {
            console.warn(`Warning: Could not read git log from ${repoName}: ${err.message}`);
        }
    }
}

// Sort by date (newest first) and limit to 10
entries.sort((a, b) => new Date(b.date) - new Date(a.date));
const limitedEntries = entries.slice(0, 10);

// Generate final JSON
const output = {
    generated: new Date().toISOString(),
    count: limitedEntries.length,
    entries: limitedEntries
};

console.log('📝 Writing changelog JSON...');
fs.writeFileSync(process.argv[2], JSON.stringify(output, null, 2));

console.log(`✅ Generated changelog with ${output.count} entries`);
if (output.count > 0) {
    console.log('\n📋 Recent entries:');
    limitedEntries.slice(0, 5).forEach(entry => {
        const dateStr = entry.date.substring(0, 10);
        console.log(`  ${dateStr} [${entry.project}] ${entry.title}`);
    });
}
EOF

# Run the Node.js script
node /tmp/changelog-generator.js "$OUTPUT_FILE"

# Validate JSON
if jq empty "$OUTPUT_FILE" 2>/dev/null; then
    echo "📄 Output: $OUTPUT_FILE"
else
    echo "❌ Invalid JSON generated"
    exit 1
fi

# Clean up
rm -f /tmp/changelog-generator.js
