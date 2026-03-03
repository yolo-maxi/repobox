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
