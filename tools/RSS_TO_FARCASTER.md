# RSS-to-Farcaster Auto-Publish

Automatically detects new blog posts in `public/feed.xml` and publishes them to Farcaster with a preview.

## Features

- **Idempotent**: Safe to run multiple times, only posts new items
- **State tracking**: Stores last-posted GUID in `.state/farcaster-last-guid.txt`
- **Safeguards**: Integrates with existing `farcaster-cast-guard.sh` for all posting
- **Smart formatting**: Auto-truncates descriptions to fit Farcaster's 320-char limit
- **Dry-run default**: Won't post by default, requires explicit `--post` flag

## Usage

```bash
# Check what would be posted (dry-run mode - default)
./tools/rss-to-farcaster.sh
./tools/rss-to-farcaster.sh --dry-run

# Actually post to Farcaster
./tools/rss-to-farcaster.sh --post

# Help
./tools/rss-to-farcaster.sh --help
```

## Example Output

### Dry-run (default):
```
📝 NEW POST DETECTED (dry-run mode)
Title: Weekly Build Report — March 3, 2026
Link: https://repo.box/blog/weekly-build-report-2026-03-03
GUID: https://repo.box/blog/weekly-build-report-2026-03-03

Cast text (222 chars):
---
New post: Weekly Build Report — March 3, 2026

346 commits across 4 projects. Oceangram v0.9.0 ships, SSS connects to Base Sepolia, repo.box gets dynamic project pages.

https://repo.box/blog/weekly-build-report-2026-03-03
---

To actually post, run: ./tools/rss-to-farcaster.sh --post
```

### When nothing to post:
```
✅ No new posts (latest GUID already posted: https://repo.box/blog/weekly-build-report-2026-03-03)
```

### Successful posting:
```
🚀 POSTING NEW BLOG POST
Title: Weekly Build Report — March 3, 2026
GUID: https://repo.box/blog/weekly-build-report-2026-03-03

[Cast guard and Farcaster posting output]
✅ Posted and updated state file
```

## Safety & Automation

### Safeguards
- Uses existing `farcaster-cast-guard.sh` workflow
- No bypass of safeguards - all security checks apply
- Only posts when explicitly using `--post` flag
- Stores state only after successful posting

### Cron Setup
For automated posting (recommended):

```bash
# Add to crontab: check every 30 minutes
*/30 * * * * cd /home/xiko/repobox && ./tools/rss-to-farcaster.sh --post >/dev/null 2>&1
```

### Manual Workflow
1. Write new blog post
2. Update RSS feed (`public/feed.xml`)
3. Run `./tools/rss-to-farcaster.sh --dry-run` to preview
4. Run `./tools/rss-to-farcaster.sh --post` to publish

## Dependencies

- `xmllint` (from `libxml2-utils` package) - for RSS parsing
- `farcaster-cast-guard.sh` - for safe posting with security checks
- Farcaster credentials configured in `~/clawd/secrets/`

## State Management

- **State file**: `.state/farcaster-last-guid.txt`
- **Contains**: GUID of the last successfully posted item
- **Behavior**: Script only posts items with GUIDs newer than stored state
- **Reset**: Delete state file to repost the latest item

## Text Formatting

Cast format:
```
New post: [TITLE]

[DESCRIPTION (truncated if needed)]

[LINK]
```

Auto-truncation:
1. Try with 150-char description
2. If too long, try 100-char description  
3. If still too long, use title + link only
4. Never exceeds Farcaster's 320-character limit

## Error Handling

- Missing RSS feed → Exit with error
- Missing xmllint → Install instructions
- Cast guard blocks → Exit with error, no state update
- Posting fails → Exit with error, no state update
- Missing required RSS fields → Exit with error and debug output

## Testing

```bash
# Test parsing
./tools/rss-to-farcaster.sh --dry-run

# Test with specific state
echo "some-old-guid" > .state/farcaster-last-guid.txt
./tools/rss-to-farcaster.sh --dry-run  # Should detect new post

echo "$(xmllint --xpath 'string(//item[1]/guid)' public/feed.xml)" > .state/farcaster-last-guid.txt
./tools/rss-to-farcaster.sh --dry-run  # Should show "No new posts"

# Reset for actual testing
rm .state/farcaster-last-guid.txt
```
