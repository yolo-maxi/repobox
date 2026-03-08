# Tools

Scripts for automating repo.box site maintenance and content generation.

## generate-changelog.sh

Generates the build-in-public changelog feed for the homepage "Latest Shipped" section.

**Usage:**
```bash
cd /home/xiko/repobox
./tools/generate-changelog.sh
```

**Output:**
- `public/data/changelog.json` - JSON feed consumed by homepage

**Data Sources:**
- `/home/xiko/kanban-projects/*.md` - Done sections from project kanban boards  
- Recent git commits from key repos: repobox, oceangram, sss

**Schedule:**
- Run after completing significant work to update homepage
- Idempotent - safe to run multiple times
- Could be automated via cron or git hooks if desired

**Format:**
Each changelog entry includes:
- `date` - ISO timestamp of completion/commit  
- `project` - Project name (repo or kanban project)
- `title` - Feature/commit description
- `type` - "feature" (from kanban) or "commit" (from git)
- `link` - Optional link (currently null, could be enhanced)

The homepage JavaScript automatically fetches and renders the latest 6 entries with relative timestamps and project badges.

## daily-activity.sh

Generates today's git commit activity across all repos for the "What We Shipped Today" section.

**Usage:**
```bash
cd /home/xiko/repobox
./tools/daily-activity.sh

# Or for specific date:
TODAY="2026-03-07" ./tools/daily-activity.sh
```

**Output:**
- `public/data/daily-activity.html` - Styled HTML section ready for homepage inclusion

**Data Sources:**
- Git commits from all repos in `/home/xiko/` (29 repos scanned)
- Only commits from the target date (midnight to 23:59 UTC)
- Excludes merge commits

**Features:**
- Groups commits by project/repo
- Shows commit time and cleaned message
- Handles empty state gracefully ("No commits yet today")
- Lightweight styling matching site aesthetic
- Cron-friendly and idempotent

## build-homepage.sh

Rebuilds the complete homepage with integrated daily activity feed.

**Usage:**
```bash
cd /home/xiko/repobox  
./tools/build-homepage.sh
```

**Process:**
1. Runs `daily-activity.sh` to generate latest activity
2. Creates backup of original homepage (if not exists)
3. Inserts activity HTML between "Things We've Shipped" and "What We Build" sections
4. Validates HTML structure
5. Rolls back on validation failure

**Output:**
- `public/index.html` - Complete homepage with activity feed
- `public/index.html.backup` - Original homepage template

**Automation Ready:**
This script can be run via cron to automatically update the daily activity:
```bash
# Run every hour during work days
0 9-18 * * 1-5 cd /home/xiko/repobox && ./tools/build-homepage.sh
```
